#!/usr/bin/env python3
"""ENGINE-UNIVERSAL-SERVICES.67(c)(iii) — the memo REHASH SPIKE, from LIFETIMES.

⛔ WHY THIS EXISTS RATHER THAN ANOTHER READING OF THE SNAPSHOT. `-0128` published
*"two success tables are live at once … that is the REHASH window, so the memo's
contribution to PEAK is 1.5x its steady-state size"* from the probe's size-class
SNAPSHOT. A snapshot cannot support that sentence, for three separate reasons:

  1. It shows CO-RESIDENCY, not SUCCESSION. Two tables of the same shape being live
     together does not say one replaced the other. `floor.sv` is the counterexample
     the tracked artifact already contained: FOUR `T=296` tables live at once, two of
     them the SAME size — which no rehash can produce, because a rehash doubles.
  2. It is taken at the HYSTERESIS instant, not at the PEAK instant. Those are
     different moments and the conclusion was stated about the peak.
  3. A count over it is not a count over the ladder: re-derived, the snapshot says
     FIVE of eight inputs, not four.

Succession is a LIFETIME property. This analyser reads the lifetime tape — one row
per allocation at or above the threshold, carrying the ordinal at which it was made
and the ordinal at which it was released — and decides, per input:

  * which allocations were live at the EXACT instant `peak_live` was set;
  * which of the live memo tables were SUPERSEDED, meaning a larger table of the same
    family was allocated while this one was still live and this one was freed after;
  * therefore how many bytes of the peak are rehash overlap, and what the memo's own
    peak/steady ratio actually is.

⛔ THE QUANTITY. Every byte here is REQUESTED LIVE BYTES, the same allocation-side
quantity `peak_live` is. Nothing in this file is a residency claim.

⛔ AND THE BOUND IS NAMED RATHER THAN IMPLIED. The overlap is what the peak-setting
instant would lose if the superseded table had been freed first. The whole-parse peak
would fall by AT MOST that, because the peak may relocate to a moment between two
recorded events — the tape records only allocations at or above its threshold and
cannot bound the live curve in between. Reported as an UPPER BOUND on the saving.

Usage: rehash_spike_report.py <tape.tsv> <meta.tsv> <classes.tsv>
"""
import sys
from collections import defaultdict

MB = 1048576.0
# ⭐ DETERMINED, NOT ASSUMED, and held fixed at the value `-0128` pinned from the
# large tables where the two candidates are 8 bytes apart. Re-deriving it here would
# hand the search a free degree of freedom it has already been observed fabricating
# with (the same 64-bucket table named twice at two widths).
GROUP_WIDTH = 8
NEVER = (1 << 64) - 1


def read_tsv(path):
    with open(path) as fh:
        head = fh.readline().rstrip("\n").split("\t")
        return [dict(zip(head, ln.rstrip("\n").split("\t"))) for ln in fh if ln.strip()]


def hashbrown_table(size, entry_size):
    """Bucket count if `size` is exactly a hashbrown table for `entry_size`, else None."""
    for k in range(0, 40):
        buckets = 1 << k
        if buckets * entry_size + buckets + GROUP_WIDTH == size:
            return buckets
    return None


def analyse(tape_path, meta_path, class_path, emit=lambda _s: None):
    """The whole per-input analysis, factored out so the SCORER reads the same numbers
    this report prints. A scorer with its own second implementation is two chances to
    be wrong and no chance to notice (`-0130`: agreeing with yourself is not
    corroboration)."""
    tape_rows, meta_rows, class_rows = (read_tsv(p) for p in (tape_path, meta_path, class_path))

    tape = defaultdict(list)
    for r in tape_rows:
        tape[r["path"]].append(
            (int(r["slot"]), int(r["size"]), int(r["alloc_seq"]), int(r["free_seq"]))
        )
    classes = defaultdict(list)
    for r in class_rows:
        classes[r["path"]].append(
            (int(r["class"]), int(r["lo"]), int(r["hi"]), int(r["live_bytes"]), int(r["live_count"]))
        )

    control_bad = 0
    control_checked = 0
    refused = []
    rows = []

    for m in meta_rows:
        path = m["path"]
        name = path.rsplit("/", 1)[-1]
        if m.get("tape_complete") != "true":
            refused.append((name, f"tape incomplete (overflow={m.get('tape_overflow')} "
                                  f"unmatched_frees={m.get('tape_unmatched_frees')})"))
            continue
        entries = tape[path]
        tape_min = int(m["tape_min_bytes"])
        # ⛔ ONE CLOCK. The probe also reports per-kind ordinals (`peak_at_alloc_calls`
        # and friends), and comparing an allocation ordinal with a free ordinal is
        # meaningless — they count different events. Everything below is the unified
        # EVENT sequence, which is why succession is decidable at all.
        peak_seq = int(m["peak_at_event_seq"])
        snap_seq = int(m["snap_at_event_seq"])
        peak_live = int(m["peak_live"])
        memo_T = int(m["memo_bucket_B"])
        fail_T = int(m["memo_fail_bucket_B"])

        def live_at(cut):
            """Allocated at or before `cut` and not yet released at `cut`."""
            return [e for e in entries if e[2] <= cut and (e[3] == NEVER or e[3] > cut)]

        # ── THE CONTROL ────────────────────────────────────────────────────────
        # The class table and the tape are maintained by two independent code paths
        # in the probe. Reconstructed from the tape at the SNAPSHOT instant, every
        # class at or above the tape threshold must reproduce the class table EXACTLY
        # — bytes and count. A mismatch invalidates the tape, and it is counted here
        # rather than smoothed away.
        recon_bytes, recon_count = defaultdict(int), defaultdict(int)
        for _, size, _, _ in live_at(snap_seq):
            k = (size.bit_length()) if size else 1
            recon_bytes[k] += size
            recon_count[k] += 1
        for k, lo, _hi, lb, lc in classes[path]:
            if lo < tape_min:
                continue
            control_checked += 1
            if recon_bytes[k] != lb or recon_count[k] != lc:
                control_bad += 1
                emit(f"⛔ CONTROL MISMATCH {name} class={k}: tape {recon_bytes[k]}B/"
                     f"{recon_count[k]} vs class table {lb}B/{lc}")
        # A class the tape reconstructs but the class table never reported is equally
        # a mismatch, so the reverse direction is checked too.
        reported = {k for k, lo, _h, _b, _c in classes[path] if lo >= tape_min}
        for k in recon_bytes:
            if k not in reported and (1 << max(k - 1, 0)) >= tape_min:
                control_checked += 1
                control_bad += 1
                emit(f"⛔ CONTROL MISMATCH {name} class={k}: tape {recon_bytes[k]}B "
                     f"but the class table reports nothing")

        # ── THE MEMO TABLES LIVE AT THE PEAK INSTANT ───────────────────────────
        peak_live_entries = live_at(peak_seq)
        memo = []
        for slot, size, a, f in peak_live_entries:
            b = hashbrown_table(size, memo_T)
            if b is not None:
                memo.append({"slot": slot, "size": size, "buckets": b, "alloc": a, "free": f})
        memo.sort(key=lambda d: -d["size"])

        # ── SUCCESSION, WHICH IS THE THING A SNAPSHOT CANNOT SEE ───────────────
        # A is SUPERSEDED by B iff B is larger, B was allocated while A was still
        # live (alloc(A) < alloc(B) < free(A)), and A is freed at all. Two tables
        # that are merely co-resident fail this and are reported as CO-RESIDENT,
        # never as a rehash.
        superseded, coresident = [], []
        for a_tab in memo[1:]:
            hit = next(
                (b for b in memo
                 if b["size"] > a_tab["size"]
                 and a_tab["alloc"] < b["alloc"]
                 and (a_tab["free"] == NEVER or b["alloc"] < a_tab["free"])
                 and a_tab["free"] != NEVER),
                None,
            )
            (superseded if hit else coresident).append(a_tab)

        memo_bytes = sum(d["size"] for d in memo)
        biggest = memo[0]["size"] if memo else 0
        overlap = sum(d["size"] for d in superseded)
        rows.append({
            "name": name,
            "bytes": int(m["bytes"]),
            "peak_live": peak_live,
            "n_memo": len(memo),
            "buckets": [d["buckets"] for d in memo],
            "memo_bytes": memo_bytes,
            "biggest": biggest,
            "n_superseded": len(superseded),
            "n_coresident": len(coresident),
            "overlap": overlap,
            "ratio": (memo_bytes / biggest) if biggest else 0.0,
            "share": (overlap / peak_live) if peak_live else 0.0,
            "thin_need_bytes": int(m["thin_need_bytes"]) if m["thin_need_bytes"] not in ("NA", "") else None,
            "recorded": int(m["tape_recorded"]),
            "entries": entries,
            "fail_T": fail_T,
            "meta": m,
        })

    return rows, refused, control_checked, control_bad


def main(argv):
    if len(argv) != 4:
        print(__doc__)
        return 2
    rows, refused, control_checked, control_bad = analyse(argv[1], argv[2], argv[3], print)

    print("=== TAPE CONTROL: the lifetime tape against the probe's own class table ===")
    print(f"  {control_checked} (input, class) pairs at or above the tape threshold checked, "
          f"{control_bad} mismatched")
    if refused:
        print("\n⛔ REFUSED INPUTS (an incomplete tape is refused, never averaged):")
        for n, why in refused:
            print(f"    {n}: {why}")

    print("\n=== MEMO TABLES LIVE AT THE EXACT INSTANT `peak_live` WAS SET ===")
    print(f"{'input':34s} {'file B':>10s} {'peak MB':>10s} {'n':>2s} {'buckets':>26s} "
          f"{'memo MB':>9s} {'ratio':>7s} {'sup':>3s} {'cor':>3s} {'overlap MB':>10s} {'share':>7s}")
    for r in rows:
        print(f"{r['name']:34s} {r['bytes']:10d} {r['peak_live']/MB:10.2f} {r['n_memo']:2d} "
              f"{str(r['buckets']):>26s} {r['memo_bytes']/MB:9.2f} {r['ratio']:7.4f} "
              f"{r['n_superseded']:3d} {r['n_coresident']:3d} {r['overlap']/MB:10.2f} "
              f"{r['share']*100:6.2f}%")

    body = [r for r in rows if r["name"] != "floor.sv"]
    over = [r for r in body if r["n_superseded"] >= 1]
    multi = [r for r in body if r["n_memo"] >= 2]
    print(f"\n  inputs with >= 2 memo tables live at the peak : {len(multi)} of {len(body)}")
    print(f"  inputs with a SUPERSEDED (rehash) table       : {len(over)} of {len(body)}")
    if over:
        shares = sorted(r["share"] for r in over)
        med = shares[len(shares) // 2] if len(shares) % 2 else (shares[len(shares)//2 - 1] + shares[len(shares)//2]) / 2
        print(f"  overlap share of peak_live over those        : min {min(shares)*100:.2f}% · "
              f"median {med*100:.2f}% · max {max(shares)*100:.2f}%")
        ratios = sorted(r["ratio"] for r in over)
        print(f"  memo peak/steady ratio over those            : min {min(ratios):.4f} · "
              f"max {max(ratios):.4f}")
    flat = [r for r in body if r["n_memo"] == 1]
    if flat:
        print(f"  memo peak/steady ratio where none overlaps   : "
              f"{sorted({round(r['ratio'], 4) for r in flat})} on {len(flat)} inputs")

    # ── PRE-SIZING: is the final table predictable from the input length? ──────
    print("\n=== CAN THE MEMO BE PRE-SIZED FROM INPUT LENGTH? (the fix pricing) ===")
    print("A pre-size removes the rehash overlap only if it lands within the 50 % the")
    print("overlap costs. The final bucket count per input byte is what decides it.")
    ratios = []
    for r in body:
        if not r["biggest"]:
            continue
        buckets = r["buckets"][0]
        per = buckets / r["bytes"]
        ratios.append((per, r["name"], buckets))
        print(f"  {r['name']:34s} final {buckets:>9d} buckets  {per:8.4f} buckets/input byte")
    if ratios:
        lo, hi = min(ratios), max(ratios)
        print(f"  spread: {lo[0]:.4f} ({lo[1]}) .. {hi[0]:.4f} ({hi[1]}) = {hi[0]/lo[0]:.2f}x")

    # ── THE CROWDED CLASSES, DECOMPOSED BY OBSERVATION ────────────────────────
    # ⛔ THIS IS THE ONE THAT REFUTES AN INFERENCE RATHER THAN CONFIRMING ONE. Naming a
    # structure inside a size-class TOTAL is a subset-sum search, and a total has many
    # decompositions. The tape prints the decomposition that actually happened, for the
    # classes where the search has the most freedom: the crowded low ones.
    print("\n=== CROWDED CLASSES, DECOMPOSED BY OBSERVATION RATHER THAN BY SEARCH ===")
    print("Every class below is reproduced to the byte by the tape (see the control above);")
    print("the LIST is what the process actually allocated, which a class total cannot say.")
    for r in rows:
        entries, meta_row = r["entries"], r["meta"]
        snap_seq = int(meta_row["snap_at_event_seq"])
        live = [e for e in entries if e[2] <= snap_seq and (e[3] == NEVER or e[3] > snap_seq)]
        by_class = defaultdict(list)
        for _slot, size, _a, _f in live:
            by_class[size.bit_length()].append(size)
        crowded = {k: sorted(v, reverse=True) for k, v in by_class.items() if len(v) >= 3}
        if not crowded:
            continue
        print(f"  {r['name']}")
        for k in sorted(crowded):
            v = crowded[k]
            print(f"    class {k:2d}  {len(v)} allocations, {sum(v)} B  {v}")

    # ── THE THIN TABLE'S LIFETIME, DIRECTLY ───────────────────────────────────
    print("\n=== THE THIN-MEMO TABLE'S LIFETIME, READ FROM THE TAPE ===")
    print("`-0128` inferred `never freed` from `end_live` minus a predicted value. The")
    print("tape reads the free ordinal itself, which is a different instrument.")
    for r in rows:
        need = r["thin_need_bytes"]
        if not need:
            continue
        hit = [e for e in r["entries"] if e[1] == need]
        if not hit:
            print(f"  {r['name']:34s} predicted {need:>12d} B — NOT FOUND in the tape")
            continue
        for _slot, size, a, f in hit:
            print(f"  {r['name']:34s} {size:>12d} B  alloc_seq={a:<10d} "
                  f"free_seq={'NEVER' if f == NEVER else f}")

    return 1 if control_bad else 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))

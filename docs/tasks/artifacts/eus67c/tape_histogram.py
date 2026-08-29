#!/usr/bin/env python3
"""ENGINE-UNIVERSAL-SERVICES.67(c)(iii) — reduce a lifetime tape to a tracked histogram.

⛔ WHY A REDUCTION AND NOT THE RAW FILE. At an 8 KiB threshold the ladder's tape is ~250 000
rows for ONE input — `br_gh330.v`, a 36 KB file — and a 20 MB blob is not raw data anyone will
read. The rehash question needs only the LARGE allocations, so the tracked tape is taken at a
16 KiB threshold; this reducer carries the 8 KiB run's finding in a form a reader can check:
per input and per exact size, how many allocations, how many were freed, and the mean lifetime
in EVENTS. Nothing here is a sample — the run it reduces is `complete=true`, and an incomplete
one is REFUSED by name rather than reduced.

⚠️ EXACT SIZE, NOT SIZE CLASS. The finding is that ONE size repeats: a class would hide that
65 443 of the 65 464 allocations in `[8192, 16384)` are the same 8192 bytes, which is what makes
them a repeated buffer rather than assorted traffic.

Usage: tape_histogram.py <tape.tsv> <meta.tsv> <out_histogram.tsv> [min_count]
"""
import collections
import csv
import sys

NEVER = (1 << 64) - 1


def main():
    if len(sys.argv) not in (4, 5):
        print(__doc__)
        return 2
    tape_p, meta_p, out_p = sys.argv[1:4]
    min_count = int(sys.argv[4]) if len(sys.argv) == 5 else 8

    meta = {r["path"]: r for r in csv.DictReader(open(meta_p), delimiter="\t")}
    incomplete = [p for p, r in meta.items() if r.get("tape_complete") != "true"]
    if incomplete:
        print("⛔ REFUSING: the tape is incomplete for "
              f"{[p.rsplit('/', 1)[-1] for p in incomplete]} — a truncated tape reduced to a "
              "histogram looks exactly like a complete one.")
        return 1

    agg = collections.defaultdict(lambda: {"n": 0, "freed": 0, "life": 0, "bytes": 0})
    for r in csv.DictReader(open(tape_p), delimiter="\t"):
        k = (r["path"], int(r["size"]))
        a = agg[k]
        a["n"] += 1
        a["bytes"] += int(r["size"])
        f = int(r["free_seq"])
        if f != NEVER:
            a["freed"] += 1
            a["life"] += f - int(r["alloc_seq"])

    with open(out_p, "w") as fh:
        fh.write("path\tsize\tcount\tfreed\tnever_freed\ttotal_bytes\tmean_lifetime_events\n")
        for (path, size), a in sorted(agg.items(), key=lambda kv: (kv[0][0], -kv[1]["n"])):
            if a["n"] < min_count:
                continue
            mean = f"{a['life'] / a['freed']:.1f}" if a["freed"] else "NA"
            fh.write(f"{path}\t{size}\t{a['n']}\t{a['freed']}\t{a['n'] - a['freed']}\t"
                     f"{a['bytes']}\t{mean}\n")

    top = sorted(agg.items(), key=lambda kv: -kv[1]["n"])[:8]
    print(f"wrote {out_p} (sizes with at least {min_count} allocations)")
    print(f"{'input':34s} {'size B':>10s} {'count':>8s} {'freed':>8s} {'mean life (events)':>19s}")
    for (path, size), a in top:
        mean = f"{a['life'] / a['freed']:.1f}" if a["freed"] else "NA"
        print(f"{path.rsplit('/', 1)[-1]:34s} {size:10d} {a['n']:8d} {a['freed']:8d} {mean:>19s}")
    return 0


if __name__ == "__main__":
    sys.exit(main())

#!/usr/bin/env python3
"""ENGINE-UNIVERSAL-SERVICES.67(b) LEG 4 — score the allocator-side census.

Reads `alloc_census.tsv` (produced by `measure_alloc_census.sh`) and prints the
ALLOCATION-SIDE attribution plus the CROSS-QUANTITY comparison against peak RSS,
keeping the two kinds of quantity apart by construction rather than by care:

  * allocation side (one kind of quantity throughout, so the split is closed):
        peak_live = ast_allocated + residual
    where `ast_allocated` is the engine's own `arena+AST ... allocated` figure —
    chunk bytes + arena spines + the AST-owned child-`Vec`/`String` heap.

  * cross-quantity (labelled, never silently mixed):
        peak_live  vs  peak_rss - floor
    Leg 3 measured 1 008 MB of allocation costing 32 768 B of RSS, so this ratio
    is a MEASUREMENT of the allocation/residency gap and not a rounding.

⛔ THE EXACTNESS TEST IS PRINTED, NOT ASSUMED. The AST legs are monotone and are
reported at end of parse. `peak_live - ast_allocated` is the non-AST live figure
EXACTLY when the live high-water is reached at the end of the allocating phase,
and a LOWER bound otherwise. `post_peak_alloc_calls` decides which, per row.

Usage: alloc_census_shares.py <alloc_census.tsv> [classes.tsv]
"""
import sys

# `generated/systemverilog_parser.rs:3291`. Kept here as a NAMED constant rather
# than a literal in the arithmetic, and re-derivable by
# `docs/tasks/artifacts/eus67/thin_scratch_census.sh`, which reports it for every
# generated family (the property is engine-universal, not a SystemVerilog one).
THIN_RULE_COUNT = 109

FLOOR_LOW = 14.72 * 1024 * 1024   # .67(a): lowest peak over any real file
FLOOR_HIGH = 18.86 * 1024 * 1024  # .67(a): highest of eight repeats on a 20-byte module
MB = 1024.0 * 1024.0


def num(s):
    if s in ("NA", "", None):
        return None
    try:
        return float(s)
    except ValueError:
        return None


def rows(path):
    with open(path) as fh:
        head = fh.readline().rstrip("\n").split("\t")
        for line in fh:
            if not line.strip():
                continue
            yield dict(zip(head, line.rstrip("\n").split("\t")))


def main():
    if len(sys.argv) < 2:
        print(__doc__)
        return 2
    data = list(rows(sys.argv[1]))
    if not data:
        print("no rows", file=sys.stderr)
        return 2

    print("=== ALLOCATION-SIDE ATTRIBUTION (one kind of quantity throughout) ===")
    hdr = ("input", "verdict", "peak_live MB", "ast_alloc MB", "residual MB",
           "residual/peak_live", "exact?", "churn", "free/alloc")
    print("\t".join(hdr))
    resid_shares, cross, churn_ok, freeratio_ok = [], [], [], []
    exactness = []
    for r in data:
        pl = num(r["peak_live"])
        ast = num(r["arena_ast_allocated_MB"])
        ab = num(r["alloc_bytes"])
        fb = num(r["free_bytes"])
        ppc = num(r["post_peak_alloc_calls"])
        ac = num(r["alloc_calls"])
        if pl is None or ast is None:
            print(f"{r['path']}\t{r['verdict']}\tNA\tNA\tNA\tNA\tNA\tNA\tNA")
            continue
        ast_b = ast * MB
        resid = pl - ast_b
        share = resid / pl if pl else None
        # ⛔ The exactness test: a peak reached at the END of the allocating phase
        # means the AST was already at its reported size when the peak was taken.
        exact = None
        if ppc is not None and ac:
            exact = ppc / ac
            exactness.append((r["path"], exact, ppc, ac))
        churn = ab / pl if (ab and pl) else None
        fr = fb / ab if (ab and fb) else None
        if share is not None:
            resid_shares.append((r["path"], share))
        if churn is not None:
            churn_ok.append((r["path"], churn))
        if fr is not None:
            freeratio_ok.append((r["path"], fr))
        print("\t".join([
            r["path"].split("/")[-1], r["verdict"],
            f"{pl / MB:.2f}", f"{ast:.2f}", f"{resid / MB:.2f}",
            f"{share:.4f}" if share is not None else "NA",
            f"{exact:.2e}" if exact is not None else "NA",
            f"{churn:.2f}" if churn is not None else "NA",
            f"{fr:.4f}" if fr is not None else "NA",
        ]))

    print()
    print("=== CROSS-QUANTITY: peak_live (ALLOCATION) vs peak_rss - floor (RESIDENCY) ===")
    print("input\tpeak_rss MB\trss-floor_low MB\tpeak_live/(rss-floor_low)\tpeak_live/(rss-floor_high)")
    for r in data:
        pl = num(r["peak_live"])
        rss = num(r["peak_rss_bytes"])
        if pl is None or rss is None:
            print(f"{r['path'].split('/')[-1]}\tNA\tNA\tNA\tNA")
            continue
        lo = rss - FLOOR_LOW
        hi = rss - FLOOR_HIGH
        rlo = pl / lo if lo > 0 else None
        rhi = pl / hi if hi > 0 else None
        if rlo is not None:
            cross.append((r["path"], rlo))
        print("\t".join([
            r["path"].split("/")[-1], f"{rss / MB:.2f}", f"{lo / MB:.2f}",
            f"{rlo:.4f}" if rlo is not None else "NA",
            f"{rhi:.4f}" if rhi is not None else "NA",
        ]))

    def band(pairs):
        if not pairs:
            return "n/a"
        vals = sorted(v for _, v in pairs)
        n = len(vals)
        med = vals[n // 2] if n % 2 else (vals[n // 2 - 1] + vals[n // 2]) / 2
        return f"min {vals[0]:.4f} · median {med:.4f} · max {vals[-1]:.4f} (n={n})"

    print()
    print("=== BANDS ===")
    print(f"residual share of peak_live : {band(resid_shares)}")
    print(f"peak_live/(rss-floor_low)   : {band(cross)}")
    print(f"alloc_bytes/peak_live       : {band(churn_ok)}")
    print(f"free_bytes/alloc_bytes      : {band(freeratio_ok)}")
    print()
    print("=== EXACTNESS (post_peak_alloc_calls / alloc_calls; ~0 ⇒ the split is EXACT) ===")
    for path, frac, ppc, ac in exactness:
        verdict = "EXACT" if frac < 1e-4 else ("NEAR" if frac < 1e-2 else "LOWER-BOUND ONLY")
        print(f"  {path.split('/')[-1]:<34} {frac:.3e}  ({int(ppc)} of {int(ac)})  {verdict}")

    if len(sys.argv) > 2:
        per = {}
        for r in rows(sys.argv[2]):
            per.setdefault(r["path"], []).append(r)

        print()
        print("=== THE DENSE THIN-MEMO SCRATCH, PREDICTED FROM SOURCE AND LOOKED FOR BY SIZE ===")
        print("`ThinMemoScratch::begin(THIN_RULE_COUNT * (len + 1))` resizes a Vec<u64>, so the")
        print("request is exactly THIN_RULE_COUNT * (bytes + 1) * 8. Class k holds [2^(k-1), 2^k),")
        print("so a request of B bytes lands in class B.bit_length().")
        print("input\tfile bytes\tpredicted B\tclass\tclass live B\tclass count\tclass/pred"
              "\tclass share of snap")
        for r in data:
            p = r["path"]
            n = int(r["bytes"]) if r["bytes"].isdigit() else None
            if n is None:
                continue
            pred = THIN_RULE_COUNT * (n + 1) * 8
            k = pred.bit_length()
            hit = [c for c in per.get(p, []) if c["class"].strip() == str(k)]
            if not hit:
                print(f"{p.split('/')[-1]}\t{n}\t{pred}\t{k}\tNOT PRESENT\tNA\tNA\tNA")
                continue
            lb = num(hit[0]["live_bytes"])
            lc = hit[0]["live_count"]
            sh = hit[0]["share"]
            # ⛔ `class/pred` == 1.00 means the class holds the thin scratch AND NOTHING
            # ELSE, so the identification is exact. A ratio above 1 means other
            # allocations share the class and the scratch cannot be isolated from this
            # table alone — reported, never rounded away.
            ratio = (lb / pred) if (lb and pred) else None
            print(f"{p.split('/')[-1]}\t{n}\t{pred}\t{k}\t{int(lb) if lb else 'NA'}\t{lc}"
                  f"\t{ratio:.4f}\t{sh}" if ratio is not None
                  else f"{p.split('/')[-1]}\t{n}\t{pred}\t{k}\t{lb}\t{lc}\tNA\t{sh}")

        print()
        print("=== SIZE CLASSES AT THE NEAR-PEAK SNAPSHOT — classes holding >= 1% of live ===")
        for path, cls in per.items():
            print(f"-- {path}")
            big = [c for c in cls if num(c["share"]) and num(c["share"]) >= 0.01]
            big.sort(key=lambda c: -(num(c["share"]) or 0))
            for c in big:
                lb = num(c["live_bytes"])
                print(f"     class={c['class']:>2} [{c['lo']},{c['hi']})"
                      f" live={(lb / MB if lb else 0):9.2f} MB"
                      f" count={c['live_count']:>12} share={c['share']}")
    return 0


if __name__ == "__main__":
    sys.exit(main())

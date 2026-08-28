#!/usr/bin/env python3
"""ENGINE-UNIVERSAL-SERVICES.67(b) — turn the ladder TSV into the shares the leaf publishes.

Reads the TSV `measure_parse_footprint.sh` writes and reports, per input, what fraction of
the FLOOR-SUBTRACTED peak RSS is (i) ParseNode headers + the three side arenas, (ii) the
AST-owned heap outside the arena, (iii) the two together.

⛔ The floor is a BAND, not a constant (`.67`(a): 14.72 MB lowest real peak .. 18.86 MB
highest of eight repeats). Every share is therefore printed at BOTH ends, and a conclusion
that only survives at one end is not a conclusion.
"""
import sys

FLOOR_LO = 14.72 * 1024 * 1024   # MB -> bytes
FLOOR_HI = 18.86 * 1024 * 1024
MB = 1024.0 * 1024.0


def num(cell):
    return None if cell in ("NA", "") else float(cell)


def main(path):
    rows = [l.rstrip("\n").split("\t") for l in open(path)]
    head, body = rows[0], rows[1:]
    idx = {name: i for i, name in enumerate(head)}
    print(f"{'input':<34}{'KB':>8}{'peak MB':>10}{'arena':>8}{'owned':>8}"
          f"{'AST%lo':>9}{'AST%hi':>9}{'own/arena':>11}{'slots/vec':>11}")
    shares_lo, shares_hi, excluded = [], [], []
    for r in body:
        rss = num(r[idx["peak_rss_bytes"]])
        arena = num(r[idx["arena_total_MB"]])
        owned = num(r[idx["owned_MB"]])
        if rss is None or arena is None or owned is None:
            print(f"{r[0].split('/')[-1]:<34}  NOT MEASURED (a missing field is not a zero)")
            continue
        name = r[0].split("/")[-1]
        kb = float(r[idx["bytes"]]) / 1024.0
        ast = arena + owned
        # Floor-subtracted marginal footprint, at both ends of the floor band.
        lo = (rss - FLOOR_HI) / MB     # smallest marginal => LARGEST share
        hi = (rss - FLOOR_LO) / MB
        # ⛔ ADMISSIBILITY BY A NAMED RULE, AND THE EXCLUSION IS COUNTED (`.67`(a)): a peak
        # under 2x the floor band's top cannot be separated from the floor's own ~19 %
        # spread, and silently keeping such a row is what biased (a)'s first pass.
        if rss < 2 * FLOOR_HI:
            excluded.append(name)
            print(f"{name:<34}{kb:>8.0f}{rss/MB:>10.0f}   EXCLUDED — peak < 2x the floor band")
            continue
        s_hi, s_lo = 100.0 * ast / lo if lo > 0 else float('nan'), 100.0 * ast / hi
        vecs = num(r[idx["child_vecs"]]) or 0.0
        slots = num(r[idx["child_slots"]]) or 0.0
        print(f"{name:<34}{kb:>8.0f}{rss/MB:>10.0f}{arena:>8.1f}{owned:>8.1f}"
              f"{s_lo:>9.1f}{s_hi:>9.1f}{owned/arena if arena else 0:>11.2f}"
              f"{(slots/vecs if vecs else 0):>11.2f}")
        shares_lo.append(s_lo); shares_hi.append(s_hi)
    if shares_lo:
        shares_lo.sort(); shares_hi.sort()
        mid = len(shares_lo) // 2
        print(f"\nadmitted {len(shares_lo)} row(s); excluded {len(excluded)}: "
              f"{', '.join(excluded) if excluded else 'none'}")
        print(f"AST share of the floor-subtracted peak: "
              f"{min(shares_lo):.1f}%-{max(shares_lo):.1f}% at the 14.72 MB floor, "
              f"{min(shares_hi):.1f}%-{max(shares_hi):.1f}% at 18.86 MB; "
              f"median {shares_lo[mid]:.1f}% / {shares_hi[mid]:.1f}%")


if __name__ == "__main__":
    main(sys.argv[1] if len(sys.argv) > 1 else "docs/tasks/artifacts/eus67/attribution_owned.tsv")

#!/usr/bin/env python3
"""D2-A speed-gate analyzer: geomean-of-mins over alternated rounds.

Input: round files  round<N>_<side>.txt  (side = base|cand), the perf probe's
stdout table. Extracts per-pattern MIN ns; per side takes the best (min) across
rounds per pattern; reports geomean(cand)/geomean(base) plus per-round geomeans.
"""
import glob
import math
import re
import sys

PAT = re.compile(r"^(\w+)\s+(\d+)\s+(\d+)\s+(\d+)\s+(\d+)\s+(\d+)\s+(\d+)\s*$")


def parse(path):
    mins = {}
    for line in open(path):
        m = PAT.match(line.strip())
        if m:
            mins[m.group(1)] = int(m.group(2))
    return mins


def geomean(vals):
    return math.exp(sum(math.log(v) for v in vals) / len(vals))


def side_rounds(side):
    return sorted(glob.glob(f"round*_{side}.txt"))


def main():
    rounds = {}
    for side in ("base", "cand"):
        for f in side_rounds(side):
            n = re.search(r"round(\d+)_", f).group(1)
            rounds.setdefault(n, {})[side] = parse(f)

    patterns = None
    best = {"base": {}, "cand": {}}
    print("per-round geomeans (ns):")
    for n in sorted(rounds, key=int):
        row = rounds[n]
        if "base" not in row or "cand" not in row:
            continue
        if patterns is None:
            patterns = sorted(row["base"])
        gb = geomean([row["base"][p] for p in patterns])
        gc = geomean([row["cand"][p] for p in patterns])
        print(f"  round {n}: base {gb:9.1f}  cand {gc:9.1f}  ratio {gc / gb:.3f}")
        for side in ("base", "cand"):
            for p, v in row[side].items():
                if p not in best[side] or v < best[side][p]:
                    best[side][p] = v

    gb = geomean([best["base"][p] for p in patterns])
    gc = geomean([best["cand"][p] for p in patterns])
    print("\nbest-mins per pattern (base -> cand, ratio):")
    for p in patterns:
        b, c = best["base"][p], best["cand"][p]
        print(f"  {p:<18} {b:>9} -> {c:>9}  {c / b:.3f}")
    print(f"\nGEOMEAN-OF-MINS: base {gb:.1f} ns -> cand {gc:.1f} ns  ratio {gc / gb:.4f}  ({(gc / gb - 1) * 100:+.1f}%)")


if __name__ == "__main__":
    main()

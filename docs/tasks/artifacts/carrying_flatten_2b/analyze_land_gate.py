#!/usr/bin/env python3
"""RGX-0078.5.j.2 STEP-2b land-gate analysis — geomean-of-best-mins, bar -2.0%."""
import glob
import math
import os
import re
import sys

OUT = os.path.dirname(os.path.abspath(__file__))
ROW = re.compile(r"^([a-z_]+)\s+(\d+)\s+\d+\s+\d+\s+\d+\s+\d+\s+\d+\s*$")


def parse_round(path):
    mins = {}
    with open(path) as f:
        for line in f:
            m = ROW.match(line)
            if m:
                mins[m.group(1)] = int(m.group(2))
    if not mins:
        sys.exit(f"no pattern rows parsed from {path}")
    return mins


def geomean(vals):
    return math.exp(sum(math.log(v) for v in vals) / len(vals))


base_rounds = sorted(glob.glob(os.path.join(OUT, "round*_base.txt")))
cand_rounds = sorted(glob.glob(os.path.join(OUT, "round*_cand.txt")))
assert len(base_rounds) == len(cand_rounds) == 5, (base_rounds, cand_rounds)

base_by_round = [parse_round(p) for p in base_rounds]
cand_by_round = [parse_round(p) for p in cand_rounds]
patterns = sorted(base_by_round[0])
assert all(sorted(r) == patterns for r in base_by_round + cand_by_round)

print("per-round geomean-of-mins (base -> cand, ratio):")
for i, (b, c) in enumerate(zip(base_by_round, cand_by_round), 1):
    gb = geomean([b[p] for p in patterns])
    gc = geomean([c[p] for p in patterns])
    print(
        f"  round {i}: {gb:9.1f} -> {gc:9.1f}  ratio={gc / gb:.4f}  ({(gc / gb - 1) * 100:+.2f}%)"
    )

best_base = {p: min(r[p] for r in base_by_round) for p in patterns}
best_cand = {p: min(r[p] for r in cand_by_round) for p in patterns}
print("\nper-pattern best-min (base -> cand, ratio):")
for p in patterns:
    ratio = best_cand[p] / best_base[p]
    print(
        f"  {p:<24} {best_base[p]:>6} -> {best_cand[p]:>8}  ratio={ratio:.4f}  ({(ratio - 1) * 100:+.2f}%)"
    )

gb = geomean(list(best_base.values()))
gc = geomean(list(best_cand.values()))
ratio = gc / gb
print(f"\n=== GEOMEAN-OF-BEST-MINS: base {gb:.1f}ns -> cand {gc:.1f}ns ===")
print(f"=== RATIO cand/base = {ratio:.4f}  =>  {(ratio - 1) * 100:+.2f}% ===")
verdict = "LAND (faster)" if ratio < 0.98 else "REFUSE (bar -2.0% not cleared)"
print(f"=== falsification bar -2.0% (ratio<0.98): {verdict} ===")

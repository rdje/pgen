#!/usr/bin/env python3
"""G1-A (`PGEN-RGX-0078-0166`) A/B land-gate analyzer — the C1/-0160 protocol verbatim.

Inputs (this directory):
  floorval_r{1,2,3}.txt              base best-of-3 floor validation (±6% vs banked)
  round{1..5}_{base,cand}.txt        alternated 5×2000 bench rounds
  corpus_{base,cand}.jsonl           full-corpus outputs (2,189 cells)
  ladder_{base,cand}.jsonl           39-cell ladder outputs

Acceptance (the -0158 design, stated pre-build):
  PRIMARY corpus geomean -1...-4% off 1,263.4 ns; falsification better than -0.5%
  (then the M1 precedent: land as proven simplification, perf claim REFUSED).
"""
import json
import math
import re
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
BANKED_FLOOR_GEOMEAN = 1937.4  # ns, the -0160 C1 banked bench floor (1d3fa0ee)

ROW = re.compile(r"^(\w+)\s+(\d+)\s+\d+\s+\d+\s+\d+\s+\d+\s+\d+\s*$")


def bench_mins(path):
    mins = {}
    for line in path.read_text().splitlines():
        m = ROW.match(line)
        if m:
            mins[m.group(1)] = int(m.group(2))
    if len(mins) != 8:
        sys.exit(f"{path}: expected 8 pattern rows, got {len(mins)}")
    return mins


def geomean(values):
    return math.exp(sum(math.log(v) for v in values) / len(values))


def best_mins(paths):
    rounds = [bench_mins(p) for p in paths]
    return {k: min(r[k] for r in rounds) for k in rounds[0]}


def load_cells(path):
    cells = {}
    for line in path.read_text().splitlines():
        row = json.loads(line)
        cells[row["id"]] = row
    return cells


def main():
    # Floor validation: best-of-3 geomean of mins vs the banked floor, ±6%.
    fv = best_mins([HERE / f"floorval_r{r}.txt" for r in (1, 2, 3)])
    fv_geo = geomean(fv.values())
    fv_delta = (fv_geo / BANKED_FLOOR_GEOMEAN - 1) * 100
    print(f"== FLOORVAL (base best-of-3) == geomean {fv_geo:,.1f} ns vs banked "
          f"{BANKED_FLOOR_GEOMEAN:,.1f} = {fv_delta:+.2f}% (gate ±6%)"
          f" -> {'OK' if abs(fv_delta) <= 6 else 'FAIL — STOP'}")

    base = best_mins([HERE / f"round{r}_base.txt" for r in (1, 2, 3, 4, 5)])
    cand = best_mins([HERE / f"round{r}_cand.txt" for r in (1, 2, 3, 4, 5)])
    print("\n== BENCH (alternated 5x2000, best-min per pattern) ==")
    for k in sorted(base):
        d = (cand[k] / base[k] - 1) * 100
        print(f"  {k:24s} {base[k]:>8,} -> {cand[k]:>8,} ns  {d:+.2f}%")
    bg, cg = geomean(base.values()), geomean(cand.values())
    bench_delta = (cg / bg - 1) * 100
    print(f"  GEOMEAN {bg:,.1f} -> {cg:,.1f} ns  {bench_delta:+.2f}%   "
          f"(~{496000 / cg:.1f}x vs 496us)  guard ±2%: "
          f"{'OK' if bench_delta <= 2.0 else 'FAIL'}")

    for name in ("corpus", "ladder"):
        b = load_cells(HERE / f"{name}_base.jsonl")
        c = load_cells(HERE / f"{name}_cand.jsonl")
        assert set(b) == set(c), f"{name}: cell-set mismatch"
        flips = [k for k in b if b[k]["actual_parse"] != c[k]["actual_parse"]]
        bmax = max(b.values(), key=lambda r: r["min_ns"])
        cmax = max(c.values(), key=lambda r: r["min_ns"])
        geo_b = geomean([r["min_ns"] for r in b.values()])
        geo_c = geomean([r["min_ns"] for r in c.values()])
        print(f"\n== {name.upper()} ({len(b)} cells) == verdict flips: {len(flips)} {flips[:5]}")
        print(f"  MAX  {bmax['min_ns']:,} ({bmax['id']}) -> {cmax['min_ns']:,} ({cmax['id']})  "
              f"{(cmax['min_ns'] / bmax['min_ns'] - 1) * 100:+.2f}%")
        print(f"  GEOMEAN {geo_b:,.1f} -> {geo_c:,.1f} ns  {(geo_c / geo_b - 1) * 100:+.2f}%")
        watch = ["pcre2:testdata/testinput2:line_725", "pcre2:testdata/testinput2:line_4674",
                 "pcre2:testdata/testinput2:line_6526", "pcre2:testdata/testinput2:line_2880",
                 "pcre2:testdata/testinput2:line_965", "pcre2:testdata/testinput2:line_6538",
                 "nest_80", "nest_40", "orig_4674", "flat_160"]
        for k in watch:
            if k in b:
                d = (c[k]["min_ns"] / b[k]["min_ns"] - 1) * 100
                print(f"    {k:48s} {b[k]['min_ns']:>10,} -> {c[k]['min_ns']:>10,}  {d:+.2f}%")
        if name == "corpus":
            bands = [(0, 10), (11, 30), (31, 60), (61, 120), (121, 200),
                     (201, 400), (401, 1000), (1001, 4000)]
            print("  byte bands (geomean delta, n):")
            for lo, hi in bands:
                ks = [k for k in b if lo <= b[k]["pattern_bytes"] <= hi]
                if not ks:
                    continue
                gb = geomean([b[k]["min_ns"] for k in ks])
                gc = geomean([c[k]["min_ns"] for k in ks])
                print(f"    {lo:>5}-{hi:<5}B n={len(ks):<5} {(gc / gb - 1) * 100:+.2f}%")


if __name__ == "__main__":
    main()

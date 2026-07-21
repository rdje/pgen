#!/usr/bin/env python3
"""Adjudicate the pre-registered PGEN-RGX-0078-0214 land gate.

Binding acceptance (the `-0197` ratchet under the `-0212` hardened
interleaved protocol): the candidate lands only when

- CUSTODY: each base sweep geomean is within +-3% of the floor-of-record
  1004.3665721515218 ns AND base1 vs base2 agree within +-2.5%
  (breach => the sweep set is invalid for adjudication — rerun, do not land);
- the POOLED (per-cell min across each side's two sweeps) candidate corpus
  geomean is numerically STRICTLY lower than the pooled base geomean
  (unrounded);
- verdicts are identical on all cells across all four sweeps;
- the pooled candidate corpus MAX stays <= the settled 425,000 ns bound.

Bench rounds are steering/reporting only; the bench floorval band (+-6%
vs the banked 1579.7 ns) is a custody tripwire.
"""

import json
import math
import re
from pathlib import Path

HERE = Path(__file__).resolve().parent
FLOOR_OF_RECORD_NS = 1004.3665721515218
BANKED_BENCH_NS = 1579.7
SETTLED_CORPUS_MAX_NS = 425_000
ROW = re.compile(r"^(\w+)\s+(\d+)\s+\d+\s+\d+\s+\d+\s+\d+\s+\d+\s*$")


def geomean(values):
    values = list(values)
    return math.exp(sum(math.log(v) for v in values) / len(values))


def bench_mins(path):
    rows = {}
    for line in path.read_text().splitlines():
        m = ROW.match(line)
        if m:
            rows[m.group(1)] = int(m.group(2))
    if len(rows) != 8:
        raise SystemExit(f"{path}: expected 8 benchmark rows, got {len(rows)}")
    return rows


def load_corpus(tag):
    rows = {}
    for line in (HERE / f"corpus_{tag}.jsonl").read_text().splitlines():
        row = json.loads(line)
        rows[row["id"]] = row
    if len(rows) != 2189:
        raise SystemExit(f"corpus_{tag}: {len(rows)} cells != 2189")
    return rows


floor_rounds = [bench_mins(HERE / f"floorval_r{r}.txt") for r in (1, 2, 3)]
floor_bench = {n: min(run[n] for run in floor_rounds) for n in floor_rounds[0]}
floor_bench_ns = geomean(floor_bench.values())
print(f"bench floorval geomean {floor_bench_ns:.1f} ns vs banked {BANKED_BENCH_NS} "
      f"= {100 * (floor_bench_ns / BANKED_BENCH_NS - 1):+.2f}%  "
      f"{'PASS' if abs(floor_bench_ns / BANKED_BENCH_NS - 1) <= 0.06 else 'CUSTODY BREACH'} (+-6%)")

base_rounds = [bench_mins(HERE / f"round{r}_base.txt") for r in range(1, 6)]
cand_rounds = [bench_mins(HERE / f"round{r}_candidate.txt") for r in range(1, 6)]
base_bench = geomean(min(run[n] for run in base_rounds) for n in base_rounds[0])
cand_bench = geomean(min(run[n] for run in cand_rounds) for n in cand_rounds[0])
print(f"bench steering: base {base_bench:.1f} -> cand {cand_bench:.1f} ns "
      f"= {100 * (cand_bench / base_bench - 1):+.2f}%")

sweeps = {tag: load_corpus(tag) for tag in ("base1", "cand1", "base2", "cand2")}

verdicts_ok = True
ids = set(sweeps["base1"])
for tag, rows in sweeps.items():
    if set(rows) != ids:
        raise SystemExit(f"{tag}: cell set differs")
for case_id in ids:
    verdict = {rows[case_id]["actual_parse"] for rows in sweeps.values()}
    if len(verdict) != 1:
        verdicts_ok = False
        print(f"VERDICT FLIP at {case_id}: "
              + ", ".join(f"{t}={r[case_id]['actual_parse']}" for t, r in sweeps.items()))
print(f"verdict flips across all four sweeps: {'0/2189  PASS' if verdicts_ok else 'PRESENT — REFUSE'}")

g = {tag: geomean(r["min_ns"] for r in rows.values()) for tag, rows in sweeps.items()}
for tag in ("base1", "base2"):
    delta = g[tag] / FLOOR_OF_RECORD_NS - 1
    print(f"custody {tag} {g[tag]!r} ns ({100 * delta:+.2f}% vs floor {FLOOR_OF_RECORD_NS!r})  "
          f"{'PASS' if abs(delta) <= 0.03 else 'BREACH'} (+-3%)")
bb = g["base1"] / g["base2"] - 1
print(f"custody base1 vs base2 {100 * bb:+.2f}%  {'PASS' if abs(bb) <= 0.025 else 'BREACH'} (+-2.5%)")
custody_ok = (abs(g["base1"] / FLOOR_OF_RECORD_NS - 1) <= 0.03
              and abs(g["base2"] / FLOOR_OF_RECORD_NS - 1) <= 0.03
              and abs(bb) <= 0.025)

pooled_base = {i: min(sweeps["base1"][i]["min_ns"], sweeps["base2"][i]["min_ns"]) for i in ids}
pooled_cand = {i: min(sweeps["cand1"][i]["min_ns"], sweeps["cand2"][i]["min_ns"]) for i in ids}
pb = geomean(pooled_base.values())
pc = geomean(pooled_cand.values())
ratio = pc / pb
print(f"pooled base {pb!r} -> pooled cand {pc!r} ns = {100 * (ratio - 1):+.4f}%  "
      f"{'STRICT DECREASE: PASS' if pc < pb else 'NO DECREASE: REFUSE'}")
pmax_b = max(pooled_base.values())
pmax_c = max(pooled_cand.values())
print(f"MAX pooled base {pmax_b:,} / pooled cand {pmax_c:,} <= {SETTLED_CORPUS_MAX_NS:,}  "
      f"{'PASS' if pmax_c <= SETTLED_CORPUS_MAX_NS else 'REFUSE'}")

BANDS = (("<1us", 0, 1_000), ("1-2.5us", 1_000, 2_500),
         ("2.5-20us", 2_500, 20_000), (">=20us", 20_000, 10**12))
parts = []
for name, lo, hi in BANDS:
    cells = [i for i in ids if lo <= pooled_base[i] < hi]
    if cells:
        r = geomean(pooled_cand[i] for i in cells) / geomean(pooled_base[i] for i in cells)
        parts.append(f"{name} {r:.4f} (n={len(cells)})")
print("per-band pooled ratios: " + " | ".join(parts))

for tag in ("cand1", "cand2"):
    print(f"single-sweep geomean {tag}: {g[tag]!r}")
single_floor = math.sqrt(g["cand1"] * g["cand2"])
print(f"single-sweep-comparable candidate floor (geomean of sweep geomeans): {single_floor!r}")

land = custody_ok and verdicts_ok and pc < pb and pmax_c <= SETTLED_CORPUS_MAX_NS
print(f"\nVERDICT: {'LAND' if land else 'REFUSE (revert per the -0197 ratchet)' }")

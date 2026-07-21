#!/usr/bin/env python3
"""PGEN-RGX-0078-0210 per-band decomposition of the paired corpus sweeps.

Reproduces the `-0209` band_analysis.txt method on this unit's banked
JSONLs: bands are cut over the BASE sweep's own timing population, the
per-band statistic is the geomean of per-cell `min_ns`, and the worst
per-cell ratios are listed so a flat-add (or its absence) is visible
directly. Run after run_ab.sh; output is the banked band_analysis.txt.
"""

import json
import math
from pathlib import Path

HERE = Path(__file__).resolve().parent


def load(path):
    rows = {}
    for line in path.read_text().splitlines():
        row = json.loads(line)
        rows[row["id"]] = row
    return rows


def geomean(values):
    values = list(values)
    return math.exp(sum(math.log(v) for v in values) / len(values))


base = load(HERE / "corpus_base.jsonl")
cand = load(HERE / "corpus_candidate.jsonl")
assert base.keys() == cand.keys(), "paired sweeps must cover identical cells"

BANDS = [
    ("sub1us", 0, 1_000),
    ("1to2p5", 1_000, 2_500),
    ("2p5to20", 2_500, 20_000),
    ("ge20us", 20_000, float("inf")),
]

lines = [
    "PGEN-RGX-0078-0210 per-band decomposition (banked either way per the prereg)",
    "bands over the BASE sweep's own timing population; geomean per band:",
]
for name, lo, hi in BANDS:
    ids = [i for i, row in base.items() if lo <= row["min_ns"] < hi]
    if not ids:
        lines.append(f"  {name:8} cells=    0")
        continue
    b = geomean(base[i]["min_ns"] for i in ids)
    c = geomean(cand[i]["min_ns"] for i in ids)
    lines.append(
        f"  {name:8} cells={len(ids):5} base={b:10.1f} cand={c:10.1f} "
        f"delta={100 * (c / b - 1):+7.2f}%"
    )

ratios = sorted(
    base, key=lambda i: cand[i]["min_ns"] / base[i]["min_ns"], reverse=True
)
lines.append("worst per-cell ratios:")
for i in ratios[:8]:
    lines.append(
        f"  {i:60} {base[i]['min_ns']:7} -> {cand[i]['min_ns']:7} "
        f"{cand[i]['min_ns'] / base[i]['min_ns']:.2f}x"
    )
lines.append("best per-cell ratios:")
for i in ratios[-8:]:
    lines.append(
        f"  {i:60} {base[i]['min_ns']:7} -> {cand[i]['min_ns']:7} "
        f"{cand[i]['min_ns'] / base[i]['min_ns']:.2f}x"
    )

out = "\n".join(lines) + "\n"
(HERE / "band_analysis.txt").write_text(out)
print(out)

#!/usr/bin/env python3
"""Adjudicate the pre-registered PGEN-RGX-0078-0212 land gate.

Binding acceptance (the `-0197` ratchet, which supersedes the `-0176`-era
bench land bound): the candidate lands only when its UNROUNDED canonical
PCRE2 external-corpus geomean is numerically and strictly lower than the
same-session immediate-base geomean, verdicts are identical on all cells,
and the candidate corpus MAX stays <= the re-settled 425,000 ns bound (director #178). Bench
rounds are steering/reporting only. The floor-validation sanity band (+-6%
against the banked bench floor) is a custody check: a breach invalidates the
sweep for rerun, it does not adjudicate.
"""

import json
import math
import re
from pathlib import Path


HERE = Path(__file__).resolve().parent
BANKED_BENCH_NS = 1619.1
SETTLED_CORPUS_MAX_NS = 425_000
ROW = re.compile(r"^(\w+)\s+(\d+)\s+\d+\s+\d+\s+\d+\s+\d+\s+\d+\s*$")


def geomean(values):
    values = list(values)
    return math.exp(sum(math.log(value) for value in values) / len(values))


def bench_mins(path):
    rows = {}
    for line in path.read_text().splitlines():
        match = ROW.match(line)
        if match:
            rows[match.group(1)] = int(match.group(2))
    if len(rows) != 8:
        raise SystemExit(f"{path}: expected 8 benchmark rows, got {len(rows)}")
    return rows


def best_mins(stem):
    rounds = [bench_mins(HERE / f"round{round}_{stem}.txt") for round in range(1, 6)]
    return {name: min(run[name] for run in rounds) for name in rounds[0]}


def load_corpus(path):
    rows = {}
    for line in path.read_text().splitlines():
        row = json.loads(line)
        rows[row["id"]] = row
    return rows


floor_rounds = [bench_mins(HERE / f"floorval_r{round}.txt") for round in range(1, 4)]
floor = {name: min(run[name] for run in floor_rounds) for name in floor_rounds[0]}
floor_ns = geomean(floor.values())
floor_ok = abs(floor_ns / BANKED_BENCH_NS - 1) <= 0.06

base = best_mins("base")
candidate = best_mins("candidate")
base_ns = geomean(base.values())
candidate_ns = geomean(candidate.values())
bench_ratio = candidate_ns / base_ns

base_corpus = load_corpus(HERE / "corpus_base.jsonl")
candidate_corpus = load_corpus(HERE / "corpus_candidate.jsonl")
if set(base_corpus) != set(candidate_corpus):
    raise SystemExit("corpus cell sets differ")
flips = [
    case_id
    for case_id in base_corpus
    if base_corpus[case_id]["actual_parse"] != candidate_corpus[case_id]["actual_parse"]
]
base_corpus_ns = geomean(row["min_ns"] for row in base_corpus.values())
candidate_corpus_ns = geomean(row["min_ns"] for row in candidate_corpus.values())
base_max = max(base_corpus.values(), key=lambda row: row["min_ns"])
candidate_max = max(candidate_corpus.values(), key=lambda row: row["min_ns"])

print(
    f"FLOORVAL bench geomean {floor_ns:.1f} ns vs banked {BANKED_BENCH_NS:.1f} ns "
    f"({floor_ns / BANKED_BENCH_NS - 1:+.2%}); custody {'OK' if floor_ok else 'BREACH'}"
)
print("BENCH (steering only) alternated 5x2000, geomean of per-pattern best minima")
for name in sorted(base):
    print(
        f"  {name:24s} {base[name]:8d} -> {candidate[name]:8d} ns "
        f"({candidate[name] / base[name] - 1:+.2%})"
    )
print(
    f"  GEOMEAN {base_ns:.1f} -> {candidate_ns:.1f} ns "
    f"({bench_ratio - 1:+.2%}); ratio={bench_ratio:.6f}"
)
print(f"CORPUS cells={len(base_corpus)} verdict_flips={len(flips)}")
print(
    f"  GEOMEAN(unrounded) {base_corpus_ns!r} -> {candidate_corpus_ns!r} ns "
    f"({candidate_corpus_ns / base_corpus_ns - 1:+.4%}); "
    f"strict_decrease: {'PASS' if candidate_corpus_ns < base_corpus_ns else 'FAIL'}"
)
print(
    f"  BASE_MAX {base_max['min_ns']} ns {base_max['id']}\n"
    f"  CANDIDATE_MAX {candidate_max['min_ns']} ns {candidate_max['id']}; "
    f"settled_bound<={SETTLED_CORPUS_MAX_NS}: "
    f"{'PASS' if candidate_max['min_ns'] <= SETTLED_CORPUS_MAX_NS else 'FAIL'}"
)

if not floor_ok:
    print("VERDICT: INVALID-CUSTODY (floor validation breach — rerun the sweep)")
    raise SystemExit(3)

passed = (
    candidate_corpus_ns < base_corpus_ns
    and not flips
    and candidate_max["min_ns"] <= SETTLED_CORPUS_MAX_NS
)
print(f"VERDICT: {'LAND' if passed else 'REVERT'}")

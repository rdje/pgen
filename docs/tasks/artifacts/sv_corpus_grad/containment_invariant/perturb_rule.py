#!/usr/bin/env python3
"""Perturbation + target-selection helper for the `contained_in_introduced_subgraph` probe.

⛔ EVERY TARGET IS DERIVED, NEVER TYPED. A probe that hard-codes "perturb `non_keyword_identifier`,
it is inside the sub-graph" is asserting the very reachability fact the predicate under test
computes — so the arm would pass whenever the predicate and the probe were wrong the SAME way. The
rules to perturb are chosen HERE, from the measured grammar's own graph and the run's own
`rule_costs.tsv`, and printed so the arm's log says which rule it used.

⚠️ It edits the BASELINE, never the fresh measurement. The gate re-measures this tree on every arm,
so the only way to synthesise a rise is to lower what the baseline claims — the same technique as
`docs/tasks/artifacts/engine_universal_services/accepted_rise_gate/perturb.py`, one level down.

  --pick {inside,outside} --introduced R…    largest measured rule in / out of the sub-graph
  --pick-alt-introduced --exclude RULE       a grammar rule whose sub-graph EXCLUDES RULE
  --apply --rule R --delta D --op {lower,raise}      edit rule_costs.tsv in place
EXIT  0 = done · 2 = refused (no qualifying target, so the arm must not pretend it ran)
"""

from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[5]
if not (ROOT / "grammars").is_dir():
    raise SystemExit(f"perturb_rule: not at the repo root (derived {ROOT})")
sys.path.insert(0, str(ROOT / "scripts"))
import parse_cost_containment as PCC  # noqa: E402

HEADER = "\t".join(PCC.RULE_COSTS_HEADER)


def load(graph_path: str, costs_path: str):
    edges = json.loads(Path(graph_path).read_text(encoding="utf-8"))["edges"]
    known = set(edges) | {r for v in edges.values() for r in v}
    return edges, known, PCC.read_rule_costs(costs_path)


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--graph", required=True)
    ap.add_argument("--costs", required=True)
    ap.add_argument("--pick", choices=("inside", "outside"))
    ap.add_argument("--pick-alt-introduced", action="store_true")
    ap.add_argument("--exclude")
    ap.add_argument("--introduced", nargs="+", default=[])
    ap.add_argument("--min-entries", type=int, default=1)
    ap.add_argument("--apply", action="store_true")
    ap.add_argument("--rule")
    ap.add_argument("--delta", type=int, default=0)
    ap.add_argument("--op", choices=("lower", "raise"), default="lower")
    a = ap.parse_args()

    edges, known, costs = load(a.graph, a.costs)

    if a.pick:
        scope = PCC.reachable(edges, a.introduced)
        want_inside = a.pick == "inside"
        cands = [(r, v["entries"]) for r, v in costs.items()
                 if v["entries"] >= a.min_entries
                 and ((PCC.origin_rule(r, known) in scope) == want_inside)]
        if not cands:
            print(f"perturb_rule: no measured rule {a.pick} the sub-graph of "
                  f"{a.introduced} holds >= {a.min_entries} entries — refusing", file=sys.stderr)
            return 2
        # deterministic: the largest, ties broken by name
        print(sorted(cands, key=lambda t: (-t[1], t[0]))[0][0])
        return 0

    if a.pick_alt_introduced:
        if not a.exclude:
            print("perturb_rule: --pick-alt-introduced needs --exclude RULE", file=sys.stderr)
            return 2
        target = PCC.origin_rule(a.exclude, known) or a.exclude
        for rule in sorted(edges):
            if target not in PCC.reachable(edges, [rule]):
                print(rule)
                return 0
        print(f"perturb_rule: every grammar rule reaches {target} — refusing", file=sys.stderr)
        return 2

    if a.apply:
        if not a.rule or a.delta <= 0:
            print("perturb_rule: --apply needs --rule and a positive --delta", file=sys.stderr)
            return 2
        if a.rule not in costs:
            print(f"perturb_rule: `{a.rule}` is not in {a.costs} — refusing", file=sys.stderr)
            return 2
        if a.op == "lower" and costs[a.rule]["entries"] < a.delta:
            print(f"perturb_rule: `{a.rule}` holds {costs[a.rule]['entries']} entries, "
                  f"less than delta {a.delta} — refusing", file=sys.stderr)
            return 2
        costs[a.rule]["entries"] += (-a.delta if a.op == "lower" else a.delta)
        with open(a.costs, "w", encoding="utf-8") as fh:
            fh.write(HEADER + "\n")
            for rule in sorted(costs):
                v = costs[rule]
                fh.write(f"{rule}\t{v['entries']}\t{v['committed']}\t{v['memo_hits']}\n")
        verb = {"lower": "lowered", "raise": "raised"}[a.op]
        print(f"perturb_rule: {verb} `{a.rule}` by {a.delta} in {a.costs}")
        return 0

    ap.print_help()
    return 2


if __name__ == "__main__":
    sys.exit(main())

#!/usr/bin/env python3
"""SV-CORPUS-GRAD.13c.2k (c) / .13c.2w — is a measured rise CONTAINED in the sub-graph the change
introduced? ⭐ THE ARM-FILE ADAPTER over the ONE predicate.

⛔ WHY THIS FILE IS NOW AN ADAPTER AND NOT AN IMPLEMENTATION (`SV-CORPUS-GRAD.13c.2w`).
`PARSE-COST-RATCHET` gained `contained_in_introduced_subgraph` as a CODED invariant — the same
question, asked of the ratchet's own `rule_costs.tsv` instead of an arm's JSON. Two spellings of one
predicate is [[one-metric-name-two-predicates-is-a-contract-defect]], measured in this repository
three days earlier on `unreachable_rules`: one metric name, two implementations, both correct about
different populations, and a contract nobody could adjudicate. ⇒ the predicate, the reference-graph
derivation and the LR-eliminator origin fold all live in `scripts/parse_cost_containment.py`; this
file supplies the ARM shape of its inputs and the arm-specific refusals.

WHAT STAYS HERE, because it is about ARMS rather than about containment:
  * two arms must have measured the SAME sample, profile and file count;
  * the arm's frozen graph must have been derived from the arm's OWN grammar sha
    (⛔ never from `generated/systemverilog.json`, a floating build artifact the arm driver rewrites
    for every arm — the first containment run in this campaign read the `designB` graph while
    analysing the `t_only` arm, caught only by its own timestamp).

THE PREDICATE (implementation: `scripts/parse_cost_containment.py`):

    containment(introduced) :=
        Δcommitted <= 0                        no accepted derivation got dearer
      ∧ no rule's entry count FELL             the change is purely additive
      ∧ every rule whose entries ROSE is reachable, in the GRAMMAR's own reference graph, from one
        of the `introduced` rules              the rise cannot escape its subtree

⛔ WHAT IT DELIBERATELY DOES NOT SAY. Containment says the rise is confined to the construct that
caused it. It does NOT say the rise was unavoidable — `.13c.2k` is the standing proof that a
contained-looking cost can still have a strictly cheaper spelling, and only another ARM can see
that. The `.13c.2k` design-B arm is the standing RED control that proves this can REFUSE.

USAGE  python3 …/containment.py <base.json> <arm.json> --graph <arm.graph.json>
                          --introduced RULE [RULE …] [--top N]
EXIT   0 = contained · 1 = NOT contained (with the escaping rules listed) · 2 = refused
"""

from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[5]
if not (ROOT / "grammars").is_dir():
    raise SystemExit(f"containment: not at the repo root (derived {ROOT}) — fix the parents[] depth")
sys.path.insert(0, str(ROOT / "scripts"))
import parse_cost_containment as PCC  # noqa: E402


def reference_graph(graph_path: Path, arm: dict) -> dict[str, list[str]]:
    """The arm's FROZEN graph, refused unless it was derived from the arm's own grammar."""
    g = json.loads(graph_path.read_text(encoding="utf-8"))
    want = arm.get("arm_grammar_sha256")
    if want is None:
        raise SystemExit(f"containment: arm `{arm['arm']}` carries no derived grammar sha "
                         f"(schema 1?) — re-measure it, refusing")
    if g["grammar_sha256"] != want:
        raise SystemExit(
            f"containment: {graph_path.name} was derived from grammar {g['grammar_sha256'][:16]}… "
            f"but arm `{arm['arm']}` names {want[:16]}… — refusing. "
            f"Re-run arm_graph.py for this arm.")
    return g["edges"]


def as_rule_costs(arm: dict) -> dict[str, dict[str, int]]:
    """An arm file's three per-rule maps in the shared `rule_costs.tsv` shape."""
    names = set(arm["rule_entries"]) | set(arm.get("rule_committed", {})) \
        | set(arm.get("rule_memo_hits", {}))
    return {r: {"entries": arm["rule_entries"].get(r, 0),
                "committed": arm.get("rule_committed", {}).get(r, 0),
                "memo_hits": arm.get("rule_memo_hits", {}).get(r, 0)} for r in names}


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("base")
    ap.add_argument("arm")
    ap.add_argument("--graph", required=True,
                    help="the ARM's frozen reference graph from arm_graph.py")
    ap.add_argument("--introduced", nargs="+", required=True,
                    help="the rule name(s) the change ADDS a reference to")
    ap.add_argument("--top", type=int, default=15)
    a = ap.parse_args()

    base = json.loads(Path(a.base).read_text(encoding="utf-8"))
    arm = json.loads(Path(a.arm).read_text(encoding="utf-8"))
    for k in ("files", "sample_sha256", "profile"):
        if base[k] != arm[k]:
            print(f"containment: arms disagree on {k} — refusing", file=sys.stderr)
            return 2

    edges = reference_graph(Path(a.graph), arm)
    try:
        holds, detail, facts = PCC.containment(
            as_rule_costs(base), as_rule_costs(arm), edges, a.introduced,
            arm["total_committed"] - base["total_committed"])
    except PCC.Refused as exc:
        print(f"containment: REFUSED — {exc}", file=sys.stderr)
        return 2

    be, xe = base["rule_entries"], arm["rule_entries"]
    print(f"CONTAINMENT: {base['arm']} -> {arm['arm']}   introduced={' '.join(a.introduced)}")
    print(f"  sub-graph reachable from the introduced rule(s) : {facts['scope']:>6,} rules")
    print(f"  rules whose entries ROSE                        : {len(facts['rose']):>6,}  "
          f"({sum(facts['rose'].values()):+,})")
    print(f"  rules whose entries FELL                        : {len(facts['fell']):>6,}  "
          f"({sum(facts['fell'].values()):+,})")
    print(f"  Δcommitted                                      : {facts['d_committed']:>+7,}")
    print(f"  risers OUTSIDE that sub-graph                   : {len(facts['escaped']):>6,}  "
          f"({sum(facts['escaped'].values()):+,})")

    if facts["escaped"]:
        print(f"\n  the {min(a.top, len(facts['escaped']))} largest escaping rules:")
        for r, d in sorted(facts["escaped"].items(), key=lambda t: -t[1])[: a.top]:
            print(f"    {r:<50} {be.get(r, 0):>13,} -> {xe.get(r, 0):>13,}  {d:>+12,}")

    print(f"\n  VERDICT: {'CONTAINED' if holds else 'NOT CONTAINED'}")
    print(f"  {detail}")
    return 0 if holds else 1


if __name__ == "__main__":
    sys.exit(main())

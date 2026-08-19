#!/usr/bin/env python3
"""SV-CORPUS-GRAD.13c.2k (c) — is a measured rise CONTAINED in the sub-graph the change introduced?

⛔ WHY THIS QUESTION AND NOT A SMALLER ONE. `PARSE-COST-RATCHET` accepts a rise only under an
INVARIANT that is code in `scripts/check_parse_cost_ratchet.sh` and is re-evaluated on the run's
own numbers. Its two existing invariants are exact arithmetic identities over three TOTALS
(`pure_memo_lookups`: Δentries == Δmemo ∧ Δcommitted == 0; `unmatched_terminal_alternatives`:
Δentries == 2·Δmemo ∧ Δcommitted == 0). Both fail on `.13c.2t` and on `.13c.2k`, and a third
identity over the same three totals would be numerology — three numbers cannot distinguish
"the added alternative speculates inside its own sub-graph" from "the parser now speculates
everywhere". ⇒ the discriminating evidence is PER-RULE, and it is already being collected by
`arm_cost.py`; this script is the predicate over it.

THE PREDICATE, stated so it can FAIL:

    containment(introduced) :=
        Δcommitted <= 0                                        no accepted derivation got dearer
      ∧ no rule's entry count FELL                             the change is purely additive
      ∧ every rule whose entry count ROSE is reachable, in the GRAMMAR's own reference graph,
        from one of the `introduced` rules                     the rise cannot escape its subtree

⭐ The reference graph comes from the ARM'S OWN FROZEN GRAPH (`arm_graph.py` -> `<arm>.graph.json`),
whose recorded `grammar_sha256` must equal the arm measurement's. ⛔ It deliberately does NOT read
`generated/systemverilog.json`: that is a floating build artifact the arm driver rewrites for every
arm, and the first run of this script read the `designB` graph while analysing the `t_only` arm —
the same class of error as `ENGINE-UNIVERSAL-SERVICES.20` slice 4, caught here by its own
timestamp. The graph is derived from the frontend's `raw_ast`, i.e. the population the code
generator consumed, not from a text scan of the grammar.

⛔ WHAT IT DELIBERATELY DOES NOT SAY. Containment says the rise is confined to the construct that
caused it. It does NOT say the rise was unavoidable, and it is not a licence for a change that
speculates across the whole grammar — that is exactly the case it must REFUSE, and the
`.13c.2k` arm is the standing RED control that proves it can.

USAGE  python3 …/containment.py <base.json> <arm.json> --graph <arm.graph.json>
                          --introduced RULE [RULE …] [--top N]
EXIT   0 = contained · 1 = NOT contained (with the escaping rules listed) · 2 = refused
"""

from __future__ import annotations

import argparse
import collections
import json
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[5]
if not (ROOT / "grammars").is_dir():
    raise SystemExit(f"containment: not at the repo root (derived {ROOT}) — fix the parents[] depth")


def reference_graph(graph_path: Path, arm: dict) -> dict[str, set[str]]:
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
    return {k: set(v) for k, v in g["edges"].items()}


def reachable(edges: dict[str, set[str]], starts: list[str]) -> set[str]:
    seen, stack = set(starts), list(starts)
    while stack:
        for child in edges.get(stack.pop(), ()):
            if child not in seen:
                seen.add(child)
                stack.append(child)
    return seen


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
    unknown = [r for r in a.introduced if r not in edges and not any(r in v for v in edges.values())]
    if unknown:
        print(f"containment: {unknown} name no rule in the grammar's reference graph — refusing",
              file=sys.stderr)
        return 2
    scope = reachable(edges, a.introduced)

    be, xe = base["rule_entries"], arm["rule_entries"]
    deltas = {r: xe.get(r, 0) - be.get(r, 0) for r in set(be) | set(xe)}
    rose = {r: d for r, d in deltas.items() if d > 0}
    fell = {r: d for r, d in deltas.items() if d < 0}
    escaped = {r: d for r, d in rose.items() if r not in scope}
    d_committed = arm["total_committed"] - base["total_committed"]

    print(f"CONTAINMENT: {base['arm']} -> {arm['arm']}   introduced={' '.join(a.introduced)}")
    print(f"  sub-graph reachable from the introduced rule(s) : {len(scope):>6,} rules")
    print(f"  rules whose entries ROSE                        : {len(rose):>6,}  "
          f"({sum(rose.values()):+,})")
    print(f"  rules whose entries FELL                        : {len(fell):>6,}  "
          f"({sum(fell.values()):+,})")
    print(f"  Δcommitted                                      : {d_committed:>+7,}")
    print(f"  risers OUTSIDE that sub-graph                   : {len(escaped):>6,}  "
          f"({sum(escaped.values()):+,})")

    legs = []
    legs.append(("Δcommitted <= 0", d_committed <= 0, f"{d_committed:+,}"))
    legs.append(("no rule's entries FELL", not fell,
                 f"{len(fell)} fell ({sum(fell.values()):+,})" if fell else "none"))
    legs.append(("every riser is in the sub-graph", not escaped,
                 f"{len(escaped)} escaped ({sum(escaped.values()):+,})" if escaped else "none"))
    print()
    for name, ok, detail in legs:
        print(f"  [{'PASS' if ok else 'FAIL'}] {name:<34} {detail}")

    if escaped:
        print(f"\n  the {min(a.top, len(escaped))} largest escaping rules:")
        for r, d in sorted(escaped.items(), key=lambda t: -t[1])[: a.top]:
            print(f"    {r:<50} {be.get(r, 0):>13,} -> {xe.get(r, 0):>13,}  {d:>+12,}")

    contained = all(ok for _, ok, _ in legs)
    print(f"\n  VERDICT: {'CONTAINED' if contained else 'NOT CONTAINED'}")
    return 0 if contained else 1


if __name__ == "__main__":
    sys.exit(main())

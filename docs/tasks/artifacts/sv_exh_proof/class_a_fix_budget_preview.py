#!/usr/bin/env python3
"""SV-EXH-PROOF.7.4.6.9 FIX_PLAN de-risk — what budget would the proposed fix actually give?

Mirrors `compute_min_full_derivation_depths` / `min_full_derivation_depth_of_node`
(stimuli_generator.rs:7920 / :7714) EXACTLY — the over-estimating structural proxy, which
charges +1 per NESTING node as well as per rule reference — and prints, per class-A rule,
what `original_max_depth * 2 + min_full_derivation_depth[rule]` would be, against the depth
the run actually needs (`purdom_depth`, from class_a_residual_depth_population.py).

This answers FIX_PLAN item (a) — "confirm the resulting budget actually clears 48-54 rather
than assuming it" — with no code change and no gate run.
"""

import json
import sys
from pathlib import Path

ARTIFACTS = Path(__file__).resolve().parent
sys.path.insert(0, str(ARTIFACTS))
from class_a_min_derivation_depth import (  # noqa: E402
    GEN_AST,
    INF,
    atom_inner_node,
    atom_reference,
    compute_min_terminal_lengths,
    compute_rule_depths,
    forced_branch_depth,
    quantifier_min,
)

GATE_MAX_DEPTH = 20  # the gate's --max-depth
FLAT_BUDGET = GATE_MAX_DEPTH * 2  # what generate_target_witnesses uses today (:5477)


def full_derivation_depth_of_node(node, depths, cache):
    """Mirror of min_full_derivation_depth_of_node (:7714). None == not yet resolvable."""
    key = id(node)
    if key in cache:
        return cache[key]
    if "Or" in node:
        values = [
            v
            for v in (
                full_derivation_depth_of_node(a, depths, cache)
                for a in node["Or"]["alternatives"]
            )
            if v is not None
        ]
        result = min(values) + 1 if values else None
    elif "Sequence" in node:
        deepest = 0
        result = 0
        for element in node["Sequence"]["elements"]:
            v = full_derivation_depth_of_node(element, depths, cache)
            if v is None:
                result = None
                break
            deepest = max(deepest, v)
        else:
            result = deepest + 1
    elif "Quantified" in node:
        q = node["Quantified"]
        if quantifier_min(q["quantifier"]) == 0:
            result = 0
        else:
            v = full_derivation_depth_of_node(q["element"], depths, cache)
            result = None if v is None else v + 1
    elif "Lookahead" in node:
        result = 0
    elif "Atom" in node:
        inner = atom_inner_node(node)
        if inner is not None:
            v = full_derivation_depth_of_node(inner, depths, cache)
            result = None if v is None else v + 1
        else:
            ref = atom_reference(node)
            if ref is None:
                result = 1
            else:
                v = depths.get(ref)
                result = None if v is None else v + 1
    else:
        result = 1
    cache[key] = result
    return result


def compute_full_derivation_depths(tree):
    depths = {}
    while True:
        changed = False
        cache = {}
        for name, body in tree.items():
            body_depth = full_derivation_depth_of_node(body, depths, cache)
            if body_depth is None:
                continue
            candidate = body_depth + 1
            if name not in depths or candidate < depths[name]:
                depths[name] = candidate
                changed = True
        if not changed:
            return depths


def main():
    with GEN_AST.open() as handle:
        tree = json.load(handle)["grammar_tree"]
    lengths = compute_min_terminal_lengths(tree)
    purdom_depths = compute_rule_depths(tree, lengths, purdom=True)
    proxy = compute_full_derivation_depths(tree)

    gap = Path(__file__).resolve().parents[4] / (
        "rust/target/sv_stimuli_quality_gate/work/profile_2017_replay_gap.json"
    )
    with gap.open() as handle:
        residual = {}
        for debt in json.load(handle)["reachable_branch_debt"]:
            residual.setdefault(debt["rule_name"], set()).add(debt["branch_index"])

    print(
        f"FIX-BUDGET-PREVIEW: flat budget today = {GATE_MAX_DEPTH} x 2 = {FLAT_BUDGET}; "
        f"proposed = {FLAT_BUDGET} + min_full_derivation_depth[rule]"
    )
    header = (
        "\nrule                                            needs  rule_proxy  A:rule  "
        "branch_proxy  B:branch"
    )
    print(header)
    worst = 0
    rule_ok = branch_ok = total = 0
    for rule in sorted(residual):
        if rule.startswith("net_declaration_"):  # class C, different mechanism
            continue
        for index in sorted(residual[rule]):
            needs = forced_branch_depth(tree, purdom_depths, lengths, True, rule, index)
            if needs == INF:
                continue
            total += 1
            if FLAT_BUDGET + (proxy.get(rule) or 0) >= needs:
                rule_ok += 1
            alt = tree[rule]["Or"]["alternatives"][index]
            bp = full_derivation_depth_of_node(alt, proxy, {})
            if bp is not None and FLAT_BUDGET + bp + 1 >= needs:
                branch_ok += 1
        needs = max(
            forced_branch_depth(tree, purdom_depths, lengths, True, rule, i)
            for i in residual[rule]
        )
        if needs == INF:
            continue
        worst_index = max(
            residual[rule],
            key=lambda i: forced_branch_depth(tree, purdom_depths, lengths, True, rule, i),
        )
        alt = tree[rule]["Or"]["alternatives"][worst_index]
        bp = full_derivation_depth_of_node(alt, proxy, {})
        rule_budget = FLAT_BUDGET + (proxy.get(rule) or 0)
        branch_budget = FLAT_BUDGET + (bp + 1 if bp is not None else 0)
        worst = max(worst, needs)
        print(
            f"{rule:<46} {needs:>5}  {str(proxy.get(rule)):>10}  "
            f"{'YES' if rule_budget >= needs else 'NO ':>6}  "
            f"{str(bp):>12}  {'YES' if branch_budget >= needs else 'NO'}"
        )
    print(f"\ndeepest class-A requirement across the residual: {worst}")
    print(
        f"per-branch verdict over all {total} class-A targets: "
        f"variant A (rule-scoped, the .5.2 formula verbatim) clears {rule_ok}/{total}; "
        f"variant B (branch-scoped) clears {branch_ok}/{total}"
    )
    return 0


if __name__ == "__main__":
    sys.exit(main())

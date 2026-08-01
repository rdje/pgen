#!/usr/bin/env python3
"""SV-EXH-PROOF.7.4.6.9 — WHY a forced class-A branch dies at max_depth=40.

Read-only static analysis over the SAME normalized gen-AST the stimuli generator
consumes (`--dump-gen-ast`).  It reproduces, for one rule's ordered choice, the depth
the generator ACTUALLY spends under the witness pass's construct mode, and contrasts
it with the depth the SHALLOWEST derivation would spend.

Two metrics, both mirrored from rust/src/ast_pipeline/stimuli_generator.rs:

  min_depth     the shallowest complete derivation — `Or` free to pick ANY alternative.
  purdom_depth  the derivation `construct_mode` actually commits to: at every `Or`,
                the alternative with the smallest `min_terminal_length_of_node`
                (:7640-7768), ties by index (:10065-10079), and NO fallback because
                `construct_mode` truncates the attempt order to 1 (:10087).

Runtime depth accounting (the real `generate_rule` recursion, not the over-estimating
structural proxy at :7714):
  * generate_rule(R, d)       FAILS when `d > max_depth`                 (:9491)
  * generate_rule(R, d)       evaluates R's body at depth `d + 1`        (:9740)
  * Atom("rule_reference", S) calls generate_rule(S, d + 1)              (:11347)
  * Quantified                descends its element at `d + 1`            (:11757)
  * Or / Sequence             pass `depth` through UNCHANGED             (:10375, :10693)
  * entry                     generate_rule(entry, 0)                    (:9163)
  * construct_mode renders a min-0 (`?`/`*`/`{0,..}`) quantifier ZERO times.

So a rule generates iff `depth_metric(R) <= max_depth`.  Both metrics are monotone
fixpoints, computed exactly the way the engine computes its own length fixpoint.

GROUND-TRUTH CONTROLS — this instrument refuses rather than guesses.  The canonical
run's own measured split of `prop_primary_sv_2017`'s 30 alternatives (4 covered /
26 residual, byte-identical in sv_2023) is replayed as a positive and a negative
control against `purdom_depth`.  Any control miss aborts with exit 2 before a number
is published.
"""

import json
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[4]
GEN_AST = ROOT / "rust/target/sv_stimuli_quality_gate/work/systemverilog_gen_ast.json"

INF = float("inf")
WITNESS_MAX_DEPTH = 40  # the gate runs --max-depth 20; generate_target_witnesses doubles it (:5477)

# The canonical run's measured split (profile_2017_replay_gap.json, 2026-08-01).
COVERED = [0, 1, 2, 28]


# --------------------------------------------------------------------------- tree
def atom_reference(node):
    token = node.get("Atom", {}).get("value", {}).get("Token")
    if not token or token[0].get("String") != "rule_reference":
        return None
    return token[1]["String"]


def atom_inner_node(node):
    value = node.get("Atom", {}).get("value", {})
    return value.get("Node") if "Node" in value else None


def quantifier_min(spec):
    if spec in ("?", "*"):
        return 0
    if spec == "+":
        return 1
    bounds = spec.strip("{}").split(",")
    try:
        return int(bounds[0].strip() or "0")
    except ValueError:
        return 1


# ------------------------------------------------- metric 1: min terminal length
def min_terminal_length_of_node(node, table, cache):
    """Mirror of `min_terminal_length_of_node` (:7640). None == not yet resolvable."""
    key = id(node)
    if key in cache:
        return cache[key]
    if "Or" in node:
        lengths = [
            length
            for length in (
                min_terminal_length_of_node(a, table, cache)
                for a in node["Or"]["alternatives"]
            )
            if length is not None
        ]
        result = min(lengths) if lengths else None
    elif "Sequence" in node:
        result = 0
        for element in node["Sequence"]["elements"]:
            length = min_terminal_length_of_node(element, table, cache)
            if length is None:
                result = None
                break
            result += length
    elif "Quantified" in node:
        quantified = node["Quantified"]
        count = quantifier_min(quantified["quantifier"])
        if count == 0:
            result = 0
        else:
            length = min_terminal_length_of_node(quantified["element"], table, cache)
            result = None if length is None else count * length
    elif "Lookahead" in node:
        result = 0
    elif "Atom" in node:
        inner = atom_inner_node(node)
        if inner is not None:
            result = min_terminal_length_of_node(inner, table, cache)
        else:
            referenced = atom_reference(node)
            result = 1 if referenced is None else table.get(referenced)
    else:
        result = 1
    cache[key] = result
    return result


def compute_min_terminal_lengths(tree):
    """Mirror of `compute_min_terminal_lengths` (:7606) — monotone fixpoint."""
    table = {}
    while True:
        changed = False
        cache = {}  # per-sweep: estimates change between sweeps, not within one
        for name, body in tree.items():
            candidate = min_terminal_length_of_node(body, table, cache)
            if candidate is None:
                continue
            if name not in table or candidate < table[name]:
                table[name] = candidate
                changed = True
        if not changed:
            return table


# ---------------------------------------------------- metric 2: generation depth
def purdom_choice(alternatives, lengths, cache):
    """The single alternative construct_mode commits to: min length, tie by index."""
    best_index, best_key = 0, None
    for index, alternative in enumerate(alternatives):
        length = min_terminal_length_of_node(alternative, lengths, cache)
        key = INF if length is None else length
        if best_key is None or key < best_key:
            best_index, best_key = index, key
    return best_index


def node_depth(node, depths, lengths, purdom, cache, length_cache):
    """Max generate_rule entry depth below `node`, relative to the node's own depth."""
    key = id(node)
    if key in cache:
        return cache[key]
    if "Or" in node:
        alternatives = node["Or"]["alternatives"]
        if purdom:
            chosen = purdom_choice(alternatives, lengths, length_cache)
            result = node_depth(
                alternatives[chosen], depths, lengths, purdom, cache, length_cache
            )
        else:
            result = min(
                (
                    node_depth(a, depths, lengths, purdom, cache, length_cache)
                    for a in alternatives
                ),
                default=INF,
            )
    elif "Sequence" in node:
        result = max(
            (
                node_depth(e, depths, lengths, purdom, cache, length_cache)
                for e in node["Sequence"]["elements"]
            ),
            default=0,
        )
    elif "Quantified" in node:
        quantified = node["Quantified"]
        if quantifier_min(quantified["quantifier"]) == 0:
            result = 0
        else:
            result = 1 + node_depth(
                quantified["element"], depths, lengths, purdom, cache, length_cache
            )
    elif "Lookahead" in node:
        result = 0
    elif "Atom" in node:
        inner = atom_inner_node(node)
        if inner is not None:
            result = node_depth(inner, depths, lengths, purdom, cache, length_cache)
        else:
            referenced = atom_reference(node)
            result = 0 if referenced is None else 1 + depths.get(referenced, INF)
    else:
        result = 0
    cache[key] = result
    return result


def compute_rule_depths(tree, lengths, purdom):
    """Monotone fixpoint of rule_depth, same shape as the engine's length fixpoint."""
    depths = {}
    while True:
        changed = False
        cache, length_cache = {}, {}
        for name, body in tree.items():
            below = node_depth(body, depths, lengths, purdom, cache, length_cache)
            if below == INF:
                continue
            candidate = 0 if below == 0 else 1 + below
            if name not in depths or candidate < depths[name]:
                depths[name] = candidate
                changed = True
        if not changed:
            return depths


def forced_branch_depth(tree, depths, lengths, purdom, rule_name, branch_index):
    """Depth of `rule_name` when its ROOT `Or` is forced to `branch_index`."""
    alternative = tree[rule_name]["Or"]["alternatives"][branch_index]
    below = node_depth(alternative, depths, lengths, purdom, {}, {})
    return 0 if below == 0 else 1 + below


def fmt(value):
    return "inf" if value == INF else str(value)


def main():
    with GEN_AST.open() as handle:
        tree = json.load(handle)["grammar_tree"]
    lengths = compute_min_terminal_lengths(tree)
    purdom_depths = compute_rule_depths(tree, lengths, purdom=True)
    min_depths = compute_rule_depths(tree, lengths, purdom=False)

    rule = sys.argv[1] if len(sys.argv) > 1 else "prop_primary_sv_2017"
    alt_count = len(tree[rule]["Or"]["alternatives"])
    residual = [i for i in range(alt_count) if i not in COVERED]

    purdom_branch = {
        i: forced_branch_depth(tree, purdom_depths, lengths, True, rule, i)
        for i in range(alt_count)
    }
    min_branch = {
        i: forced_branch_depth(tree, min_depths, lengths, False, rule, i)
        for i in range(alt_count)
    }

    failures = []
    for i in COVERED:
        if purdom_branch[i] > WITNESS_MAX_DEPTH:
            failures.append(
                f"POSITIVE CONTROL MISS: covered branch #{i} has purdom_depth="
                f"{fmt(purdom_branch[i])} > max_depth {WITNESS_MAX_DEPTH}"
            )
    for i in residual:
        if purdom_branch[i] <= WITNESS_MAX_DEPTH:
            failures.append(
                f"NEGATIVE CONTROL MISS: residual branch #{i} has purdom_depth="
                f"{fmt(purdom_branch[i])} <= max_depth {WITNESS_MAX_DEPTH}"
            )
    if failures:
        print("MIN-DERIVATION-DEPTH: REFUSING — ground-truth controls failed", file=sys.stderr)
        for line in failures:
            print("  " + line, file=sys.stderr)
        return 2

    print(
        f"MIN-DERIVATION-DEPTH: grammar=systemverilog rules={len(tree)} rule={rule} "
        f"witness_max_depth={WITNESS_MAX_DEPTH} controls=PASS "
        f"(positive {len(COVERED)}/{len(COVERED)}, negative {len(residual)}/{len(residual)})"
    )
    print("\nalt  purdom_depth  min_depth  measured   verdict")
    for i in range(alt_count):
        state = "COVERED " if i in COVERED else "residual"
        over = "over budget" if purdom_branch[i] > WITNESS_MAX_DEPTH else "fits"
        rescuable = (
            "  <-- the SHALLOWEST derivation FITS"
            if purdom_branch[i] > WITNESS_MAX_DEPTH >= min_branch[i]
            else ""
        )
        print(
            f"#{i:<3} {fmt(purdom_branch[i]):>12}  {fmt(min_branch[i]):>9}  {state}   "
            f"{over}{rescuable}"
        )

    print("\nwhole-rule metrics (Or free to choose, i.e. the unforced witness):")
    for name in (
        "prop_primary_sv_2017",
        "property_expr",
        "prop_or_sv_2017",
        "prop_iff_sv_2017",
        "sequence_expr",
        "expression",
        "primary_sv_2017",
        "expression_base",
        "tagged_union_expression_sv_2017",
        "systemverilog_file",
    ):
        if name in tree:
            print(
                f"  {name:<34} purdom_depth={fmt(purdom_depths.get(name, INF)):<6}"
                f" min_depth={fmt(min_depths.get(name, INF)):<6}"
                f" min_terminal_length={lengths.get(name)}"
            )
    return 0


if __name__ == "__main__":
    sys.exit(main())

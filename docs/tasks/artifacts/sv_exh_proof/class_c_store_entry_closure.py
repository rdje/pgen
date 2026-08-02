#!/usr/bin/env python3
"""SV-EXH-PROOF.7.4.6.12 — the class-C STORE-ENTRY-BLOCKED verdict, re-derived independently.

The engine decides whether to raise a witness target's entry with
`witness_target_is_store_entry_blocked` (rust/src/ast_pipeline/stimuli_generator.rs).  This
script answers the SAME question from the gate's OWN normalized grammar, in a different
language, so the engine's verdict is not the only thing asserting it:

    for a target, does its MANDATORY rendering force a POSITIVE store gate whose fact-kind NO
    rule in the target rule's own reachable closure can `@emit_fact`?

If yes, a witness rooted at that rule starts from an EMPTY store, the gate can never be
satisfied, and the generation-side prune rejects at ANY budget and ANY seed — which is exactly
the class-C residual (`STORE-AWARE-GEN: … fact_count_at_least predicate unsatisfiable (zero
source facts)`).

Two halves, both mirroring the engine:
  * MANDATORY-descent walk (`mandatory_reach_gate` / `mandatory_node_gated`) — an `Or` is gated
    only when EVERY alternative is, a `Sequence` when ANY element is, a min-0 quantifier never
    propagates, lookaheads render nothing.  ⛔ A whole-closure "does any gate exist below here"
    scan is NOT the same question and gets `net_declaration_sv_2017` wrong: its gated rules all
    sit behind ungated escapes.  That miscalibration is what the escape control below catches.
  * POSITIVE polarity only (`StoreGateScope::PositiveOnly`) — `has_fact` /
    `fact_attribute_equals` / `fact_count_at_least`.  A `lacks_fact` gate is SATISFIED by an
    empty store, so counting it would declare a witnessable target unwitnessable.

⭐ GROUND TRUTH — it refuses rather than guesses.  Before publishing any verdict it runs THREE
controls: a rule the project has already measured as blocked, a rule whose closure is full of
gated rules that are all escapable (the discriminating control), and the branch of that same
rule which IS blocked.  Any control missing its expected verdict aborts with exit 2.

Reads only artifacts an ordinary gate run already leaves behind — no gate re-run, no engine
call.  Every path is repo-root relative and stays on the repository volume.

  Usage: python3 docs/tasks/artifacts/sv_exh_proof/class_c_store_entry_closure.py [TARGET ...]
         TARGET is `rule` (a rule target) or `rule#N` (root-`Or` alternative N).
         Default: the class-C rule plus both profiles' consequent branch targets.

MEASURED 2026-08-02 (PGEN-SV-EXH-PROOF-0170), the gate's profile-2017 normalized grammar:
  wildcard_escape_nettype_identifier    closure=12   emittable={}  => BLOCKED
  net_declaration_sv_2017               (rule)                     => not blocked (escapes)
  net_declaration_sv_2017#2             (the wildcard-escape alt)  => BLOCKED
  the sole emitter of `wildcard_import_open` is `package_import_item`, in NEITHER closure.
"""

from __future__ import annotations

import collections
import json
import os
import re
import sys

ROOT = os.path.abspath(os.path.join(os.path.dirname(__file__), "..", "..", "..", ".."))
GEN_AST = os.path.join(
    ROOT, "rust", "target", "sv_stimuli_quality_gate", "work", "systemverilog_gen_ast.json"
)

# The POSITIVE store-gate primitives — the engine's `StoreGateScope::PositiveOnly` vocabulary.
POSITIVE_GATES = {"has_fact", "fact_attribute_equals", "fact_count_at_least"}

# (target, expected blocked) — see the module docstring for why each is trustworthy.
CONTROLS = (
    ("wildcard_escape_nettype_identifier", True),  # the measured class-C rule
    ("net_declaration_sv_2017", False),  # gated rules, all behind escapes
    ("net_declaration_sv_2017#2", True),  # …and the one alternative that is not
)

DEFAULT_TARGETS = [
    "wildcard_escape_nettype_identifier",
    "net_declaration_sv_2017#2",
    "net_declaration_sv_2023#2",
]

TARGET_RE = re.compile(r"^(?P<rule>[^#]+)(?:#(?P<branch>\d+))?$")


def load_grammar() -> dict:
    if not os.path.isfile(GEN_AST):
        sys.stderr.write(
            f"error: missing '{os.path.relpath(GEN_AST, ROOT)}' — run the SV stimuli quality gate first\n"
        )
        raise SystemExit(2)
    with open(GEN_AST, encoding="utf-8") as handle:
        return json.load(handle)


def token_pair(node):
    """('rule_reference', name) for a rule-reference atom, else None."""
    if not isinstance(node, dict) or "Atom" not in node:
        return None
    value = node["Atom"]["value"]
    if "Token" not in value:
        return None
    parts = value["Token"]
    if len(parts) >= 2:
        return parts[0].get("String"), parts[1].get("String")
    return None


def rule_references(node, out):
    """Every `rule_reference` token anywhere in a node (the reach-graph edge set)."""
    if not isinstance(node, dict):
        return out
    if "Atom" in node:
        pair = token_pair(node)
        if pair and pair[0] == "rule_reference":
            out.append(pair[1])
        else:
            inner = node["Atom"]["value"].get("Node")
            if inner is not None:
                rule_references(inner, out)
    elif "Sequence" in node:
        for element in node["Sequence"]["elements"]:
            rule_references(element, out)
    elif "Or" in node:
        for alternative in node["Or"]["alternatives"]:
            rule_references(alternative, out)
    elif "Quantified" in node:
        rule_references(node["Quantified"]["element"], out)
    elif "Lookahead" in node:
        rule_references(node["Lookahead"]["element"], out)
    return out


def closure(tree, entry):
    seen, queue = {entry}, [entry]
    while queue:
        rule = queue.pop(0)
        if rule not in tree:
            continue
        for referenced in rule_references(tree[rule], []):
            if referenced in tree and referenced not in seen:
                seen.add(referenced)
                queue.append(referenced)
    return seen


def quantifier_min(quantifier: str) -> int:
    """Mirrors `parse_quantifier_bounds`' minimum for the shapes a grammar can write."""
    if quantifier in ("?", "*"):
        return 0
    if quantifier == "+":
        return 1
    stripped = quantifier.strip("{}")
    head = stripped.split(",")[0].strip()
    return int(head) if head.isdigit() else 0


def emit_kinds(annotations) -> collections.defaultdict:
    """rule -> the fact-KINDS it `@emit_fact`s (rule-level AND branch-level)."""
    kinds = collections.defaultdict(set)

    def scan(entries, rule):
        for entry in entries or []:
            if entry.get("name") != "emit_fact":
                continue
            value = entry.get("ast", {}).get("Structured", {}).get("value", {})
            for prop in value.get("Object", []):
                if prop["key"] == "kind":
                    payload = prop["value"]
                    kind = payload.get("Identifier") or payload.get("String")
                    if kind:
                        kinds[rule].add(kind)

    for rule, entries in annotations.get("semantic_annotations", {}).items():
        scan(entries, rule)
    for rule, branches in annotations.get("branch_semantic_annotations", {}).items():
        for branch in branches:
            scan(branch, rule)
    return kinds


def positive_gate_kinds(annotations) -> collections.defaultdict:
    """rule -> the fact-KINDS its POSITIVE `@predicate` gates consult."""
    kinds = collections.defaultdict(set)

    def scan(entries, rule):
        for entry in entries or []:
            if entry.get("name") != "predicate":
                continue
            value = entry.get("ast", {}).get("Structured", {}).get("value", {})
            props = {p["key"]: p["value"] for p in value.get("Object", [])}
            name = props.get("name", {})
            if (name.get("Identifier") or name.get("String")) not in POSITIVE_GATES:
                continue
            args = props.get("args", {}).get("Array", [])
            if args:
                kind = args[0].get("Identifier") or args[0].get("String")
                if kind:
                    kinds[rule].add(kind)

    for rule, entries in annotations.get("semantic_annotations", {}).items():
        scan(entries, rule)
    for rule, branches in annotations.get("branch_semantic_annotations", {}).items():
        for branch in branches:
            scan(branch, rule)
    return kinds


class Walker:
    """Mirrors `mandatory_reach_gate` / `mandatory_node_gated` at `StoreGateScope::PositiveOnly`."""

    def __init__(self, tree, gates, available):
        self.tree = tree
        self.gates = gates
        self.available = available
        self.witnessed = []  # (gated_rule, kind) pairs that made the verdict true

    def rule_gated(self, rule, visited):
        if rule in visited:
            return False  # cycle: no NEW unsatisfiable gate below an in-progress rule
        visited.add(rule)
        try:
            unsatisfiable = [k for k in self.gates.get(rule, set()) if k not in self.available]
            if unsatisfiable:
                self.witnessed.append((rule, sorted(unsatisfiable)[0]))
                return True
            node = self.tree.get(rule)
            return self.node_gated(node, visited) if node is not None else False
        finally:
            visited.discard(rule)

    def node_gated(self, node, visited):
        if not isinstance(node, dict):
            return False
        if "Or" in node:
            alternatives = node["Or"]["alternatives"]
            if not alternatives:
                return False
            return all(self.node_gated(alt, visited) for alt in alternatives)
        if "Sequence" in node:
            return any(self.node_gated(el, visited) for el in node["Sequence"]["elements"])
        if "Quantified" in node:
            spec = node["Quantified"]
            if quantifier_min(spec.get("quantifier", "?")) >= 1:
                return self.node_gated(spec["element"], visited)
            return False
        if "Lookahead" in node:
            return False
        if "Atom" in node:
            value = node["Atom"]["value"]
            if "Node" in value:
                return self.node_gated(value["Node"], visited)
            pair = token_pair(node)
            if pair and pair[0] == "rule_reference":
                # A reference missing from the active tree cannot be rendered ⇒ no escape.
                return self.rule_gated(pair[1], visited) if pair[1] in self.tree else True
        return False


def verdict(tree, emitters, gates, rule, branch):
    """(blocked, closure_size, emittable_kinds, [(gated_rule, kind)])"""
    reach = closure(tree, rule)
    available = set()
    for member in reach:
        available |= emitters.get(member, set())
    walker = Walker(tree, gates, available)
    blocked = False
    if branch is not None:
        node = tree.get(rule)
        alternatives = node.get("Or", {}).get("alternatives", []) if isinstance(node, dict) else []
        if branch < len(alternatives):
            blocked = walker.node_gated(alternatives[branch], set())
    if not blocked:
        blocked = walker.rule_gated(rule, set())
    return blocked, len(reach), available, walker.witnessed


def parse_target(text):
    match = TARGET_RE.match(text)
    if not match:
        raise SystemExit(f"error: unparseable target '{text}' (expected `rule` or `rule#N`)")
    branch = match.group("branch")
    return match.group("rule"), (int(branch) if branch is not None else None)


def main() -> int:
    grammar = load_grammar()
    tree = grammar["grammar_tree"]
    annotations = grammar["annotations"]
    emitters = emit_kinds(annotations)
    gates = positive_gate_kinds(annotations)

    # ⭐ Ground-truth controls FIRST — publish nothing if any misses.
    for text, expected in CONTROLS:
        rule, branch = parse_target(text)
        if rule not in tree:
            sys.stderr.write(f"REFUSE: control target '{text}' is absent from the grammar\n")
            return 2
        blocked, _, _, _ = verdict(tree, emitters, gates, rule, branch)
        if blocked is not expected:
            sys.stderr.write(
                f"REFUSE: control '{text}' expected blocked={expected}, measured {blocked} — "
                "the instrument is mis-calibrated; fix it before trusting any verdict\n"
            )
            return 2
    print(
        "controls OK: "
        + "  ".join(f"{t}={'BLOCKED' if e else 'not blocked'}" for t, e in CONTROLS)
    )

    for text in sys.argv[1:] or DEFAULT_TARGETS:
        rule, branch = parse_target(text)
        if rule not in tree:
            print(f"{text}: ABSENT from this profile's grammar")
            continue
        blocked, size, available, witnessed = verdict(tree, emitters, gates, rule, branch)
        print(
            f"{text}: closure={size} emittable={sorted(available) or '{}'} "
            f"=> {'BLOCKED' if blocked else 'not blocked'}"
        )
        for gated_rule, kind in witnessed:
            producers = sorted(r for r, k in emitters.items() if kind in k)
            print(
                f"    mandatory gate on '{gated_rule}' consults '{kind}'; "
                f"producers={producers} (none inside the closure)"
            )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

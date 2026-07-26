#!/usr/bin/env python3
"""LEX-ADJACENCY.1 — size the transitive rule-closure of a candidate lexical-token rule.

The PARSE half of the no-layout primitive can be emitted either STATICALLY (a
specialized no-layout twin per rule in the annotated rule's closure — zero runtime
cost) or DYNAMICALLY (a runtime depth counter mirroring the generator's
`atomic_token_depth` — a per-rule-entry cost the ⭐ speed north star forbids).
Which one is affordable is a MEASUREMENT, not a preference: it depends entirely on
how big the closure is. This script measures it.

Read-only. Deterministic. Input is the normalized generation-input AST
(`--dump-gen-ast`), i.e. exactly the IR codegen consumes, so the closure this
reports is the closure codegen would specialize.

    ./rust/target/debug/ast_pipeline grammars/systemverilog.ebnf --generate-parser \
        --dump-gen-ast /tmp/sv_gen_ast.json --eliminate-left-recursion --output /tmp/p.rs
    python3 docs/tasks/artifacts/lex_adjacency/closure_probe.py /tmp/sv_gen_ast.json \
        time_literal number time_unit trivia
"""

from __future__ import annotations

import json
import sys


def rule_references(node, out: set[str]) -> set[str]:
    """Collect every `rule_reference` target reachable inside one rule's body node."""
    if isinstance(node, dict):
        for key, value in node.items():
            if key == "Token" and isinstance(value, list) and len(value) == 2:
                kind = value[0].get("String") if isinstance(value[0], dict) else None
                name = value[1].get("String") if isinstance(value[1], dict) else None
                if kind == "rule_reference" and name:
                    out.add(name)
            rule_references(value, out)
    elif isinstance(node, list):
        for item in node:
            rule_references(item, out)
    return out


def closure(grammar_tree: dict, root: str) -> set[str]:
    """The transitive rule closure of `root`, including `root` itself."""
    seen: set[str] = set()
    stack = [root]
    while stack:
        rule = stack.pop()
        if rule in seen or rule not in grammar_tree:
            continue
        seen.add(rule)
        stack.extend(rule_references(grammar_tree[rule], set()) - seen)
    return seen


def main(argv: list[str]) -> int:
    if len(argv) < 3:
        print(f"usage: {argv[0]} <gen_ast.json> <rule> [<rule> ...]", file=sys.stderr)
        return 2
    with open(argv[1], encoding="utf-8") as handle:
        grammar_tree = json.load(handle)["grammar_tree"]
    print(f"grammar rules: {len(grammar_tree)}")
    for root in argv[2:]:
        if root not in grammar_tree:
            print(f"{root:16s} ABSENT from the grammar tree")
            continue
        members = closure(grammar_tree, root)
        print(f"{root:16s} closure size = {len(members):5d}")
        print(f"{'':16s} members: {', '.join(sorted(members))}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv))

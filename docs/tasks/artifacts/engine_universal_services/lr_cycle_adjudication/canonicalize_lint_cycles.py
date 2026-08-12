#!/usr/bin/env python3
"""`ENGINE-UNIVERSAL-SERVICES.13` — how many DISTINCT left-recursive cycles does a grammar have?

`--lint-grammar` reports one warning **per participating rule**, because `detect_left_recursion`
starts a DFS from every rule and reports any that closes back on itself. A single 12-rule cycle is
therefore printed 12 times (once per rule that can start it), and the headline
`left_recursion_unhandled=30` counts *rule rows*, not cycles.

That distinction decides how big `.13`'s adjudication actually is. Canonicalising each reported
path by rotation collapses SystemVerilog's **30 rule rows to 7 distinct cycles** and `ebnf`'s **5 to
3** — so the acceptance-(a) worklist is 10 rows, not 35.

⛔ GROUND TRUTH — it refuses rather than guesses. Every parsed row must satisfy the shape the lint
guarantees (`rule -> … -> rule`, first element == last element == the reporting rule); a row that
does not is an error, not a skipped line. And a grammar whose headline count disagrees with the
number of rows actually parsed aborts before printing a number.

  python3 docs/tasks/artifacts/engine_universal_services/lr_cycle_adjudication/canonicalize_lint_cycles.py
  python3 …/canonicalize_lint_cycles.py grammars/systemverilog.ebnf grammars/ebnf.ebnf
"""
from __future__ import annotations

import re
import subprocess
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))

from _repo_root import repo_root  # noqa: E402  (path is set immediately above)

DEFAULT_GRAMMARS = ("grammars/systemverilog.ebnf", "grammars/ebnf.ebnf")
CYCLE_RE = re.compile(r"rule '([A-Za-z_0-9]+)' is left-recursive \(cycle: ([^)]*)\)")
HEADLINE_RE = re.compile(r"left_recursion_unhandled=(\d+)")


def lint(root: Path, grammar: str) -> str:
    binary = root / "rust/target/debug/ast_pipeline"
    if not binary.is_file():
        raise SystemExit(
            f"REFUSE: {binary} is missing — build it with:\n"
            '  (cd rust && cargo build --features "generated_parsers ebnf_dual_run" --bin ast_pipeline)'
        )
    result = subprocess.run(
        [str(binary), str(root / grammar), "--lint-grammar"],
        capture_output=True,
        text=True,
        env={**__import__("os").environ, "PGEN_LINT_DUMP_ALL": "1"},
    )
    return result.stdout + result.stderr


def canonical(rule: str, path: str) -> tuple[str, ...]:
    nodes = [node.strip() for node in path.split("->")]
    if len(nodes) < 2 or nodes[0] != rule or nodes[-1] != rule:
        raise SystemExit(
            f"REFUSE: cycle row for '{rule}' is not the shape the lint guarantees "
            f"(rule -> … -> rule): {path!r}"
        )
    core = nodes[:-1]
    pivot = min(range(len(core)), key=lambda index: core[index])
    return tuple(core[pivot:] + core[:pivot])


def report(root: Path, grammar: str) -> int:
    text = lint(root, grammar)
    headline = HEADLINE_RE.search(text)
    rows = CYCLE_RE.findall(text)
    if headline is None:
        raise SystemExit(f"REFUSE: no left_recursion_unhandled headline in the lint of {grammar}")
    declared = int(headline.group(1))
    if declared != len(rows):
        raise SystemExit(
            f"REFUSE: {grammar} headline says {declared} unhandled cycles but "
            f"{len(rows)} rows were parsed — is PGEN_LINT_DUMP_ALL honoured?"
        )

    distinct: dict[tuple[str, ...], list[str]] = {}
    for rule, path in rows:
        distinct.setdefault(canonical(rule, path), []).append(rule)

    print(f"\n=== {grammar}: {declared} reported rule rows -> {len(distinct)} DISTINCT cycles")
    for index, (cycle, reporters) in enumerate(distinct.items(), 1):
        print(f"  [{index}] len={len(cycle):2d}  reported by {len(reporters):2d} rule(s)")
        print("       " + " -> ".join(cycle) + f" -> {cycle[0]}")
    return len(distinct)


def main(argv: list[str]) -> int:
    root = repo_root()
    grammars = argv[1:] or list(DEFAULT_GRAMMARS)
    total = sum(report(root, grammar) for grammar in grammars)
    print(f"\nTOTAL distinct cycles across {len(grammars)} grammar(s): {total}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv))

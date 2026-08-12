#!/usr/bin/env python3
"""Which LRM literal `{ … }` did the extractor read as EBNF repetition? (`SV-CORPUS-GRAD.13c.2e`)

⛔ WHY THIS EXISTS. `.13c.2e` closes ONE instance of the dropped-delimiter class — IEEE
1800-2017 A.2.11 `select_condition ::= binsof ( bins_expression ) [ intersect {
covergroup_range_list } ]`, whose LITERAL braces reached `systemverilog.ebnf` as
`covergroup_range_list*`. The leaf's own caution was *"do not assume this is the only place"*,
and the honest way to discharge that is a census, not a paragraph. This is the census.

THE DISCRIMINATOR, stated so it can be argued with
------------------------------------------------
IEEE 1800 Annex A writes repetition as `{ x }` and literal SystemVerilog braces as `{ x }` —
the SAME characters. Nothing typographic separates them, which is exactly how the extractor
fell in (`SV-0049`'s note records the collision landing two lines apart in the same clause).
The separating rule this instrument uses is SEMANTIC and decidable:

    a `{ X }` whose X is itself a LIST nonterminal (`*_list`) is LITERAL brace syntax,
    because a list production already carries its own repetition — wrapping it in
    repetition metasyntax would mean "zero or more comma-separated lists, with no
    separator between them", which no clause in the standard means.

Rows are therefore classified into three tiers rather than one verdict:

  DEFECT   `{ X_list }` that the grammar renders as `X_list*` with NO `lbrace`/`rbrace`.
  CORRECT  `{ X_list }` that the grammar renders with `lbrace … rbrace` (the sibling
           productions that were already right, and stay right).
  REPEAT   `{ X }` where X is not a list ⇒ ordinary repetition metasyntax, not a defect.

⚠️ HONEST BOUNDS, stated rather than papered over.
  1. The tier rule keys on the `_list` SUFFIX. A literal-brace production whose inner
     nonterminal is not named `*_list` would land in REPEAT and be missed. That is a real
     hole, and it is why the REPEAT tier is PRINTED in full rather than summarised — the
     reader is meant to be able to scan it, which is cheap at this size.
  2. The grammar side is read textually (rule head + continuation lines), not through the
     EBNF frontend. It sees the file as written, which is what the question is about.
  3. A rule may be profile-split (`X_sv_2017` / `X_sv_2023`); all three spellings are probed.

GROUND TRUTH — the instrument refuses rather than guesses
--------------------------------------------------------
Four controls run before any verdict is printed, and a failure exits 2 instead of publishing
([[a-check-whose-inputs-all-pass-has-not-been-tested]]):

  C1  the LRM parse found a plausible number of productions (both editions, >800 each);
  C2  the grammar parse resolved `select_condition` — the name-resolution control, which is
      what fails silently if the rule-body joiner breaks;
  C3  CORRECT contains BOTH `expression_or_dist` and `inside_expression` — the two known
      literal-brace list productions this repo already models right (`SV-0049` fixed the
      first; the second was never wrong). If they stop appearing, the tier rule broke.
  C4  REPEAT contains `specify_block` — an unambiguous repetition row. If a genuine
      repetition starts reading as a defect, this fires.

⭐ NOTE ON C3 AND THE POST-FIX RUN. Before `.13c.2e` lands, DEFECT = {`select_condition`};
after it lands, DEFECT is EMPTY and `select_condition` joins CORRECT. Both are expected
outputs of the same instrument — the before/after pair is the leaf's evidence, so this script
deliberately does NOT pin `select_condition` to a tier. C3 pins rules the fix does not touch.

Usage (from anywhere; paths resolve against the repository root, directive 12):
  python3 docs/tasks/artifacts/sv_corpus_grad/intersect_braces/lrm_brace_transcription_sweep.py
  python3 …/lrm_brace_transcription_sweep.py --quiet     # tiers only, no REPEAT listing

  # the leaf's BEFORE side, reproducible from git rather than from a stashed copy:
  git show fdade1ce:grammars/systemverilog.ebnf > /tmp/before.ebnf
  python3 …/lrm_brace_transcription_sweep.py --grammar /tmp/before.ebnf --quiet
"""

from __future__ import annotations

import argparse
import re
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
from _repo_root import repo_root  # noqa: E402

ROOT = repo_root()
LRM_EDITIONS = ("2017", "2023")
GRAMMAR = ROOT / "grammars" / "systemverilog.ebnf"

PRODUCTION = re.compile(r"^([a-z_][a-z0-9_]*)\s*::=\s*(.*)$")
RULE_HEAD = re.compile(r"^([a-z_][a-z0-9_]*)\s*:=\s*(.*)$")
BRACE_GROUP = re.compile(r"\{\s*([a-z_][a-z0-9_]*)\s*\}")


def load_lrm(edition: str) -> dict[str, set[str]]:
    """Every `name ::= rhs` in one LRM edition's extracted markdown."""
    md_dir = ROOT / "docs" / "systemverilog" / edition / "md"
    productions: dict[str, set[str]] = {}
    for path in sorted(md_dir.glob("*.md")):
        with path.open(errors="replace") as handle:
            for line in handle:
                match = PRODUCTION.match(line.strip())
                if match:
                    productions.setdefault(match.group(1), set()).add(match.group(2).strip())
    return productions


def load_grammar(path: Path | None = None) -> dict[str, str]:
    """Each rule's body, continuation lines folded in (the file as written)."""
    bodies: dict[str, str] = {}
    current: str | None = None
    for line in (path or GRAMMAR).read_text().splitlines():
        head = RULE_HEAD.match(line)
        if head:
            current = head.group(1)
            bodies[current] = head.group(2)
        elif current and line.startswith(" "):
            bodies[current] += " " + line.strip()
    return bodies


def renders_as_repetition(body: str, inner: str) -> bool:
    """Does the grammar spell `inner` as a `*` repetition (rather than a braced single)?"""
    name = re.escape(inner)
    return bool(
        re.search(rf"\b{name}\s*\*", body) or re.search(rf"\(\s*{name}\s*\)\s*\*", body)
    )


def classify(grammar_path: Path | None = None) -> tuple[list[tuple], list[tuple], list[tuple], dict[str, int]]:
    grammar = load_grammar(grammar_path)
    defect, correct, repeat = [], [], []
    counts: dict[str, int] = {}
    for edition in LRM_EDITIONS:
        productions = load_lrm(edition)
        counts[edition] = len(productions)
        for name, right_hand_sides in productions.items():
            candidates = (name, f"{name}_sv_2017", f"{name}_sv_2023")
            for rhs in right_hand_sides:
                for match in BRACE_GROUP.finditer(rhs):
                    inner = match.group(1)
                    for candidate in candidates:
                        body = grammar.get(candidate)
                        if body is None:
                            continue
                        if not renders_as_repetition(body, inner):
                            continue
                        braced = "lbrace" in body and "rbrace" in body
                        row = (edition, name, candidate, inner)
                        if not inner.endswith("_list"):
                            repeat.append(row)
                        elif braced:
                            correct.append(row)
                        else:
                            defect.append(row)
    # A rule whose LRM `{ X_list }` IS braced in the grammar never renders X_list as `*`,
    # so it cannot reach the loop above. Probe those directly for the C3 control.
    for edition in LRM_EDITIONS:
        for name, right_hand_sides in load_lrm(edition).items():
            for rhs in right_hand_sides:
                for match in BRACE_GROUP.finditer(rhs):
                    inner = match.group(1)
                    if not inner.endswith("_list"):
                        continue
                    for candidate in (name, f"{name}_sv_2017", f"{name}_sv_2023"):
                        body = grammar.get(candidate)
                        if body is None:
                            continue
                        if re.search(rf"lbrace\s+{re.escape(inner)}\s+rbrace", body):
                            correct.append((edition, name, candidate, inner))
    dedupe = lambda rows: sorted(set(rows))
    return dedupe(defect), dedupe(correct), dedupe(repeat), counts


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--quiet", action="store_true", help="omit the REPEAT listing")
    parser.add_argument(
        "--grammar",
        type=Path,
        default=None,
        help="read the grammar from this file instead of grammars/systemverilog.ebnf "
             "(use `git show <rev>:grammars/systemverilog.ebnf` for a BEFORE run)",
    )
    args = parser.parse_args()

    defect, correct, repeat, counts = classify(args.grammar)
    grammar = load_grammar(args.grammar)

    failures = []
    for edition, count in counts.items():
        if count < 800:
            failures.append(f"C1: only {count} productions parsed from the {edition} LRM")
    if "select_condition" not in grammar:
        failures.append("C2: the grammar reader did not resolve `select_condition`")
    correct_names = {row[1] for row in correct}
    for expected in ("expression_or_dist", "inside_expression"):
        if expected not in correct_names:
            failures.append(f"C3: `{expected}` is missing from the CORRECT tier")
    if "specify_block" not in {row[1] for row in repeat}:
        failures.append("C4: `specify_block` is missing from the REPEAT tier")
    if failures:
        for line in failures:
            print(f"REFUSE — {line}", file=sys.stderr)
        return 2

    print("LRM literal-brace transcription sweep — `{ X }` groups the grammar renders as `X*`")
    print(f"  LRM productions read: " + ", ".join(f"{k}={v}" for k, v in sorted(counts.items())))
    shown = args.grammar if args.grammar else GRAMMAR.relative_to(ROOT)
    print(f"  grammar rules read:   {len(grammar)}  ({shown})")
    print()
    print(f"DEFECT  — literal `{{ X_list }}` rendered as `X_list*`, no braces: {len(defect)}")
    for edition, name, candidate, inner in defect:
        print(f"    [{edition}] {name} -> grammar `{candidate}` renders `{inner}*`")
    print()
    print(f"CORRECT — literal `{{ X_list }}` modelled with `lbrace … rbrace`: {len(correct)}")
    for edition, name, candidate, inner in correct:
        print(f"    [{edition}] {name} -> grammar `{candidate}` braces `{inner}`")
    print()
    print(f"REPEAT  — `{{ X }}` where X is not a list ⇒ genuine repetition: {len(repeat)}")
    if not args.quiet:
        for edition, name, candidate, inner in repeat:
            print(f"    [{edition}] {name} -> `{inner}`")
    print()
    print(f"VERDICT: {len(defect)} mis-transcribed literal-brace production(s).")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

#!/usr/bin/env python3
"""Which corpus rows can the `select_condition` brace fix legally move? (`SV-CORPUS-GRAD.13c.2e`)

The TARGETING instrument, in the sense `.3.8` established and `.3.18` reused: the set it
prints BEFORE the fix is the set the fix is allowed to move, so *"flipped-but-not-keyed = 0"*
is checkable rather than asserted.

⛔⛔ THE TRAP THIS INSTRUMENT EXISTS TO AVOID — `intersect` IS TWO DIFFERENT OPERATORS.
IEEE 1800-2017 spells the SVA sequence operator `sequence_expr ::= sequence_expr intersect
sequence_expr` (A.2.10, clause 16.9.6) with the SAME keyword as the covergroup select
condition `binsof ( bins_expression ) [ intersect { covergroup_range_list } ]` (A.2.11). A
census keyed on the bare word `intersect` mixes them: measured on this corpus, a bare-word
key returns 35 failing rows of which only 29 are `select_condition`, and it reports 7
"brace-less intersect" passing rows that are ALL SVA sequences the fix does not touch. Keying
on the bare word would therefore have manufactured an over-acceptance population that does
not exist, and then "explained" its own artifact.

⇒ The predicate here requires the `binsof ( … )` head: an `intersect` is keyed only when it
is preceded, modulo white space, by a `binsof ( … )` call. That is the shape `select_condition`
actually derives.

THREE POPULATIONS, because the fix moves acceptance in BOTH directions
---------------------------------------------------------------------
  fail + range      a FAILING row whose `intersect { … }` list carries a `[a:b]` range.
                    A range is not an expression, so the pre-fix concatenation route could
                    not derive it — these are the REJECT→ACCEPT candidates.
  pass              a PASSING row with a binsof-intersect. Must still pass; this is the
                    regression tripwire for a fix that NARROWS the language.
  pass + brace-less a PASSING row whose binsof-intersect has NO braces. The LRM has no such
                    production, so each is an accepts-invalid instance the fix should turn
                    into a reject.

⚠️ HONEST BOUNDS, stated rather than papered over.
  1. The scan is LEXICAL. A `binsof`/`intersect` inside a string literal or a comment is
     counted, and a macro-assembled one is missed. The binding no-regression proof for this
     leaf is the GLOBAL per-file transition census (`analyze_transitions.py`, the `.3.4` LAW),
     never this scan — this scan only says which movements are ATTRIBUTABLE.
  2. `fail + range` is an UPPER bound on the yield, not a prediction: a corpus file fails for
     whatever reason it hits FIRST, so a keyed row may stay failing on an unrelated defect.
     A keyed row that does not flip is not a bug in the fix; an UNKEYED row that flips is.
  3. `binsof\\s*\\([^()]*\\)` does not nest, so a `binsof(f(x))` head is missed. Measured on
     this corpus: 0 such rows, so the simplification costs nothing here — but it would have
     to be revisited on another corpus.

Usage (from anywhere; paths resolve against the repository root, directive 12):
  python3 docs/tasks/artifacts/sv_corpus_grad/intersect_braces/keyed_intersect_rows.py \
      --results rust/target/sv_axis2_baseline/results.pre_13c_2e.tsv
"""

from __future__ import annotations

import argparse
import re
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
from _repo_root import repo_root  # noqa: E402

ROOT = repo_root()

# A `select_condition` head. The negative lookahead in NOBRACE deliberately allows white
# space before rejecting, so `intersect  {` is NOT reported as brace-less.
SELECT_CONDITION = re.compile(r"binsof\s*\([^()]*\)\s*intersect", re.S)
WITH_RANGE = re.compile(
    r"binsof\s*\([^()]*\)\s*intersect\s*\{[^}]*\[[^\]]*:[^\]]*\][^}]*\}", re.S
)
BRACE_LESS = re.compile(r"binsof\s*\([^()]*\)\s*intersect\s*(?!\s*\{)\S", re.S)
# The control: an SVA sequence `intersect`, which this fix must NOT touch.
SEQUENCE_INTERSECT = re.compile(r"(?<!\))\s\bintersect\b", re.S)


def read_results(path: Path) -> dict[str, str]:
    outcomes: dict[str, str] = {}
    for line in path.read_text().splitlines():
        parts = line.split("\t")
        if len(parts) >= 3:
            outcomes[parts[2]] = parts[1]
    return outcomes


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--results",
        type=Path,
        default=Path("rust/target/sv_axis2_baseline/results.pre_13c_2e.tsv"),
        help="a run_external_corpus.sh results.tsv (repo-root-relative or absolute)",
    )
    args = parser.parse_args()
    results_path = args.results if args.results.is_absolute() else ROOT / args.results
    if not results_path.is_file():
        print(f"REFUSE: no results file at {results_path}", file=sys.stderr)
        return 2

    outcomes = read_results(results_path)
    if len(outcomes) < 1000:
        print(f"REFUSE: only {len(outcomes)} rows read — that is not a corpus", file=sys.stderr)
        return 2

    fail_keyed, fail_range, pass_keyed, pass_braceless, sva_only = [], [], [], [], []
    for rel, status in sorted(outcomes.items()):
        path = ROOT / rel
        try:
            text = path.read_text(errors="replace")
        except OSError:
            continue
        if not SELECT_CONDITION.search(text):
            if "intersect" in text:
                sva_only.append(rel)
            continue
        if status == "fail":
            fail_keyed.append(rel)
            if WITH_RANGE.search(text):
                fail_range.append(rel)
        else:
            pass_keyed.append(rel)
            if BRACE_LESS.search(text):
                pass_braceless.append(rel)

    print(f"corpus rows: {len(outcomes)}   results: {results_path.relative_to(ROOT)}")
    print()
    print(f"KEYED, FAILING (binsof-intersect):        {len(fail_keyed)}")
    print(f"  ... carrying a RANGE in the list:       {len(fail_range)}   <- REJECT->ACCEPT candidates")
    for rel in fail_range:
        print(f"      {rel}")
    print(f"KEYED, PASSING (binsof-intersect):        {len(pass_keyed)}   <- must not regress")
    for rel in pass_keyed:
        print(f"      {rel}")
    print(f"  ... BRACE-LESS (no LRM production):     {len(pass_braceless)}   <- ACCEPT->REJECT candidates")
    for rel in pass_braceless:
        print(f"      {rel}")
    print()
    print(f"NOT KEYED but containing `intersect`:     {len(sva_only)}   <- SVA sequence operator, untouched")
    print()
    print("A flip inside the keyed sets is attributable to this fix. A flip OUTSIDE them is not,")
    print("and is the signal to stop and diagnose rather than to explain.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

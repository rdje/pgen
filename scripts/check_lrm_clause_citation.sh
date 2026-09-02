#!/usr/bin/env bash
# scripts/check_lrm_clause_citation.sh
#
# LRM-CLAUSE-CITATION enforcement gate (SV-CORPUS-GRAD.13q / .13q(c)).
#
# THE RULE: a cited IEEE clause number must CONTAIN the LRM line it is cited with.
#
# ⛔ THE FOUNDING MEASUREMENT. A conformance claim here is four measurements wearing one sentence —
# the QUOTED TEXT, the LRM FILE, the LINE NUMBER, and the CLAUSE NUMBER. Three are checkable by
# anyone with `grep`; the fourth was checkable by nobody, and it was wrong. `SV-CORPUS-GRAD.13p`
# located the gate-delay cardinality sentence at 1800-2017 `section-28-…md:396` and 1364-2005
# `section-7-…md:376`, quoted it correctly, and labelled it §28.16 / §7.14 — a real clause ("Gate
# and net delays") whose number sat four lines from the located text inside §28.3.3's own
# cross-reference *"Delays are discussed in more detail in 28.16."* Seven cited lines, seven wrong
# clause labels, every quotation right — shipped to release ledger `SV-0077`, the downstream
# SystemVerilog contract, the parser book and a grammar comment, and repeated by everything that
# quoted them. Nothing in the grammar fix was wrong, which is exactly why the class needs a gate:
# a wrong LABEL on a right claim survives every other check in this repository.
#
# ⛔ IT JUDGES ONLY WHAT IT CAN BIND, AND PRINTS THE SIZE OF THE REST. A line reference is judged
# only when its file is pinned structurally, it sits in the clause's own bracket scope, and the
# resolved file is clause-addressable. Everything else is COUNTED (`no_line_ref`, `no_edition`,
# `annex_ref`, `unaddressable_section`, `unresolved_file`) and printed beside the verdict — because
# a census that gets cleaner because it stopped looking is the failure mode that looks most like
# success, and this instrument caught its own author committing it (judged rows fell 35 -> 17
# mid-fix while the report stayed green).
#
# ⭐ THERE IS NO ALLOWANCE LIST. Two citation shapes legitimately break the enclosure premise and
# BOTH are recognised structurally, never by path: the defined-here/absent-there sentence
# (*"defined in §8.26.1 but NOT referenced at `section-8-classes.md:95`"* — two correct
# parentheticals about different places) is unbound by bracket scope, and the mis-titled corpus
# file (17 of 124 vendored section files carry a name their own headings refute) is unbound by
# clause-addressability. The first cut carried those as seven hand-written exceptions keyed by
# line numbers that move; a gate whose green depends on a hand-maintained exception list is not a
# gate, it is a promise. Removing the list cost ZERO reach — `agree` is identical either way.
#
# ⛔⛔ SV-CORPUS-GRAD.13u — THE ADDRESSABILITY RULE WAS CORRECTED AND A PUBLISHED NUMBER WITHDRAWN.
# Its first cut tested a FILE against its own FILENAME (the modal heading major) and reported
# "17 of 124 vendored section files carry a filename their own headings refute". That is FALSE.
# The modal major measures which clause is BIGGEST, not whether the filename lies:
# `section-15-…md` opens at 15.1 with 26 clause-15 headings — correctly named, the defect being a
# missing SPLIT — and `section-29-…md` is 29.1→29.10 and is not defective at all. Spanning is the
# NORM (34/55, 25/41, 17/28 per edition), so it is no signal. Both questions are now per-(file,
# CITED CLAUSE): `addressable_for` asks whether the file carries any heading of that major, and
# `enclosing` filters candidates by the CITED major rather than the file's modal one — the modal
# form was latently wrong for a spanning file, skipping all 26 clause-15 headings. The filename's
# number is read NOWHERE any more. The corrected rule judges MORE (agree 33 → 35) with
# `unaddressable_section` 2 → 0: the hole the first cut reported in its own reach was the
# instrument's, not the corpus's. The residual corpus question is `SV-CORPUS-GRAD.13t`.
#
# GROUND TRUTH runs on every invocation (18 pairs, including the founding §28.16 -> :396 known-bad,
# an ancestor-citation known-good, and the two files SV-CORPUS-GRAD.13u's predecessor got wrong,
# pinned in both directions) and a miss exits 7 — MISCALIBRATED — rather than reporting a clean tree.
#
# `--self-test` adds a RED control per structural rule, and EACH ARM CARRIES ITS OWN SUBJECT: three
# synthetic probes driven through the real pipeline and asserted in BOTH directions. That shape was
# learned rather than designed. The arms used to mutate each rule on the REAL TREE; when .13u
# corrected the addressability rule, no citation in the repository landed on an unaddressable pair
# any more, the mutation changed nothing, and the arm correctly reported itself INERT rather than
# passing — a control whose subject the tree happens not to contain is not a passing control, it is
# an absent one (SV-CORPUS-GRAD.87(e)). The tree-level run is kept as corroboration and reports
# NOT EXERCISED, never a pass, when this repository does not contain the shape; a GREEN control
# asserts the tree re-reads identically afterwards.
#
# Cost: ~0.43 s over the whole tracked tree, no build, no parser, reads and mutates nothing.
set -euo pipefail
ROOT="$(git rev-parse --show-toplevel)"
cd "$ROOT"

CENSUS="stimuli/sv/lrm_clause_citation_census.py"
[ -f "$CENSUS" ] || {
  echo "lrm-clause-citation: FAIL — $CENSUS not found (the doctrine's own oracle is missing)" >&2
  exit 1
}

out="$(python3 "$CENSUS" 2>&1)"; rc=$?
printf '%s\n' "$out" | sed 's/^/lrm-clause-citation: /'

case "$rc" in
  0) echo "lrm-clause-citation: OK" ;;
  7) echo "lrm-clause-citation: FAIL — the census is MISCALIBRATED; its verdict about the tree" >&2
     echo "       means nothing until its ground truth passes. Re-run: python3 $CENSUS --self-test" >&2
     exit 1 ;;
  *) echo "lrm-clause-citation: FAIL — a cited clause number does not contain the line it cites." >&2
     echo "       A clause number is a MEASUREMENT, not a label: the innermost numbered heading at" >&2
     echo "       or before the cited line IS the clause that contains it. Re-derive it and correct" >&2
     echo "       the citation — do NOT move the line number to fit the clause." >&2
     exit 1 ;;
esac

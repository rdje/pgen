#!/usr/bin/env bash
# ENGINE-UNIVERSAL-SERVICES.17 slice 3 — the guard census run through the REAL PLANNER, pinned.
#
# WHAT IT MEASURES. Slice 2 priced option (iii) at `guard-feasible 16/28` and left slice 3 one
# mandatory first check, in its own words:
#
#   "whether the annotation-composability check also passes at `property_expr` is NOT measured here
#    (the wrapper refuses it for a missing return annotation on `property_expr_sv_2017`
#    alternative 5) and is slice 3's first check."
#
# `--report-indirect-lr-plan --indirect-lr-plan-guard-dry-run` answers it by admitting the
# guard-feasible candidates into the REAL elimination driver on a CLONE of the grammar, so the
# verdict comes from the shipped planner rather than from a second implementation of it.
#
#   D1  SV: the answer                 would_absorb=2 would_refuse=0 — BOTH knots reach a plan
#   D2  SV: the two rules              casting_type, property_expr — the SVA knot composes
#   D3  SV: the payoff, MEASURED       left-recursive rule rows 28 -> 0 on the rewritten clone
#   D4  ⛔ SV: the instrument's INPUT   1069 annotated rules — without this, `would_refuse=0` could
#                                      equally mean "the grammar declares no annotations at all"
#   D5  wrapper: the CONTRAST          would_absorb=0 would_refuse=13, every one of them
#                                      "declares no return annotation"
#   D6  ⛔ wrapper: WHY, in one number  21 annotated rules — the wrapper's own entrypoints, and
#                                      essentially nothing from the 148 KB LRM-GENERATED body it
#                                      includes. Its refusals are a fact about IT, and transfer to
#                                      nothing about the grammar that ships.
#   D7  ebnf: nothing owed             would_absorb=0 would_refuse=0 — its knot is already absorbed
#   D8  ⛔ the default report is UNMOVED no dry-run section without the flag
#
# ⭐ GROUND TRUTH (`feedback_instrument_needs_ground_truth`). Every case DECLARES the value it
# requires and this script compares against it, so any disagreement is a non-zero exit naming the
# case. D4/D6 are the falsifiability pair: they pin the instrument's own INPUT, because
# `compose_route_template` returns "nothing to compose" on a grammar with no annotations — so a
# `would_refuse=0` read without them is indistinguishable from a vacuous pass. D8 pins that the
# instrument is opt-in, i.e. that no other reader of this report saw anything change.
#
# ⛔ IF A CASE FAILS, RE-ADJUDICATE — DO NOT EDIT THE EXPECTATION. D1/D2 are the evidence that
# `.15`'s founding premise is re-adjudicated; D5/D6 are the evidence that the wrapper's refusal is
# NOT evidence about the shipped grammar.
#
# ⛔⛔ WHAT NONE OF IT CLAIMS. The dry run emits NO guard, so the grammar it builds is the one
# `.13` slice 5 measured as a REGRESSION (`initial k = 8'(1);` ACCEPT -> REJECT when `casting_type`
# is eliminated unguarded). These cases pin PLAN-STAGE reachability — annotation composability, the
# trial re-lint, the ambiguity check — and nothing about what the rewritten grammar parses.
#
# ⛔ IT TOUCHES NOTHING. The dry run works on a clone; the shipped grammar, `generated/` and the
# scratch slot are all untouched, and the report is always rc 0.
#
# HOW TO RUN (from the repository root):
#   bash docs/tasks/artifacts/engine_universal_services/guard_dry_run/probe.sh
set -uo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../../../.." && pwd)"
cd "$ROOT" || exit 1

PIPELINE="rust/target/debug/ast_pipeline"

if [ ! -x "$PIPELINE" ]; then
  echo "probe: $PIPELINE not built. Run:" >&2
  echo "  (cd rust && cargo build --features \"generated_parsers ebnf_dual_run\" --bin ast_pipeline)" >&2
  exit 2
fi

# The under-featured-binary trap (`CI-PARITY-GATE-ROT.24`): an under-featured binary errors on
# stderr and yields an EMPTY row on stdout, and two empty result sets diff clean. Refuse up front.
if ! scripts/require_ast_pipeline_features.sh "$PIPELINE" ebnf_dual_run >/dev/null 2>&1; then
  scripts/require_ast_pipeline_features.sh "$PIPELINE" ebnf_dual_run >&2
  exit 2
fi

dry_run_report() {
  "$PIPELINE" "$1" --report-indirect-lr-plan --indirect-lr-plan-guard-dry-run 2>/dev/null
}

SV_REPORT="$(dry_run_report grammars/systemverilog.ebnf)"
WRAPPER_REPORT="$(dry_run_report grammars/systemverilog_lrm_profiled_wrapper.ebnf)"
EBNF_REPORT="$(dry_run_report grammars/ebnf.ebnf)"
SV_PLAIN="$("$PIPELINE" grammars/systemverilog.ebnf --report-indirect-lr-plan 2>/dev/null)"

fail=0

# check <case-label> <what> <got> <want>
check() {
  local label="$1" what="$2" got="$3" want="$4"
  if [ "$got" = "$want" ]; then
    printf '  %-4s %-52s => %-40s ✅\n' "$label" "$what" "$got"
  else
    printf '  %-4s %-52s => %-40s ⛔ want %s\n' "$label" "$what" "$got" "$want"
    fail=1
  fi
}

# `would_absorb=N would_refuse=M` reduced to `N/M`.
verdict_of() {
  grep -m1 '^    would_absorb=' <<< "$1" \
    | sed -E 's/.*would_absorb=([0-9]+) would_refuse=([0-9]+).*/\1\/\2/'
}

# `left_recursive_rule_rows A -> B` reduced to `A->B`.
rows_of() {
  grep -m1 '^    would_absorb=' <<< "$1" \
    | sed -E 's/.*left_recursive_rule_rows ([0-9]+) -> ([0-9]+).*/\1->\2/'
}

annotated_rules_of() {
  grep -m1 '^    inputs: annotations=' <<< "$1" \
    | sed -E 's/.*rules_with_branch_return_annotations=([0-9]+).*/\1/'
}

absorbed_rules_of() {
  grep "^    ✅ would absorb " <<< "$1" | sed -E "s/.*'([^']+)'.*/\1/" | paste -sd, -
}

echo "ENGINE-UNIVERSAL-SERVICES.17 slice 3 — the guard census, run through the real planner"
echo

check D1 "systemverilog: would_absorb / would_refuse" \
  "$(verdict_of "$SV_REPORT")" "2/0"

check D2 "systemverilog: the rules that reach a plan" \
  "$(absorbed_rules_of "$SV_REPORT")" "casting_type,property_expr"

check D3 "systemverilog: left-recursive rule rows, on the clone" \
  "$(rows_of "$SV_REPORT")" "28->0"

# ⛔ D4/D6 ARE THE FALSIFIABILITY PAIR, not decoration. `compose_route_template` returns "nothing to
# compose" when a grammar declares NO annotations, so `would_refuse=0` has two readings — the chain
# composes, or there was nothing to compose. These two numbers separate them, and they are also the
# whole root cause of the D1-vs-D5 split: the same language, one hand-annotated view (1069 rules)
# and one LRM-generated view whose 148 KB body carries essentially none (21, and they are the
# wrapper's own entrypoint rules).
check D4 "systemverilog: rules with branch return annotations" \
  "$(annotated_rules_of "$SV_REPORT")" "1069"

check D5 "wrapper: would_absorb / would_refuse" \
  "$(verdict_of "$WRAPPER_REPORT")" "0/13"

# ⛔ Scoped to `would still REFUSE`, i.e. the DRY-RUN section alone. The first draft of this case
# grepped the whole report for the refusal text and counted 16 — the dry run's 13 plus the THREE
# `⛔ REFUSED` lines the real pass already prints for the candidates it admits today. Two different
# populations summed into one number that looked plausible; the bank caught it because the
# expectation was declared before the number was read.
check D5b "wrapper: dry-run refusals that are 'no return annotation'" \
  "$(grep 'would still REFUSE' <<< "$WRAPPER_REPORT" | grep -c 'declares no return annotation')" "13"

check D6 "wrapper: rules with branch return annotations" \
  "$(annotated_rules_of "$WRAPPER_REPORT")" "21"

check D7 "ebnf: would_absorb / would_refuse" \
  "$(verdict_of "$EBNF_REPORT")" "0/0"

# ⛔ D8 — the instrument is OPT-IN, and that is what makes every other reader of this report
# unaffected. A dry-run section appearing without the flag would mean the report now applies plans
# to a clone on every invocation, which is a different tool than the one slice 2's bank pins.
check D8 "the default report carries no dry-run section" \
  "$(grep -c 'GUARD DRY-RUN' <<< "$SV_PLAIN")" "0"

echo
if [ "$fail" -eq 0 ]; then
  echo "GUARD-DRY-RUN: 9/9 as declared — option (iii)'s plan-stage reachability is unchanged."
else
  echo "GUARD-DRY-RUN: MISMATCH — what option (iii) unlocks has moved." >&2
  echo "  Do NOT edit the expectation to match; re-adjudicate in ENGINE-UNIVERSAL-SERVICES.17." >&2
fi
exit "$fail"

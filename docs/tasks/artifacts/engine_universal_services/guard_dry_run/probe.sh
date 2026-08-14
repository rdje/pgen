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
# ⭐⭐ SLICE 7 ADDED THE EMISSION, so the cases below pin what the planner now SYNTHESIZES:
#
#   D9  SV: the guard chains            guard_chains=3 guard_rules=6
#   D10 SV: which chains, with POSITIONS casting_type ×2 [loop], property_expr ×1 [loop+trailing] —
#                                      and the split is slice 5's per-candidate `seed:` verdict
#                                      reproduced by a DIFFERENT code path
#   D11 ⛔⛔ the SHIPPED path emits NONE  indirect_guard_chains=0 on all three grammars. This is the
#                                      check on "a starvation-safe candidate has no site to guard",
#                                      which is otherwise an argument.
#   D12 ⛔ the census UNDER-COUNTS       `casting_type` reports `variants=1` and the planner emits
#       the CHAINS                     TWO chains. `guard_variants` keys on a residual BYTE SET and
#                                      the shipped guard is STRUCTURAL: `tick lparen
#                                      constant_expression rparen` and `tick lparen expression
#                                      rparen` share a FIRST set and are different sub-parses. The
#                                      census figure is a LOWER BOUND on the chain count.
#   D13 ⛔⛔ it UNDER-COUNTS the CLONES   `hops=` is the SHORTEST transparency distance; `chain=` is
#       too, for a DIFFERENT reason    every rule on any transparent path. They disagree wherever
#                                      transparency BRANCHES — SV's dialect twins do, so 5 rules get
#                                      cloned where `hops=3` reads as 4. Pinned as a COUNT of
#                                      disagreeing sites (6 SV / 5 wrapper), not as one example.
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
# ⛔⛔ WHAT NONE OF IT CLAIMS — AND THE BOUND MOVED AT SLICE 7, INWARD. Through slice 6 the dry run
# emitted no guard at all, so the grammar it built was the one `.13` slice 5 measured as a REGRESSION
# (`initial k = 8'(1);` ACCEPT -> REJECT when `casting_type` is eliminated unguarded). It now
# synthesizes the guarded clone chains (D9-D13b). What is STILL unclaimed is every parse: nothing here
# generates a parser or runs an input, so these cases pin plan-stage outcomes and the SHAPE of the
# emission, and nothing about what the rewritten grammar accepts or rejects.
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
checks=0

# check <case-label> <what> <got> <want>
#
# ⛔ `checks` is incremented HERE and the summary prints `$checks/$checks`, so the headline total is
# DERIVED from the cases that ran. `.17` slice 6b measured the alternative failing three times in one
# session: a hand-typed total is a number nothing re-derives, so it rots the moment a case is added
# (`docs/DERIVED_STATE_CONTAINMENT.md` R1/R3).
check() {
  local label="$1" what="$2" got="$3" want="$4"
  checks=$((checks + 1))
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

# ---- `.17` slice 7 — the EMISSION.

# `guard_chains=N guard_rules=M` reduced to `N/M`.
guards_of() {
  grep -m1 '^    would_absorb=' <<< "$1" \
    | sed -E 's/.*guard_chains=([0-9]+) guard_rules=([0-9]+).*/\1\/\2/'
}

# Each synthesized chain as `<guarded rule>[<positions>]`, in emission order.
chains_of() {
  grep '^    🛡  guard ' <<< "$1" \
    | sed -E "s/.*guard '([^']+)' \[([^]]+)\].*/\1[\2]/" | paste -sd, -
}

# The SHIPPED pass's own guard counter, from the header line every invocation prints.
shipped_guards_of() {
  grep -m1 '^indirect_eliminated_base_rules=' <<< "$1" \
    | sed -E 's/.*indirect_guard_chains=([0-9]+).*/\1/'
}

check D9 "systemverilog: guard_chains / guard_rules" \
  "$(guards_of "$SV_REPORT")" "3/6"

# ⛔ THE POSITIONS ARE THE CROSS-CHECK, not decoration. `casting_type` is `seed_routes=0/10` and
# `property_expr` `78/80` on the `seed:` line — a `GuardAssessment` verdict. These strings are what
# the EMITTER independently decided to write into the grammar. Two code paths, one answer; if they
# ever disagree, one of them is wrong and this case names which candidate.
check D10 "systemverilog: the chains, with their guard positions" \
  "$(chains_of "$SV_REPORT")" \
  "casting_type_lr_guard0[loop],casting_type_lr_guard1[loop],property_expr_lr_guard0[loop+trailing]"

# ⛔⛔ D11 IS THE ONE THAT PROTECTS EVERY SHIPPED PARSER. The guard planner runs unconditionally; what
# keeps it silent on the shipped path is that the admission criterion is "no surviving starvation
# site" and a guard exists only for one. A non-zero here means a parser-behaviour change landed —
# owed a two-sided repro ratchet and a corpus re-measure — whether or not anyone meant it.
check D11 "systemverilog: the SHIPPED pass synthesizes no guard" \
  "$(shipped_guards_of "$SV_PLAIN")" "0"
check D11b "wrapper + ebnf: the SHIPPED pass synthesizes no guard" \
  "$(shipped_guards_of "$WRAPPER_REPORT"),$(shipped_guards_of "$EBNF_REPORT")" "0,0"

# ⛔ D12 — the census figure is a LOWER BOUND on the chain count, pinned as a PAIR so the gap is the
# measurement. `guard_variants` keys on the residual's FIRST BYTE SET, which is what the DEAD byte-
# test form would have compared; the shipped guard is a structural sub-parse, so two residuals with
# one FIRST set are two rules. Read either number alone and option (iii)'s rule-name price is wrong.
# ⛔ Both halves anchor STRUCTURALLY, per `.17` slice 5's own lesson: the census side on the
# `[candidate] casting_type ` row (the trailing space excludes `casting_type_sv_2017` and friends)
# and the emission side on the `🛡  guard '` marker in its only legal position — NOT on the bare rule
# name, which also appears inside every `rules:` list.
check D12 "casting_type: census variants vs chains the planner emits" \
  "$(grep -A2 "^\[candidate\] casting_type " <<< "$SV_REPORT" | grep -m1 -oE 'variants=[0-9]+' | cut -d= -f2),$(grep -c "^    🛡  guard 'casting_type_lr_guard" <<< "$SV_REPORT")" \
  "1,2"

# ⛔⛔ D13 — THE SECOND UNDER-PRICE, and it is the same shape as D12 one field over. `hops=` is the
# SHORTEST transparency distance; `chain=` is every rule on ANY transparent path, and the clone price
# is the chain. They disagree wherever transparency BRANCHES, which SystemVerilog's dialect twins do:
# `primary` reaches `cast` through `primary_sv_2017` AND `primary_sv_2023`, so five rules get cloned
# where `hops=3` reads as four. Pinned as a COUNT of disagreeing sites rather than as a rule name, so
# it stays a measurement of the population rather than of one example.
#
# The extraction is per-site by construction: `chain=` closes the `[guard=…]` bracket group, one per
# site line and nowhere else (`.17` slice 5's anchoring rule).
branching_sites_of() {
  grep -oE 'hops=[0-9]+ chain=[^]]+' <<< "$1" | awk -F'[= ]' '
    { hops = $2; n = split($0, parts, "chain="); c = parts[2];
      links = split(c, _, ">");
      if (hops != links - 1) branching++ }
    END { print branching + 0 }'
}
check D13 "systemverilog: sites where the chain BRANCHES (hops < clones)" \
  "$(branching_sites_of "$SV_PLAIN")" "6"
check D13b "wrapper: sites where the chain BRANCHES (hops < clones)" \
  "$(branching_sites_of "$WRAPPER_REPORT")" "5"

echo
if [ "$fail" -eq 0 ]; then
  echo "GUARD-DRY-RUN: $checks/$checks as declared — option (iii)'s plan-stage reachability and the shape it emits are unchanged."
else
  echo "GUARD-DRY-RUN: MISMATCH ($checks case(s) checked) — what option (iii) unlocks has moved." >&2
  echo "  Do NOT edit the expectation to match; re-adjudicate in ENGINE-UNIVERSAL-SERVICES.17." >&2
fi
exit "$fail"

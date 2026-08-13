#!/usr/bin/env bash
# ENGINE-UNIVERSAL-SERVICES.17 slice 2 — the GUARD-FEASIBILITY census, pinned.
#
# WHAT IT MEASURES. Slice 1 left option (iii) — a call-site follow-restriction guard synthesized
# onto the sheared clone — as the front-runner with ONE open question, and the leaf required it be
# answered against the SHIPPED grammar rather than a synthetic:
#
#   "is the holder's residual FIRST set statically computable at each of the 28 rows, and does the
#    guard stay sound when that residual is nullable or when two holders of the same clone disagree?"
#
# `--report-indirect-lr-plan` now answers all three from the shipped grammars, and this bank pins
# the answers so a later change cannot move them silently:
#
#   C1  SystemVerilog headline           guard-feasible 16/28, against starvation-safe 0/28
#   C2  the cast/call knot's DOMINATOR   constant_primary  FEASIBLE  variants=1  max_hops=1
#   C3  the SVA property knot's DOMINATOR property_expr    FEASIBLE  variants=1  max_hops=0
#   C4  FIRST is computable EVERYWHERE   undecidable=0 over every surviving site, both grammars
#   C5  disjointness is DEAD in SV       no_competition=0 — `trivia` is nullable and leads every
#                                        token, so every FIRST set contains `/` and no two are
#                                        ever disjoint (grammars/systemverilog.ebnf:619)
#   C6  the LRM wrapper agrees           guard-feasible 13/18, against starvation-safe 3/18
#   C7  ⛔ the qualifier on all of it     EVERY guard byte test is OVER-approximated (157/157) —
#                                        sound, but not a PROOF the knot closes
#
# ⭐ GROUND TRUTH (`feedback_instrument_needs_ground_truth`). Every case DECLARES the value it
# requires and this script compares against it, so the bank is self-checking: any disagreement is a
# non-zero exit naming the case. C4 and C5 are the two cases that must stay at ZERO, so the bank
# carries expectations in both directions rather than only "the number I saw".
#
# ⛔ IF A CASE FAILS, RE-ADJUDICATE — DO NOT EDIT THE EXPECTATION. These numbers are `.17`'s
# pricing of option (iii); a change in them is a change in what the fix costs, and slice 3 reads
# them as its input. C5 in particular is a claim about the grammar's LAYOUT model, not about a
# count: if it ever becomes non-zero, the disjointness refinement has become live and more of the
# 28 rows can be discharged with no guard at all.
#
# ⛔ IT TOUCHES NOTHING. `--report-indirect-lr-plan` is pure analysis on the post-elimination
# gen-AST — it plans nothing, writes no grammar byte, acquires no scratch-slot restore obligation
# (`.13` slice 4b's class) and is always rc 0.
#
# HOW TO RUN (from the repository root):
#   bash docs/tasks/artifacts/engine_universal_services/guard_feasibility/probe.sh
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

SV_REPORT="$("$PIPELINE" grammars/systemverilog.ebnf --report-indirect-lr-plan 2>/dev/null)"
WRAPPER_REPORT="$("$PIPELINE" grammars/systemverilog_lrm_profiled_wrapper.ebnf \
  --report-indirect-lr-plan 2>/dev/null)"

fail=0

# check <case-label> <what> <got> <want>
check() {
  local label="$1" what="$2" got="$3" want="$4"
  if [ "$got" = "$want" ]; then
    printf '  %-4s %-52s => %-34s ✅\n' "$label" "$what" "$got"
  else
    printf '  %-4s %-52s => %-34s ⛔ want %s\n' "$label" "$what" "$got" "$want"
    fail=1
  fi
}

# The `guard:` line of one candidate, reduced to `VERDICT variants=N max_hops=M`.
guard_line_of() {
  awk -v rule="$2" '
    $1 == "[candidate]" { current = $2 }
    current == rule && $1 == "guard:" {
      verdict = $2
      for (i = 3; i <= NF; i++) {
        if ($i ~ /^variants=/) variants = $i
        if ($i ~ /^max_hops=/) hops = $i
      }
      print verdict, variants, hops
      exit
    }' <<< "$1"
}

# One bucket of the census line, or 0 when the bucket is absent (an absent bucket IS a zero here —
# the line only prints the buckets that occurred).
census_bucket_of() {
  local line bucket
  line="$(grep -m1 '^guard-verdict census' <<< "$1" || true)"
  bucket="$(grep -oE "$2=[0-9]+" <<< "$line" | head -1 || true)"
  printf '%s' "${bucket:-$2=0}"
}

echo "ENGINE-UNIVERSAL-SERVICES.17 slice 2 — guard-feasibility census, pinned to the shipped grammars"
echo

check C1 "systemverilog: starvation-safe / guard-feasible" \
  "$(grep -m1 '^starvation-safe candidates:' <<< "$SV_REPORT" | awk '{print $3}') / $(grep -m1 '^guard-feasible candidates:' <<< "$SV_REPORT" | awk '{print $3}')" \
  "0/28 / 16/28"

check C2 "systemverilog: cast/call dominant base 'constant_primary'" \
  "$(guard_line_of "$SV_REPORT" constant_primary)" \
  "FEASIBLE variants=1 max_hops=1"

check C3 "systemverilog: SVA property dominant base 'property_expr'" \
  "$(guard_line_of "$SV_REPORT" property_expr)" \
  "FEASIBLE variants=1 max_hops=0"

check C4a "systemverilog: residual FIRST computable at every site" \
  "$(census_bucket_of "$SV_REPORT" undecidable)" "undecidable=0"
check C4b "wrapper: residual FIRST computable at every site" \
  "$(census_bucket_of "$WRAPPER_REPORT" undecidable)" "undecidable=0"

check C5a "systemverilog: FIRST-disjointness refinement is dead (trivia)" \
  "$(census_bucket_of "$SV_REPORT" no_competition)" "no_competition=0"
check C5b "wrapper: FIRST-disjointness refinement is dead (trivia)" \
  "$(census_bucket_of "$WRAPPER_REPORT" no_competition)" "no_competition=0"

check C6 "wrapper: starvation-safe / guard-feasible" \
  "$(grep -m1 '^starvation-safe candidates:' <<< "$WRAPPER_REPORT" | awk '{print $3}') / $(grep -m1 '^guard-feasible candidates:' <<< "$WRAPPER_REPORT" | awk '{print $3}')" \
  "3/18 / 13/18"

# C7 — ⛔ THE QUALIFIER ON EVERY OTHER CASE. `~` marks an OVER-APPROXIMATED byte set: the guard is
# still sound (it can only fail to fire, never over-accept) but it is NOT a proof that the knot
# closes. Measured: EVERY site on both grammars is approximated — not one exact guard exists — for
# two compounding reasons, a multi-element residual is not byte-decided by construction, and SV's
# nullable `trivia` puts `/` in every token's FIRST set. ⇒ slice 3 cannot treat the byte test as the
# cheap win; a trivia-aware STRUCTURAL lookahead is effectively mandatory on this grammar.
# ⛔ Pinned ABSOLUTELY, and read from the UNCAPPED report. The first draft of this case compared
# `$approx/$sites` against `$sites/$sites` — an expectation derived from the very number it was
# checking, so it would have passed on 0/0 and on any future count alike. A tautological assertion
# is the `a-check-whose-inputs-all-pass-has-not-been-tested` shape, in the gate meant to prevent it.
# It also read the DEFAULT report, whose per-candidate site list is capped at 5, so it measured 123
# of the 157 sites and called that "every".
SV_REPORT_ALL="$(PGEN_INDIRECT_LR_DUMP_ALL=1 "$PIPELINE" grammars/systemverilog.ebnf \
  --report-indirect-lr-plan 2>/dev/null)"
sv_sites="$(grep -c 'first=' <<< "$SV_REPORT_ALL" || true)"
sv_approx="$(grep -o 'first=[^]]*' <<< "$SV_REPORT_ALL" | grep -c '~' || true)"
check C7 "systemverilog: OVER-approximated guard byte tests / all sites (uncapped)" \
  "$sv_approx/$sv_sites" "157/157"

echo
if [ "$fail" -eq 0 ]; then
  echo "GUARD-FEASIBILITY-CENSUS: 9/9 as declared — option (iii)'s price on the shipped grammars is unchanged."
else
  echo "GUARD-FEASIBILITY-CENSUS: MISMATCH — option (iii)'s price has moved." >&2
  echo "  Do NOT edit the expectation to match; re-adjudicate in ENGINE-UNIVERSAL-SERVICES.17." >&2
fi
exit "$fail"

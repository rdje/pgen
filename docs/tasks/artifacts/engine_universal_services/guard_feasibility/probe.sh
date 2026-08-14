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
#   C7  ⛔ the qualifier on all of it     EVERY guard byte test is OVER-approximated (129/129) —
#                                        sound, but not a PROOF the knot closes
#                                        ⛔ the DENOMINATOR was 157 until `.17` slice 5 and it was
#                                        WRONG: see the adjudication above the case itself.
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

# ⛔⛔ `.17` SLICE 9 — THE SYSTEMVERILOG REPORTS ARE TAKEN UNDER THE NARROWED ADMISSION, AND EVERY
# DECLARED VALUE BELOW IS UNCHANGED BECAUSE OF IT.
#
# This bank is a CENSUS OF THE KNOTS THE ELIMINATOR HAS NOT ABSORBED — starvation-safe vs
# guard-feasible, the residual FIRST sets, the seed tails. Its subject only exists while the knots
# survive. Slice 9 flipped the shipped admission, so a bare report now arrives with the cast/call and
# property knots ALREADY absorbed and prints `candidates=0`: an empty census, which would read as
# "the price of option (iii) is zero" when it means "option (iii) has shipped".
#
# ⇒ the lever names the population these rows are about. ⭐ The precedent is inside this same file:
# C10 has always passed `--no-eliminate-indirect-left-recursion` to see `ebnf`'s knot before the pass
# eats it, for exactly this reason and with the reason written beside it. This is that pattern
# applied to the two knots that changed hands.
#
# ⛔ The WRAPPER reports deliberately do NOT take the lever: its 13 candidates are refused for a
# missing return annotation under either admission, so its rows measure the same population either
# way — and leaving them on the shipped path is what makes C6/C7b a control on the flip's blast
# radius rather than a copy of the SV rows.
NARROW="--indirect-lr-admit-starvation-safe-only"

SV_REPORT="$("$PIPELINE" grammars/systemverilog.ebnf --report-indirect-lr-plan "$NARROW" 2>/dev/null)"
WRAPPER_REPORT="$("$PIPELINE" grammars/systemverilog_lrm_profiled_wrapper.ebnf \
  --report-indirect-lr-plan 2>/dev/null)"

fail=0
# ⛔ DERIVED, NEVER STORED (`docs/DERIVED_STATE_CONTAINMENT.md` R1/R3). The summary line used to
# carry a HAND-TYPED total, and it was edited three times in one session (9 → 10 → 18) — a number a
# command answers exactly, stored where it can rot. `checks` is now incremented by `check` itself,
# so the headline cannot disagree with the number of cases that actually ran.
checks=0

# check <case-label> <what> <got> <want>
check() {
  local label="$1" what="$2" got="$3" want="$4"
  checks=$((checks + 1))
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
#
# ⛔⛔ AND THE THIRD DRAFT IS THIS ONE, BECAUSE THE SECOND ONE'S DENOMINATOR WAS NEVER THE SITE
# COUNT — `.17` slice 5, found when a NEW report line made the ambiguity visible.
# `grep -c 'first='` counts LINES containing that substring, and the per-candidate summary line
# `guard: … suffix_first=…` contains it too. So `157` was `129` starvation sites **plus 28
# per-candidate lines** — one per candidate, which is exactly the 28 the report iterates. Adding
# slice 5's `seed_first=` line took it to 185 and made the conflation impossible to miss.
# ⭐ THE SUBSTANCE IS UNCHANGED AND WAS NEVER AT RISK: every site is `~` and so is every candidate
# summary, so "not one exact guard exists" held under either count. What was wrong is the number
# quoted for it in TOOLBOX, the leaf, the decision record and this bank.
# ⛔ The expectation below is therefore RE-ADJUDICATED, not edited to fit: 129 = 126 surviving
# sites (`⛔ starved by`) + 3 benign ones (`·  benign site`), cross-checked against a second,
# independent extraction on `— residual '`, and against the surviving-site census 68+29+29 = 126.
# The extraction is now anchored so it cannot drift again: `[guard=` opens exactly one bracket group
# per site line and appears nowhere else, and `~ hops=` is the approximation marker in its ONLY
# position — which also stops a `~` BYTE inside a rendered set from being counted as the marker.
SV_REPORT_ALL="$(PGEN_INDIRECT_LR_DUMP_ALL=1 "$PIPELINE" grammars/systemverilog.ebnf \
  --report-indirect-lr-plan "$NARROW" 2>/dev/null)"
sv_sites="$(grep -o '\[guard=' <<< "$SV_REPORT_ALL" | wc -l | tr -d ' ')"
sv_approx="$(grep -o '~ hops=' <<< "$SV_REPORT_ALL" | wc -l | tr -d ' ')"
check C7 "systemverilog: OVER-approximated guard byte tests / all sites (uncapped)" \
  "$sv_approx/$sv_sites" "129/129"

# C7b — the same qualifier on the OTHER grammar, which the 157-era bank never counted at all. It is
# not redundant with C7: the wrapper carries the same nullable-`trivia` layout tier but a completely
# different rule population, so an exact guard appearing there would refute the structural half of
# C7's explanation while leaving the layout half standing.
WRAPPER_REPORT_ALL="$(PGEN_INDIRECT_LR_DUMP_ALL=1 "$PIPELINE" \
  grammars/systemverilog_lrm_profiled_wrapper.ebnf --report-indirect-lr-plan 2>/dev/null)"
wrapper_sites="$(grep -o '\[guard=' <<< "$WRAPPER_REPORT_ALL" | wc -l | tr -d ' ')"
wrapper_approx="$(grep -o '~ hops=' <<< "$WRAPPER_REPORT_ALL" | wc -l | tr -d ' ')"
check C7b "wrapper: OVER-approximated guard byte tests / all sites (uncapped)" \
  "$wrapper_approx/$wrapper_sites" "77/77"

# ---- `.17` slice 5 — THE SEED TERM (the TRAILING guard position), pinned here because it prices
# option (iii) exactly as C1–C7 price the loop position, and because it is the term the candidate
# ORDERING was forbidden to encode until it existed.
#
# C8 is the census; C9 is the case that makes it a discriminator rather than a restatement. ⛔ If C9
# ever reads the same verdict on both rules, the seed term has stopped separating candidates on one
# knot and the ordering question it was built to answer has no input again.
seed_bucket_of() {
  local line bucket
  line="$(grep -m1 '^seed-verdict census' <<< "$1" || true)"
  bucket="$(grep -oE "$2=[0-9]+" <<< "$line" | head -1 || true)"
  printf '%s' "${bucket:-$2=0}"
}
# The `seed:` line of one candidate, reduced to `seed_routes=N/M`.
seed_routes_of() {
  awk -v rule="$2" '
    $1 == "[candidate]" { current = $2 }
    current == rule && $1 == "seed:" {
      for (i = 2; i <= NF; i++) if ($i ~ /^seed_routes=/) { print $i; exit }
    }' <<< "$1"
}

check C8a "systemverilog: sites that need the TRAILING guard" \
  "$(seed_bucket_of "$SV_REPORT" trailing_guard_required)" "trailing_guard_required=32"
check C8b "systemverilog: sites where the rewrite adds no over-long seed" \
  "$(seed_bucket_of "$SV_REPORT" no_seed_tail)" "no_seed_tail=65"
check C8c "systemverilog: seed FIRST computable at every site" \
  "$(seed_bucket_of "$SV_REPORT" seed_undecidable)" "seed_undecidable=0"
check C8d "systemverilog: candidates needing the trailing guard emitted" \
  "$(grep -m1 '^candidates needing the TRAILING guard' <<< "$SV_REPORT" | awk '{print $7}')" \
  "15/28"

# C9 — ⛔⛔ THE DISCRIMINATOR, and the correction it carries. `.17` slice 4 named
# `casting_type_lr_base` as the base that would carry the sheared `constant_cast` clone and so
# regress `initial k = int'(1);`. Measured here: `casting_type`'s routes CLOSE at `cast` /
# `constant_cast`, each of which has exactly ONE alternative — the cycle-closing one — so the shear
# leaves no clone and no seed (`seed_routes=0/10`). The rule whose base really carries that seed is
# `constant_primary` (`10/10`). Ground truth for the survival half: the guard dry run's own clone
# set contains no `casting_type_lr_seed_cast` and no `casting_type_lr_seed_constant_cast`.
check C9a "systemverilog: 'casting_type' (the driver's pick) owes no trailing guard" \
  "$(seed_routes_of "$SV_REPORT" casting_type)" "seed_routes=0/10"
check C9b "systemverilog: 'constant_primary' carries the seed slice 4 described" \
  "$(seed_routes_of "$SV_REPORT" constant_primary)" "seed_routes=10/10"
check C9c "systemverilog: the eliminator builds no clone for the dropped closers" \
  "$("$PIPELINE" grammars/systemverilog.ebnf --report-indirect-lr-plan "$NARROW" \
      --indirect-lr-plan-guard-dry-run --indirect-lr-plan-json /dev/stdout 2>/dev/null \
    | grep -cE '"casting_type_lr_seed_(cast|constant_cast)"' || true)" "0"
# ⛔ `ebnf` is the grammar whose knot the pass ACTUALLY absorbs today, so a non-zero here would mean
# the shipped rewrite has an unguarded seed starvation in it right now.
EBNF_REPORT="$("$PIPELINE" grammars/ebnf.ebnf --report-indirect-lr-plan \
  --no-eliminate-indirect-left-recursion 2>/dev/null)"
check C10 "ebnf: the knot the pass already absorbs owes no trailing guard" \
  "$(grep -m1 '^candidates needing the TRAILING guard' <<< "$EBNF_REPORT" | awk '{print $7}')" \
  "0/5"

echo
if [ "$fail" -eq 0 ]; then
  echo "GUARD-FEASIBILITY-CENSUS: $checks/$checks as declared — option (iii)'s price on the shipped grammars is unchanged."
else
  echo "GUARD-FEASIBILITY-CENSUS: MISMATCH — option (iii)'s price has moved." >&2
  echo "  Do NOT edit the expectation to match; re-adjudicate in ENGINE-UNIVERSAL-SERVICES.17." >&2
fi
exit "$fail"

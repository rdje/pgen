#!/usr/bin/env bash
# ENGINE-UNIVERSAL-SERVICES.17 slice 1 — the quantifier-policy control bank.
#
# WHAT IT MEASURES. Five controls that pin PGEN's ACTUAL give-back laws, so `.17`'s design
# argues from measurement instead of from the PEG formalism:
#
#   Q1  `( "a" )* "a"`            on "aaa"  MUST REJECT  — the quantifier is POSSESSIVE
#   Q2  `( "a" | "ab" ) "c"`      on "abc"  MUST ACCEPT  — ⛔⛔ SEE THE CORRECTION BELOW
#   Q3  `( "a" | "ab" )`          on "ab"   MUST ACCEPT  — default policy is longest_match
#   Q4  `( "a" &"a" )* "a"`       on "aaa"  MUST ACCEPT  — a stop-guard rescues Q1 …
#       same grammar             on "a"    MUST ACCEPT  — … in both directions
#   Q5  guarded star, no residual on "aaa"  MUST REJECT  — so the guard is CALL-SITE scoped
#   Q5b Q5 with the guard removed on "aaa"  MUST ACCEPT  — Q5's one-difference control
#
# ⛔⛔ CORRECTION TO Q2's READING (2026-08-13, `.17` slice 4 — the VERDICT is unchanged and must
# stay ACCEPT; what was wrong is the MECHANISM slice 1 read out of it). Q2 was recorded as *"the
# choice DOES give back — `"a"` wins, `"c"` fails, the choice retries `"ab"`"*, and that is the
# premise of `.17` slice 1's FINDING 1 and of the decision record it produced. It does not follow:
# Q3 on the very next line establishes that the default policy is `longest_match`, so `"ab"` wins
# OUTRIGHT and `"c"` then matches the single remaining byte. The parse completes on the first and
# only alternative the choice ever kept ⇒ **Q2's ACCEPT is predicted identically with and without a
# give-back, so it discriminates nothing.**
#
# The discriminating shape — one where both alternatives match and only the SHORTER lets the caller
# finish — is `ch := "a" | "ab"` with `scratch := ch "bc"` on "abc", and it **REJECTS**. Measured on
# BOTH oracles in `../guard_effectiveness/` (cases C1/C2/C3), located at
# `ast_based_generator.rs:5037` (a single `best_content` winner slot, losers discarded).
# ⇒ read [[project_pgen_gives_back_at_neither_combinator]], NOT the superseded
# [[project_pgen_gives_back_at_the_choice_but_not_at_the_quantifier]].
#
# ⭐ Q1 / Q3 / Q4 / Q5 / Q5b are UNAFFECTED — each is a genuine one-difference control whose verdict
# separates the hypotheses it was run for. Only Q2 was over-read.
#
# ⭐ GROUND TRUTH (`feedback_instrument_needs_ground_truth`). Every case DECLARES its required
# verdict and this script compares against it, so the bank is self-checking in BOTH directions:
# it carries cases that must accept AND cases that must reject, and any disagreement is a
# non-zero exit naming the case. An instrument that can only report cannot notice that it
# broke; this one refuses.
#
# ⛔ IT DOES NOT TOUCH THE SCRATCH SLOT. Every case is a standalone `.ebnf` driven through
# `ast_pipeline --interpret-parse`, so there is no `grammars/scratch/scratch.ebnf` to restore
# and no git-ignored `generated/scratch_parser.rs` to leave behind. That half-restore is
# exactly what `.13` slice 4b root-caused after slice 3's probe left two gates RED for a day;
# the cheapest fix for that class is not to acquire the obligation.
#
# ⚠️ HONEST BOUND ON THE ORACLE. `--interpret-parse` is authoritative BY VERIFICATION, and its
# one measured hole is un-eliminated left recursion (`.14`). No case here contains any
# recursion at all — they are pure choice / quantifier / lookahead / sequence shapes, which the
# structural combinator suite pins byte-identical to the compile-and-run oracle (35/35). Q1 and
# Q4 were additionally confirmed on the REAL generated parser through the scratch slot; see
# README.md for those two commands and their output.
#
# HOW TO RUN (from the repository root):
#   bash docs/tasks/artifacts/engine_universal_services/quantifier_policy/probe.sh
set -uo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../../../.." && pwd)"
cd "$ROOT" || exit 1

HERE="docs/tasks/artifacts/engine_universal_services/quantifier_policy"
PIPELINE="rust/target/debug/ast_pipeline"
WORK="rust/target/es17_probe"   # repo volume, git-ignored (policy 13: project data stays on the repo volume)

if [ ! -x "$PIPELINE" ]; then
  echo "probe: $PIPELINE not built. Run:" >&2
  echo "  (cd rust && cargo build --features \"generated_parsers ebnf_dual_run\" --bin ast_pipeline)" >&2
  exit 2
fi

# The 1.4 under-featured-binary trap: an under-featured binary errors on stderr and yields an
# EMPTY verdict on stdout, and two empty result sets diff clean. Refuse up front.
if ! scripts/require_ast_pipeline_features.sh "$PIPELINE" ebnf_dual_run >/dev/null 2>&1; then
  scripts/require_ast_pipeline_features.sh "$PIPELINE" ebnf_dual_run >&2
  exit 2
fi

mkdir -p "$WORK"
printf 'aaa' > "$WORK/aaa.txt"
printf 'a'   > "$WORK/a.txt"
printf 'abc' > "$WORK/abc.txt"
printf 'ab'  > "$WORK/ab.txt"

fail=0

# run <case-label> <grammar-basename> <input-basename> <ACCEPT|REJECT>
run() {
  local label="$1" grammar="$2" input="$3" want="$4"
  local line got
  line="$("$PIPELINE" "$HERE/$grammar" --interpret-parse "$WORK/$input" 2>/dev/null | grep '^INTERPRET-PARSE:' || true)"

  if [ -z "$line" ]; then
    printf '  %-6s %-38s %-6s => NO VERDICT LINE  ⛔\n' "$label" "$grammar" "$input"
    fail=1
    return
  fi

  case "$line" in
    *accepted=true*)  got=ACCEPT ;;
    *accepted=false*) got=REJECT ;;
    *)                got=UNPARSEABLE ;;
  esac

  if [ "$got" = "$want" ]; then
    printf '  %-6s %-38s %-6s => %-6s (want %-6s) ✅\n' "$label" "$grammar" "$input" "$got" "$want"
  else
    printf '  %-6s %-38s %-6s => %-6s (want %-6s) ⛔\n' "$label" "$grammar" "$input" "$got" "$want"
    printf '         %s\n' "$line"
    fail=1
  fi
}

echo "ENGINE-UNIVERSAL-SERVICES.17 — quantifier give-back control bank"
echo

run Q1  q1_possessive_star.ebnf              aaa.txt REJECT
run Q2  q2_choice_gives_back.ebnf            abc.txt ACCEPT
run Q3  q3_default_is_longest_match.ebnf     ab.txt  ACCEPT
run Q4a q4_stop_guard_rescues.ebnf           aaa.txt ACCEPT
run Q4b q4_stop_guard_rescues.ebnf           a.txt   ACCEPT
run Q5  q5_guard_must_be_callsite_scoped.ebnf aaa.txt REJECT
run Q5b q5b_unguarded_control.ebnf           aaa.txt ACCEPT

echo
if [ "$fail" -eq 0 ]; then
  echo "QUANTIFIER-POLICY-CONTROLS: 7/7 as declared — the VERDICTS hold as recorded."
  echo "  ⛔ Q2's verdict is not evidence of a give-back — see the CORRECTION in this file's header."
else
  echo "QUANTIFIER-POLICY-CONTROLS: MISMATCH — an engine law recorded in .17 no longer holds." >&2
  echo "  Do NOT adjust the expectations to match; the leaf's design rests on them." >&2
fi
exit "$fail"

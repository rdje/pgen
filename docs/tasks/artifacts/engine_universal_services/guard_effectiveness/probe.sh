#!/usr/bin/env bash
# ENGINE-UNIVERSAL-SERVICES.17 slice 4 — the guard EFFECTIVENESS bank.
#
# WHAT IT MEASURES. Slice 3 handed slice 4 one burden in one sentence: *"the guard's effectiveness —
# that a trivia-aware structural lookahead refuses exactly the fatal iteration on real SystemVerilog
# text — is still unmeasured."* This bank answers it with a three-way one-difference ladder over the
# post-rewrite shape, carrying SystemVerilog's own layout model:
#
#   G0  prim := prim_base ( prim_suffix )*                             no guard      — must STARVE
#   G1  prim := prim_base ( prim_suffix &residual_first_byte )*        byte set      — slice 2's cheap form
#   G2  prim := prim_base ( prim_suffix &( … ) )*                      the residual  — slice 2's mandated form
#   G6  prim := prim_base ( prim_suffix )* &( … )                      TRAILING only — G2's exact complement
#   G3  prim := prim_base ( prim_suffix &( … ) )* &( … )               BOTH          — the shape slice 4 adopts
#   G4  G3 plus a second, residual-free holder of `prim`               — the guard must be CALL-SITE scoped
#   G5  G4 with the trailing guard removed                             — G4's one-difference control
#   G7  G4's holders, guard moved onto a CLONE CHAIN                   — ⭐ `.17` slice 6: the shape the
#                                                                        decision actually specifies
#
# ⭐⭐ G4 ↔ G7 is slice 6's pair, and it closes the hole slice 4 left. G3 has the power and G4 shows
# the damage; NOTHING in this bank had ever expressed the combination the decision names — the guard
# on a clone reached only from the residual-bearing holder. G7 is that shape: same two holders, same
# seven inputs, and the only difference is WHERE the two lookaheads live. It must accept all six
# starvation rows AND `e7`, the row G4 rejects. It is also the rule-for-rule TARGET the eliminator's
# guard planner has to synthesize (`.17` slice 7), so a later planner can be checked against a
# measured grammar instead of against a design note.
#
# ⭐⭐ G2 vs G6 is the slice's second finding: the two guard POSITIONS close DISJOINT starvations.
# Per-iteration stops the LOOP at the right count and cannot touch an over-long SEED (e5, where the
# loop runs zero times); trailing refuses an over-long SEED and cannot touch the LOOP (by then the
# possessive `*` has committed to the maximum count, with no give-back to a shorter one). Only G3
# accepts all six. ⇒ ONE clone chain carrying TWO lookaheads, not two chains.
#
# and three controls that re-adjudicate an engine law `.17` slice 1 recorded and slice 2/3 reasoned
# from:
#
#   C1  ch := "a" | "ab"  ·  scratch := ch "bc"   on "abc"  must REJECT — the choice does NOT give back
#   C2  C1 under @branch_policy: ordered          on "abc"  must ACCEPT — C1's one-difference control
#   C3  slice 1's Q2, as a rule                   on "abc"  must ACCEPT — reproduced, and shown non-discriminating
#
# ⭐ GROUND TRUTH (`feedback_instrument_needs_ground_truth`). Every case DECLARES its verdict below
# and the script compares against it, so the bank is self-checking in BOTH directions — both an
# ACCEPT and a REJECT population — and any disagreement is a non-zero exit naming the case. ⛔ Do NOT
# adjust an expectation to match a new measurement: `.17`'s design rests on these exact rows, and a
# bank that edits its own expectations is a bank that cannot notice it broke.
# ⛔ THE SPLIT IS PRINTED, NOT WRITTEN HERE (`docs/DERIVED_STATE_CONTAINMENT.md` R1/R3). This comment
# used to state it — "19 cases must ACCEPT and 18 must REJECT" — and BOTH numbers were wrong for the
# 37-row bank they described (it was 21/16). Nothing read them, which is exactly why they rotted. The
# run now derives and prints the split, so the only place it appears is the place it is measured, and
# a row declaring neither verdict fails the bank instead of being counted as nothing.
#
# ⛔ G4 and G5 are run on `e1` and `e7` ONLY, and that is deliberate. Their second alternative
# (`kw_k eq prim semi`) absorbs `e2`–`e6` on its own, so those rows would pass under BOTH hypotheses
# and prove nothing about scoping — the very defect this bank's C3 exists to name in slice 1's Q2.
# A case that does not discriminate is left out rather than run and over-read.
#
# ⛔⛔ BOTH ORACLES, ALWAYS — this is `.13` slice 3's recorded lesson, in this exact rule family.
# A first draft of that measurement ran the interpreter alone and got a table wrong in BOTH
# directions; a single-oracle run here does not under-report, it MISREPORTS. GEN is the real
# generated parser through the scratch slot (authoritative BY CONSTRUCTION); INTERP is
# `--interpret-parse` (authoritative BY VERIFICATION only, with `.14` the tracked divergence).
# A row where they disagree is reported as DIVERGE and fails the bank.
#
# ⛔ SCRATCH-SLOT OBLIGATION. The GEN arm loads each grammar into `grammars/scratch/scratch.ebnf`,
# so this script acquires the obligation `.13` slice 4b root-caused: `generated/scratch_parser.rs`
# is git-ignored, so restoring the fixture WITHOUT regenerating leaves a clean-looking tree and two
# RED gates. `restore_scratch` therefore checks out AND regenerates, on ANY exit path.
#
# HOW TO RUN (from the repository root):
#   bash docs/tasks/artifacts/engine_universal_services/guard_effectiveness/probe.sh
#   bash docs/tasks/artifacts/engine_universal_services/guard_effectiveness/probe.sh --interp-only
#
# `--interp-only` skips the scratch cycle (~6 min per grammar) for a fast re-read of the INTERP
# column. It can never be the basis of a claim about the engine on its own.
set -uo pipefail

HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT="$(cd "$HERE/../../../../.." && pwd)"
cd "$ROOT" || exit 2

AST_PIPELINE="rust/target/debug/ast_pipeline"
PROBE_BIN="rust/target/debug/parseability_probe"
SCRATCH="grammars/scratch/scratch.ebnf"
WORK="rust/target/es17_guard_effectiveness"   # repo volume, git-ignored (policy 13)
INTERP_ONLY=0
[[ "${1:-}" == "--interp-only" ]] && INTERP_ONLY=1

[[ -x "$AST_PIPELINE" ]] || {
  echo "probe: missing $AST_PIPELINE — build it:" >&2
  echo "  (cd rust && cargo build --features \"generated_parsers ebnf_dual_run\" --bin ast_pipeline)" >&2
  exit 2
}
# The under-featured-binary trap: an under-featured binary errors on stderr and yields an EMPTY
# verdict on stdout, and two empty result sets diff clean. Refuse up front.
if ! scripts/require_ast_pipeline_features.sh "$AST_PIPELINE" ebnf_dual_run >/dev/null 2>&1; then
  scripts/require_ast_pipeline_features.sh "$AST_PIPELINE" ebnf_dual_run >&2
  exit 2
fi
if [[ $INTERP_ONLY -eq 0 ]]; then
  [[ -x "$PROBE_BIN" ]] || {
    echo "probe: missing $PROBE_BIN — build it:" >&2
    echo "  (cd rust && cargo build --features generated_parsers --bin parseability_probe)" >&2
    exit 2
  }
fi

mkdir -p "$WORK" || exit 2

# ---- the inputs. `k = … ;` is the enclosing statement that makes the holder MANDATORY; without it
# a shorter parse hides the starvation. Written with printf so the tick needs no shell escaping.
printf 'k = n%s(n);'          "'"     > "$WORK/e1.txt"   # the starvation: the holder needs its residual
printf 'k = n%s(n)%s(n);'     "'" "'" > "$WORK/e2.txt"   # the chain must still take ONE iteration
printf 'k = n%s(n)/*c*/;'     "'"     > "$WORK/e3.txt"   # ⭐ E1 with a comment AT the guard boundary
printf 'k = n%s(n)/*c*/%s(n);' "'" "'" > "$WORK/e4.txt"  # E2 with a comment AT the guard boundary
printf 'k = t%s(n);'          "'"     > "$WORK/e5.txt"   # ⛔ the seed clone starves the holder: a CHOICE, not the `*`
printf 'k = t%s(n)%s(n);'     "'" "'" > "$WORK/e6.txt"   # seeded chain plus one iteration
printf 'k = n;'                       > "$WORK/e7.txt"   # the RESIDUAL-FREE holder's input (G4/G5 only)
printf 'abc'                          > "$WORK/abc.txt"  # the give-back controls

# ---- the case table: <grammar> <input> <ACCEPT|REJECT> <one-line note>
CASES=(
  "g0_unguarded          e1  REJECT  the starvation, reproduced post-rewrite"
  "g0_unguarded          e2  REJECT  and it survives a longer chain"
  "g0_unguarded          e3  REJECT  and a comment does not change it"
  "g0_unguarded          e4  REJECT  nor does a comment mid-chain"
  "g0_unguarded          e5  REJECT  ⛔ starved by the SEED clone via a choice, not by the loop"
  "g0_unguarded          e6  REJECT  seeded chain starves too"

  "g1_byte_guard         e1  ACCEPT  the cheap guard closes the plain starvation"
  "g1_byte_guard         e2  ACCEPT  and does not over-refuse the chain"
  "g1_byte_guard         e3  REJECT  ⛔⛔ THE SLIP: '/' is in FIRST(residual), so the guard passes on a comment"
  "g1_byte_guard         e4  ACCEPT  a comment mid-chain is survivable (the iteration itself fails)"
  "g1_byte_guard         e5  REJECT  ⛔ the choice starvation is OUT OF REACH of any guard on the star"
  "g1_byte_guard         e6  ACCEPT  seeded chain closes"

  "g2_structural_guard   e1  ACCEPT  the structural guard closes the plain starvation"
  "g2_structural_guard   e2  ACCEPT  and does not over-refuse the chain"
  "g2_structural_guard   e3  ACCEPT  ⭐⭐ THE DISCRIMINATOR: trivia-aware, so the comment does not slip it"
  "g2_structural_guard   e4  ACCEPT  and a comment mid-chain is still absorbed"
  "g2_structural_guard   e5  REJECT  ⛔⛔ (iii) IS NOT SUFFICIENT — this row is the whole finding"
  "g2_structural_guard   e6  ACCEPT  seeded chain closes"

  "g6_trailing_guard_alone  e1  REJECT  the trailing guard cannot undo a loop that already committed"
  "g6_trailing_guard_alone  e2  REJECT  nor a longer one"
  "g6_trailing_guard_alone  e3  REJECT  nor one with a comment"
  "g6_trailing_guard_alone  e4  REJECT  nor a comment mid-chain"
  "g6_trailing_guard_alone  e5  ACCEPT  ⭐⭐ but it DOES refuse the over-long SEED — G2's exact complement"
  "g6_trailing_guard_alone  e6  REJECT  seeded chain still starves without the per-iteration guard"

  "g3_trailing_guard     e1  ACCEPT  both positions: the loop starvation closes"
  "g3_trailing_guard     e2  ACCEPT  and the chain is not over-refused"
  "g3_trailing_guard     e3  ACCEPT  and the comment does not slip it"
  "g3_trailing_guard     e4  ACCEPT  and a comment mid-chain is absorbed"
  "g3_trailing_guard     e5  ACCEPT  ⭐⭐ AND the seed starvation closes — the only rung that gets all six"
  "g3_trailing_guard     e6  ACCEPT  seeded chain closes"

  "g4_trailing_guard_needs_a_clone  e1  ACCEPT  the residual-BEARING holder is unharmed"
  "g4_trailing_guard_needs_a_clone  e7  REJECT  ⛔⛔ but the residual-FREE holder breaks ⇒ the guard is CALL-SITE scoped"
  "g5_trailing_guard_clone_control  e1  ACCEPT  G4's one-difference control"
  "g5_trailing_guard_clone_control  e7  ACCEPT  and it parses what G4 refused ⇒ G4's REJECT is the guard, not the shape"

  "g7_guarded_clone_chain  e1  ACCEPT  ⭐⭐ the CALL-SITE-SCOPED shape: the guarded chain closes the loop starvation"
  "g7_guarded_clone_chain  e2  ACCEPT  and does not over-refuse the chain"
  "g7_guarded_clone_chain  e3  ACCEPT  and the comment does not slip it"
  "g7_guarded_clone_chain  e4  ACCEPT  and a comment mid-chain is absorbed"
  "g7_guarded_clone_chain  e5  ACCEPT  and the SEED starvation closes — the clone's own choice falls back to kw"
  "g7_guarded_clone_chain  e6  ACCEPT  seeded chain closes"
  "g7_guarded_clone_chain  e7  ACCEPT  ⭐⭐ AND the residual-FREE holder is UNHARMED — the row G4 REJECTS"

  "c1_choice_never_gives_back  abc  REJECT ⛔⛔ refutes slice 1 FINDING 1 — the choice does NOT give back"
  "c2_ordered_control          abc  ACCEPT  C1's one-difference control: the parse EXISTS, the engine refused to reach it"
  "c3_slice1_q2_reproduced     abc  ACCEPT  slice 1's Q2 verdict, and it is consistent with NO give-back"
)

# ⭐ Restore the scratch slot on ANY exit, including failure or Ctrl-C, and REGENERATE — the fixture
# alone is half the slot (`.13` slice 4b). Skipped when the GEN arm never ran.
restore_scratch() {
  [[ $INTERP_ONLY -eq 1 ]] && return 0
  [[ -n "${GEN_ARM_RAN:-}" ]] || return 0
  git checkout -- "$SCRATCH" 2>/dev/null || return 0
  make -C rust SHELL=/bin/bash focus_scratch >"$WORK/restore_focus.log" 2>&1 || {
    echo "⛔ scratch slot RESTORE-REGENERATE failed — generated/scratch_parser.rs still holds the" >&2
    echo "   last probe's grammar. Re-run by hand before committing anything:" >&2
    echo "     make -C rust SHELL=/bin/bash focus_scratch    # log: $WORK/restore_focus.log" >&2
  }
}
trap restore_scratch EXIT

interp_verdict() {  # $1 = grammar file, $2 = input file
  local line
  line="$("$AST_PIPELINE" "$1" --interpret-parse "$2" 2>/dev/null | grep -m1 '^INTERPRET-PARSE:')"
  [[ -z "$line" ]] && { echo "NOVERDICT"; return; }
  [[ "$line" == *"accepted=true"* ]] && echo "ACCEPT" || echo "REJECT"
}

gen_verdict() {  # $1 = input file
  local out
  out="$("$PROBE_BIN" --parse scratch "$1" 2>&1)"
  [[ "$out" == *"parse_full passed"* ]] && echo "ACCEPT" || echo "REJECT"
}

load_scratch() {  # $1 = grammar basename
  GEN_ARM_RAN=1
  cp "$HERE/$1.ebnf" "$SCRATCH" || return 1
  make -C rust SHELL=/bin/bash focus_scratch >"$WORK/$1.focus.log" 2>&1 || {
    echo "focus_scratch FAILED for $1 — see $WORK/$1.focus.log" >&2; return 1; }
  (cd rust && cargo build --features generated_parsers --bin parseability_probe) \
    >"$WORK/$1.build.log" 2>&1 || {
    echo "probe rebuild FAILED for $1 — see $WORK/$1.build.log" >&2; return 1; }
}

echo "ENGINE-UNIVERSAL-SERVICES.17 slice 4 — guard effectiveness bank"
[[ $INTERP_ONLY -eq 1 ]] && echo "⚠️  --interp-only: the GEN column is SKIPPED and no claim about the engine may rest on this run."
echo
printf '  %-26s %-4s %-8s %-8s %-8s %s\n' GRAMMAR IN GEN INTERP WANT ''

fail=0
loaded=""
for spec in "${CASES[@]}"; do
  read -r grammar input want note <<<"$spec"

  if [[ $INTERP_ONLY -eq 0 && "$loaded" != "$grammar" ]]; then
    load_scratch "$grammar" || exit 2
    loaded="$grammar"
  fi

  interp="$(interp_verdict "$HERE/$grammar.ebnf" "$WORK/$input.txt")"
  if [[ $INTERP_ONLY -eq 1 ]]; then gen="(skip)"; else gen="$(gen_verdict "$WORK/$input.txt")"; fi

  mark="✅"
  if [[ "$interp" != "$want" ]]; then mark="⛔"; fail=1; fi
  if [[ $INTERP_ONLY -eq 0 ]]; then
    if [[ "$gen" != "$want" ]]; then mark="⛔"; fail=1; fi
    if [[ "$gen" != "$interp" ]]; then mark="⛔ DIVERGE(.14)"; fail=1; fi
  fi

  printf '  %-26s %-4s %-8s %-8s %-8s %s  %s\n' "$grammar" "$input" "$gen" "$interp" "$want" "$mark" "$note"
done

echo
# ⛔ DERIVED, never stored: the total and the ACCEPT/REJECT split are counted from the CASES table at
# run time. A prose count of a table is a claim with no gate behind it (`.17` slice 6).
# ⭐ The two guards below are PLANT-PROVEN, not asserted: a row with verdict `MAYBE` exits rc 1 with
# `45 rows — 28 must ACCEPT, 16 must REJECT` and the malformed-table message.
total=${#CASES[@]}
want_accept=0
want_reject=0
for spec in "${CASES[@]}"; do
  read -r _g _i w _rest <<<"$spec"
  [[ "$w" == ACCEPT ]] && want_accept=$((want_accept + 1))
  [[ "$w" == REJECT ]] && want_reject=$((want_reject + 1))
done
printf 'ground truth, both directions: %d rows — %d must ACCEPT, %d must REJECT\n' \
  "$total" "$want_accept" "$want_reject"
if [[ $((want_accept + want_reject)) -ne $total ]]; then
  echo "⛔ a CASES row declares neither ACCEPT nor REJECT — the table is malformed" >&2
  fail=1
fi
if [[ $want_accept -eq 0 || $want_reject -eq 0 ]]; then
  echo "⛔ the bank has lost one of its two directions — a one-sided bank proves nothing" >&2
  fail=1
fi
if [[ $fail -eq 0 ]]; then
  echo "GUARD-EFFECTIVENESS: $total/$total as declared — the byte guard slips on trivia (E3), the two"
  echo "  guard POSITIONS close disjoint starvations (G2 vs G6), only BOTH close all six (G3), the"
  echo "  guard is call-site scoped (G4 vs G5), a CLONE CHAIN gets both (G7), and the choice never"
  echo "  gives back (C1 vs C2)."
else
  echo "GUARD-EFFECTIVENESS: MISMATCH — a row disagrees with the verdict .17 slice 4 recorded." >&2
  echo "  Do NOT adjust the expectations to match; the leaf's decision rests on them." >&2
fi
exit "$fail"

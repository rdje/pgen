#!/usr/bin/env bash
# SV-CORPUS-GRAD.13e.10 (c4) SIZING — what a `@profiles` gate actually does, measured before the
# grammar is touched.
#
# WHY THIS FILE EXISTS
# --------------------
# (c) confirmed 17 `verilog_2005` over-acceptances and (c3) pinned them. (c4) is the grammar fix,
# and the leaf's own instruction is SIZE IT FIRST. Sizing turned up two things that would have made
# the obvious fix wrong, and both are measurements rather than readings:
#
#   1. ⛔ A `@profiles` GATE INVERTED EVERY NEGATIVE LOOKAHEAD THAT REFERENCED THE GATED RULE.
#      `!X` means "refuse if X matches here". Gate X out of a profile and X matched nothing, so
#      `!X` succeeded VACUOUSLY — the STRICT profile became MORE permissive at that site, which is
#      exactly backwards. Silent: no lint, no warning, and it failed in the ACCEPTING direction.
#      ⭐ REPAIRED by `ENGINE-UNIVERSAL-SERVICES.46` (c): a negative lookahead's body is evaluated
#      with `@profiles` gating IGNORED, in codegen and the interpreter together. Arm [2]'s second
#      row is flipped accordingly and now pins the monotonic verdict.
#
#   2. ⭐ THE LINT ALREADY NAMES THE COMPANION EDITS. Gating `tick` produces 11 `profile_orphans`
#      ERRORS, each carrying a DERIVED minimal fix. The other 16 terminals gate at orphans=0.
#
# ⛔⛔ AND THE FIRST TWO ATTEMPTS TO MEASURE (1) WERE BOTH WORTHLESS, WHICH IS THE REAL LESSON:
#   * attempt A put `@profiles` INLINE after `:=` (branch-level). The gate was never applied, and
#     the CONTROL is what said so — a required `tick` still parsed under the strict profile.
#   * attempt B fixed that but could not DISCRIMINATE: with nothing else able to consume the `'`,
#     a vacuous `!tick` and a refusing `!tick` both end in a reject. Two hypotheses, one reading.
#   The shape that works adds a CATCH-ALL that can consume the guarded token, so the hypotheses
#   predict OPPOSITE verdicts.
#
# Usage: bash docs/tasks/artifacts/sv_corpus_grad/profile_gate_sizing/probe.sh

set -uo pipefail
ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../../../.." && pwd)"
cd "$ROOT_DIR" || exit 2

PIPE="rust/target/debug/ast_pipeline"
HERE="docs/tasks/artifacts/sv_corpus_grad/profile_gate_sizing"
SCRATCH="rust/target/profile_gate_sizing_probe"

if [[ ! -x "$PIPE" ]]; then
    printf 'REFUSED: %s is absent — build it first. A probe that cannot measure must refuse\n' "$PIPE" >&2
    printf 'rather than report a flattering zero.\n' >&2
    exit 2
fi

rm -rf "$SCRATCH"; mkdir -p "$SCRATCH"
trap 'rm -rf "$SCRATCH"' EXIT
pass=0; fail=0

verdict() { # verdict <grammar> <input-text> <profile>
    printf '%s' "$2" > "$SCRATCH/in.txt"
    "$PIPE" "$1" --interpret-parse "$SCRATCH/in.txt" --grammar-profile "$3" 2>&1 \
        | grep -o 'accepted=[a-z]*' | head -1
}
expect() { # expect <label> <actual> <wanted>
    if [[ "$2" == "$3" ]]; then printf '  ✅ %-58s %s\n' "$1" "$2"; pass=$((pass+1))
    else printf '  ❌ %-58s %s (wanted %s)\n' "$1" "$2" "$3"; fail=$((fail+1)); fi
}

echo "PROFILE-GATE SIZING PROBE — $ROOT_DIR"
echo
echo "[1] ⭐ CONTROL FIRST — the @profiles gate is ACTIVE (or nothing below means anything)"
# A REQUIRED gated rule: present under `loose`, absent under `strict`. If both accept, the gate is
# not applied and every verdict in this probe is worthless. This arm caught exactly that once:
# `@profiles` written INLINE after `:=` is a BRANCH-level gate and does not gate the rule.
expect "gated rule REQUIRED, profile=loose  -> accept" "$(verdict "$HERE/gate_active_control.ebnf" "1'" loose)"  accepted=true
expect "gated rule REQUIRED, profile=strict -> reject" "$(verdict "$HERE/gate_active_control.ebnf" "1'" strict)" accepted=false

echo
echo "[2] ⭐ THE FINDING, NOW REPAIRED — \`!X\` still refuses once its subject is gated away"
# Discriminating by construction: a catch-all can consume the guarded token, so
#   H1 (the lookahead's subject still matches inside `!`) -> strict REJECTS
#   H2 (the subject matches nothing, so `!X` is vacuous)  -> strict ACCEPTS
#
# ⭐⭐⭐ THIS ARM MEASURED `accepted=true` — H2, the DEFECT — from the day it was written until
# `ENGINE-UNIVERSAL-SERVICES.46` (c) repaired the semantics: a negative lookahead's body is now
# evaluated with `@profiles` gating IGNORED, in codegen AND the interpreter together. It is flipped
# here to pin the MONOTONIC verdict, and it must never read `accepted=true` again — that would mean
# gating had gone back to ADDING strings to the narrower profile.
expect "!X refuses under loose  (X live)            -> reject" "$(verdict "$HERE/neg_lookahead_discriminating.ebnf" "1'" loose)"  accepted=false
expect "!X still refuses under strict (X gated)     -> reject" "$(verdict "$HERE/neg_lookahead_discriminating.ebnf" "1'" strict)" accepted=false

echo
echo "[3] sanity — the catch-all path itself works under both profiles"
expect "catch-all consumes a plain char, loose"  "$(verdict "$HERE/neg_lookahead_discriminating.ebnf" "1a" loose)"  accepted=true
expect "catch-all consumes a plain char, strict" "$(verdict "$HERE/neg_lookahead_discriminating.ebnf" "1a" strict)" accepted=true

echo
printf 'RESULT: pass=%d fail=%d\n' "$pass" "$fail"
[[ $fail -eq 0 ]] || exit 1
exit 0

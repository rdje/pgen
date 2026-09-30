#!/usr/bin/env bash
# ENGINE-UNIVERSAL-SERVICES.98 correction (PGEN-ENGINE-UNIVERSAL-SERVICES-0190) — the certificate
# probe that shows the (C) arm injecting a newline after a single-codepoint whitespace class.
#
# regex-syntax lowers `[ ]` to a Literal, so `[ ]+` reaches the compound-repetition arm (C) that
# `.98` added, which — as shipped in -0188/-0189 — carried no whitespace-only exemption and answered
# by first set: a `"\n"` separator after every inline-trivia run. In the SystemVerilog preprocessor
# grammar `\n` terminates a directive, so the injected newline breaks the sample the generator
# renders and the registered parser rejects it. The probe narrows `space_or_tab := /[ \t]+/` to
# `/[ ]+/` in a scratch copy (the registered parser still accepts everything the narrowed grammar
# can render) and runs the certificate on a BEFORE and an AFTER generator; the untouched grammar is
# the control (its `[ \t]+` is a two-codepoint Class and takes the class arm on both binaries).
#
#   PGEN_BEFORE_BIN=<ast_pipeline built from e4d928f35 or earlier (-0188..-0189)> \
#   PGEN_AFTER_BIN=<ast_pipeline built from this commit or later> \
#   bash docs/tasks/artifacts/engine_universal_services/eus98_correction/probe.sh
#
# Expected (measured 2026-09-30, count 40): scratch BEFORE sample_parse_failures=24/27/28 at seeds
# 0/7/42, scratch AFTER 5/1/3; control 1/0/1 on BOTH binaries (the class arm untouched); every arm
# UNKNOWN=0. The AFTER residue is the grammar's own (mid-sample failures, no injected newline). Output lives under rust/target/ (DATA-LOCALITY).
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../../../.." && pwd)"
BEFORE="${PGEN_BEFORE_BIN:?set PGEN_BEFORE_BIN to a pre-fix ast_pipeline}"
AFTER="${PGEN_AFTER_BIN:-$ROOT/rust/target/debug/ast_pipeline}"
OUT="${PGEN_EUS98_CORR_OUT:-$ROOT/rust/target/eus98_corr}"
G="$ROOT/grammars/systemverilog_preprocessor.ebnf"
S="$OUT/pp_scratch"; mkdir -p "$S"
sed 's|^space_or_tab := /\[ \\t\]+/|space_or_tab := /[ ]+/|' "$G" > "$S/systemverilog_preprocessor.ebnf"
grep -q '^space_or_tab := /\[ \]+/' "$S/systemverilog_preprocessor.ebnf" || { echo "probe: the scratch narrowing did not apply"; exit 2; }
fail=0
for arm in "before-scratch:$BEFORE:$S/systemverilog_preprocessor.ebnf" "after-scratch:$AFTER:$S/systemverilog_preprocessor.ebnf" "before-control:$BEFORE:$G" "after-control:$AFTER:$G"; do
  name=${arm%%:*}; rest=${arm#*:}; bin=${rest%%:*}; g=${rest#*:}
  for seed in 0 7 42; do
    log="$OUT/cert_${name}_seed${seed}.log"
    "$bin" "$g" --report-certificate-coverage --count 40 --seed "$seed" > "$log" 2>&1 || true
    line="$(grep -m1 'CERTIFICATE-COVERAGE:' "$log" || true)"
    spf="$(grep -oE 'sample_parse_failures=[0-9]+' <<<"$line" | cut -d= -f2)"
    unk="$(grep -oE 'UNKNOWN=[0-9]+' <<<"$line" | cut -d= -f2)"
    printf 'EUS98-CORR: arm=%-14s seed=%-2s UNKNOWN=%s sample_parse_failures=%s\n' "$name" "$seed" "${unk:-?}" "${spf:-?}"
    [ -n "$spf" ] || fail=1
  done
done
# The verdict: (1) the fix lowers the scratch arm's failures at every seed; (2) the control reads the
# same on both binaries (the class arm is untouched). The AFTER scratch residue is NOT asserted equal
# to the control: the narrowed grammar draws a different sample stream, so it carries the grammar's
# own residue at other counts (5/1/3 vs 1/0/1 measured) — its failures sit mid-sample at positions
# 217+, like the control's, not at position 0 behind an injected newline like BEFORE's.
spf() { grep -oE 'sample_parse_failures=[0-9]+' "$OUT/cert_$1_seed$2.log" | head -1 | cut -d= -f2; }
for seed in 0 7 42; do
  b="$(spf before-scratch "$seed")"; a="$(spf after-scratch "$seed")"
  [ -n "$b" ] && [ -n "$a" ] && [ "$b" -gt "$a" ] || { echo "EUS98-CORR: seed $seed before-scratch=$b is not above after-scratch=$a"; fail=1; }
  bc="$(spf before-control "$seed")"; ac="$(spf after-control "$seed")"
  [ "$bc" = "$ac" ] || { echo "EUS98-CORR: seed $seed control moved: before=$bc after=$ac"; fail=1; }
done
[ "$fail" -eq 0 ] && echo "EUS98-CORR: OK (before-scratch > after-scratch and control unmoved at seeds 0/7/42)" || echo "EUS98-CORR: FAIL"
exit "$fail"

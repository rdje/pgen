#!/usr/bin/env bash
# ENGINE-UNIVERSAL-SERVICES.119 — the compound-tail separator (`.98` (C)) is offered only where the
# grammar skips layout.
#
# `.98` (C) made `regex_tail_greedy_blocker` answer a greedy repetition over a COMPOUND sub-pattern;
# the preprocessor grammar's three comment-aware text tokens end in one whose first set holds a space
# and no newline, so the generator baked a newline after them — and in that `@whitespace_sensitive`
# grammar a newline ends the directive. This probe measures, generator against generator:
#
#   ARM 1  the certificate on the UNTOUCHED preprocessor grammar, seeds 0/7/42, count 40:
#          `sample_parse_failures` on a BEFORE binary and on the AFTER binary.
#   ARM 2  stream identity: `--generate-stimuli` on every shipped grammar, BEFORE vs AFTER, per seed.
#          A grammar that declares no `@whitespace_sensitive` must render BYTE-IDENTICAL samples (the
#          fix cannot reach it); the two that declare it (regex, systemverilog_preprocessor) may move.
#
#   PGEN_BEFORE_BIN=<ast_pipeline built from 9f00c30d3 or earlier, back to b69eceb39 (-0188)> \
#   PGEN_AFTER_BIN=<ast_pipeline built from the fix or later>   (default rust/target/debug/ast_pipeline) \
#   bash docs/tasks/artifacts/engine_universal_services/eus119_layout_sensitive_separator/probe.sh
#
# Measured 2026-10-01: ARM 1 BEFORE 1/0/1, AFTER 0/0/0 (the pre-`.98` engine also reads 0/0/0);
# ARM 2 every non-sensitive grammar identical at every seed. Output lives under rust/target/ (DATA-LOCALITY).
set -uo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../../../.." && pwd)"
BEFORE="${PGEN_BEFORE_BIN:?set PGEN_BEFORE_BIN to a pre-fix ast_pipeline}"
AFTER="${PGEN_AFTER_BIN:-$ROOT/rust/target/debug/ast_pipeline}"
OUT="${PGEN_EUS119_OUT:-$ROOT/rust/target/eus119/probe}"; mkdir -p "$OUT"
COUNT="${PGEN_EUS119_COUNT:-20}"
fail=0
G="$ROOT/grammars/systemverilog_preprocessor.ebnf"
for arm in "before:$BEFORE" "after:$AFTER"; do
  name=${arm%%:*}; bin=${arm#*:}
  for seed in 0 7 42; do
    log="$OUT/cert_${name}_seed${seed}.log"
    "$bin" "$G" --report-certificate-coverage --count 40 --seed "$seed" > "$log" 2>&1 || true
    line="$(grep -m1 'CERTIFICATE-COVERAGE:' "$log" || true)"
    spf="$(grep -oE 'sample_parse_failures=[0-9]+' <<<"$line" | cut -d= -f2)"
    unk="$(grep -oE 'UNKNOWN=[0-9]+' <<<"$line" | cut -d= -f2)"
    printf 'EUS119: arm=cert-%-6s seed=%-2s UNKNOWN=%s sample_parse_failures=%s\n' "$name" "$seed" "${unk:-?}" "${spf:-?}"
    [ -n "$spf" ] || fail=1
    if [ "$name" = after ] && [ "${spf:-1}" != 0 ]; then echo "EUS119: the AFTER generator still renders a sample the parser rejects (seed $seed)"; fail=1; fi
  done
done
before_total=0; for seed in 0 7 42; do n="$(grep -oE 'sample_parse_failures=[0-9]+' "$OUT/cert_before_seed$seed.log" | head -1 | cut -d= -f2)"; before_total=$((before_total + ${n:-0})); done
[ "$before_total" -gt 0 ] || { echo "EUS119: the BEFORE generator reads no failure — it is not a pre-fix binary, the probe proves nothing"; fail=1; }
identical=0; moved=0
for g in "$ROOT"/grammars/*.ebnf; do
  gname="$(basename "$g" .ebnf)"
  sensitive=no; grep -q '^@whitespace_sensitive' "$g" && sensitive=yes
  for seed in 0 7 42; do
    "$BEFORE" "$g" --generate-stimuli --count "$COUNT" --seed "$seed" > "$OUT/stim_before_${gname}_$seed.txt" 2>"$OUT/stim_before_${gname}_$seed.err"; rb=$?
    "$AFTER"  "$g" --generate-stimuli --count "$COUNT" --seed "$seed" > "$OUT/stim_after_${gname}_$seed.txt"  2>"$OUT/stim_after_${gname}_$seed.err";  ra=$?
    if cmp -s "$OUT/stim_before_${gname}_$seed.txt" "$OUT/stim_after_${gname}_$seed.txt" && [ "$rb" = "$ra" ]; then
      identical=$((identical + 1))
    else
      moved=$((moved + 1)); printf 'EUS119: arm=stream grammar=%s seed=%s sensitive=%s MOVED (rc %s -> %s)\n' "$gname" "$seed" "$sensitive" "$rb" "$ra"
      [ "$sensitive" = yes ] || { echo "EUS119: a grammar that skips layout moved — the fix reached it"; fail=1; }
    fi
  done
done
printf 'EUS119: arm=stream grammars=%s seeds=3 count=%s identical=%s moved=%s\n' "$(ls "$ROOT"/grammars/*.ebnf | wc -l | tr -d ' ')" "$COUNT" "$identical" "$moved"
[ "$fail" -eq 0 ] && echo "EUS119: OK (certificate BEFORE > 0 and AFTER 0/0/0; every layout-skipping grammar byte-identical)" || echo "EUS119: FAIL"
exit "$fail"

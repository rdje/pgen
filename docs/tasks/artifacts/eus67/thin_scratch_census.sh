#!/usr/bin/env bash
# docs/tasks/artifacts/eus67/thin_scratch_census.sh — ENGINE-UNIVERSAL-SERVICES.67(b) LEG 4.
#
# The dense thin-memo scratch table is sized `THIN_RULE_COUNT * (input.len() + 1)`
# slots of `u64` (`generated/<g>_parser.rs` constructor -> `ThinMemoScratch::begin`,
# `rust/src/ast_pipeline/mod.rs`), so its cost is `THIN_RULE_COUNT * 8` BYTES PER
# INPUT BYTE, paid before the parse reads a token and regardless of what the file
# contains. This census reports that constant for every generated family, because
# the property is ENGINE-UNIVERSAL and quoting it for SystemVerilog alone would
# read as a SystemVerilog defect.
#
# ⛔ It reports what it can PARSE OUT of the generated sources and NOTHING else: a
# family whose constant is absent prints ABSENT rather than 0, since a missing
# constant and a zero-cost table are different facts.
set -uo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../../.." && pwd)"
cd "$ROOT"
printf 'grammar\tthin_rule_count\trule_count\tbytes_per_input_byte\tMB_per_MB_of_input\n'
found=0
# ⛔ THE GLOB IS `generated/*.rs`, NOT `generated/*_parser.rs`, AND THAT IS A
# CORRECTION THIS SCRIPT ALREADY NEEDED ONCE. The EBNF parser ships as
# `generated/ebnf.rs` — no `_parser` suffix — so the obvious glob silently
# censused 10 files of 11 and reported a complete table. A census over a
# NAMING CONVENTION must model the exceptions to it.
for f in generated/*.rs; do
  [ -r "$f" ] || continue
  g="$(basename "$f" .rs)"; g="${g%_parser}"
  thin=$(sed -n 's/.*const THIN_RULE_COUNT: usize = \([0-9]*\)usize;.*/\1/p' "$f" | head -1)
  rules=$(sed -n 's/.*const RULE_COUNT: usize = \([0-9]*\)usize;.*/\1/p' "$f" | head -1)
  if [ -z "$thin" ]; then
    printf '%s\tABSENT\t%s\tABSENT\tABSENT\n' "$g" "${rules:-ABSENT}"
    continue
  fi
  found=$((found + 1))
  printf '%s\t%s\t%s\t%s\t%s\n' "$g" "$thin" "${rules:-ABSENT}" "$((thin * 8))" "$((thin * 8))"
done
[ "$found" -gt 0 ] || { echo "no generated parser declared THIN_RULE_COUNT — regenerate first" >&2; exit 2; }

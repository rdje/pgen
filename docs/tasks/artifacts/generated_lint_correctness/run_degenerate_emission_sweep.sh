#!/usr/bin/env bash
# GENERATED-LINT-CORRECTNESS.1 — measure the statically-degenerate emissions in
# every generated parser, BEFORE -> AFTER the codegen fold.
#
# The BEFORE arm is the artifact tree as it stands when `--capture-before` runs
# (regenerate it from a HEAD-vintage binary first if the tree is not clean).
# Both arms read the artifacts at their CANONICAL output paths, so the sizes are
# directly comparable and the `.5`-era embedded-output-path trap does not apply.
#
# Usage:
#   run_degenerate_emission_sweep.sh --capture-before
#   run_degenerate_emission_sweep.sh --capture-after
#   run_degenerate_emission_sweep.sh --report
set -uo pipefail
cd "$(dirname "$0")/../../../.." || exit 1
REPO="$PWD"
OUT="$REPO/rust/target/generated_lint_correctness"
mkdir -p "$OUT"

ARTIFACTS="
generated/json_parser.rs
generated/regex_parser.rs
generated/return_annotation_parser.rs
generated/rtl_const_expr_parser.rs
generated/rtl_frontend_parser.rs
generated/scratch_parser.rs
generated/semantic_annotation_parser.rs
generated/systemverilog_parser.rs
generated/systemverilog_preprocessor_parser.rs
generated/vhdl_parser.rs
generated/ebnf.rs
"

# Every statically-decided form the three emitters could produce. `eq_op` (a
# clippy CORRECTNESS lint, deny-by-default) fires only on the two identical-
# operand rows; the rest are dead weight that rustc folds but still ships.
PATTERNS=(
  '"longest_match" == "ordered"'
  '"longest_match" == "priority_first"'
  '"priority_first" == "ordered"'
  '"priority_first" == "priority_first"'   # clippy::eq_op
  '"ordered" == "ordered"'                 # clippy::eq_op
  '"ordered" == "priority_first"'
  'skip_leading_whitespace && false'       # clippy::overly_complex_bool_expr
  'skip_leading_whitespace && true'
)

capture() {
  local label="$1" f pat n total bytes
  : > "$OUT/census_$label.txt"
  total=0
  for f in $ARTIFACTS; do
    [ -f "$REPO/$f" ] || { printf '%-44s MISSING\n' "$(basename "$f")" >> "$OUT/census_$label.txt"; continue; }
    bytes=$(wc -c < "$REPO/$f" | tr -d ' ')
    printf '%-44s bytes=%s\n' "$(basename "$f")" "$bytes" >> "$OUT/census_$label.txt"
    for pat in "${PATTERNS[@]}"; do
      n=$(grep -c -- "$pat" "$REPO/$f")
      total=$((total + n))
      [ "$n" -gt 0 ] && printf '    %-42s %s\n' "$pat" "$n" >> "$OUT/census_$label.txt"
    done
  done
  echo "TOTAL_DEGENERATE_$label=$total" >> "$OUT/census_$label.txt"
  echo "captured $label -> $OUT/census_$label.txt (total degenerate = $total)"
}

case "${1:---report}" in
  --capture-before) capture before ;;
  --capture-after)  capture after ;;
  --report)
    echo "===== DEGENERATE-EMISSION SWEEP: BEFORE -> AFTER ====="
    paste <(grep -E '^\S+ +bytes=' "$OUT/census_before.txt") \
          <(grep -E '^\S+ +bytes=' "$OUT/census_after.txt") 2>/dev/null |
      awk '{printf "  %-44s %14s -> %-14s\n", $1, $2, $4}'
    echo
    grep '^TOTAL_DEGENERATE' "$OUT/census_before.txt" "$OUT/census_after.txt"
    ;;
  *) echo "unknown mode: $1"; exit 2 ;;
esac

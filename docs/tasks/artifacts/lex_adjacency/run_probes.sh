#!/usr/bin/env bash
# LEX-ADJACENCY.1 — re-run the design leaf's generator-shape probes.
#
# Read-only: needs only the DEBUG `ast_pipeline` built with the dual feature
# surface (`generated_parsers ebnf_dual_run`); no grammar edit, no parser regen,
# no probe rebuild. Deterministic (fixed seed 0).
#
#   bash docs/tasks/artifacts/lex_adjacency/run_probes.sh
#
# What each probe answers:
#   probe_atomic_triggers   — do the TWO inferred atomicity triggers (`-> $text`
#                             on every branch, `@transform`) close the INTERIOR
#                             seam of a two-element `number unit` rule?
#   probe_exterior_boundary — does an atomic rule ALSO fuse its LEFT-EXTERIOR
#                             boundary against a preceding keyword? (the real
#                             SystemVerilog shape `timeunit 10ns;`)
set -uo pipefail
HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO="$(cd "$HERE/../../../.." && pwd)"
PIPE="$REPO/rust/target/debug/ast_pipeline"
OUT="$(mktemp -d)"; trap 'rm -rf "$OUT"' EXIT

echo "feature surface: $("$PIPE" --report-feature-surface 2>&1 | head -1)"
echo

echo "=== probe_atomic_triggers.ebnf (A control / D '-> \$text' / E '@transform') ==="
"$PIPE" "$HERE/probe_atomic_triggers.ebnf" --generate-stimuli \
  --output "$OUT/p4.txt" --count 80 --seed 0 2>&1 | grep -E "^Stimuli coverage" | sed 's/^/  /'
for tag in A D E; do
  n=$(grep -c "^$tag" "$OUT/p4.txt")
  echo "  variant $tag ($n samples):"
  grep "^$tag" "$OUT/p4.txt" | sort -u | head -6 | sed 's/^/     /'
done
echo

echo "=== probe_exterior_boundary.ebnf (<A> control / <D> atomic, both after the keyword) ==="
"$PIPE" "$HERE/probe_exterior_boundary.ebnf" --generate-stimuli \
  --output "$OUT/p5.txt" --count 60 --seed 0 2>&1 | grep -E "^Stimuli coverage" | sed 's/^/  /'
for tag in '<A>' '<D>'; do
  n=$(grep -cF "$tag" "$OUT/p5.txt")
  echo "  variant $tag ($n samples):"
  grep -F "$tag" "$OUT/p5.txt" | sort -u | head -6 | sed 's/^/     /'
done
echo

echo "=== the two engine sites the design targets (source greps) ==="
echo "  PARSE  half — hard-coded no-skip rule-NAME allowlist:"
# run the source greps FROM the repo root so every reported path is repo-relative
cd "$REPO" || exit 1
grep -rn 'string_content_double" | "string_content_single' rust/src/ --include=*.rs \
  | sed 's/^/     /'
echo "  PARSE  half — the allowlist is LIVE (return_annotation emits the false arm):"
# `generated/` is gitignored; regenerate with `make -C rust focus_<grammar>` if absent.
# `grep -c` exits 1 on zero matches, so each count is guarded to keep the report clean.
count_sites() {  # <file> <pattern>
  [ -r "$1" ] || { printf 'absent'; return; }
  # `grep -c` already prints the count (including 0); it merely EXITS 1 on zero
  # matches, so only the status needs swallowing — never re-print here.
  grep -c "$2" "$1" 2>/dev/null || true
}
for g in return_annotation systemverilog; do
  f="generated/${g}_parser.rs"
  printf '     %s: match_regex(...) total=%s no-skip=%s\n' \
    "$f" \
    "$(count_sites "$f" 'match_regex(')" \
    "$(count_sites "$f" 'match_regex([^)]*, *false)')"
done
GEN=rust/src/ast_pipeline/stimuli_generator.rs
echo "  GENERATE half — atomicity INFERRED from the return shape:"
grep -n 'fn rule_is_lexically_atomic' "$GEN" | sed "s|^|     $GEN:|"
echo "  GENERATE half — interior + exterior flags driven by ONE bool:"
grep -n 'let is_atomic = self.rule_is_lexically_atomic' "$GEN" | sed "s|^|     $GEN:|"
grep -n 'self.atomic_token_depth += 1;' "$GEN" | sed "s|^|     $GEN:|"
grep -n 'self.last_terminal_from_atomic_rule = is_atomic;' "$GEN" | sed "s|^|     $GEN:|"

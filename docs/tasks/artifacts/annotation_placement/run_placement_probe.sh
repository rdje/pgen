#!/usr/bin/env bash
# ANNOTATION-PLACEMENT.1 — measure what PGEN does with the SAME semantic annotation
# in each placement the meta-grammar admits. Read-only; debug `ast_pipeline` only.
set -uo pipefail
HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO="$(cd "$HERE/../../../.." && pwd)"
cd "$REPO" || exit 1
PIPE=rust/target/debug/ast_pipeline
OUT="$(mktemp -d)"; trap 'rm -rf "$OUT"' EXIT

echo "=== 1. LINTER verdict on all three placements ==="
"$PIPE" "$HERE/placement_probe.ebnf" --lint-grammar 2>&1 | tail -3 | sed 's/^/  /'
echo "  lint exit=$?"
echo
echo "=== 2. which placements REACH the generated parser ==="
"$PIPE" "$HERE/placement_probe.ebnf" --generate-parser --eliminate-left-recursion \
  --output "$OUT/pp.rs" > "$OUT/pp.log" 2>&1
echo "  generate exit=$?"
for m in marker_rule marker_branch marker_mid; do
  printf '  %-14s -> %s occurrence(s) in the emitted parser\n' \
    "$m" "$(grep -c "$m" "$OUT/pp.rs" 2>/dev/null || true)"
done
echo
echo "=== 3. is ANY diagnostic emitted for a placement that was dropped? ==="
if grep -iE "marker_mid|mid_sequence|drop|ignor|unsupported|warn" "$OUT/pp.log" | head -5; then :; else
  echo "  (none — the drop is SILENT and the exit code is 0)"
fi

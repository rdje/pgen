#!/usr/bin/env bash
# SV-CORPUS-GRAD.3.11 — re-run the three generator-shape probes that decided the
# design (design_adjudication.txt). Read-only; needs only the DEBUG ast_pipeline,
# no parser regen and no probe rebuild.
set -uo pipefail
REPO=/Volumes/SSD/Documents/github/pgen
PIPE="$REPO/rust/target/debug/ast_pipeline"
HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
OUT="$(mktemp -d)"; trap 'rm -rf "$OUT"' EXIT
for g in gen_probe gen_probe2 gen_probe3; do
  echo "=== $g.ebnf ==="
  "$PIPE" "$HERE/$g.ebnf" --generate-stimuli --output "$OUT/$g.txt" --count 60 --seed 0 2>&1 \
    | grep -E "^Stimuli coverage" | sed 's/^/  /'
  # show a few samples of EACH variant tag (A/B/C) — the bare `//...` rows are
  # the line_comment rule generating itself, not a time-literal shape
  for tag in A B C; do
    n=$(grep -cE "^$tag " "$OUT/$g.txt")
    [ "$n" -eq 0 ] && continue
    echo "    variant $tag ($n samples):"
    grep -E "^$tag " "$OUT/$g.txt" | sort -u | head -6 | sed 's/^/      /'
  done
  echo
done

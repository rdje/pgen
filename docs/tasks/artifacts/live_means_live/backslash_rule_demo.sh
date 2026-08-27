#!/usr/bin/env bash
# backslash_rule_demo.sh — two backslash rules in ONE markdown table row, and only one is real.
# Owning record: docs/decisions/two-escaping-rules-that-look-alike-and-only-one-is-real.md
# Owning leaf:   `LIVE-DOC-CONTAINMENT.11`
#
# ONE row, TWO backslashes, OPPOSITE outcomes:
#   `\`   -> renders as <code>\</code>.  The backslash did NOT escape the backtick; it is the
#            span's CONTENT. CommonMark parses code spans BEFORE backslash escapes, and states
#            that backslash escapes do not work inside them.
#   a\|b  -> renders as a|b.  The backslash DID escape the pipe and is consumed, because GFM
#            splits table cells BEFORE inline parsing, so the pipe rule runs first.
#
# Getting the first one wrong reads a shipped contract at 4 defective cells where both renderers
# see 0, every one of them correct. That mis-rule was one step from becoming a commit-blocking arm.
#
# Exits 0 only when both halves still behave as stated. Needs pandoc; without it, NOT EVALUATED.
set -uo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../../.." && pwd)"
W="$ROOT/rust/target/backslash_rule_demo"

command -v pandoc >/dev/null 2>&1 || {
  echo "backslash-rule: NOT EVALUATED — pandoc is absent, so no renderer can be driven" >&2; exit 0; }

mkdir -p "$W"
printf '| A | B |\n|---|---|\n| x | span `\\` here, pipe a\\|b there |\n' > "$W/t.md"
# ⛔ -tex_math_dollars: pandoc's gfm reader enables TeX math on $…$, which GFM does not have.
out="$(pandoc -f gfm-tex_math_dollars -t html "$W/t.md" 2>/dev/null | tr -d '\n' | sed 's/.*<tbody>//')"
printf '%s\n' "$out"

fail=0
case "$out" in
  *'<code>\</code>'*) echo "backslash-rule: HELD — a backslash does NOT escape a code-span delimiter" ;;
  *) echo "backslash-rule: MISSED — '\`\\\`' no longer renders as a code span holding a backslash" >&2; fail=1 ;;
esac
case "$out" in
  *'a|b'*) echo "backslash-rule: HELD — a backslash DOES escape a table pipe, and is consumed" ;;
  *) echo "backslash-rule: MISSED — 'a\\|b' no longer renders as a literal pipe in one cell" >&2; fail=1 ;;
esac
exit "$fail"

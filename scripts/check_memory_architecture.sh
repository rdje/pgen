#!/usr/bin/env bash
# scripts/check_memory_architecture.sh
# SV-EXH-PROOF/MEMORY-ARCH (PGEN-MEMORY-ARCH-0005): single source of truth for the
# durable-memory-architecture invariants (per MEMORY_ARCHITECTURE.md §9 E2).
# Exits NONZERO on any breach. Called by .githooks/pre-commit (E3) and CI (E4).
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"; cd "$ROOT"
# Layer-A resume-pointer caps. BOTH are load-bearing — see the note below before touching either.
#
# ⛔ WHY TWO CAPS, MEASURED IN THIS REPOSITORY (README-POLICY.2, 2026-07-30):
#   From adoption until today this check enforced the LINE cap alone, exactly as the portable
#   standard's own reference script prescribes (MEMORY_ARCHITECTURE.md §9). Measured at the start
#   of the trim, MEMORY.md was:
#       60 lines  -> PASSING, exactly at the 60-line ceiling
#       138,403 bytes -> UNBOUNDED, 2,306 bytes per line, single longest line 18,816 bytes
#   ⇒ a file whose own header calls it a "bounded resume pointer … keep ≤ ~50 lines" was a 138 KB
#   document, and this guard was green the entire time. The line cap held the line COUNT and held
#   back nothing. The "Current state" block — whose heading literally reads "OVERWRITE this block
#   each update — do not append" — carried 18 distinct sessions and 81.3% of the file.
#   The same class was independently found at README.md:115 (4,369 bytes on ONE line, no shared
#   code path), which is why scripts/check_readme_stability.sh shipped with both caps on day one.
#   ⭐ Line and byte caps are COMPLEMENTS, not redundancy: neither wrapped prose nor very long
#   lines can bypass the budget.
#
# ⛔ NEVER raise a cap to land content. Layer A holds where-we-are-NOW; anything else belongs in
#   docs/tasks/ (layer B) or docs/decisions/ (layer C), both of which are durable and addressable.
#   A cap increase requires an explicit reviewed decision recorded in docs/tasks/README-POLICY.md.
#
# The values: chosen AFTER the trim (39 lines / 5,720 bytes), leaving deliberately PROPORTIONAL
# headroom so neither cap is the soft one that absorbs all the growth — 28% on lines, 25% on
# bytes. The line cap is lowered 60 -> 50 to match the standard this script enforces, which says
# "≤ ~50 lines" in both MEMORY_ARCHITECTURE.md §6 and MEMORY.md's own header; 60 was looser than
# the rule it was policing. At 50 lines the byte cap allows ~143 bytes/line — the shape of an
# actual pointer file — so the two caps bind at the same style of document rather than one
# shadowing the other.
CAP="${MEMORY_POINTER_LINE_CAP:-50}"
BYTE_CAP="${MEMORY_POINTER_BYTE_CAP:-7168}"
fail=0
note(){ printf 'memory-arch: %s\n' "$1" >&2; fail=1; }

# E2.1 — the standard itself must be present (system of record).
[ -f MEMORY_ARCHITECTURE.md ] || note "MEMORY_ARCHITECTURE.md is missing (the memory system of record)"

# E2.2 — layer A: bounded resume pointer present and within BOTH caps.
if [ -f MEMORY.md ]; then
  n=$(wc -l < MEMORY.md | tr -d ' ')
  b=$(wc -c < MEMORY.md | tr -d ' ')
  [ "$n" -le "$CAP" ] || note "MEMORY.md is $n lines (> cap $CAP) — it is layer A (resume pointer); demote content to docs/tasks/ (B) or docs/decisions/ (C)"
  # The byte cap is what makes the line cap mean something: 60 lines carried 138,403 bytes here.
  [ "$b" -le "$BYTE_CAP" ] || note "MEMORY.md is $b bytes (> cap $BYTE_CAP) — long lines bypass the line cap; demote content to docs/tasks/ (B) or docs/decisions/ (C). Do NOT raise the cap to fit the content."
else
  note "MEMORY.md (layer A resume pointer) is missing"
fi

# E2.3 — bootstrap pointers (E1): present and pointing at the system of record.
for f in AGENTS.md CLAUDE.md; do
  if [ -f "$f" ]; then
    grep -q "MEMORY_ARCHITECTURE.md" "$f" || note "$f does not point at MEMORY_ARCHITECTURE.md"
  else
    note "$f bootstrap pointer is missing"
  fi
done

# E2.4 — layer B: the task-tree system exists.
[ -f docs/TASK_TREE.md ] || note "docs/TASK_TREE.md (layer B index) is missing"
[ -d docs/tasks ] || note "docs/tasks/ (layer B task-trees) is missing"

# E2.5 — layer C: decisions dir + index present, and the index is not empty while
# records exist (a cheap in-sync sanity check, not a full reconcile).
if [ -d docs/decisions ]; then
  [ -f docs/decisions/INDEX.md ] || note "docs/decisions/INDEX.md (layer C index) is missing"
  rec_count=$(find docs/decisions -maxdepth 1 -name '*.md' ! -name 'INDEX.md' | wc -l | tr -d ' ')
  if [ -f docs/decisions/INDEX.md ] && [ "$rec_count" -gt 0 ]; then
    idx_rows=$(grep -cE '^\| \[' docs/decisions/INDEX.md || true)
    [ "$idx_rows" -gt 0 ] || note "docs/decisions/ has $rec_count records but INDEX.md lists none (out of sync)"
  fi
else
  note "docs/decisions/ (layer C) is missing"
fi

if [ "$fail" -eq 0 ]; then echo "memory-arch: OK"; fi
exit $fail

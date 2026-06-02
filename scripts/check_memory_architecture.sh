#!/usr/bin/env bash
# scripts/check_memory_architecture.sh
# SV-EXH-PROOF/MEMORY-ARCH (PGEN-MEMORY-ARCH-0005): single source of truth for the
# durable-memory-architecture invariants (per MEMORY_ARCHITECTURE.md §9 E2).
# Exits NONZERO on any breach. Called by .githooks/pre-commit (E3) and CI (E4).
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"; cd "$ROOT"
# Layer-A resume-pointer line cap (the standard's knob; default 60).
CAP="${MEMORY_POINTER_LINE_CAP:-60}"
fail=0
note(){ printf 'memory-arch: %s\n' "$1" >&2; fail=1; }

# E2.1 — the standard itself must be present (system of record).
[ -f MEMORY_ARCHITECTURE.md ] || note "MEMORY_ARCHITECTURE.md is missing (the memory system of record)"

# E2.2 — layer A: bounded resume pointer present and within the cap.
if [ -f MEMORY.md ]; then
  n=$(wc -l < MEMORY.md)
  [ "$n" -le "$CAP" ] || note "MEMORY.md is $n lines (> cap $CAP) — it is layer A (resume pointer); demote content to docs/tasks/ (B) or docs/decisions/ (C)"
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

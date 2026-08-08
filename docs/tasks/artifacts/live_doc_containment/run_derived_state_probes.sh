#!/usr/bin/env bash
# LIVE-DOC-CONTAINMENT.3 — probes for the E2.6 derived-state sub-check in
# scripts/check_memory_architecture.sh (docs/DERIVED_STATE_CONTAINMENT.md R1/R3, §6).
#
# ⭐ WHY THESE EXIST IN THIS SHAPE. A size/shape guard run over an ALREADY-FIXED file returns 0
# whether it works or is blind — that is the CTRL-1 lesson `README-POLICY.2` paid for, where the
# retired line-only guard was re-executed from `git show HEAD:` to prove the fix was earned rather
# than assumed. So the negative control here is not a synthetic fixture: it is the REAL pre-fix
# `MEMORY.md`, replayed out of git history, which must be REJECTED.
#
# Run from the repository root:  bash docs/tasks/artifacts/live_doc_containment/run_derived_state_probes.sh
set -uo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../../.." && pwd)"; cd "$ROOT"

CHECK="scripts/check_memory_architecture.sh"
WORK="rust/target/derived_state_probe"          # repo volume, per the data-locality policy
# The last commit that still carried BOTH derivable-exact fields (`push`, `latest_commit`).
PREFIX_REV="${DERIVED_STATE_PREFIX_REV:-ff96f98c~1}"

pass=0; fail=0
ok(){   printf '  ✅ %s\n' "$1"; pass=$((pass+1)); }
bad(){  printf '  ❌ %s\n' "$1"; fail=$((fail+1)); }

mkdir -p "$WORK"
trap 'rm -rf "$WORK"' EXIT

printf '\n=== P1 — the check is syntactically sound ===\n'
if bash -n "$CHECK" 2>/dev/null; then ok "bash -n clean"; else bad "bash -n FAILED"; fi

printf '\n=== P2 — GREEN: the current, compliant pointer passes ===\n'
if bash "$CHECK" >/dev/null 2>&1; then ok "current tree passes"; else bad "current tree FAILS (unexpected)"; fi

printf '\n=== P3 — CTRL-1 (the load-bearing one): the REAL pre-fix MEMORY.md must be REJECTED ===\n'
if git show "${PREFIX_REV}:MEMORY.md" > "$WORK/MEMORY.md" 2>/dev/null; then
  out="$(DERIVED_STATE_SURFACES="$WORK/MEMORY.md" bash "$CHECK" 2>&1)"; rc=$?
  hits="$(printf '%s\n' "$out" | grep -c 'derived-state:')"
  if [ "$rc" -ne 0 ]; then ok "rejected the real pre-fix pointer (exit $rc)"; else bad "BLIND — accepted the real pre-fix pointer"; fi
  if [ "$hits" -eq 2 ]; then ok "found exactly the 2 known fields (push, latest_commit), deduped"
  else bad "expected 2 findings, got $hits — a double-report or a miss"; fi
  printf '%s\n' "$out" | grep 'derived-state:' | sed 's/^memory-arch: /     /' | cut -c1-118
else
  bad "could not replay ${PREFIX_REV}:MEMORY.md (shallow clone? set DERIVED_STATE_PREFIX_REV)"
fi

printf '\n=== P4 — CTRL-2: a BLINDED scanner must REFUSE, not silently pass ===\n'
# An instrument with no ground truth is a confident guess. Break both patterns and confirm the
# controls catch it BEFORE any verdict is published.
sed 's/|unpushed|commits_ahead/|__disabled__|commits_ahead/; s/(ahead|behind)/(__nope__|__nope2__)/' \
  "$CHECK" > "$WORK/blinded.sh"
out="$(bash "$WORK/blinded.sh" 2>&1)"; rc=$?
if [ "$rc" -ne 0 ] && printf '%s\n' "$out" | grep -q 'controls MISSED'; then
  ok "blinded scanner REFUSES (exit $rc, 'controls MISSED')"
else
  bad "blinded scanner did NOT refuse — the ground truth is decorative"
fi

printf '\n=== P5 — the compliant DERIVATION form is not a false positive ===\n'
printf '%s\n' '- **active_work_unit**: `T` -> `.2`' \
  '> unpushed is DERIVED: `git rev-list --count origin/main..HEAD`' > "$WORK/compliant.md"
if DERIVED_STATE_SURFACES="$WORK/compliant.md" bash "$CHECK" >/dev/null 2>&1; then
  ok "a pointer carrying the derivation (not the value) passes"
else
  bad "FALSE POSITIVE on the compliant form — R3/R5 would be unimplementable"
fi

printf '\n=== RESULT: %d passed, %d failed ===\n' "$pass" "$fail"
[ "$fail" -eq 0 ]

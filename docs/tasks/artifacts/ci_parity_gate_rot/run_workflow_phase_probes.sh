#!/usr/bin/env bash
# docs/tasks/artifacts/ci_parity_gate_rot/run_workflow_phase_probes.sh
#
# CI-PARITY-GATE-ROT.3 — RED / GREEN / CONTROL arms for the workflow-phase changes.
#
# ⭐ WHY BEFORE→AFTER AND NOT JUST AFTER. Every arm runs against BOTH the pre-change gate
# (`git show <base>:rust/scripts/ci_workflow_local_gate.sh`) and the working-tree gate, so the
# claim is a MEASURED DELTA rather than "the new code passes its own tests". The RED arms must
# flip (pass-before → block-after); the CONTROL arms must be IDENTICAL on both sides — that is
# what proves the change did not over-broaden. `GENERATED-LINT-CORRECTNESS.4` shipped this shape
# and `.5`'s control arms caught a real bug in the harness; both lessons are taken as read here.
#
# ⛔ REPRODUCTION TRAP INHERITED FROM `.1`: the gate resolves its own root with
# `${BASH_SOURCE[0]}/../..`, so BOTH copies must live in `rust/scripts/` or ROOT_DIR lands one
# directory off and every arm reports nonsense (the tell is a doubled `rust/rust/target/...`).
#
# Usage:  bash docs/tasks/artifacts/ci_parity_gate_rot/run_workflow_phase_probes.sh [base-ref]
# Exit 0 iff every arm reaches its expected verdict on both sides.
set -uo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../../.." && pwd)"; cd "$ROOT"

BASE_REF="${1:-HEAD}"
AFTER="rust/scripts/ci_workflow_local_gate.sh"
BEFORE="rust/scripts/.ciparity_before_gate.sh"

git show "$BASE_REF:$AFTER" > "$BEFORE" 2>/dev/null || {
  echo "probes: ✗ cannot materialise the BEFORE gate from $BASE_REF" >&2; exit 1; }
chmod +x "$BEFORE"
trap 'rm -f "$ROOT/$BEFORE"' EXIT

PASS=0; FAIL=0

# run_arm <id> <expect-before: pass|block> <expect-after: pass|block> <needle-after|-> <env...> -- <gate>
run_arm() {
  local id="$1" expect_before="$2" expect_after="$3" needle="$4"; shift 4
  local side gate rc out verdict expect ok
  ok=1
  for side in before after; do
    if [ "$side" = before ]; then gate="$BEFORE"; expect="$expect_before"; else gate="$AFTER"; expect="$expect_after"; fi
    out=$(env "$@" bash "$ROOT/$gate" 2>&1); rc=$?
    if [ "$rc" -eq 0 ]; then verdict=pass; else verdict=block; fi
    if [ "$verdict" != "$expect" ]; then
      printf 'FAIL  %-34s %-6s expected %-5s got %-5s (rc=%d)\n' "$id" "$side" "$expect" "$verdict" "$rc"
      printf '%s\n' "$out" | tail -n 4 | sed 's/^/          /'
      ok=0
    elif [ "$side" = after ] && [ "$needle" != "-" ]; then
      if printf '%s\n' "$out" | grep -qF -- "$needle"; then
        printf 'ok    %-34s %-6s %-5s + needle present\n' "$id" "$side" "$verdict"
      else
        printf 'FAIL  %-34s %-6s %-5s but the expected message is MISSING: %s\n' "$id" "$side" "$verdict" "$needle"
        ok=0
      fi
    else
      printf 'ok    %-34s %-6s %s\n' "$id" "$side" "$verdict"
    fi
  done
  if [ "$ok" -eq 1 ]; then PASS=$((PASS + 1)); else FAIL=$((FAIL + 1)); fi
  printf '%s\n' "  ----"
}

printf '%s\n' "=============================================================================="
printf 'WORKFLOW-PHASE PROBES  (before=%s, after=working tree)\n' "$BASE_REF"
printf '%s\n' "=============================================================================="

# ─────────────────────────────────────────────────────────────────────────────────────────────
# RED-1 — the measured vacuous green: a filter name matching nothing certified parity.
#         BEFORE: exit 0, "✅ ... parity gate passed" having replayed ZERO workflows.
# ─────────────────────────────────────────────────────────────────────────────────────────────
run_arm "RED-1 unknown-filter-name" pass block \
  "unknown PGEN_CI_WORKFLOW_LOCAL_FILTER entry 'typo-that-matches-nothing'" \
  PGEN_CI_WORKFLOW_LOCAL_FILTER=typo-that-matches-nothing

# ─────────────────────────────────────────────────────────────────────────────────────────────
# RED-2 — one REAL name plus one typo. The dangerous shape: the run does real work, so the
#         green looks earned, while a whole workflow the operator believed they were replaying
#         was silently absent.
# ─────────────────────────────────────────────────────────────────────────────────────────────
run_arm "RED-2 one-real-one-typo" pass block \
  "unknown PGEN_CI_WORKFLOW_LOCAL_FILTER entry 'mdbook-docs-gatee'" \
  PGEN_CI_WORKFLOW_LOCAL_FILTER=branch-protection-contract-gate,mdbook-docs-gatee

# ─────────────────────────────────────────────────────────────────────────────────────────────
# RED-3 — the missing-generated cascade must now carry its CAUSE. BEFORE the operator got
#         `could not compile pgen (lib)` and nothing else, which points the diagnosis at the
#         Rust sources instead of at the export model.
# ─────────────────────────────────────────────────────────────────────────────────────────────
run_arm "RED-3 missing-generated-explained" block block \
  "this is NOT a defect in the Rust sources" \
  PGEN_CI_WORKFLOW_LOCAL_FILTER=annotation-contract-gate

# ─────────────────────────────────────────────────────────────────────────────────────────────
# CONTROL-1/2 — ⭐ THE ARMS THAT CAUGHT THE OVER-BROADENING. The first implementation refused
#         up front whenever generated/ was absent, which would have turned these two GREEN runs
#         RED: neither replay needs a generated parser. They must be identical on both sides.
# ─────────────────────────────────────────────────────────────────────────────────────────────
run_arm "CTRL-1 shell-only-replay-still-ok" pass pass \
  "all selected local workflow commands passed" \
  PGEN_CI_WORKFLOW_LOCAL_FILTER=branch-protection-contract-gate

run_arm "CTRL-2 mdbook-replay-still-ok" pass pass "-" \
  PGEN_CI_WORKFLOW_LOCAL_FILTER=mdbook-docs-gate

# ─────────────────────────────────────────────────────────────────────────────────────────────
# CONTROL-3 — ⭐ the explanation must NOT fire on an unrelated failure. A genuine gate defect
#         mislabelled as "your environment is unprepared" is worse than no explanation at all:
#         it teaches the operator to ignore a real red. Forced by pointing the branch-protection
#         gate at a policy file that does not exist.
# ─────────────────────────────────────────────────────────────────────────────────────────────
printf 'CTRL-3 unrelated-failure-not-mislabelled\n'
out=$(PGEN_BRANCH_PROTECTION_POLICY_JSON=/nonexistent/policy.json \
      PGEN_CI_WORKFLOW_LOCAL_FILTER=branch-protection-contract-gate \
      bash "$ROOT/$AFTER" 2>&1); rc=$?
if [ "$rc" -eq 0 ]; then
  printf 'FAIL  %-34s expected block, got pass\n' "CTRL-3"; FAIL=$((FAIL + 1))
elif printf '%s\n' "$out" | grep -qF "this is NOT a defect in the Rust sources"; then
  printf 'FAIL  %-34s the missing-generated explanation fired on an UNRELATED failure\n' "CTRL-3"
  FAIL=$((FAIL + 1))
elif printf '%s\n' "$out" | grep -qF "branch protection policy file not found"; then
  printf 'ok    %-34s blocked for its OWN reason, unexplained by the new block\n' "CTRL-3"
  PASS=$((PASS + 1))
else
  printf 'FAIL  %-34s blocked, but for an unrecognised reason\n' "CTRL-3"; FAIL=$((FAIL + 1))
fi
printf '%s\n' "  ----"

# ─────────────────────────────────────────────────────────────────────────────────────────────
# GREEN-1 — the required-artifact set is DERIVED from rust/src/lib.rs, and the derivation must
#         discriminate: exactly the literal-path include!()s (no cfg, absence = hard rustc
#         error) and NONE of the cfg-guarded include!(env!()) ones (absence = feature disabled).
#         Agreement is not evidence unless the two classes are told apart.
# ─────────────────────────────────────────────────────────────────────────────────────────────
printf 'GREEN-1 derivation-discriminates\n'
derived=$(grep -oE 'include!\("\.\./\.\./generated/[a-z_]+\.rs"\)' rust/src/lib.rs |
          sed -E 's|.*(generated/[a-z_]+\.rs).*|\1|' | sort -u | tr '\n' ' ')
guarded=$(grep -c 'include!(env!' rust/src/lib.rs)
expected="generated/return_annotation_parser.rs generated/semantic_annotation_parser.rs "
if [ "$derived" = "$expected" ] && [ "$guarded" -ge 9 ]; then
  printf 'ok    %-34s literal=[%s] cfg-guarded-ignored=%s\n' "GREEN-1" "${derived% }" "$guarded"
  PASS=$((PASS + 1))
else
  printf 'FAIL  %-34s derived=[%s] guarded=%s\n' "GREEN-1" "$derived" "$guarded"; FAIL=$((FAIL + 1))
fi
printf '%s\n' "  ----"

printf '%s\n' "------------------------------------------------------------------------------"
printf 'arms=%d  PASS=%d  FAIL=%d\n' "$((PASS + FAIL))" "$PASS" "$FAIL"
[ "$FAIL" -eq 0 ]

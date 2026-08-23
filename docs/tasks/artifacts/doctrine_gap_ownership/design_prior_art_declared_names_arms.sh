#!/usr/bin/env bash
# DOCTRINE-GAP-OWNERSHIP.11 — the two-arm control + the red control for the
# DESIGN-PRIOR-ART declared-name blindness, re-runnable from any cwd.
#
#   ARM BEFORE   HEAD's enforcer on the same staged set   -> must FAIL, naming `handles`
#   ARM AFTER    the patched enforcer, same staged set    -> must PASS
#   RED          a genuinely novel directive name staged  -> must STILL FAIL
#
# ⛔ Exit codes are captured with the command's OWN status, never through a pipe: a
#    `| head` swallows it and every arm reads rc=0, which is how the first version of
#    this control reported the defect as absent (layer A: a nonzero exit is not proof a
#    guard fired — and neither is a zero one proof it did not).
# ⛔ The BEFORE arm is copied into `scripts/` before it runs. The enforcer derives the
#    repo root as `dirname "$0"/..`, so running it from a scratch directory makes it look
#    for staged files in `rust/target/...`, find none, and exit 0 — a green arm that
#    proves nothing. The first run of this control did exactly that.
set -uo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")" && while [ ! -d .git ] && [ "$PWD" != / ]; do cd ..; done; pwd)"
cd "$ROOT" || exit 2
BEFORE="scripts/.dpa_before_arm.sh"
LEAF="docs/tasks/DOCTRINE-GAP-OWNERSHIP.md"
cleanup() { rm -f "$BEFORE"; }
trap cleanup EXIT

run() { local out rc; out="$( { bash "$1" >/dev/null; } 2>&1 )"; rc=$?
        printf '  rc=%s  %s\n' "$rc" "$(printf '%s' "$out" | head -2 | tr '\n' ' ')"; }

git show HEAD:scripts/check_design_prior_art.sh > "$BEFORE" 2>/dev/null || {
  echo "design_prior_art arms: REFUSED — cannot read the enforcer at HEAD"; exit 2; }
echo "### ARM BEFORE — HEAD's enforcer, current staged set"
run "$BEFORE"
echo "### ARM AFTER  — the patched enforcer, current staged set"
run scripts/check_design_prior_art.sh
echo "### RED CONTROL — a genuinely novel directive name must STILL be blocked"
printf '\n- probe: a genuinely novel surface `@%s` appears here.\n' "zzz_not_a_real_directive" >> "$LEAF"
git add "$LEAF"
run scripts/check_design_prior_art.sh
git restore --staged "$LEAF" >/dev/null 2>&1; git checkout -- "$LEAF"
echo "  probe reverted: $(grep -c 'zzz_not_a_real_directive' "$LEAF") occurrences remain"

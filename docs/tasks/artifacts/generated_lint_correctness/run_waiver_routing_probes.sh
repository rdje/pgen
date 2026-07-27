#!/usr/bin/env bash
# docs/tasks/artifacts/generated_lint_correctness/run_waiver_routing_probes.sh
#
# WAIVER-ROUTING (10th enforced doctrine) — RED / GREEN / CONTROL probes, plus a sweep of the
# EXISTING corpus for waiver notes that are already sitting unrouted.
#
# The class: `docs/tasks/RGX-0090.md:131` carried a hand-written waiver note INSIDE a ticked ROOT
# CAUSE box — "the parse/perf diagnosis-toolbox signatures do not apply" — and nothing happened
# for months. That note was a precise, correct bug report about a MISSING diagnosis family, filed
# by someone who had done the work and hit the boundary. `GENERATED-LINT-CORRECTNESS.4` later
# re-derived the identical gap from scratch.
#
# ⛔ THE DESIGN CONSTRAINT THE RED ARMS PROTECT: the doctrine must NOT punish honesty. Forbidding
# waiver language would delete the signal — authors would stop writing the note. So a waiver stays
# LEGAL and only has to name an owner. GREEN-1/2 pin that a one-token citation discharges it;
# CTRL-3 pins that the historical record is never retroactively bound.
set -uo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../../.." && pwd)"
CHECK="${ROOT_DIR}/scripts/check_waiver_routing.sh"
WORK="${ROOT_DIR}/rust/target/doctrine_checks/waiver_routing_probes"

rm -rf "${WORK}"; mkdir -p "${WORK}"
pass_count=0; fail_count=0

new_repo() {
  local d="${WORK}/$1"
  rm -rf "$d"; mkdir -p "$d/docs/tasks" "$d/scripts"
  git -C "$d" init -q
  git -C "$d" config user.email probe@pgen.local
  git -C "$d" config user.name probe
  cp "${CHECK}" "$d/scripts/check_waiver_routing.sh"
  git -C "$d" add -A >/dev/null 2>&1
  git -C "$d" commit -qm base >/dev/null 2>&1
  printf '%s' "$d"
}

run_arm() {
  local name="$1" expect_rc="$2" d="$3" rc=0
  local log="${WORK}/${name}.log"
  ( cd "$d" && bash scripts/check_waiver_routing.sh ) >"$log" 2>&1 || rc=$?
  if [ "$rc" = "$expect_rc" ]; then
    printf 'PROBE %-9s PASS  exit=%s (expected %s)\n' "$name" "$rc" "$expect_rc"
    pass_count=$((pass_count + 1))
  else
    printf 'PROBE %-9s FAIL  exit=%s (expected %s)\n' "$name" "$rc" "$expect_rc"
    sed 's/^/  /' "$log" | tail -n 8
    fail_count=$((fail_count + 1))
  fi
}

# ---- RED-1: a bare waiver claim, no owner named -----------------------------------------------
d="$(new_repo red1)"
printf '# TREE-A\n\n- [x] **ROOT CAUSE** — the diagnosis-toolbox signatures do not apply to this defect class.\n' \
  >"$d/docs/tasks/TREE-A.md"
git -C "$d" add -A >/dev/null 2>&1
run_arm RED-1 1 "$d"

# ---- RED-2: the REAL RGX-0090 text, verbatim --------------------------------------------------
d="$(new_repo red2)"
printf '# TREE-A\n\n- [x] ROOT CAUSE: (Waiver note: like RGX-0091 this is a BUILD-FLOW defect - the parse/perf diagnosis-toolbox signatures do not apply; root cause is backed by the recorded scratch reproduction runs above.)\n' \
  >"$d/docs/tasks/TREE-A.md"
git -C "$d" add -A >/dev/null 2>&1
run_arm RED-2 1 "$d"

# ---- RED-3: an explicit env waiver, unrouted --------------------------------------------------
d="$(new_repo red3)"
printf '# TREE-A\n\nLanded with PGEN_DIAG_EVIDENCE_WAIVER set because the gate cannot see this surface.\n' \
  >"$d/docs/tasks/TREE-A.md"
git -C "$d" add -A >/dev/null 2>&1
run_arm RED-3 1 "$d"

# ---- GREEN-1: same claim, owner named on the SAME line ----------------------------------------
d="$(new_repo green1)"
printf '# TREE-A\n\n- [x] **ROOT CAUSE** — the diagnosis-toolbox signatures do not apply to this defect class (gate gap owned by GENERATED-LINT-CORRECTNESS.5).\n' \
  >"$d/docs/tasks/TREE-A.md"
git -C "$d" add -A >/dev/null 2>&1
run_arm GREEN-1 0 "$d"

# ---- GREEN-2: discharged by a slice id --------------------------------------------------------
d="$(new_repo green2)"
printf '# TREE-A\n\nThe checklist does not apply to this docs-only slice; tracked as PGEN-CI-PARITY-GATE-ROT-0002.\n' \
  >"$d/docs/tasks/TREE-A.md"
git -C "$d" add -A >/dev/null 2>&1
run_arm GREEN-2 0 "$d"

# ---- CTRL-1: ordinary prose that merely contains the words ------------------------------------
d="$(new_repo ctrl1)"
printf '# TREE-A\n\nWe apply the fix at the codegen site. The rule does not apply layout to terminals.\n' \
  >"$d/docs/tasks/TREE-A.md"
git -C "$d" add -A >/dev/null 2>&1
run_arm CTRL-1 0 "$d"

# ---- CTRL-2: nothing staged -------------------------------------------------------------------
d="$(new_repo ctrl2)"
run_arm CTRL-2 0 "$d"

# ---- CTRL-3: a PRE-EXISTING waiver, untouched — the historical record is never retro-bound -----
d="$(new_repo ctrl3)"
printf '# TREE-A\n\n- [x] ROOT CAUSE: (Waiver note: the diagnosis signatures do not apply here.)\n' \
  >"$d/docs/tasks/TREE-A.md"
git -C "$d" add -A >/dev/null 2>&1
git -C "$d" commit -qm "historical waiver, already landed" >/dev/null 2>&1
printf '\nAn unrelated later addition.\n' >>"$d/docs/tasks/TREE-A.md"
git -C "$d" add -A >/dev/null 2>&1
run_arm CTRL-3 0 "$d"

# ---- SWEEP: unrouted waivers already sitting in the tracked corpus ----------------------------
echo
echo "=== SWEEP: waiver-shaped claims already in docs/tasks/ (informational) ==="
# Sourced from the enforcer so the sweep can never measure a different rule than the gate applies.
eval "$(grep -E "^WAIVER_RE=" "${CHECK}")"
eval "$(grep -E "^OWNER_RE=" "${CHECK}")"
total=0; routed=0; unrouted=0
while IFS= read -r f; do
  while IFS= read -r line; do
    [ -n "$line" ] || continue
    total=$((total + 1))
    ln="${line%%:*}"; lo=$(( ln > 6 ? ln - 6 : 1 )); hi=$(( ln + 6 ))
    if sed -n "${lo},${hi}p" "${ROOT_DIR}/$f" | grep -qE "$OWNER_RE"; then routed=$((routed + 1))
    else unrouted=$((unrouted + 1)); printf '  UNROUTED  %s\n' "$f"; printf '            %.150s\n' "$(printf '%s' "$line" | sed 's/^[0-9]*://')"; fi
  done < <(grep -nE "$WAIVER_RE" "${ROOT_DIR}/$f" 2>/dev/null || true)
done < <(cd "${ROOT_DIR}" && git ls-files 'docs/tasks/*.md')
printf 'sweep: %d waiver-shaped lines in the tracked corpus — %d already name an owner, %d do not\n' \
  "$total" "$routed" "$unrouted"

printf '\nprobes: %d passed, %d failed\n' "$pass_count" "$fail_count"
[ "$fail_count" -eq 0 ] || exit 1
exit 0

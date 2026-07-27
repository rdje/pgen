#!/usr/bin/env bash
# docs/tasks/artifacts/generated_lint_correctness/run_diag_evidence_probes.sh
# GENERATED-LINT-CORRECTNESS.3 — RED / GREEN / CONTROL probes for the two hardenings applied to
# scripts/check_diagnosis_evidence.sh, plus a REGRESSION SWEEP over every real task leaf.
#
# The two hardenings:
#   (1) BOX-SCOPED evidence — the signature backing a ticked box must sit inside THAT BOX'S OWN
#       bullet. Closes cross-FILE leakage (a co-staged tree file supplying the token — measured:
#       that is how GENERATED-LINT-CORRECTNESS.1 passed) and incidental-PROSE leakage (the
#       whole-file grep matching a mention anywhere — the deferred watch item in
#       docs/decisions/project_build_integrity_compiler_root_cause_signature.md).
#   (2) A FOURTH signature group for CODEGEN-EMISSION defects.
#
# Arms:
#   RED-1   boxes in leaf A, signature only in co-staged leaf B   -> BLOCK (the .1 hole)
#   RED-2   signature present in the file but OUTSIDE the box     -> BLOCK (the prose hole)
#   RED-3   ROOT CAUSE box unticked                              -> BLOCK (unchanged behaviour)
#   GREEN-1 signature inside the box, correctness family          -> ALLOW
#   GREEN-2 signature inside the box, CODEGEN-EMISSION family     -> ALLOW (group 4; would have
#                                                                   been BLOCKED before this leaf)
#   CTRL-1  no code staged at all                                 -> ALLOW (check not required)
#   SWEEP   every docs/tasks/*.md carrying a checklist is replayed against the new box-scoped
#           logic, reporting any leaf that would now fail — so a regression is measured, not hoped.
#
# Scratch lives under rust/target/ (repo volume) per the project data-locality policy.
# Usage: bash docs/tasks/artifacts/generated_lint_correctness/run_diag_evidence_probes.sh
set -uo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../../.." && pwd)"
CHECK="${ROOT_DIR}/scripts/check_diagnosis_evidence.sh"
WORK="${ROOT_DIR}/rust/target/doctrine_checks/diag_evidence_probes"

rm -rf "${WORK}"; mkdir -p "${WORK}"
pass_count=0; fail_count=0

# Each arm builds a THROWAWAY git repo so staging is real (the check reads `git diff --cached`).
new_repo() {
  local d="${WORK}/$1"
  rm -rf "$d"; mkdir -p "$d/docs/tasks" "$d/rust/src" "$d/scripts"
  git -C "$d" init -q
  git -C "$d" config user.email probe@pgen.local
  git -C "$d" config user.name probe
  cp "${CHECK}" "$d/scripts/check_diagnosis_evidence.sh"
  printf 'fn main() {}\n' >"$d/rust/src/lib.rs"   # the staged CODE change that triggers the check
  printf '%s' "$d"
}

run_arm() {
  # run_arm <name> <expected_exit> <expected_substring> <repo_dir>
  local name="$1" expect_rc="$2" expect_msg="$3" d="$4" rc=0
  local log="${WORK}/${name}.log"
  ( cd "$d" && bash scripts/check_diagnosis_evidence.sh ) >"$log" 2>&1 || rc=$?
  local ok_msg="yes"
  [ -n "$expect_msg" ] && { grep -qF -- "$expect_msg" "$log" || ok_msg="no"; }
  if [ "$rc" = "$expect_rc" ] && [ "$ok_msg" = "yes" ]; then
    printf 'PROBE %-8s PASS  exit=%s (expected %s)%s\n' "$name" "$rc" "$expect_rc" \
      "${expect_msg:+ — matched: ${expect_msg}}"
    pass_count=$((pass_count + 1))
  else
    printf 'PROBE %-8s FAIL  exit=%s (expected %s, msg_match=%s)\n' "$name" "$rc" "$expect_rc" "$ok_msg"
    sed 's/^/  /' "$log" | tail -n 20
    fail_count=$((fail_count + 1))
  fi
}

checklist_boxes() {
  # $1 = text placed INSIDE the ROOT CAUSE box body (may be empty)
  cat <<EOF
#### Acceptance Checklist (enforced)

- [x] **ROOT CAUSE (WHY + WHERE)** — the emission site was enumerated.
      ${1}
- [x] **ADDRESSED (verified)** — before -> after on the symptom.
- [x] **NO REGRESSION** — cert coverage deterministic at seeds 0/7/42, fully_certified.
EOF
}

# ---- RED-1: boxes in leaf A, diagnosis signature only in co-staged leaf B ----
d="$(new_repo red1)"
{ echo "# TREE-A"; checklist_boxes ""; } >"$d/docs/tasks/TREE-A.md"
{ echo "# TREE-B"; echo; echo "Unrelated leaf. It happens to quote --report-certificate-coverage and"; \
  echo "[plannable-probe] verdicts for its own reasons."; } >"$d/docs/tasks/TREE-B.md"
git -C "$d" add -A >/dev/null 2>&1
run_arm RED-1 1 "NOT INSIDE THAT BOX" "$d"

# ---- RED-2: signature in the same FILE but outside the ticked box -----------
d="$(new_repo red2)"
{ echo "# TREE-A"; checklist_boxes ""; echo; \
  echo "Narrative section: we also ran --report-certificate-coverage and read the"; \
  echo "[plannable-probe] verdicts, but not as part of the ROOT CAUSE box."; } >"$d/docs/tasks/TREE-A.md"
git -C "$d" add -A >/dev/null 2>&1
run_arm RED-2 1 "NOT INSIDE THAT BOX" "$d"

# ---- RED-3: ROOT CAUSE box left unticked (unchanged behaviour) --------------
d="$(new_repo red3)"
{ echo "# TREE-A"; echo; \
  echo "- [ ] **ROOT CAUSE (WHY + WHERE)** — pending, --report-certificate-coverage not yet run."; \
  echo "- [x] **ADDRESSED (verified)** — before -> after."; \
  echo "- [x] **NO REGRESSION** — seeds 0/7/42."; } >"$d/docs/tasks/TREE-A.md"
git -C "$d" add -A >/dev/null 2>&1
run_arm RED-3 1 "UNTICKED" "$d"

# ---- GREEN-1: correctness-family signature inside the box ------------------
d="$(new_repo green1)"
{ echo "# TREE-A"; checklist_boxes \
  'Diagnosed with --report-certificate-coverage and the [plannable-probe] verdicts.'; } \
  >"$d/docs/tasks/TREE-A.md"
git -C "$d" add -A >/dev/null 2>&1
run_arm GREEN-1 0 "OK (task leaf passes" "$d"

# ---- GREEN-2: CODEGEN-EMISSION signature inside the box (the new group 4) ---
d="$(new_repo green2)"
{ echo "# TREE-A"; checklist_boxes \
  'The emission site interpolates a codegen constant; the lint lane reports clippy::eq_op over generated/systemverilog_parser.rs.'; } \
  >"$d/docs/tasks/TREE-A.md"
git -C "$d" add -A >/dev/null 2>&1
run_arm GREEN-2 0 "OK (task leaf passes" "$d"

# ---- CTRL-1: no code staged -> the checklist is not required ---------------
d="$(new_repo ctrl1)"
rm -f "$d/rust/src/lib.rs"
{ echo "# TREE-A"; echo "docs-only change"; } >"$d/docs/tasks/TREE-A.md"
git -C "$d" add -A >/dev/null 2>&1
run_arm CTRL-1 0 "no code change staged" "$d"

# ---- SWEEP: would the new box-scoping false-fail any REAL leaf? ------------
# For every tracked task file that carries a ticked ROOT CAUSE box, report whether its diagnosis
# signature is inside that box. A leaf that is now unbacked is NOT automatically a defect (older
# leaves predate this rule and are already committed) — but it MUST be measured, not assumed away.
echo
echo "=== SWEEP: box-scoped backing across every real task leaf ==="
sweep_total=0; sweep_backed=0; sweep_unbacked=0
unbacked_list="${WORK}/unbacked_leaves.txt"; : >"$unbacked_list"
# ⚠️ SOURCED, NOT COPIED (GENERATED-LINT-CORRECTNESS.4). This sweep originally carried its own
# hand-copied duplicate of DIAGNOSIS_SIG. The moment `.4` corrected group 2's tool vocabulary and
# added the ops/build-flow family, that copy would have measured the STALE rule and reported a
# gap the live enforcer no longer has — the duplicated-metadata class
# ([[feedback_duplicated_metadata_needs_derived_drift_gate]]) inside the very driver that exists to
# verify the enforcer. Reading the assignment out of the enforcer makes drift impossible.
eval "$(grep -E '^DIAGNOSIS_SIG=' "${CHECK}")"
DIAG_SIG="${DIAGNOSIS_SIG}"
[ -n "${DIAG_SIG}" ] || { echo "probe: could not source DIAGNOSIS_SIG from ${CHECK}" >&2; exit 1; }

box_body_probe() {
  awk -v start="$1" '
    function isbox(l)     { return match(l, /^[ \t]*[-*][ \t]*\[[ xX]\][ \t]/) }
    function indent(l, i) { i = match(l, /[^ \t]/); return (i == 0 ? 0 : i - 1) }
    NR <  start { next }
    NR == start { boxind = indent($0); print; next }
    {
      if (isbox($0) && indent($0) <= boxind) exit
      if ($0 ~ /^#/) exit
      print
    }
  ' "$2"
}

# Sourced for the same anti-drift reason as DIAGNOSIS_SIG above.
eval "$(grep -E '^ROOT_KW=' "${CHECK}")"
HDR_RE="^[[:space:]]*[-*][[:space:]]*\[[xX]\][[:space:]].*(${ROOT_KW})"
while IFS= read -r f; do
  hdrs="$(grep -nEi -- "$HDR_RE" "${ROOT_DIR}/$f" 2>/dev/null | cut -d: -f1 || true)"
  [ -n "$hdrs" ] || continue
  sweep_total=$((sweep_total + 1))
  backed=no
  while IFS= read -r ln; do
    [ -n "$ln" ] || continue
    if box_body_probe "$ln" "${ROOT_DIR}/$f" | grep -Eq -- "$DIAG_SIG"; then backed=yes; break; fi
  done <<<"$hdrs"
  if [ "$backed" = yes ]; then sweep_backed=$((sweep_backed + 1));
  else sweep_unbacked=$((sweep_unbacked + 1)); echo "$f" >>"$unbacked_list"; fi
done < <(cd "${ROOT_DIR}" && git ls-files 'docs/tasks/*.md')

printf 'sweep: %d leaf files carry a ticked ROOT CAUSE box; %d are box-scoped-backed, %d are not\n' \
  "$sweep_total" "$sweep_backed" "$sweep_unbacked"
if [ "$sweep_unbacked" -gt 0 ]; then
  echo "sweep: files whose ROOT CAUSE box would NOT be backed under the new rule (informational —"
  echo "sweep: these are already-committed leaves; the rule binds NEW commits):"
  sed 's/^/  /' "$unbacked_list"
fi

printf '\nprobes: %d passed, %d failed\n' "$pass_count" "$fail_count"
[ "$fail_count" -eq 0 ] || exit 1
exit 0

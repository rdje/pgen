#!/usr/bin/env bash
# docs/tasks/artifacts/generated_lint_correctness/run_diag_evidence_leaf_scope_probes.sh
#
# GENERATED-LINT-CORRECTNESS.7 — RED / GREEN / CONTROL probes for the LEAF-SCOPING fix to
# scripts/check_diagnosis_evidence.sh.
#
# ⭐ THE MISBEHAVIOUR. `.3` scoped the SIGNATURE to its own box but never scoped the BOX to the
# change: any ticked box in any staged task file could satisfy a requirement. So for every tree
# already holding one compliant leaf, box-scoping was VACUOUS — a new leaf could carry no
# checklist at all and inherit a finished leaf's boxes. RED-H1/H2 are that hole, reproduced.
#
# ⛔ THE POINT OF THE GREEN ARMS. The fix must not make the gate punitive. A leaf whose checklist
# landed in an EARLIER commit must still pass when a follow-up commit edits that same leaf
# (GREEN-2), and a commit that only DELETES lines inside its leaf must still pass (GREEN-3) —
# otherwise ordinary multi-commit leaves would be forced to waive, which is how a gate teaches
# authors to bypass it (the `.4`/`.6` lesson).
#
# Arms:
#   RED-H1  new leaf with NO checklist, old completed leaf in the SAME file  -> BLOCK (was PASS)
#   RED-H2  owning leaf in file B, backed boxes only in unrelated file A     -> BLOCK (was PASS)
#   RED-H3  the three boxes split across two staged files                    -> BLOCK (was PASS)
#   GREEN-1 the owning leaf writes all three boxes in this commit            -> ALLOW
#   GREEN-2 checklist committed EARLIER; this commit edits the same leaf     -> ALLOW
#   GREEN-3 this commit only DELETES lines inside its own leaf               -> ALLOW
#   CTRL-1  ROOT CAUSE box unticked                                          -> BLOCK (unchanged)
#   CTRL-2  signature present but OUTSIDE the ticked box                     -> BLOCK (unchanged)
#   CTRL-3  no code staged at all                                            -> ALLOW (unchanged)
#
# Before->after replay (produced, not asserted):
#   git show HEAD:scripts/check_diagnosis_evidence.sh > rust/target/head_check.sh
#   PGEN_DIAG_CHECK_OVERRIDE=rust/target/head_check.sh bash <this script>
# Under the pre-`.7` enforcer RED-H1/H2/H3 MUST pass (the hole) while every GREEN/CTRL arm keeps
# its verdict; if a RED arm already blocked there, the fix bought nothing.
#
# Scratch lives under rust/target/ (repo volume) per the project data-locality policy.
set -uo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../../.." && pwd)"
CHECK="${PGEN_DIAG_CHECK_OVERRIDE:-${ROOT_DIR}/scripts/check_diagnosis_evidence.sh}"
[ "${CHECK#/}" = "$CHECK" ] && CHECK="${ROOT_DIR}/${CHECK}"
WORK="${ROOT_DIR}/rust/target/doctrine_checks/diag_evidence_leaf_scope_probes"

rm -rf "${WORK}"; mkdir -p "${WORK}"
pass_count=0; fail_count=0

# A complete, compliant checklist block for a leaf.
checklist() {
  printf -- '- [x] **ROOT CAUSE (WHY + WHERE)** — %s\n' "${1:-\`--report-certificate-coverage\` showed UNKNOWN=3; the [plannable-probe] verdict named the rule.}"
  printf -- '- [x] **ADDRESSED (verified)** — UNKNOWN 3 -> 0 on the same command.\n'
  printf -- '- [x] **NO REGRESSION** — cert deterministic at seeds 0/7/42, fully_certified.\n'
}

new_repo() {
  local d="${WORK}/$1"
  rm -rf "$d"; mkdir -p "$d/docs/tasks" "$d/rust/src" "$d/scripts"
  git -C "$d" init -q
  git -C "$d" config user.email probe@pgen.local
  git -C "$d" config user.name probe
  cp "${CHECK}" "$d/scripts/check_diagnosis_evidence.sh"
  # The enforcer copy must be COMMITTED, not left staged: `.5` widened `code_changed` to include
  # scripts/check_*.sh, so a staged copy would make every arm look like a code change and break
  # the "no code change staged" control.
  git -C "$d" add scripts/check_diagnosis_evidence.sh >/dev/null 2>&1
  git -C "$d" commit -qm "probe base: the enforcer itself" >/dev/null 2>&1
  printf '%s' "$d"
}

code_change() { printf 'fn main() {}\n' >"$1/rust/src/lib.rs"; }

run_arm() {
  # run_arm <name> <expected_exit> <expected_substring> <repo_dir>
  local name="$1" expect_rc="$2" expect_msg="$3" d="$4" rc=0
  local log="${WORK}/${name}.log"
  ( cd "$d" && bash scripts/check_diagnosis_evidence.sh ) >"$log" 2>&1 || rc=$?
  local ok_msg="yes"
  [ -n "$expect_msg" ] && { grep -qF -- "$expect_msg" "$log" || ok_msg="no"; }
  if [ "$rc" = "$expect_rc" ] && [ "$ok_msg" = "yes" ]; then
    printf 'PROBE %-8s PASS  exit=%s (expected %s)\n' "$name" "$rc" "$expect_rc"
    pass_count=$((pass_count + 1))
  else
    printf 'PROBE %-8s FAIL  exit=%s (expected %s, msg_match=%s)\n' "$name" "$rc" "$expect_rc" "$ok_msg"
    sed 's/^/  /' "$log" | tail -n 12
    fail_count=$((fail_count + 1))
  fi
}

NOT_OWNED="LEAF THIS CHANGE DID NOT TOUCH"
ACROSS="ACROSS DIFFERENT task files"
NOT_IN_BOX="NOT INSIDE THAT BOX"
OK_MSG="OK (task leaf passes"

# ------------------------------------ RED ARMS ------------------------------------------------
# H1 — the borrow within one file. Leaf `.1` is committed and finished; leaf `.2` owns today's
# code change and writes nothing. Only `.2`'s section is touched.
d="$(new_repo redh1)"
{ printf '# TREE-A\n\n### `.1` — an old, completed leaf\n\n#### Acceptance Checklist (enforced)\n\n'; checklist; } >"$d/docs/tasks/TREE-A.md"
git -C "$d" add -A >/dev/null 2>&1; git -C "$d" commit -qm "leaf .1 landed" >/dev/null 2>&1
printf '\n### `.2` — the NEW leaf owning today'"'"'s code change\n\n- Status: in progress, no checklist written.\n' >>"$d/docs/tasks/TREE-A.md"
code_change "$d"; git -C "$d" add -A >/dev/null 2>&1
run_arm RED-H1 1 "$NOT_OWNED" "$d"

# H2 — the borrow across files. TREE-B owns the change; TREE-A is an unrelated tree whose
# checklist section is not touched (only its title line is).
d="$(new_repo redh2)"
{ printf '# TREE-A\n\n### `.1` — unrelated, finished\n\n#### Acceptance Checklist (enforced)\n\n'; checklist; } >"$d/docs/tasks/TREE-A.md"
printf '# TREE-B\n\n### `.1` — placeholder\n' >"$d/docs/tasks/TREE-B.md"
git -C "$d" add -A >/dev/null 2>&1; git -C "$d" commit -qm "both trees exist" >/dev/null 2>&1
printf '# TREE-A (title touched by a typo fix)\n' >"$d/docs/tasks/TREE-A.md.new"
{ printf '# TREE-A — typo fixed\n'; tail -n +2 "$d/docs/tasks/TREE-A.md"; } >"$d/docs/tasks/TREE-A.md.tmp"
mv "$d/docs/tasks/TREE-A.md.tmp" "$d/docs/tasks/TREE-A.md"; rm -f "$d/docs/tasks/TREE-A.md.new"
printf '\n### `.2` — the leaf that ACTUALLY owns today'"'"'s change\n\n- No checklist written.\n' >>"$d/docs/tasks/TREE-B.md"
code_change "$d"; git -C "$d" add -A >/dev/null 2>&1
run_arm RED-H2 1 "$NOT_OWNED" "$d"

# H3 — the three boxes satisfied only across two files, each box freshly written.
d="$(new_repo redh3)"
{ printf '# TREE-A\n\n### `.1` — owns the change\n\n'
  printf -- '- [x] **ROOT CAUSE (WHY + WHERE)** — `--report-certificate-coverage` UNKNOWN=3, [plannable-probe] parsed=true.\n'
  printf -- '- [x] **ADDRESSED (verified)** — UNKNOWN 3 -> 0.\n'; } >"$d/docs/tasks/TREE-A.md"
{ printf '# TREE-B\n\n### `.1` — a different tree, touched today\n\n'
  printf -- '- [x] **NO REGRESSION** — cert deterministic at seeds 0/7/42, fully_certified.\n'; } >"$d/docs/tasks/TREE-B.md"
code_change "$d"; git -C "$d" add -A >/dev/null 2>&1
run_arm RED-H3 1 "$ACROSS" "$d"

# ----------------------------------- GREEN ARMS -----------------------------------------------
d="$(new_repo green1)"
{ printf '# TREE-A\n\n### `.1` — the owning leaf\n\n#### Acceptance Checklist (enforced)\n\n'; checklist; } >"$d/docs/tasks/TREE-A.md"
code_change "$d"; git -C "$d" add -A >/dev/null 2>&1
run_arm GREEN-1 0 "$OK_MSG" "$d"

# GREEN-2 — the multi-commit leaf. The checklist landed in an earlier commit; today's follow-up
# edits the SAME leaf's narrative and touches no box. This MUST still pass.
d="$(new_repo green2)"
{ printf '# TREE-A\n\n### `.1` — the owning leaf\n\n#### Acceptance Checklist (enforced)\n\n'; checklist
  printf '\n#### Notes\n\n- initial note\n'; } >"$d/docs/tasks/TREE-A.md"
git -C "$d" add -A >/dev/null 2>&1; git -C "$d" commit -qm "leaf .1 checklist landed" >/dev/null 2>&1
printf -- '- a follow-up note added by today'"'"'s commit, inside the same leaf\n' >>"$d/docs/tasks/TREE-A.md"
code_change "$d"; git -C "$d" add -A >/dev/null 2>&1
run_arm GREEN-2 0 "$OK_MSG" "$d"

# GREEN-3 — a deletion-only edit inside the owning leaf (pure-deletion hunks must still count as
# touching the section, else a docs-trimming commit is blocked for the wrong reason).
d="$(new_repo green3)"
{ printf '# TREE-A\n\n### `.1` — the owning leaf\n\n#### Acceptance Checklist (enforced)\n\n'; checklist
  printf '\n#### Notes\n\n- line to delete\n- line to keep\n'; } >"$d/docs/tasks/TREE-A.md"
git -C "$d" add -A >/dev/null 2>&1; git -C "$d" commit -qm "leaf .1 landed" >/dev/null 2>&1
grep -v 'line to delete' "$d/docs/tasks/TREE-A.md" >"$d/docs/tasks/TREE-A.tmp" && mv "$d/docs/tasks/TREE-A.tmp" "$d/docs/tasks/TREE-A.md"
code_change "$d"; git -C "$d" add -A >/dev/null 2>&1
run_arm GREEN-3 0 "$OK_MSG" "$d"

# ---------------------------------- CONTROL ARMS ----------------------------------------------
d="$(new_repo ctrl1)"
{ printf '# TREE-A\n\n### `.1` — the owning leaf\n\n'
  printf -- '- [ ] **ROOT CAUSE (WHY + WHERE)** — pending; `--trace-rules` not yet run.\n'
  printf -- '- [x] **ADDRESSED (verified)** — before -> after.\n'
  printf -- '- [x] **NO REGRESSION** — seeds 0/7/42.\n'; } >"$d/docs/tasks/TREE-A.md"
code_change "$d"; git -C "$d" add -A >/dev/null 2>&1
run_arm CTRL-1 1 "UNTICKED" "$d"

d="$(new_repo ctrl2)"
{ printf '# TREE-A\n\n### `.1` — the owning leaf\n\n'
  printf -- '- [x] **ROOT CAUSE (WHY + WHERE)** — we investigated and are confident.\n'
  printf -- '- [x] **ADDRESSED (verified)** — before -> after.\n'
  printf -- '- [x] **NO REGRESSION** — seeds 0/7/42, fully_certified.\n'
  printf '\nNarrative outside the box: `--report-certificate-coverage` UNKNOWN=0, [plannable-probe] parsed=true.\n'; } >"$d/docs/tasks/TREE-A.md"
code_change "$d"; git -C "$d" add -A >/dev/null 2>&1
run_arm CTRL-2 1 "$NOT_IN_BOX" "$d"

d="$(new_repo ctrl3)"
printf '# TREE-A\n\ndocs-only change\n' >"$d/docs/tasks/TREE-A.md"
git -C "$d" add -A >/dev/null 2>&1
run_arm CTRL-3 0 "no code change staged" "$d"

printf '\nprobes: %d passed, %d failed  (enforcer: %s)\n' "$pass_count" "$fail_count" "$CHECK"
[ "$fail_count" -eq 0 ] || exit 1
exit 0

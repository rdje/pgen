#!/usr/bin/env bash
# docs/tasks/artifacts/generated_lint_correctness/run_proof_surface_scope_probes.sh
#
# GENERATED-LINT-CORRECTNESS.5 — RED / GREEN / CONTROL probes for widening `code_changed` in
# scripts/check_diagnosis_evidence.sh to cover THE PROOF SURFACE (the doctrine enforcers, the gate
# scripts, their Makefile wiring, the git hooks, the CI workflows, build.rs).
#
# The defect being closed: measured across 2,618 commits, 521 touched that surface and 397 of them
# (76%) staged no path from the old predicate, so the acceptance gate answered "no code change
# staged" and required nothing — of exactly the commits that could have caught every rot class
# found in sessions #214-#216.
#
# ⛔ THE RISK THIS MUST NOT INTRODUCE is over-binding: a corpus file, a test fixture or an ordinary
# tool must NOT suddenly demand a root-cause checklist. CTRL-3/4/5 exist to pin that boundary, and
# they are the arms to watch if this predicate is ever widened again.
#
# Arms:
#   RED-1  a gate script staged, no checklist            -> BLOCK  (was ALLOW before .5)
#   RED-2  a doctrine enforcer staged, no checklist      -> BLOCK  (was ALLOW before .5)
#   RED-3  rust/Makefile staged, no checklist            -> BLOCK  (was ALLOW before .5)
#   RED-4  a CI workflow staged, no checklist            -> BLOCK  (was ALLOW before .5)
#   RED-5  rust/build.rs staged, no checklist            -> BLOCK  (was ALLOW before .5)
#   GREEN-1 a gate script + an ops/build-flow-backed checklist -> ALLOW
#   GREEN-2 a doctrine enforcer + the same               -> ALLOW
#   CTRL-1 rust/src staged + checklist                   -> ALLOW  (unchanged behaviour)
#   CTRL-2 rust/src staged, no checklist                 -> BLOCK  (unchanged behaviour)
#   CTRL-3 docs only                                     -> ALLOW  (must NOT be bound)
#   CTRL-4 a test corpus / fixture only                  -> ALLOW  (must NOT be bound)
#   CTRL-5 an ordinary non-gate tool script only         -> ALLOW  (must NOT be bound)
#
# Replay against the pre-.5 enforcer to produce the before->after:
#   git show HEAD:scripts/check_diagnosis_evidence.sh > rust/target/head_check.sh
#   PGEN_DIAG_CHECK_OVERRIDE=$PWD/rust/target/head_check.sh bash <this script>
# Under the old predicate the five RED arms MUST pass-through as "no code change staged".
set -uo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../../.." && pwd)"
CHECK="${PGEN_DIAG_CHECK_OVERRIDE:-${ROOT_DIR}/scripts/check_diagnosis_evidence.sh}"
WORK="${ROOT_DIR}/rust/target/doctrine_checks/proof_surface_scope_probes"

rm -rf "${WORK}"; mkdir -p "${WORK}"
pass_count=0; fail_count=0

# new_repo <name> <staged-file-path>  — builds a throwaway repo staging exactly that one file.
#
# ⚠️ THE HARNESS MUST COMMIT ITS OWN COPY OF THE ENFORCER FIRST, and this is not incidental
# plumbing — the CONTROL arms caught it. The probe repo needs `scripts/check_diagnosis_evidence.sh`
# to run the check at all, but `.5`'s widened predicate matches `scripts/check_*.sh` (correctly:
# editing the enforcer MUST require a checklist). Leaving that copy in the staged set made every
# arm look like a proof-surface change, so CTRL-3/4/5 (docs / corpus / ordinary tool — all of which
# must stay UNBOUND) failed for a reason that had nothing to do with the predicate under test.
# Committing it first takes it out of `git diff --cached`, so each arm stages EXACTLY its one file.
# Same lesson as `.3`: write the control arms, and believe them when they fail.
new_repo() {
  local d="${WORK}/$1" f="$2"
  rm -rf "$d"; mkdir -p "$d/docs/tasks" "$d/scripts"
  git -C "$d" init -q
  git -C "$d" config user.email probe@pgen.local
  git -C "$d" config user.name probe
  cp "${CHECK}" "$d/scripts/check_diagnosis_evidence.sh"
  git -C "$d" add scripts/check_diagnosis_evidence.sh >/dev/null 2>&1
  git -C "$d" commit -qm "probe base: the enforcer itself, committed so it is not in the staged set" >/dev/null 2>&1
  mkdir -p "$d/$(dirname "$f")"
  printf '# probe\n' > "$d/$f"
  printf '%s' "$d"
}

run_arm() {
  local name="$1" expect_rc="$2" expect_msg="$3" d="$4" rc=0
  local log="${WORK}/${name}.log"
  ( cd "$d" && bash scripts/check_diagnosis_evidence.sh ) >"$log" 2>&1 || rc=$?
  local ok_msg="yes"
  [ -n "$expect_msg" ] && { grep -qF -- "$expect_msg" "$log" || ok_msg="no"; }
  if [ "$rc" = "$expect_rc" ] && [ "$ok_msg" = "yes" ]; then
    printf 'PROBE %-9s PASS  exit=%s (expected %s)\n' "$name" "$rc" "$expect_rc"
    pass_count=$((pass_count + 1))
  else
    printf 'PROBE %-9s FAIL  exit=%s (expected %s, msg_match=%s)\n' "$name" "$rc" "$expect_rc" "$ok_msg"
    sed 's/^/  /' "$log" | tail -n 6
    fail_count=$((fail_count + 1))
  fi
}

write_leaf() {   # write_leaf <repo> ; a full ops/build-flow-backed acceptance checklist
  printf '# TREE-A\n\n#### Acceptance Checklist (enforced)\n\n' >"$1/docs/tasks/TREE-A.md"
  {
    printf -- '- [x] **ROOT CAUSE (WHY + WHERE)** — the recipe swallowed a nonzero exit; `make -n` shows the compound and `git ls-files` confirms the artifact is untracked.\n'
    printf -- '- [x] **ADDRESSED (verified)** — before -> after on the symptom.\n'
    printf -- '- [x] **NO REGRESSION** — cert coverage deterministic at seeds 0/7/42, fully_certified.\n'
  } >>"$1/docs/tasks/TREE-A.md"
}

NO_CODE="no code change staged"
NEEDS="NO owning task-tree leaf"
OK_MSG="OK (task leaf passes"

# ---------------------------------- RED ARMS --------------------------------------------------
for spec in "RED-1:rust/scripts/some_quality_gate.sh" \
            "RED-2:scripts/check_something.sh" \
            "RED-3:rust/Makefile" \
            "RED-4:.github/workflows/some-gate.yml" \
            "RED-5:rust/build.rs"; do
  name="${spec%%:*}"; path="${spec#*:}"
  d="$(new_repo "${name,,}" "$path")"
  git -C "$d" add -A >/dev/null 2>&1
  run_arm "$name" 1 "$NEEDS" "$d"
done

# --------------------------------- GREEN ARMS -------------------------------------------------
d="$(new_repo green1 "rust/scripts/some_quality_gate.sh")"; write_leaf "$d"
git -C "$d" add -A >/dev/null 2>&1
run_arm GREEN-1 0 "$OK_MSG" "$d"

d="$(new_repo green2 "scripts/check_something.sh")"; write_leaf "$d"
git -C "$d" add -A >/dev/null 2>&1
run_arm GREEN-2 0 "$OK_MSG" "$d"

# -------------------------------- CONTROL ARMS ------------------------------------------------
d="$(new_repo ctrl1 "rust/src/lib.rs")"; write_leaf "$d"
git -C "$d" add -A >/dev/null 2>&1
run_arm CTRL-1 0 "$OK_MSG" "$d"

d="$(new_repo ctrl2 "rust/src/lib.rs")"
git -C "$d" add -A >/dev/null 2>&1
run_arm CTRL-2 1 "$NEEDS" "$d"

# ⛔ The over-binding boundary. These MUST stay unbound.
d="$(new_repo ctrl3 "docs/book/src/some-chapter.md")"
git -C "$d" add -A >/dev/null 2>&1
run_arm CTRL-3 0 "$NO_CODE" "$d"

d="$(new_repo ctrl4 "rust/test_data/grammar_quality/some_corpus.txt")"
git -C "$d" add -A >/dev/null 2>&1
run_arm CTRL-4 0 "$NO_CODE" "$d"

d="$(new_repo ctrl5 "tools/some_helper.pl")"
git -C "$d" add -A >/dev/null 2>&1
run_arm CTRL-5 0 "$NO_CODE" "$d"

printf '\nprobes: %d passed, %d failed\n' "$pass_count" "$fail_count"
[ "$fail_count" -eq 0 ] || exit 1
exit 0

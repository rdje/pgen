#!/usr/bin/env bash
# docs/tasks/artifacts/generated_lint_correctness/run_diag_evidence_family5_probes.sh
#
# GENERATED-LINT-CORRECTNESS.4 — RED / GREEN / CONTROL probes for the two changes this leaf makes
# to scripts/check_diagnosis_evidence.sh:
#
#   (A) GROUP 2's tool vocabulary CORRECTED — the performance family named `cargo flamegraph` /
#       `self-time` (generic Rust) while this repo's SPEED campaign measures with macOS
#       `/usr/bin/sample`, `otool -tV`, `spindump`/`filtercalltree` and `ITIMER_PROF`.
#   (B) A FIFTH signature family, OPS / BUILD-FLOW, for defects in the repository's own
#       operational surface (scripts, Makefiles, hooks, tracking state), which have no parse to
#       trace, no run to sample, no rustc error and no codegen emission.
#
# ⛔ THE POINT OF THE RED ARMS. `.3`'s probes caught a real bug in its FIRST implementation (the
# keyword matched the BODY not the HEADER, so a box merely MENTIONING "root cause" stood in). The
# danger with THIS leaf is the opposite one: an over-broad ops token that degrades the gate to
# "mention a shell command". RED-4/RED-5 exist to prove the two most tempting over-broad forms —
# a bare `grep` mention and a bare `file.rs:NNN` citation — are still REFUSED.
#
# Arms:
#   RED-1  fabricated trust-me prose, no signature anywhere      -> BLOCK
#   RED-2  a bare `file.rs:NNN` citation and nothing else        -> BLOCK  (the "cite a line
#                                                                  number" degradation .4 refuses)
#   RED-3  "verified by grep" prose                              -> BLOCK  (a claim, not output;
#                                                                  measured to match 16 real boxes)
#   RED-4  ops signature present but OUTSIDE the ticked box      -> BLOCK  (box-scoping still holds
#                                                                  for the new family)
#   GREEN-1 corrected group 2: /usr/bin/sample                   -> ALLOW  (BLOCKED before .4)
#   GREEN-2 corrected group 2: otool -tV annotated disassembly   -> ALLOW  (BLOCKED before .4)
#   GREEN-3 group 5: the REAL RGX-0090 Makefile exit-swallow box -> ALLOW  (BLOCKED before .4)
#   GREEN-4 group 5: the REAL SV-REPLAY-DEBT ARG_MAX/E2BIG box   -> ALLOW  (BLOCKED before .4)
#   GREEN-5 group 5: the REAL REPO-HYGIENE git ls-files box      -> ALLOW  (BLOCKED before .4)
#   CTRL-1  pre-existing correctness family still passes         -> ALLOW  (no regression)
#   CTRL-2  pre-existing codegen-emission family still passes    -> ALLOW  (no regression)
#   CTRL-3  ROOT CAUSE box unticked                              -> BLOCK  (unchanged behaviour)
#   CTRL-4  no code staged at all                                -> ALLOW  (check not required)
#
# GREEN-3/4/5 deliberately quote the REAL box text from the tracked leaves rather than synthetic
# prose: the claim being tested is "the corpus this family was designed for now passes", and only
# the real text can test that.
#
# Scratch lives under rust/target/ (repo volume) per the project data-locality policy.
# Usage: bash docs/tasks/artifacts/generated_lint_correctness/run_diag_evidence_family5_probes.sh
set -uo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../../.." && pwd)"
# PGEN_DIAG_CHECK_OVERRIDE lets the SAME arms be replayed against an OLDER enforcer, which is how
# the before->after is produced rather than asserted:
#   git show HEAD:scripts/check_diagnosis_evidence.sh > rust/target/head_check.sh
#   PGEN_DIAG_CHECK_OVERRIDE=rust/target/head_check.sh bash <this script>
# Under the pre-.4 enforcer the five GREEN arms MUST fail; if they pass, the change bought nothing.
CHECK="${PGEN_DIAG_CHECK_OVERRIDE:-${ROOT_DIR}/scripts/check_diagnosis_evidence.sh}"
WORK="${ROOT_DIR}/rust/target/doctrine_checks/diag_evidence_family5_probes"

rm -rf "${WORK}"; mkdir -p "${WORK}"
pass_count=0; fail_count=0

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
    printf 'PROBE %-9s PASS  exit=%s (expected %s)\n' "$name" "$rc" "$expect_rc"
    pass_count=$((pass_count + 1))
  else
    printf 'PROBE %-9s FAIL  exit=%s (expected %s, msg_match=%s)\n' "$name" "$rc" "$expect_rc" "$ok_msg"
    sed 's/^/  /' "$log" | tail -n 20
    fail_count=$((fail_count + 1))
  fi
}

# leaf <root-cause-body-text> [trailing-out-of-box-narrative]
leaf() {
  printf '# TREE-A\n\n#### Acceptance Checklist (enforced)\n\n'
  printf -- '- [x] **ROOT CAUSE (WHY + WHERE)** — %s\n' "$1"
  printf -- '- [x] **ADDRESSED (verified)** — before -> after on the symptom.\n'
  printf -- '- [x] **NO REGRESSION** — cert coverage deterministic at seeds 0/7/42, fully_certified.\n'
  [ -n "${2:-}" ] && printf '\n%s\n' "$2"
  return 0
}

NOT_IN_BOX="NOT INSIDE THAT BOX"
OK_MSG="OK (task leaf passes"

# ---------------------------------- RED ARMS --------------------------------------------------
d="$(new_repo red1)"
leaf 'We investigated thoroughly and are confident the cause is the branch selector.' \
  >"$d/docs/tasks/TREE-A.md"
git -C "$d" add -A >/dev/null 2>&1
run_arm RED-1 1 "$NOT_IN_BOX" "$d"

d="$(new_repo red2)"
leaf 'The defect is at `rust/src/ast_pipeline/ast_based_generator.rs:4687`, in the tournament emitter.' \
  >"$d/docs/tasks/TREE-A.md"
git -C "$d" add -A >/dev/null 2>&1
run_arm RED-2 1 "$NOT_IN_BOX" "$d"

d="$(new_repo red3)"
leaf 'Verified by grep that no caller remains; a repo-wide grep confirms zero occurrences.' \
  >"$d/docs/tasks/TREE-A.md"
git -C "$d" add -A >/dev/null 2>&1
run_arm RED-3 1 "$NOT_IN_BOX" "$d"

d="$(new_repo red4)"
leaf 'The recipe swallows the failure.' \
  'Narrative section, outside the box: `git ls-files generated/ | wc -l` returns 0 and `make -n` shows the recipe.' \
  >"$d/docs/tasks/TREE-A.md"
git -C "$d" add -A >/dev/null 2>&1
run_arm RED-4 1 "$NOT_IN_BOX" "$d"

# --------------------------------- GREEN ARMS -------------------------------------------------
d="$(new_repo green1)"
leaf '`/usr/bin/sample` malloc-free decomposition: value machinery ≈66% (CONSTRUCT 26.1% + TEARDOWN 18.8%) ≫ MATCH/DISPATCH 18.9%.' \
  >"$d/docs/tasks/TREE-A.md"
git -C "$d" add -A >/dev/null 2>&1
run_arm GREEN-1 0 "$OK_MSG" "$d"

d="$(new_repo green2)"
leaf 'Instruction-decisive on the custody-verified floor probe (`otool -tV`): `slice_error_fail` 1,660→681.' \
  >"$d/docs/tasks/TREE-A.md"
git -C "$d" add -A >/dev/null 2>&1
run_arm GREEN-2 0 "$OK_MSG" "$d"

# GREEN-3/4/5 quote the REAL tracked box text this family was designed for.
d="$(new_repo green3)"
leaf 'reproduced tool-first in a scratch CARGO_TARGET_DIR — step C WITHOUT `--bootstrap-mode` prints the REFUSED error and exits **1**; the swallow is the `;`-chained `@if` seed compound in `rust/Makefile:regex_parser_bootstrap`, whose exit status is the last command'"'"'s. `make -n regex_parser_bootstrap` shows the compound.' \
  >"$d/docs/tasks/TREE-A.md"
git -C "$d" add -A >/dev/null 2>&1
run_arm GREEN-3 0 "$OK_MSG" "$d"

d="$(new_repo green4)"
leaf '`--argjson cases "$realistic_cases_json"` (`sv_stimuli_quality_gate.sh:2793`) puts the ~1 MB cases array on execve argv: measured 1,034,802 B vs `ARG_MAX=1,048,576` with env ⇒ E2BIG.' \
  >"$d/docs/tasks/TREE-A.md"
git -C "$d" add -A >/dev/null 2>&1
run_arm GREEN-4 0 "$OK_MSG" "$d"

d="$(new_repo green5)"
leaf 'accumulated editor/session residue and legacy pre-Rust-era run outputs; all confirmed **untracked** via `git ls-files --error-unmatch` per file, and unreferenced by any gate/script/doc.' \
  >"$d/docs/tasks/TREE-A.md"
git -C "$d" add -A >/dev/null 2>&1
run_arm GREEN-5 0 "$OK_MSG" "$d"

# -------------------------------- CONTROL ARMS ------------------------------------------------
d="$(new_repo ctrl1)"
leaf 'Diagnosed with --report-certificate-coverage and the [plannable-probe] verdicts.' \
  >"$d/docs/tasks/TREE-A.md"
git -C "$d" add -A >/dev/null 2>&1
run_arm CTRL-1 0 "$OK_MSG" "$d"

d="$(new_repo ctrl2)"
leaf 'The emission site interpolates a codegen constant; the lint lane reports clippy::eq_op over the artifacts.' \
  >"$d/docs/tasks/TREE-A.md"
git -C "$d" add -A >/dev/null 2>&1
run_arm CTRL-2 0 "$OK_MSG" "$d"

d="$(new_repo ctrl3)"
{ printf '# TREE-A\n\n'
  printf -- '- [ ] **ROOT CAUSE (WHY + WHERE)** — pending; `/usr/bin/sample` not yet run.\n'
  printf -- '- [x] **ADDRESSED (verified)** — before -> after.\n'
  printf -- '- [x] **NO REGRESSION** — seeds 0/7/42.\n'; } >"$d/docs/tasks/TREE-A.md"
git -C "$d" add -A >/dev/null 2>&1
run_arm CTRL-3 1 "UNTICKED" "$d"

d="$(new_repo ctrl4)"
rm -f "$d/rust/src/lib.rs"
printf '# TREE-A\n\ndocs-only change\n' >"$d/docs/tasks/TREE-A.md"
git -C "$d" add -A >/dev/null 2>&1
run_arm CTRL-4 0 "no code change staged" "$d"

printf '\nprobes: %d passed, %d failed\n' "$pass_count" "$fail_count"
[ "$fail_count" -eq 0 ] || exit 1
exit 0

#!/usr/bin/env bash
# docs/tasks/artifacts/generated_lint_correctness/run_generated_clippy_gate_probes.sh
# GENERATED-LINT-CORRECTNESS.3 — RED / GREEN / CONTROL probes for
# rust/scripts/generated_clippy_correctness_gate.sh.
#
# A gate is only worth its green if it can be shown to go RED. Each arm below drives the gate
# into ONE failure mode and asserts the exit code AND the message, so "it passed" is evidence
# rather than a hope. The expensive arm (a real ~12 GB cargo clippy over 210.8 MB of generated
# code) is deliberately NOT here — it is the gate's own GREEN run, recorded separately.
#
#   RED-A  a REQUIRED generated artifact is absent            -> exit 2, names the artifact
#   RED-B  a pinned roster lint is not in clippy::correctness -> exit 2, names the departure
#   RED-C  the contract's roster_size disagrees with its list -> exit 2, self-inconsistency
#   RED-D  a real correctness finding is reported             -> exit 1, names lint+file+line
#   RED-E  artifacts on disk but NOT compiled in (vacuity)    -> exit 2, REFUSES to return green
#   CTRL   the shipped contract, clean fixture                -> exit 0
#
# RED-D/RED-E/CTRL use PGEN_GENERATED_CLIPPY_FIXTURE, the gate's recorded-JSON seam, so the
# census + verdict logic is exercised without a rebuild. All scratch lives under rust/target/
# (repo volume, gitignored) per the project data-locality policy.
#
# Usage: bash docs/tasks/artifacts/generated_lint_correctness/run_generated_clippy_gate_probes.sh
set -uo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../../.." && pwd)"
GATE="${ROOT_DIR}/rust/scripts/generated_clippy_correctness_gate.sh"
CONTRACT="${ROOT_DIR}/rust/test_data/grammar_quality/generated_clippy_correctness_contract_v0.json"
WORK="${ROOT_DIR}/rust/target/generated_clippy_correctness_gate/probes"

rm -rf "${WORK}"; mkdir -p "${WORK}"

pass_count=0; fail_count=0

arm() {
  # arm <name> <expected_exit> <expected_substring> -- <env assignments...>
  local name="$1" expect_rc="$2" expect_msg="$3"; shift 3
  [ "${1:-}" = "--" ] && shift
  local log="${WORK}/${name}.log" rc=0
  env "$@" PGEN_GENERATED_CLIPPY_REPORT_DIR="${WORK}/${name}.report" \
    bash "${GATE}" >"${log}" 2>&1 || rc=$?
  local ok_rc="no" ok_msg="no"
  [ "${rc}" = "${expect_rc}" ] && ok_rc="yes"
  grep -qF -- "${expect_msg}" "${log}" && ok_msg="yes"
  if [ "${ok_rc}" = "yes" ] && [ "${ok_msg}" = "yes" ]; then
    printf 'PROBE %-7s PASS  exit=%s (expected %s) and message matched: %s\n' \
      "${name}" "${rc}" "${expect_rc}" "${expect_msg}"
    pass_count=$((pass_count + 1))
  else
    printf 'PROBE %-7s FAIL  exit=%s (expected %s, match=%s) expected message: %s\n' \
      "${name}" "${rc}" "${expect_rc}" "${ok_msg}" "${expect_msg}"
    printf '  ---- %s ----\n' "${log}"
    sed 's/^/  /' "${log}" | tail -n 25
    fail_count=$((fail_count + 1))
  fi
}

# ---- fixture builders -------------------------------------------------------
# A build-script-executed message carrying every cfg the contract requires.
cfgs_json() {
  jq -c '[.generated_artifacts[] | select(.required) | select(.cfg_guarded) | .cfg]' "${CONTRACT}"
}

fixture_clean="${WORK}/fixture_clean.json"
{
  printf '{"reason":"build-script-executed","package_id":"pgen 0.1.0","cfgs":%s,"env":[],"out_dir":"/x"}\n' "$(cfgs_json)"
  # A style warning over generated code: present in real runs, and correctly IGNORED by the
  # gate (it censuses level=="error" only). Guards against the census over-reporting.
  printf '%s\n' '{"reason":"compiler-message","message":{"level":"warning","message":"variable does not need to be mutable","code":{"code":"clippy::unused_mut"},"spans":[{"is_primary":true,"file_name":"src/../../generated/systemverilog_parser.rs","line_start":424242}]}}'
} >"${fixture_clean}"

fixture_finding="${WORK}/fixture_finding.json"
{
  cat "${fixture_clean}"
  printf '%s\n' '{"reason":"compiler-message","message":{"level":"error","message":"equal expressions as operands to `==`","code":{"code":"clippy::eq_op"},"spans":[{"is_primary":true,"file_name":"src/../../generated/systemverilog_parser.rs","line_start":991234}]}}'
} >"${fixture_finding}"

# Same as clean, but build.rs emitted NO cfgs — exactly what a clean checkout / CI runner /
# ci_workflow_local_gate export dir produces, because generated/ is untracked.
fixture_vacuous="${WORK}/fixture_vacuous.json"
printf '%s\n' '{"reason":"build-script-executed","package_id":"pgen 0.1.0","cfgs":[],"env":[],"out_dir":"/x"}' >"${fixture_vacuous}"

# ---- RED-A: a required artifact is absent -----------------------------------
# Re-point one required artifact at a path that does not exist, leaving the real tree alone.
contract_missing="${WORK}/contract_missing_artifact.json"
jq '(.generated_artifacts[] | select(.path == "generated/regex_parser.rs") | .path)
      = "generated/DOES_NOT_EXIST_probe_parser.rs"' "${CONTRACT}" >"${contract_missing}"
arm RED-A 2 "missing: generated/DOES_NOT_EXIST_probe_parser.rs" -- \
  PGEN_GENERATED_CLIPPY_CONTRACT="${contract_missing}"

# ---- RED-B: a pinned roster lint left clippy::correctness -------------------
contract_drift="${WORK}/contract_roster_drift.json"
jq '.lint_policy.correctness_roster += ["pgen_probe_lint_that_is_not_correctness"]
    | .lint_policy.correctness_roster_size += 1' "${CONTRACT}" >"${contract_drift}"
arm RED-B 2 "departed: clippy::pgen_probe_lint_that_is_not_correctness" -- \
  PGEN_GENERATED_CLIPPY_CONTRACT="${contract_drift}"

# ---- RED-C: contract self-inconsistency -------------------------------------
contract_badsize="${WORK}/contract_bad_size.json"
jq '.lint_policy.correctness_roster_size = 999' "${CONTRACT}" >"${contract_badsize}"
arm RED-C 2 "correctness_roster_size=999" -- \
  PGEN_GENERATED_CLIPPY_CONTRACT="${contract_badsize}"

# ---- RED-D: a real correctness finding is caught ----------------------------
arm RED-D 1 "clippy::eq_op" -- \
  PGEN_GENERATED_CLIPPY_FIXTURE="${fixture_finding}"

# ---- RED-E: the vacuity refusal (THE one that matters) ----------------------
arm RED-E 2 "not-compiled-in: generated/systemverilog_parser.rs" -- \
  PGEN_GENERATED_CLIPPY_FIXTURE="${fixture_vacuous}"

# ---- CONTROL: shipped contract, clean stream -> pass ------------------------
arm CTRL 0 "✅ PASS" -- \
  PGEN_GENERATED_CLIPPY_FIXTURE="${fixture_clean}"

printf '\nprobes: %d passed, %d failed\n' "${pass_count}" "${fail_count}"
[ "${fail_count}" -eq 0 ] || exit 1
exit 0

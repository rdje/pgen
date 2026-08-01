#!/usr/bin/env bash
# SV-EXH-PROOF.7.4.6.10 — GROUND TRUTH for the closed-loop residual ratchet.
#
# The leaf's Goal is emphatic: "⛔ PROVE BOTH DIRECTIONS FIRE before trusting it — an unproven
# ratchet is an instrument without ground truth". This probe is that proof, and it does NOT
# stub the ratchet: every case drives the REAL gate script end-to-end, through the real call
# site, with only the CONTRACT'S PINNED NUMBER planted.
#
# It runs at a deliberately tiny but REAL configuration (one profile, count 1, replay count 1,
# target_max_attempts 64) so a case costs minutes rather than the canonical ~29 minutes. The
# residual at that configuration is its own number — the probe LEARNS it from case `unpinned`
# and plants around it, so the probe never hard-codes a measurement it did not take.
#
# Cases, and what each one is the control for:
#   contract_drift  exit 2  the contract's own closed-loop configuration was edited without
#                           re-pinning -> REFUSE. Without this the ratchet could be disarmed by
#                           a one-line contract edit, which is the drift class the leaf exists
#                           to end. Fails in ~1 s, before any generation.
#   unpinned        exit 1  a profile with no pinned ceiling is a blind spot, not a pass.
#   pinned_exact    exit 0  ⭐ the POSITIVE control — the ratchet must also STAY QUIET when the
#                           residual is exactly at the pin, or "it fails" proves nothing.
#   regressed       exit 1  residual ABOVE the pin (pin = N-1) -> regression.
#   improved        exit 1  residual BELOW the pin (pin = N+1) -> "lower the ceiling", so a win
#                           is banked instead of silently lost.
#   env_override    exit 0  an env override moves the run off the pinned configuration -> SKIP,
#                           not fail. This is the promotion gates (`sv_parse_full_ratio_*`,
#                           `sv_declared_shadow_*`), which run this gate at their own counts and
#                           seeds against the DEFAULT contract; the ratchet must not break them.
#
# ⚠️ MEMORY: the gate's `build_ast_pipeline_for_sv_generation` stage is the peak, not the
# generation — a first attempt at `--budget-mb 12288` was killed there at a measured tree RSS of
# 12 415 MB (`reason=rss-budget`, machine still 75% free), so the cases below cap cargo's build
# parallelism. Run the driver with headroom:
#   scripts/run_with_memory_guard.sh --budget-mb 16384 --timeout-s 7200 -- \
#     bash docs/tasks/artifacts/sv_exh_proof/run_replay_target_ratchet_probes.sh
#
# Usage:  bash docs/tasks/artifacts/sv_exh_proof/run_replay_target_ratchet_probes.sh
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../../.." && pwd)"
RUST_DIR="$ROOT_DIR/rust"
GATE="$RUST_DIR/scripts/sv_stimuli_quality_gate.sh"
BASE_CONTRACT="$RUST_DIR/test_data/grammar_quality/systemverilog_core_v0_contract.json"
# Repo-volume only, derived from the repository root — never /tmp (data-locality policy).
PROBE_DIR="$RUST_DIR/target/sv_exh_proof_ratchet_probe"
PROBE_CONTRACT="$PROBE_DIR/probe_contract.json"

rm -rf "$PROBE_DIR"
mkdir -p "$PROBE_DIR"

# The tiny-but-real probe configuration. The disabled surfaces (semantic suites, perf budgets,
# realistic corpus, parseability shadow) are all OUTSIDE the ratchet's configuration string, so
# switching them off makes the probe cheap without changing what is under test.
write_probe_contract() {
    local measured_configuration="$1"
    local profiles_json="$2"
    jq --arg configuration "$measured_configuration" --argjson profiles "$profiles_json" '
        .sample_count = 1
        | .seed_base = 12001
        | .lrm_profiles.required_profiles = ["2017"]
        | .closed_loop.replay_sample_count = 1
        | .closed_loop.target_max_attempts = 64
        | .closed_loop.parseability_shadow_enabled = false
        | .closed_loop.replay_target_ceilings = {
              enforce: true,
              measured_configuration: $configuration,
              profiles: $profiles
          }
        | .semantic_contracts.enforce_declared_identifier_suite = false
        | .semantic_contracts.enforce_width_compatibility_suite = false
        | .semantic_contracts.enforce_port_binding_legality_suite = false
        | .semantic_contracts.enforce_package_qualification_suite = false
        | .semantic_contracts.enforce_context_legality_suite = false
        | .performance_budgets.enforce = false
        | .nexsim_realistic_corpus.enforce = false
    ' "$BASE_CONTRACT" >"$PROBE_CONTRACT"
}

# $1 = case name, $2.. = extra environment assignments for the gate
run_case() {
    local case_name="$1"
    shift
    local log="$PROBE_DIR/${case_name}.log"
    local exit_code=0
    (
        cd "$RUST_DIR"
        env \
            PGEN_SV_STIMULI_QUALITY_CONTRACT="$PROBE_CONTRACT" \
            PGEN_SV_STIMULI_QUALITY_STATE_DIR="$PROBE_DIR/state_${case_name}" \
            PGEN_SV_STIMULI_DIFF_MODE=0 \
            PGEN_SV_STIMULI_PERF_BUDGET_MODE=0 \
            PGEN_SV_STIMULI_REALISTIC_CORPUS_MODE=0 \
            PGEN_SV_STIMULI_CARGO_BUILD_JOBS=2 \
            "$@" \
            bash "$GATE"
    ) >"$log" 2>&1 || exit_code=$?
    printf '%s\n' "$exit_code"
}

failures=0
report() {
    local case_name="$1" expected_exit="$2" actual_exit="$3" expected_text="$4"
    local log="$PROBE_DIR/${case_name}.log"
    local verdict="PASS"
    if [[ "$actual_exit" != "$expected_exit" ]] || ! grep -qF "$expected_text" "$log"; then
        verdict="FAIL"
        failures=$((failures + 1))
    fi
    printf '[ratchet-probe] %-14s exit=%s (expected %s)  %s\n' "$case_name" "$actual_exit" "$expected_exit" "$verdict"
    printf '                 expected text: %s\n' "$expected_text"
    # The match line is DISPLAY only — the verdict above already ran `grep -qF`. Capture first so
    # a miss prints "<NOTHING>": in a pipeline the trailing `sed` would always succeed and swallow
    # grep's non-zero status, quietly printing an empty line where the evidence should be.
    local matched
    matched="$(grep -nF "$expected_text" "$log" | head -n 1 || true)"
    printf '                 matched: %s\n' "${matched:-<NOTHING>}"
}

echo "=== case contract_drift — a contract edit without a re-pin must REFUSE ==="
write_probe_contract "" '{}'
drift_exit="$(run_case contract_drift)"
report contract_drift 2 "$drift_exit" "the contract's own closed-loop configuration no longer matches"

# The drift refusal PRINTS the contract-only configuration string, so the probe reads the pinned
# configuration out of the instrument instead of duplicating the gate's own builder here.
PROBE_CONFIGURATION="$(sed -n 's/^  contract: \[\(.*\)\]$/\1/p' "$PROBE_DIR/contract_drift.log" | head -n 1)"
if [[ -z "$PROBE_CONFIGURATION" ]]; then
    echo "error: could not read the probe configuration out of the drift refusal" >&2
    exit 2
fi
echo "[ratchet-probe] probe configuration: $PROBE_CONFIGURATION"

echo "=== case unpinned — an unpinned profile is a blind spot, and this run measures N ==="
write_probe_contract "$PROBE_CONFIGURATION" '{}'
unpinned_exit="$(run_case unpinned)"
report unpinned 1 "$unpinned_exit" "has NO pinned closed-loop residual ceiling"

RESIDUAL="$(sed -n "s/.*has NO pinned closed-loop residual ceiling (measured \([0-9]\{1,\}\)).*/\1/p" "$PROBE_DIR/unpinned.log" | head -n 1)"
if [[ -z "$RESIDUAL" ]]; then
    echo "error: could not read the measured residual out of the unpinned refusal" >&2
    exit 2
fi
if (( RESIDUAL < 1 )); then
    echo "error: the probe configuration has a residual of ${RESIDUAL}; the regression control needs a pin of ${RESIDUAL}-1" >&2
    exit 2
fi
echo "[ratchet-probe] probe residual N = $RESIDUAL"

echo "=== case pinned_exact — the POSITIVE control: at the pin the ratchet stays quiet ==="
write_probe_contract "$PROBE_CONFIGURATION" "{\"2017\": ${RESIDUAL}}"
exact_exit="$(run_case pinned_exact)"
report pinned_exact 0 "$exact_exit" "closed_loop_replay_target_ceiling_profiles_checked: 1/1"

echo "=== case regressed — residual ABOVE the pin fails as a regression ==="
write_probe_contract "$PROBE_CONFIGURATION" "{\"2017\": $((RESIDUAL - 1))}"
regressed_exit="$(run_case regressed)"
report regressed 1 "$regressed_exit" "closed-loop residual REGRESSED: ${RESIDUAL} > pinned ceiling $((RESIDUAL - 1))"

echo "=== case improved — residual BELOW the pin fails until the win is banked ==="
write_probe_contract "$PROBE_CONFIGURATION" "{\"2017\": $((RESIDUAL + 1))}"
improved_exit="$(run_case improved)"
report improved 1 "$improved_exit" "closed-loop residual IMPROVED to ${RESIDUAL} (pinned ceiling $((RESIDUAL + 1))) -- lower the ceiling"

echo "=== case env_override — an env override SKIPS the ratchet, it does not fail the run ==="
write_probe_contract "$PROBE_CONFIGURATION" "{\"2017\": ${RESIDUAL}}"
override_exit="$(run_case env_override PGEN_SV_STIMULI_QUALITY_SEED_BASE=990001)"
report env_override 0 "$override_exit" "environment overrides moved the run off the pinned configuration"

echo
if (( failures > 0 )); then
    echo "[ratchet-probe] ${failures} case(s) FAILED — the ratchet does not have ground truth"
    exit 1
fi
echo "[ratchet-probe] all 6 cases PASSED — both ratchet directions fire, the positive control stays quiet, contract drift refuses, and an env override skips"

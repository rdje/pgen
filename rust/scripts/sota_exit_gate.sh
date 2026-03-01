#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
RUST_DIR="$ROOT_DIR/rust"

STATE_DIR="${PGEN_SOTA_EXIT_STATE_DIR:-$RUST_DIR/target/sota_exit_gate}"
LOG_DIR="$STATE_DIR/logs"
SUMMARY_CSV="$STATE_DIR/summary.csv"
SUMMARY_TXT="$STATE_DIR/summary.txt"
POLICY_FILE="${PGEN_SOTA_POLICY_FILE:-$RUST_DIR/config/sota_exit_policy.env}"

if [[ ! -f "$POLICY_FILE" ]]; then
    echo "error: SOTA policy file not found at '$POLICY_FILE'" >&2
    exit 2
fi

# shellcheck disable=SC1090
source "$POLICY_FILE"

POLICY_VERSION="${PGEN_SOTA_POLICY_VERSION:-1}"
POLICY_REQUIRED_CHECKS="${PGEN_SOTA_POLICY_REQUIRED_CHECKS:-differential_baseline_contract fixed_point_gate annotation_contract_gate annotation_nonbootstrap_e2e_gate ebnf_stimuli_quality_gate stimuli_module_parity_gate differential_regression_gate performance_gate embedding_api_gate}"
POLICY_RUN_EBNF_READINESS="${PGEN_SOTA_POLICY_RUN_EBNF_READINESS:-1}"
POLICY_REQUIRE_EBNF_STRICT="${PGEN_SOTA_POLICY_REQUIRE_EBNF_STRICT:-0}"
POLICY_RUN_EBNF_DUAL_RUN_DIFF="${PGEN_SOTA_POLICY_RUN_EBNF_DUAL_RUN_DIFF:-1}"
POLICY_REQUIRE_EBNF_DUAL_RUN_STRICT="${PGEN_SOTA_POLICY_REQUIRE_EBNF_DUAL_RUN_STRICT:-0}"
POLICY_RUN_HDL_FRONTEND_READINESS="${PGEN_SOTA_POLICY_RUN_HDL_FRONTEND_READINESS:-0}"
POLICY_REQUIRE_HDL_FRONTEND_STRICT="${PGEN_SOTA_POLICY_REQUIRE_HDL_FRONTEND_STRICT:-0}"
POLICY_RUN_SV_PREPROCESSOR_QUALITY="${PGEN_SOTA_POLICY_RUN_SV_PREPROCESSOR_QUALITY:-0}"
POLICY_REQUIRE_SV_PREPROCESSOR_QUALITY_STRICT="${PGEN_SOTA_POLICY_REQUIRE_SV_PREPROCESSOR_QUALITY_STRICT:-0}"
POLICY_RUN_SV_STIMULI_QUALITY="${PGEN_SOTA_POLICY_RUN_SV_STIMULI_QUALITY:-0}"
POLICY_REQUIRE_SV_STIMULI_QUALITY_STRICT="${PGEN_SOTA_POLICY_REQUIRE_SV_STIMULI_QUALITY_STRICT:-0}"
POLICY_SV_STIMULI_ENFORCE_MIN_PARSE_FULL_PASS_RATIO="${PGEN_SOTA_POLICY_SV_STIMULI_ENFORCE_MIN_PARSE_FULL_PASS_RATIO:-0}"
POLICY_SV_STIMULI_MIN_PARSE_FULL_PASS_RATIO="${PGEN_SOTA_POLICY_SV_STIMULI_MIN_PARSE_FULL_PASS_RATIO:-0}"
POLICY_RUN_SV_DECLARED_SHADOW_PROMOTION="${PGEN_SOTA_POLICY_RUN_SV_DECLARED_SHADOW_PROMOTION:-0}"
POLICY_REQUIRE_SV_DECLARED_SHADOW_PROMOTION_STRICT="${PGEN_SOTA_POLICY_REQUIRE_SV_DECLARED_SHADOW_PROMOTION_STRICT:-0}"
POLICY_RUN_VHDL_STIMULI_QUALITY="${PGEN_SOTA_POLICY_RUN_VHDL_STIMULI_QUALITY:-0}"
POLICY_REQUIRE_VHDL_STIMULI_QUALITY_STRICT="${PGEN_SOTA_POLICY_REQUIRE_VHDL_STIMULI_QUALITY_STRICT:-0}"
POLICY_ALLOW_INFORMATIONAL_FAILURES="${PGEN_SOTA_POLICY_ALLOW_INFORMATIONAL_FAILURES:-1}"

RUN_EBNF_READINESS="${PGEN_SOTA_RUN_EBNF_READINESS:-$POLICY_RUN_EBNF_READINESS}"
REQUIRE_EBNF_STRICT="${PGEN_SOTA_REQUIRE_EBNF_STRICT:-$POLICY_REQUIRE_EBNF_STRICT}"
RUN_EBNF_DUAL_RUN_DIFF="${PGEN_SOTA_RUN_EBNF_DUAL_RUN_DIFF:-$POLICY_RUN_EBNF_DUAL_RUN_DIFF}"
REQUIRE_EBNF_DUAL_RUN_STRICT="${PGEN_SOTA_REQUIRE_EBNF_DUAL_RUN_STRICT:-$POLICY_REQUIRE_EBNF_DUAL_RUN_STRICT}"
RUN_HDL_FRONTEND_READINESS="${PGEN_SOTA_RUN_HDL_FRONTEND_READINESS:-$POLICY_RUN_HDL_FRONTEND_READINESS}"
REQUIRE_HDL_FRONTEND_STRICT="${PGEN_SOTA_REQUIRE_HDL_FRONTEND_STRICT:-$POLICY_REQUIRE_HDL_FRONTEND_STRICT}"
RUN_SV_PREPROCESSOR_QUALITY="${PGEN_SOTA_RUN_SV_PREPROCESSOR_QUALITY:-$POLICY_RUN_SV_PREPROCESSOR_QUALITY}"
REQUIRE_SV_PREPROCESSOR_QUALITY_STRICT="${PGEN_SOTA_REQUIRE_SV_PREPROCESSOR_QUALITY_STRICT:-$POLICY_REQUIRE_SV_PREPROCESSOR_QUALITY_STRICT}"
RUN_SV_STIMULI_QUALITY="${PGEN_SOTA_RUN_SV_STIMULI_QUALITY:-$POLICY_RUN_SV_STIMULI_QUALITY}"
REQUIRE_SV_STIMULI_QUALITY_STRICT="${PGEN_SOTA_REQUIRE_SV_STIMULI_QUALITY_STRICT:-$POLICY_REQUIRE_SV_STIMULI_QUALITY_STRICT}"
SV_STIMULI_ENFORCE_MIN_PARSE_FULL_PASS_RATIO="${PGEN_SOTA_SV_STIMULI_ENFORCE_MIN_PARSE_FULL_PASS_RATIO:-$POLICY_SV_STIMULI_ENFORCE_MIN_PARSE_FULL_PASS_RATIO}"
SV_STIMULI_MIN_PARSE_FULL_PASS_RATIO="${PGEN_SOTA_SV_STIMULI_MIN_PARSE_FULL_PASS_RATIO:-$POLICY_SV_STIMULI_MIN_PARSE_FULL_PASS_RATIO}"
RUN_SV_DECLARED_SHADOW_PROMOTION="${PGEN_SOTA_RUN_SV_DECLARED_SHADOW_PROMOTION:-$POLICY_RUN_SV_DECLARED_SHADOW_PROMOTION}"
REQUIRE_SV_DECLARED_SHADOW_PROMOTION_STRICT="${PGEN_SOTA_REQUIRE_SV_DECLARED_SHADOW_PROMOTION_STRICT:-$POLICY_REQUIRE_SV_DECLARED_SHADOW_PROMOTION_STRICT}"
RUN_VHDL_STIMULI_QUALITY="${PGEN_SOTA_RUN_VHDL_STIMULI_QUALITY:-$POLICY_RUN_VHDL_STIMULI_QUALITY}"
REQUIRE_VHDL_STIMULI_QUALITY_STRICT="${PGEN_SOTA_REQUIRE_VHDL_STIMULI_QUALITY_STRICT:-$POLICY_REQUIRE_VHDL_STIMULI_QUALITY_STRICT}"
ALLOW_INFORMATIONAL_FAILURES="${PGEN_SOTA_ALLOW_INFORMATIONAL_FAILURES:-$POLICY_ALLOW_INFORMATIONAL_FAILURES}"
REQUIRED_CHECKS="${PGEN_SOTA_REQUIRED_CHECKS:-$POLICY_REQUIRED_CHECKS}"

if ! [[ "$RUN_EBNF_READINESS" =~ ^[01]$ ]]; then
    echo "error: PGEN_SOTA_RUN_EBNF_READINESS must be 0 or 1" >&2
    exit 2
fi
if ! [[ "$REQUIRE_EBNF_STRICT" =~ ^[01]$ ]]; then
    echo "error: PGEN_SOTA_REQUIRE_EBNF_STRICT must be 0 or 1" >&2
    exit 2
fi
if ! [[ "$RUN_EBNF_DUAL_RUN_DIFF" =~ ^[01]$ ]]; then
    echo "error: PGEN_SOTA_RUN_EBNF_DUAL_RUN_DIFF must be 0 or 1" >&2
    exit 2
fi
if ! [[ "$REQUIRE_EBNF_DUAL_RUN_STRICT" =~ ^[01]$ ]]; then
    echo "error: PGEN_SOTA_REQUIRE_EBNF_DUAL_RUN_STRICT must be 0 or 1" >&2
    exit 2
fi
if ! [[ "$RUN_HDL_FRONTEND_READINESS" =~ ^[01]$ ]]; then
    echo "error: PGEN_SOTA_RUN_HDL_FRONTEND_READINESS must be 0 or 1" >&2
    exit 2
fi
if ! [[ "$REQUIRE_HDL_FRONTEND_STRICT" =~ ^[01]$ ]]; then
    echo "error: PGEN_SOTA_REQUIRE_HDL_FRONTEND_STRICT must be 0 or 1" >&2
    exit 2
fi
if ! [[ "$RUN_SV_PREPROCESSOR_QUALITY" =~ ^[01]$ ]]; then
    echo "error: PGEN_SOTA_RUN_SV_PREPROCESSOR_QUALITY must be 0 or 1" >&2
    exit 2
fi
if ! [[ "$REQUIRE_SV_PREPROCESSOR_QUALITY_STRICT" =~ ^[01]$ ]]; then
    echo "error: PGEN_SOTA_REQUIRE_SV_PREPROCESSOR_QUALITY_STRICT must be 0 or 1" >&2
    exit 2
fi
if ! [[ "$RUN_SV_STIMULI_QUALITY" =~ ^[01]$ ]]; then
    echo "error: PGEN_SOTA_RUN_SV_STIMULI_QUALITY must be 0 or 1" >&2
    exit 2
fi
if ! [[ "$REQUIRE_SV_STIMULI_QUALITY_STRICT" =~ ^[01]$ ]]; then
    echo "error: PGEN_SOTA_REQUIRE_SV_STIMULI_QUALITY_STRICT must be 0 or 1" >&2
    exit 2
fi
if ! [[ "$SV_STIMULI_ENFORCE_MIN_PARSE_FULL_PASS_RATIO" =~ ^[01]$ ]]; then
    echo "error: PGEN_SOTA_SV_STIMULI_ENFORCE_MIN_PARSE_FULL_PASS_RATIO must be 0 or 1" >&2
    exit 2
fi
if ! [[ "$SV_STIMULI_MIN_PARSE_FULL_PASS_RATIO" =~ ^[0-9]+$ ]] || [[ "$SV_STIMULI_MIN_PARSE_FULL_PASS_RATIO" -lt 0 ]] || [[ "$SV_STIMULI_MIN_PARSE_FULL_PASS_RATIO" -gt 100 ]]; then
    echo "error: PGEN_SOTA_SV_STIMULI_MIN_PARSE_FULL_PASS_RATIO must be an integer between 0 and 100" >&2
    exit 2
fi
if ! [[ "$RUN_SV_DECLARED_SHADOW_PROMOTION" =~ ^[01]$ ]]; then
    echo "error: PGEN_SOTA_RUN_SV_DECLARED_SHADOW_PROMOTION must be 0 or 1" >&2
    exit 2
fi
if ! [[ "$REQUIRE_SV_DECLARED_SHADOW_PROMOTION_STRICT" =~ ^[01]$ ]]; then
    echo "error: PGEN_SOTA_REQUIRE_SV_DECLARED_SHADOW_PROMOTION_STRICT must be 0 or 1" >&2
    exit 2
fi
if ! [[ "$RUN_VHDL_STIMULI_QUALITY" =~ ^[01]$ ]]; then
    echo "error: PGEN_SOTA_RUN_VHDL_STIMULI_QUALITY must be 0 or 1" >&2
    exit 2
fi
if ! [[ "$REQUIRE_VHDL_STIMULI_QUALITY_STRICT" =~ ^[01]$ ]]; then
    echo "error: PGEN_SOTA_REQUIRE_VHDL_STIMULI_QUALITY_STRICT must be 0 or 1" >&2
    exit 2
fi
if ! [[ "$ALLOW_INFORMATIONAL_FAILURES" =~ ^[01]$ ]]; then
    echo "error: PGEN_SOTA_ALLOW_INFORMATIONAL_FAILURES must be 0 or 1" >&2
    exit 2
fi
if ! [[ "$POLICY_VERSION" =~ ^[0-9]+$ ]]; then
    echo "error: PGEN_SOTA_POLICY_VERSION must be an integer" >&2
    exit 2
fi
if [[ -z "$REQUIRED_CHECKS" ]]; then
    echo "error: required check list cannot be empty (PGEN_SOTA_REQUIRED_CHECKS)" >&2
    exit 2
fi

mkdir -p "$LOG_DIR"

echo "check,required,status,notes,log" >"$SUMMARY_CSV"

required_failures=0
all_failures=0

run_check() {
    local name="$1"
    local required="$2"
    local notes="$3"
    shift 3

    local log_file="$LOG_DIR/${name}.log"

    echo "==> ${name} (${required})"
    if (
        cd "$ROOT_DIR"
        "$@"
    ) >"$log_file" 2>&1; then
        echo "    ok (${log_file})"
        echo "${name},${required},pass,\"${notes}\",${log_file}" >>"$SUMMARY_CSV"
        return 0
    fi

    echo "    fail (${log_file})"
    echo "${name},${required},fail,\"${notes}\",${log_file}" >>"$SUMMARY_CSV"
    all_failures=$((all_failures + 1))
    if [[ "$required" == "required" ]]; then
        required_failures=$((required_failures + 1))
    fi
    return 0
}

echo "==> PGEN SOTA exit gate"
echo "state_dir: $STATE_DIR"
echo "policy_file: $POLICY_FILE"
echo "policy_version: $POLICY_VERSION"
echo "required_checks: $REQUIRED_CHECKS"
echo "run_ebnf_readiness: $RUN_EBNF_READINESS"
echo "require_ebnf_strict: $REQUIRE_EBNF_STRICT"
echo "run_ebnf_dual_run_diff: $RUN_EBNF_DUAL_RUN_DIFF"
echo "require_ebnf_dual_run_strict: $REQUIRE_EBNF_DUAL_RUN_STRICT"
echo "run_hdl_frontend_readiness: $RUN_HDL_FRONTEND_READINESS"
echo "require_hdl_frontend_strict: $REQUIRE_HDL_FRONTEND_STRICT"
echo "run_sv_preprocessor_quality: $RUN_SV_PREPROCESSOR_QUALITY"
echo "require_sv_preprocessor_quality_strict: $REQUIRE_SV_PREPROCESSOR_QUALITY_STRICT"
echo "run_sv_stimuli_quality: $RUN_SV_STIMULI_QUALITY"
echo "require_sv_stimuli_quality_strict: $REQUIRE_SV_STIMULI_QUALITY_STRICT"
echo "sv_stimuli_enforce_min_parse_full_pass_ratio: $SV_STIMULI_ENFORCE_MIN_PARSE_FULL_PASS_RATIO"
echo "sv_stimuli_min_parse_full_pass_ratio: $SV_STIMULI_MIN_PARSE_FULL_PASS_RATIO"
echo "run_sv_declared_shadow_promotion: $RUN_SV_DECLARED_SHADOW_PROMOTION"
echo "require_sv_declared_shadow_promotion_strict: $REQUIRE_SV_DECLARED_SHADOW_PROMOTION_STRICT"
echo "run_vhdl_stimuli_quality: $RUN_VHDL_STIMULI_QUALITY"
echo "require_vhdl_stimuli_quality_strict: $REQUIRE_VHDL_STIMULI_QUALITY_STRICT"
echo "allow_informational_failures: $ALLOW_INFORMATIONAL_FAILURES"

run_required_check_by_name() {
    local check_name="$1"
    case "$check_name" in
        differential_baseline_contract)
            run_check \
                "differential_baseline_contract" \
                "required" \
                "tracked differential baselines exist and parse as JSON arrays" \
                perl -MJSON::PP -e '
                    use strict;
                    use warnings;
                    use JSON::PP qw(decode_json);
                    foreach my $path (@ARGV) {
                        open my $fh, "<", $path or die "open $path: $!";
                        local $/;
                        my $json = <$fh>;
                        close $fh;
                        my $data = decode_json($json);
                        die "expected object for $path"
                            unless ref($data) eq "HASH";
                        die "missing allowed_mismatches in $path"
                            unless exists $data->{allowed_mismatches};
                        die "allowed_mismatches must be array in $path"
                            unless ref($data->{allowed_mismatches}) eq "ARRAY";
                    }
                    print "ok\n";
                ' \
                "$RUST_DIR/test_data/differential_baseline/return_annotation_baseline.json" \
                "$RUST_DIR/test_data/differential_baseline/semantic_annotation_baseline.json"
            ;;
        fixed_point_gate)
            run_check \
                "fixed_point_gate" \
                "required" \
                "deterministic bootstrap artifacts" \
                make -C rust SHELL=/bin/bash fixed_point_gate
            ;;
        annotation_contract_gate)
            run_check \
                "annotation_contract_gate" \
                "required" \
                "annotation validator/contracts/robustness" \
                make -C rust SHELL=/bin/bash annotation_contract_gate
            ;;
        annotation_100_gate)
            run_check \
                "annotation_100_gate" \
                "required" \
                "aggregate annotation proof gate (coverage/typed-ast/runtime/determinism/parity)" \
                make -C rust SHELL=/bin/bash annotation_100_gate
            ;;
        annotation_nonbootstrap_e2e_gate)
            run_check \
                "annotation_nonbootstrap_e2e_gate" \
                "required" \
                "non-bootstrap annotation parser/stimuli end-to-end checks" \
                make -C rust SHELL=/bin/bash annotation_nonbootstrap_e2e_gate
            ;;
        ebnf_stimuli_quality_gate)
            run_check \
                "ebnf_stimuli_quality_gate" \
                "required" \
                "non-annotation EBNF parser/stimuli closed-loop quality checks" \
                make -C rust SHELL=/bin/bash ebnf_stimuli_quality_gate
            ;;
        stimuli_module_parity_gate)
            run_check \
                "stimuli_module_parity_gate" \
                "required" \
                "in-memory stimuli vs generated stimuli-module parity (samples/coverage/gap)" \
                make -C rust SHELL=/bin/bash stimuli_module_parity_gate
            ;;
        differential_regression_gate)
            run_check \
                "differential_regression_gate" \
                "required" \
                "no new bootstrap/generated mismatches" \
                make -C rust SHELL=/bin/bash differential_regression_gate
            ;;
        performance_gate)
            run_check \
                "performance_gate" \
                "required" \
                "performance thresholds with generated parsers" \
                make -C rust SHELL=/bin/bash performance_gate
            ;;
        embedding_api_gate)
            run_check \
                "embedding_api_gate" \
                "required" \
                "stable embedding API contract checks" \
                make -C rust SHELL=/bin/bash embedding_api_gate
            ;;
        *)
            echo "error: unknown required check '$check_name' in required check list" >&2
            exit 2
            ;;
    esac
}

for check_name in $REQUIRED_CHECKS; do
    run_required_check_by_name "$check_name"
done

if [[ "$RUN_EBNF_READINESS" -eq 1 ]]; then
    if [[ "$REQUIRE_EBNF_STRICT" -eq 1 ]]; then
        run_check "ebnf_frontend_gate" "required" "strict EBNF frontend readiness" \
            make -C rust SHELL=/bin/bash ebnf_frontend_gate
    else
        run_check "ebnf_frontend_readiness" "informational" "report-only EBNF frontend readiness" \
            make -C rust SHELL=/bin/bash ebnf_frontend_readiness
    fi
fi

if [[ "$RUN_EBNF_DUAL_RUN_DIFF" -eq 1 ]]; then
    if [[ "$REQUIRE_EBNF_DUAL_RUN_STRICT" -eq 1 ]]; then
        run_check "ebnf_frontend_dual_run_gate" "required" "strict Perl-vs-Rust EBNF dual-run differential" \
            make -C rust SHELL=/bin/bash ebnf_frontend_dual_run_gate
    else
        run_check "ebnf_frontend_dual_run_diff" "informational" "report-only Perl-vs-Rust EBNF dual-run differential" \
            make -C rust SHELL=/bin/bash ebnf_frontend_dual_run_diff
    fi
fi

if [[ "$RUN_HDL_FRONTEND_READINESS" -eq 1 ]]; then
    if [[ "$REQUIRE_HDL_FRONTEND_STRICT" -eq 1 ]]; then
        run_check "hdl_frontend_gate" "required" "strict HDL frontend readiness (systemverilog + vhdl roster)" \
            make -C rust SHELL=/bin/bash hdl_frontend_gate
    else
        run_check "hdl_frontend_readiness" "informational" "report-only HDL frontend readiness (systemverilog + vhdl roster)" \
            make -C rust SHELL=/bin/bash hdl_frontend_readiness
    fi
fi

if [[ "$RUN_SV_PREPROCESSOR_QUALITY" -eq 1 ]]; then
    if [[ "$REQUIRE_SV_PREPROCESSOR_QUALITY_STRICT" -eq 1 ]]; then
        run_check "sv_preprocessor_quality_gate" "required" "strict SystemVerilog preprocessor closed-loop quality gate" \
            make -C rust SHELL=/bin/bash sv_preprocessor_quality_gate
    else
        run_check "sv_preprocessor_quality_gate" "informational" "report-only SystemVerilog preprocessor closed-loop quality gate" \
            make -C rust SHELL=/bin/bash sv_preprocessor_quality_gate
    fi
fi

if [[ "$RUN_SV_STIMULI_QUALITY" -eq 1 ]]; then
    if [[ "$REQUIRE_SV_STIMULI_QUALITY_STRICT" -eq 1 ]]; then
        run_check "sv_stimuli_quality_gate" "required" "strict preprocess-first SystemVerilog stimuli quality gate" \
            env \
                PGEN_SV_STIMULI_QUALITY_ENFORCE_MIN_PARSE_FULL_PASS_RATIO="$SV_STIMULI_ENFORCE_MIN_PARSE_FULL_PASS_RATIO" \
                PGEN_SV_STIMULI_QUALITY_MIN_PARSE_FULL_PASS_RATIO="$SV_STIMULI_MIN_PARSE_FULL_PASS_RATIO" \
                make -C rust SHELL=/bin/bash sv_stimuli_quality_gate
    else
        run_check "sv_stimuli_quality_gate" "informational" "report-only preprocess-first SystemVerilog stimuli quality gate" \
            env \
                PGEN_SV_STIMULI_QUALITY_ENFORCE_MIN_PARSE_FULL_PASS_RATIO="$SV_STIMULI_ENFORCE_MIN_PARSE_FULL_PASS_RATIO" \
                PGEN_SV_STIMULI_QUALITY_MIN_PARSE_FULL_PASS_RATIO="$SV_STIMULI_MIN_PARSE_FULL_PASS_RATIO" \
                make -C rust SHELL=/bin/bash sv_stimuli_quality_gate
    fi
fi

if [[ "$RUN_SV_DECLARED_SHADOW_PROMOTION" -eq 1 ]]; then
    if [[ "$REQUIRE_SV_DECLARED_SHADOW_PROMOTION_STRICT" -eq 1 ]]; then
        run_check "sv_declared_shadow_promotion_gate" "required" "strict declared-shadow promotion-trial gate" \
            env PGEN_SV_DECLARED_SHADOW_PROMOTION_MODE=1 make -C rust SHELL=/bin/bash sv_declared_shadow_promotion_gate
    else
        run_check "sv_declared_shadow_promotion_gate" "informational" "report-only declared-shadow promotion-trial gate" \
            env PGEN_SV_DECLARED_SHADOW_PROMOTION_MODE=auto make -C rust SHELL=/bin/bash sv_declared_shadow_promotion_gate
    fi
fi

if [[ "$RUN_VHDL_STIMULI_QUALITY" -eq 1 ]]; then
    if [[ "$REQUIRE_VHDL_STIMULI_QUALITY_STRICT" -eq 1 ]]; then
        run_check "vhdl_stimuli_quality_gate" "required" "strict VHDL stimuli quality gate" \
            make -C rust SHELL=/bin/bash vhdl_stimuli_quality_gate
    else
        run_check "vhdl_stimuli_quality_gate" "informational" "report-only VHDL stimuli quality gate" \
            make -C rust SHELL=/bin/bash vhdl_stimuli_quality_gate
    fi
fi

{
    echo "PGEN SOTA Exit Gate Summary"
    echo "state_dir: $STATE_DIR"
    echo "required_failures: $required_failures"
    echo "all_failures: $all_failures"
    echo
    if command -v column >/dev/null 2>&1; then
        column -s, -t "$SUMMARY_CSV"
    else
        cat "$SUMMARY_CSV"
    fi
} >"$SUMMARY_TXT"

cat "$SUMMARY_TXT"

if [[ "$required_failures" -ne 0 ]]; then
    echo "❌ SOTA exit gate failed: ${required_failures} required check(s) failed." >&2
    exit 1
fi

informational_failures=$((all_failures - required_failures))
if [[ "$informational_failures" -ne 0 && "$ALLOW_INFORMATIONAL_FAILURES" -eq 0 ]]; then
    echo "❌ SOTA exit gate failed: ${informational_failures} informational check(s) failed while policy disallows informational failures." >&2
    exit 1
fi

if [[ "$all_failures" -ne 0 ]]; then
    echo "⚠️  SOTA exit gate passed required checks with ${all_failures} total failure(s)." >&2
    echo "ℹ️  Failures are in informational checks only." >&2
else
    echo "✅ SOTA exit gate passed."
fi

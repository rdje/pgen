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
POLICY_REQUIRED_CHECKS="${PGEN_SOTA_POLICY_REQUIRED_CHECKS:-differential_baseline_contract fixed_point_gate annotation_contract_gate differential_regression_gate performance_gate embedding_api_gate}"
POLICY_RUN_EBNF_READINESS="${PGEN_SOTA_POLICY_RUN_EBNF_READINESS:-1}"
POLICY_REQUIRE_EBNF_STRICT="${PGEN_SOTA_POLICY_REQUIRE_EBNF_STRICT:-0}"
POLICY_ALLOW_INFORMATIONAL_FAILURES="${PGEN_SOTA_POLICY_ALLOW_INFORMATIONAL_FAILURES:-1}"

RUN_EBNF_READINESS="${PGEN_SOTA_RUN_EBNF_READINESS:-$POLICY_RUN_EBNF_READINESS}"
REQUIRE_EBNF_STRICT="${PGEN_SOTA_REQUIRE_EBNF_STRICT:-$POLICY_REQUIRE_EBNF_STRICT}"
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

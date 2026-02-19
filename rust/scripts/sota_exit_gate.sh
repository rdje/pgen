#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
RUST_DIR="$ROOT_DIR/rust"

STATE_DIR="${PGEN_SOTA_EXIT_STATE_DIR:-$RUST_DIR/target/sota_exit_gate}"
LOG_DIR="$STATE_DIR/logs"
SUMMARY_CSV="$STATE_DIR/summary.csv"
SUMMARY_TXT="$STATE_DIR/summary.txt"

RUN_EBNF_READINESS="${PGEN_SOTA_RUN_EBNF_READINESS:-1}"
REQUIRE_EBNF_STRICT="${PGEN_SOTA_REQUIRE_EBNF_STRICT:-0}"

if ! [[ "$RUN_EBNF_READINESS" =~ ^[01]$ ]]; then
    echo "error: PGEN_SOTA_RUN_EBNF_READINESS must be 0 or 1" >&2
    exit 2
fi
if ! [[ "$REQUIRE_EBNF_STRICT" =~ ^[01]$ ]]; then
    echo "error: PGEN_SOTA_REQUIRE_EBNF_STRICT must be 0 or 1" >&2
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
echo "run_ebnf_readiness: $RUN_EBNF_READINESS"
echo "require_ebnf_strict: $REQUIRE_EBNF_STRICT"

run_check "fixed_point_gate" "required" "deterministic bootstrap artifacts" \
    make -C rust SHELL=/bin/bash fixed_point_gate
run_check "annotation_contract_gate" "required" "annotation validator/contracts/robustness" \
    make -C rust SHELL=/bin/bash annotation_contract_gate
run_check "differential_regression_gate" "required" "no new bootstrap/generated mismatches" \
    make -C rust SHELL=/bin/bash differential_regression_gate
run_check "performance_gate" "required" "performance thresholds with generated parsers" \
    make -C rust SHELL=/bin/bash performance_gate
run_check "embedding_api_gate" "required" "stable embedding API contract checks" \
    make -C rust SHELL=/bin/bash embedding_api_gate

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

if [[ "$all_failures" -ne 0 ]]; then
    echo "⚠️  SOTA exit gate passed required checks with ${all_failures} total failure(s)." >&2
    echo "ℹ️  Failures are in informational checks only." >&2
else
    echo "✅ SOTA exit gate passed."
fi

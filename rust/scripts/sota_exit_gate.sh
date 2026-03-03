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
POLICY_SV_DECLARED_SHADOW_PROMOTION_TRIALS="${PGEN_SOTA_POLICY_SV_DECLARED_SHADOW_PROMOTION_TRIALS:-3}"
POLICY_SV_DECLARED_SHADOW_PROMOTION_COUNT="${PGEN_SOTA_POLICY_SV_DECLARED_SHADOW_PROMOTION_COUNT:-6}"
POLICY_SV_DECLARED_SHADOW_PROMOTION_SEED_BASE="${PGEN_SOTA_POLICY_SV_DECLARED_SHADOW_PROMOTION_SEED_BASE:-12001}"
POLICY_SV_DECLARED_SHADOW_PROMOTION_TARGET_MAX_ATTEMPTS="${PGEN_SOTA_POLICY_SV_DECLARED_SHADOW_PROMOTION_TARGET_MAX_ATTEMPTS:-400}"
POLICY_SV_DECLARED_SHADOW_PROMOTION_PARSE_FULL_MODE="${PGEN_SOTA_POLICY_SV_DECLARED_SHADOW_PROMOTION_PARSE_FULL_MODE:-auto}"
POLICY_SV_DECLARED_SHADOW_PROMOTION_MIN_CHECKED="${PGEN_SOTA_POLICY_SV_DECLARED_SHADOW_PROMOTION_MIN_CHECKED:-2}"
POLICY_SV_DECLARED_SHADOW_PROMOTION_SEMANTIC_CLOSURE_MODE="${PGEN_SOTA_POLICY_SV_DECLARED_SHADOW_PROMOTION_SEMANTIC_CLOSURE_MODE:-1}"
POLICY_SV_DECLARED_SHADOW_PROMOTION_STIMULI_MODE="${PGEN_SOTA_POLICY_SV_DECLARED_SHADOW_PROMOTION_STIMULI_MODE:-sv_file}"
POLICY_SV_DECLARED_SHADOW_PROMOTION_DECLARED_SHADOW_PARSEABLE_ONLY="${PGEN_SOTA_POLICY_SV_DECLARED_SHADOW_PROMOTION_DECLARED_SHADOW_PARSEABLE_ONLY:-1}"
POLICY_RUN_SV_PARSE_FULL_RATIO_PROMOTION="${PGEN_SOTA_POLICY_RUN_SV_PARSE_FULL_RATIO_PROMOTION:-0}"
POLICY_REQUIRE_SV_PARSE_FULL_RATIO_PROMOTION_STRICT="${PGEN_SOTA_POLICY_REQUIRE_SV_PARSE_FULL_RATIO_PROMOTION_STRICT:-0}"
POLICY_SV_PARSE_FULL_RATIO_PROMOTION_TARGET_MIN_RATIO="${PGEN_SOTA_POLICY_SV_PARSE_FULL_RATIO_PROMOTION_TARGET_MIN_RATIO:-20}"
POLICY_SV_PARSE_FULL_RATIO_PROMOTION_TRIALS="${PGEN_SOTA_POLICY_SV_PARSE_FULL_RATIO_PROMOTION_TRIALS:-3}"
POLICY_SV_PARSE_FULL_RATIO_PROMOTION_COUNT="${PGEN_SOTA_POLICY_SV_PARSE_FULL_RATIO_PROMOTION_COUNT:-6}"
POLICY_SV_PARSE_FULL_RATIO_PROMOTION_SEED_BASE="${PGEN_SOTA_POLICY_SV_PARSE_FULL_RATIO_PROMOTION_SEED_BASE:-12001}"
POLICY_SV_PARSE_FULL_RATIO_PROMOTION_PARSE_FULL_MODE="${PGEN_SOTA_POLICY_SV_PARSE_FULL_RATIO_PROMOTION_PARSE_FULL_MODE:-auto}"
POLICY_SV_PARSE_FULL_RATIO_PROMOTION_SEMANTIC_CLOSURE_MODE="${PGEN_SOTA_POLICY_SV_PARSE_FULL_RATIO_PROMOTION_SEMANTIC_CLOSURE_MODE:-0}"
POLICY_SV_PARSE_FULL_RATIO_PROMOTION_STIMULI_MODE="${PGEN_SOTA_POLICY_SV_PARSE_FULL_RATIO_PROMOTION_STIMULI_MODE:-sv_file}"
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
SV_DECLARED_SHADOW_PROMOTION_TRIALS="${PGEN_SOTA_SV_DECLARED_SHADOW_PROMOTION_TRIALS:-$POLICY_SV_DECLARED_SHADOW_PROMOTION_TRIALS}"
SV_DECLARED_SHADOW_PROMOTION_COUNT="${PGEN_SOTA_SV_DECLARED_SHADOW_PROMOTION_COUNT:-$POLICY_SV_DECLARED_SHADOW_PROMOTION_COUNT}"
SV_DECLARED_SHADOW_PROMOTION_SEED_BASE="${PGEN_SOTA_SV_DECLARED_SHADOW_PROMOTION_SEED_BASE:-$POLICY_SV_DECLARED_SHADOW_PROMOTION_SEED_BASE}"
SV_DECLARED_SHADOW_PROMOTION_TARGET_MAX_ATTEMPTS="${PGEN_SOTA_SV_DECLARED_SHADOW_PROMOTION_TARGET_MAX_ATTEMPTS:-$POLICY_SV_DECLARED_SHADOW_PROMOTION_TARGET_MAX_ATTEMPTS}"
SV_DECLARED_SHADOW_PROMOTION_PARSE_FULL_MODE="${PGEN_SOTA_SV_DECLARED_SHADOW_PROMOTION_PARSE_FULL_MODE:-$POLICY_SV_DECLARED_SHADOW_PROMOTION_PARSE_FULL_MODE}"
SV_DECLARED_SHADOW_PROMOTION_MIN_CHECKED="${PGEN_SOTA_SV_DECLARED_SHADOW_PROMOTION_MIN_CHECKED:-$POLICY_SV_DECLARED_SHADOW_PROMOTION_MIN_CHECKED}"
SV_DECLARED_SHADOW_PROMOTION_SEMANTIC_CLOSURE_MODE="${PGEN_SOTA_SV_DECLARED_SHADOW_PROMOTION_SEMANTIC_CLOSURE_MODE:-$POLICY_SV_DECLARED_SHADOW_PROMOTION_SEMANTIC_CLOSURE_MODE}"
SV_DECLARED_SHADOW_PROMOTION_STIMULI_MODE="${PGEN_SOTA_SV_DECLARED_SHADOW_PROMOTION_STIMULI_MODE:-$POLICY_SV_DECLARED_SHADOW_PROMOTION_STIMULI_MODE}"
SV_DECLARED_SHADOW_PROMOTION_DECLARED_SHADOW_PARSEABLE_ONLY="${PGEN_SOTA_SV_DECLARED_SHADOW_PROMOTION_DECLARED_SHADOW_PARSEABLE_ONLY:-$POLICY_SV_DECLARED_SHADOW_PROMOTION_DECLARED_SHADOW_PARSEABLE_ONLY}"
RUN_SV_PARSE_FULL_RATIO_PROMOTION="${PGEN_SOTA_RUN_SV_PARSE_FULL_RATIO_PROMOTION:-$POLICY_RUN_SV_PARSE_FULL_RATIO_PROMOTION}"
REQUIRE_SV_PARSE_FULL_RATIO_PROMOTION_STRICT="${PGEN_SOTA_REQUIRE_SV_PARSE_FULL_RATIO_PROMOTION_STRICT:-$POLICY_REQUIRE_SV_PARSE_FULL_RATIO_PROMOTION_STRICT}"
SV_PARSE_FULL_RATIO_PROMOTION_TARGET_MIN_RATIO="${PGEN_SOTA_SV_PARSE_FULL_RATIO_PROMOTION_TARGET_MIN_RATIO:-$POLICY_SV_PARSE_FULL_RATIO_PROMOTION_TARGET_MIN_RATIO}"
SV_PARSE_FULL_RATIO_PROMOTION_TRIALS="${PGEN_SOTA_SV_PARSE_FULL_RATIO_PROMOTION_TRIALS:-$POLICY_SV_PARSE_FULL_RATIO_PROMOTION_TRIALS}"
SV_PARSE_FULL_RATIO_PROMOTION_COUNT="${PGEN_SOTA_SV_PARSE_FULL_RATIO_PROMOTION_COUNT:-$POLICY_SV_PARSE_FULL_RATIO_PROMOTION_COUNT}"
SV_PARSE_FULL_RATIO_PROMOTION_SEED_BASE="${PGEN_SOTA_SV_PARSE_FULL_RATIO_PROMOTION_SEED_BASE:-$POLICY_SV_PARSE_FULL_RATIO_PROMOTION_SEED_BASE}"
SV_PARSE_FULL_RATIO_PROMOTION_PARSE_FULL_MODE="${PGEN_SOTA_SV_PARSE_FULL_RATIO_PROMOTION_PARSE_FULL_MODE:-$POLICY_SV_PARSE_FULL_RATIO_PROMOTION_PARSE_FULL_MODE}"
SV_PARSE_FULL_RATIO_PROMOTION_SEMANTIC_CLOSURE_MODE="${PGEN_SOTA_SV_PARSE_FULL_RATIO_PROMOTION_SEMANTIC_CLOSURE_MODE:-$POLICY_SV_PARSE_FULL_RATIO_PROMOTION_SEMANTIC_CLOSURE_MODE}"
SV_PARSE_FULL_RATIO_PROMOTION_STIMULI_MODE="${PGEN_SOTA_SV_PARSE_FULL_RATIO_PROMOTION_STIMULI_MODE:-$POLICY_SV_PARSE_FULL_RATIO_PROMOTION_STIMULI_MODE}"
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
if ! [[ "$SV_DECLARED_SHADOW_PROMOTION_TRIALS" =~ ^[0-9]+$ ]] || [[ "$SV_DECLARED_SHADOW_PROMOTION_TRIALS" -lt 1 ]]; then
    echo "error: PGEN_SOTA_SV_DECLARED_SHADOW_PROMOTION_TRIALS must be an integer >= 1" >&2
    exit 2
fi
if ! [[ "$SV_DECLARED_SHADOW_PROMOTION_COUNT" =~ ^[0-9]+$ ]] || [[ "$SV_DECLARED_SHADOW_PROMOTION_COUNT" -lt 1 ]]; then
    echo "error: PGEN_SOTA_SV_DECLARED_SHADOW_PROMOTION_COUNT must be an integer >= 1" >&2
    exit 2
fi
if ! [[ "$SV_DECLARED_SHADOW_PROMOTION_SEED_BASE" =~ ^[0-9]+$ ]]; then
    echo "error: PGEN_SOTA_SV_DECLARED_SHADOW_PROMOTION_SEED_BASE must be an integer >= 0" >&2
    exit 2
fi
if ! [[ "$SV_DECLARED_SHADOW_PROMOTION_TARGET_MAX_ATTEMPTS" =~ ^[0-9]+$ ]] || [[ "$SV_DECLARED_SHADOW_PROMOTION_TARGET_MAX_ATTEMPTS" -lt 1 ]]; then
    echo "error: PGEN_SOTA_SV_DECLARED_SHADOW_PROMOTION_TARGET_MAX_ATTEMPTS must be an integer >= 1" >&2
    exit 2
fi
if [[ "$SV_DECLARED_SHADOW_PROMOTION_PARSE_FULL_MODE" != "auto" && "$SV_DECLARED_SHADOW_PROMOTION_PARSE_FULL_MODE" != "0" && "$SV_DECLARED_SHADOW_PROMOTION_PARSE_FULL_MODE" != "1" ]]; then
    echo "error: PGEN_SOTA_SV_DECLARED_SHADOW_PROMOTION_PARSE_FULL_MODE must be one of: auto, 0, 1" >&2
    exit 2
fi
if ! [[ "$SV_DECLARED_SHADOW_PROMOTION_MIN_CHECKED" =~ ^[0-9]+$ ]] || [[ "$SV_DECLARED_SHADOW_PROMOTION_MIN_CHECKED" -lt 1 ]]; then
    echo "error: PGEN_SOTA_SV_DECLARED_SHADOW_PROMOTION_MIN_CHECKED must be an integer >= 1" >&2
    exit 2
fi
if ! [[ "$SV_DECLARED_SHADOW_PROMOTION_SEMANTIC_CLOSURE_MODE" =~ ^[01]$ ]]; then
    echo "error: PGEN_SOTA_SV_DECLARED_SHADOW_PROMOTION_SEMANTIC_CLOSURE_MODE must be 0 or 1" >&2
    exit 2
fi
if [[ "$SV_DECLARED_SHADOW_PROMOTION_STIMULI_MODE" != "sv_file" && "$SV_DECLARED_SHADOW_PROMOTION_STIMULI_MODE" != "sv_snippet" && "$SV_DECLARED_SHADOW_PROMOTION_STIMULI_MODE" != "sv_pp_file" && "$SV_DECLARED_SHADOW_PROMOTION_STIMULI_MODE" != "sv_pp_snippet" && "$SV_DECLARED_SHADOW_PROMOTION_STIMULI_MODE" != "sv_semantic_file" ]]; then
    echo "error: PGEN_SOTA_SV_DECLARED_SHADOW_PROMOTION_STIMULI_MODE must be one of: sv_file, sv_snippet, sv_pp_file, sv_pp_snippet, sv_semantic_file" >&2
    exit 2
fi
if ! [[ "$SV_DECLARED_SHADOW_PROMOTION_DECLARED_SHADOW_PARSEABLE_ONLY" =~ ^[01]$ ]]; then
    echo "error: PGEN_SOTA_SV_DECLARED_SHADOW_PROMOTION_DECLARED_SHADOW_PARSEABLE_ONLY must be 0 or 1" >&2
    exit 2
fi
if ! [[ "$RUN_SV_PARSE_FULL_RATIO_PROMOTION" =~ ^[01]$ ]]; then
    echo "error: PGEN_SOTA_RUN_SV_PARSE_FULL_RATIO_PROMOTION must be 0 or 1" >&2
    exit 2
fi
if ! [[ "$REQUIRE_SV_PARSE_FULL_RATIO_PROMOTION_STRICT" =~ ^[01]$ ]]; then
    echo "error: PGEN_SOTA_REQUIRE_SV_PARSE_FULL_RATIO_PROMOTION_STRICT must be 0 or 1" >&2
    exit 2
fi
if ! [[ "$SV_PARSE_FULL_RATIO_PROMOTION_TARGET_MIN_RATIO" =~ ^[0-9]+$ ]] || [[ "$SV_PARSE_FULL_RATIO_PROMOTION_TARGET_MIN_RATIO" -lt 0 ]] || [[ "$SV_PARSE_FULL_RATIO_PROMOTION_TARGET_MIN_RATIO" -gt 100 ]]; then
    echo "error: PGEN_SOTA_SV_PARSE_FULL_RATIO_PROMOTION_TARGET_MIN_RATIO must be an integer between 0 and 100" >&2
    exit 2
fi
if ! [[ "$SV_PARSE_FULL_RATIO_PROMOTION_TRIALS" =~ ^[0-9]+$ ]] || [[ "$SV_PARSE_FULL_RATIO_PROMOTION_TRIALS" -lt 1 ]]; then
    echo "error: PGEN_SOTA_SV_PARSE_FULL_RATIO_PROMOTION_TRIALS must be an integer >= 1" >&2
    exit 2
fi
if ! [[ "$SV_PARSE_FULL_RATIO_PROMOTION_COUNT" =~ ^[0-9]+$ ]] || [[ "$SV_PARSE_FULL_RATIO_PROMOTION_COUNT" -lt 1 ]]; then
    echo "error: PGEN_SOTA_SV_PARSE_FULL_RATIO_PROMOTION_COUNT must be an integer >= 1" >&2
    exit 2
fi
if ! [[ "$SV_PARSE_FULL_RATIO_PROMOTION_SEED_BASE" =~ ^[0-9]+$ ]]; then
    echo "error: PGEN_SOTA_SV_PARSE_FULL_RATIO_PROMOTION_SEED_BASE must be an integer >= 0" >&2
    exit 2
fi
if [[ "$SV_PARSE_FULL_RATIO_PROMOTION_PARSE_FULL_MODE" != "auto" && "$SV_PARSE_FULL_RATIO_PROMOTION_PARSE_FULL_MODE" != "0" && "$SV_PARSE_FULL_RATIO_PROMOTION_PARSE_FULL_MODE" != "1" ]]; then
    echo "error: PGEN_SOTA_SV_PARSE_FULL_RATIO_PROMOTION_PARSE_FULL_MODE must be one of: auto, 0, 1" >&2
    exit 2
fi
if ! [[ "$SV_PARSE_FULL_RATIO_PROMOTION_SEMANTIC_CLOSURE_MODE" =~ ^[01]$ ]]; then
    echo "error: PGEN_SOTA_SV_PARSE_FULL_RATIO_PROMOTION_SEMANTIC_CLOSURE_MODE must be 0 or 1" >&2
    exit 2
fi
if [[ "$SV_PARSE_FULL_RATIO_PROMOTION_STIMULI_MODE" != "sv_file" && "$SV_PARSE_FULL_RATIO_PROMOTION_STIMULI_MODE" != "sv_snippet" && "$SV_PARSE_FULL_RATIO_PROMOTION_STIMULI_MODE" != "sv_pp_file" && "$SV_PARSE_FULL_RATIO_PROMOTION_STIMULI_MODE" != "sv_pp_snippet" && "$SV_PARSE_FULL_RATIO_PROMOTION_STIMULI_MODE" != "sv_semantic_file" ]]; then
    echo "error: PGEN_SOTA_SV_PARSE_FULL_RATIO_PROMOTION_STIMULI_MODE must be one of: sv_file, sv_snippet, sv_pp_file, sv_pp_snippet, sv_semantic_file" >&2
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
SV_DECLARED_SHADOW_PROMOTION_REPORT_JSON="<unset>"
SV_DECLARED_SHADOW_PROMOTION_RECOMMENDATION="<unset>"
SV_DECLARED_SHADOW_PROMOTION_ELIGIBLE="<unset>"
SV_DECLARED_SHADOW_PROMOTION_TOTALS_FAILED="<unset>"
SV_DECLARED_SHADOW_PROMOTION_TOTALS_CHECKED="<unset>"
SV_DECLARED_SHADOW_PROMOTION_PRIMARY_NON_SHADOW_BLOCKER="<unset>"
SV_DECLARED_SHADOW_PROMOTION_REPORT_DECLARED_SHADOW_PARSEABLE_ONLY="<unset>"
SV_DECLARED_SHADOW_PROMOTION_FAILED_TRIAL_COUNT="<unset>"
SV_DECLARED_SHADOW_PROMOTION_NON_SHADOW_BLOCKED_TRIAL_COUNT="<unset>"
SV_PARSE_FULL_RATIO_PROMOTION_REPORT_JSON="<unset>"
SV_PARSE_FULL_RATIO_PROMOTION_RECOMMENDATION="<unset>"
SV_PARSE_FULL_RATIO_PROMOTION_PRIMARY_NON_RATIO_BLOCKER="<unset>"
SV_PARSE_FULL_RATIO_PROMOTION_OBSERVED_RATIO_AVG="<unset>"

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
echo "sv_declared_shadow_promotion_trials: $SV_DECLARED_SHADOW_PROMOTION_TRIALS"
echo "sv_declared_shadow_promotion_count: $SV_DECLARED_SHADOW_PROMOTION_COUNT"
echo "sv_declared_shadow_promotion_seed_base: $SV_DECLARED_SHADOW_PROMOTION_SEED_BASE"
echo "sv_declared_shadow_promotion_target_max_attempts: $SV_DECLARED_SHADOW_PROMOTION_TARGET_MAX_ATTEMPTS"
echo "sv_declared_shadow_promotion_parse_full_mode: $SV_DECLARED_SHADOW_PROMOTION_PARSE_FULL_MODE"
echo "sv_declared_shadow_promotion_min_checked: $SV_DECLARED_SHADOW_PROMOTION_MIN_CHECKED"
echo "sv_declared_shadow_promotion_semantic_closure_mode: $SV_DECLARED_SHADOW_PROMOTION_SEMANTIC_CLOSURE_MODE"
echo "sv_declared_shadow_promotion_stimuli_mode: $SV_DECLARED_SHADOW_PROMOTION_STIMULI_MODE"
echo "sv_declared_shadow_promotion_declared_shadow_parseable_only: $SV_DECLARED_SHADOW_PROMOTION_DECLARED_SHADOW_PARSEABLE_ONLY"
echo "run_sv_parse_full_ratio_promotion: $RUN_SV_PARSE_FULL_RATIO_PROMOTION"
echo "require_sv_parse_full_ratio_promotion_strict: $REQUIRE_SV_PARSE_FULL_RATIO_PROMOTION_STRICT"
echo "sv_parse_full_ratio_promotion_target_min_ratio: $SV_PARSE_FULL_RATIO_PROMOTION_TARGET_MIN_RATIO"
echo "sv_parse_full_ratio_promotion_trials: $SV_PARSE_FULL_RATIO_PROMOTION_TRIALS"
echo "sv_parse_full_ratio_promotion_count: $SV_PARSE_FULL_RATIO_PROMOTION_COUNT"
echo "sv_parse_full_ratio_promotion_seed_base: $SV_PARSE_FULL_RATIO_PROMOTION_SEED_BASE"
echo "sv_parse_full_ratio_promotion_parse_full_mode: $SV_PARSE_FULL_RATIO_PROMOTION_PARSE_FULL_MODE"
echo "sv_parse_full_ratio_promotion_semantic_closure_mode: $SV_PARSE_FULL_RATIO_PROMOTION_SEMANTIC_CLOSURE_MODE"
echo "sv_parse_full_ratio_promotion_stimuli_mode: $SV_PARSE_FULL_RATIO_PROMOTION_STIMULI_MODE"
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
    SV_DECLARED_SHADOW_PROMOTION_STAGE_STATE_DIR="${STATE_DIR}/work/sv_declared_shadow_promotion_gate"
    SV_DECLARED_SHADOW_PROMOTION_STAGE_REPORT_JSON="${SV_DECLARED_SHADOW_PROMOTION_STAGE_STATE_DIR}/work/systemverilog_declared_identifier_promotion_report.json"

    if [[ "$REQUIRE_SV_DECLARED_SHADOW_PROMOTION_STRICT" -eq 1 ]]; then
        run_check "sv_declared_shadow_promotion_gate" "required" "strict declared-shadow promotion-trial gate" \
            env \
                PGEN_SV_DECLARED_SHADOW_PROMOTION_MODE=1 \
                PGEN_SV_DECLARED_SHADOW_PROMOTION_TRIALS="$SV_DECLARED_SHADOW_PROMOTION_TRIALS" \
                PGEN_SV_DECLARED_SHADOW_PROMOTION_COUNT="$SV_DECLARED_SHADOW_PROMOTION_COUNT" \
                PGEN_SV_DECLARED_SHADOW_PROMOTION_SEED_BASE="$SV_DECLARED_SHADOW_PROMOTION_SEED_BASE" \
                PGEN_SV_DECLARED_SHADOW_PROMOTION_TARGET_MAX_ATTEMPTS="$SV_DECLARED_SHADOW_PROMOTION_TARGET_MAX_ATTEMPTS" \
                PGEN_SV_DECLARED_SHADOW_PROMOTION_PARSE_FULL_MODE="$SV_DECLARED_SHADOW_PROMOTION_PARSE_FULL_MODE" \
                PGEN_SV_DECLARED_SHADOW_PROMOTION_MIN_CHECKED="$SV_DECLARED_SHADOW_PROMOTION_MIN_CHECKED" \
                PGEN_SV_DECLARED_SHADOW_PROMOTION_SEMANTIC_CLOSURE_MODE="$SV_DECLARED_SHADOW_PROMOTION_SEMANTIC_CLOSURE_MODE" \
                PGEN_SV_DECLARED_SHADOW_PROMOTION_STIMULI_MODE="$SV_DECLARED_SHADOW_PROMOTION_STIMULI_MODE" \
                PGEN_SV_DECLARED_SHADOW_PROMOTION_DECLARED_SHADOW_PARSEABLE_ONLY="$SV_DECLARED_SHADOW_PROMOTION_DECLARED_SHADOW_PARSEABLE_ONLY" \
                PGEN_SV_DECLARED_SHADOW_PROMOTION_STATE_DIR="$SV_DECLARED_SHADOW_PROMOTION_STAGE_STATE_DIR" \
                make -C rust SHELL=/bin/bash sv_declared_shadow_promotion_gate
    else
        run_check "sv_declared_shadow_promotion_gate" "informational" "report-only declared-shadow promotion-trial gate" \
            env \
                PGEN_SV_DECLARED_SHADOW_PROMOTION_MODE=auto \
                PGEN_SV_DECLARED_SHADOW_PROMOTION_TRIALS="$SV_DECLARED_SHADOW_PROMOTION_TRIALS" \
                PGEN_SV_DECLARED_SHADOW_PROMOTION_COUNT="$SV_DECLARED_SHADOW_PROMOTION_COUNT" \
                PGEN_SV_DECLARED_SHADOW_PROMOTION_SEED_BASE="$SV_DECLARED_SHADOW_PROMOTION_SEED_BASE" \
                PGEN_SV_DECLARED_SHADOW_PROMOTION_TARGET_MAX_ATTEMPTS="$SV_DECLARED_SHADOW_PROMOTION_TARGET_MAX_ATTEMPTS" \
                PGEN_SV_DECLARED_SHADOW_PROMOTION_PARSE_FULL_MODE="$SV_DECLARED_SHADOW_PROMOTION_PARSE_FULL_MODE" \
                PGEN_SV_DECLARED_SHADOW_PROMOTION_MIN_CHECKED="$SV_DECLARED_SHADOW_PROMOTION_MIN_CHECKED" \
                PGEN_SV_DECLARED_SHADOW_PROMOTION_SEMANTIC_CLOSURE_MODE="$SV_DECLARED_SHADOW_PROMOTION_SEMANTIC_CLOSURE_MODE" \
                PGEN_SV_DECLARED_SHADOW_PROMOTION_STIMULI_MODE="$SV_DECLARED_SHADOW_PROMOTION_STIMULI_MODE" \
                PGEN_SV_DECLARED_SHADOW_PROMOTION_DECLARED_SHADOW_PARSEABLE_ONLY="$SV_DECLARED_SHADOW_PROMOTION_DECLARED_SHADOW_PARSEABLE_ONLY" \
                PGEN_SV_DECLARED_SHADOW_PROMOTION_STATE_DIR="$SV_DECLARED_SHADOW_PROMOTION_STAGE_STATE_DIR" \
                make -C rust SHELL=/bin/bash sv_declared_shadow_promotion_gate
    fi

    if [[ -f "$SV_DECLARED_SHADOW_PROMOTION_STAGE_REPORT_JSON" ]]; then
        declared_shadow_recommendation="$(jq -er '.recommendation // "unknown"' "$SV_DECLARED_SHADOW_PROMOTION_STAGE_REPORT_JSON" 2>/dev/null || echo "unknown")"
        declared_shadow_eligible="$(jq -er '(.eligibility.eligible_for_runtime_enforcement // false) | if . then "true" else "false" end' "$SV_DECLARED_SHADOW_PROMOTION_STAGE_REPORT_JSON" 2>/dev/null || echo "unknown")"
        declared_shadow_failed="$(jq -er '.totals.failed // "unknown"' "$SV_DECLARED_SHADOW_PROMOTION_STAGE_REPORT_JSON" 2>/dev/null || echo "unknown")"
        declared_shadow_checked="$(jq -er '.totals.checked // "unknown"' "$SV_DECLARED_SHADOW_PROMOTION_STAGE_REPORT_JSON" 2>/dev/null || echo "unknown")"
        declared_shadow_primary_non_shadow_blocker="$(jq -er '.blockers.primary_non_shadow_blocker // "unknown"' "$SV_DECLARED_SHADOW_PROMOTION_STAGE_REPORT_JSON" 2>/dev/null || echo "unknown")"
        declared_shadow_parseable_only="$(jq -er '.declared_shadow_parseable_only // "unknown"' "$SV_DECLARED_SHADOW_PROMOTION_STAGE_REPORT_JSON" 2>/dev/null || echo "unknown")"
        declared_shadow_failed_trial_count="$(jq -er '.blockers.failed_trial_count // "unknown"' "$SV_DECLARED_SHADOW_PROMOTION_STAGE_REPORT_JSON" 2>/dev/null || echo "unknown")"
        declared_shadow_non_shadow_blocked_trial_count="$(jq -er '.blockers.non_shadow_blocked_trial_count // "unknown"' "$SV_DECLARED_SHADOW_PROMOTION_STAGE_REPORT_JSON" 2>/dev/null || echo "unknown")"
        SV_DECLARED_SHADOW_PROMOTION_REPORT_JSON="$SV_DECLARED_SHADOW_PROMOTION_STAGE_REPORT_JSON"
        SV_DECLARED_SHADOW_PROMOTION_RECOMMENDATION="$declared_shadow_recommendation"
        SV_DECLARED_SHADOW_PROMOTION_ELIGIBLE="$declared_shadow_eligible"
        SV_DECLARED_SHADOW_PROMOTION_TOTALS_FAILED="$declared_shadow_failed"
        SV_DECLARED_SHADOW_PROMOTION_TOTALS_CHECKED="$declared_shadow_checked"
        SV_DECLARED_SHADOW_PROMOTION_PRIMARY_NON_SHADOW_BLOCKER="$declared_shadow_primary_non_shadow_blocker"
        SV_DECLARED_SHADOW_PROMOTION_REPORT_DECLARED_SHADOW_PARSEABLE_ONLY="$declared_shadow_parseable_only"
        SV_DECLARED_SHADOW_PROMOTION_FAILED_TRIAL_COUNT="$declared_shadow_failed_trial_count"
        SV_DECLARED_SHADOW_PROMOTION_NON_SHADOW_BLOCKED_TRIAL_COUNT="$declared_shadow_non_shadow_blocked_trial_count"
    else
        SV_DECLARED_SHADOW_PROMOTION_REPORT_JSON="<missing>"
    fi

    echo "sv_declared_shadow_promotion_report_json: $SV_DECLARED_SHADOW_PROMOTION_REPORT_JSON"
    echo "sv_declared_shadow_promotion_recommendation: $SV_DECLARED_SHADOW_PROMOTION_RECOMMENDATION"
    echo "sv_declared_shadow_promotion_eligible_for_runtime_enforcement: $SV_DECLARED_SHADOW_PROMOTION_ELIGIBLE"
    echo "sv_declared_shadow_promotion_totals_failed: $SV_DECLARED_SHADOW_PROMOTION_TOTALS_FAILED"
    echo "sv_declared_shadow_promotion_totals_checked: $SV_DECLARED_SHADOW_PROMOTION_TOTALS_CHECKED"
    echo "sv_declared_shadow_promotion_primary_non_shadow_blocker: $SV_DECLARED_SHADOW_PROMOTION_PRIMARY_NON_SHADOW_BLOCKER"
    echo "sv_declared_shadow_promotion_declared_shadow_parseable_only: $SV_DECLARED_SHADOW_PROMOTION_REPORT_DECLARED_SHADOW_PARSEABLE_ONLY"
    echo "sv_declared_shadow_promotion_failed_trial_count: $SV_DECLARED_SHADOW_PROMOTION_FAILED_TRIAL_COUNT"
    echo "sv_declared_shadow_promotion_non_shadow_blocked_trial_count: $SV_DECLARED_SHADOW_PROMOTION_NON_SHADOW_BLOCKED_TRIAL_COUNT"
fi

if [[ "$RUN_SV_PARSE_FULL_RATIO_PROMOTION" -eq 1 ]]; then
    SV_PARSE_FULL_RATIO_PROMOTION_STAGE_STATE_DIR="${STATE_DIR}/work/sv_parse_full_ratio_promotion_gate"
    SV_PARSE_FULL_RATIO_PROMOTION_STAGE_REPORT_JSON="${SV_PARSE_FULL_RATIO_PROMOTION_STAGE_STATE_DIR}/work/systemverilog_parse_full_ratio_promotion_report.json"

    if [[ "$REQUIRE_SV_PARSE_FULL_RATIO_PROMOTION_STRICT" -eq 1 ]]; then
        run_check "sv_parse_full_ratio_promotion_gate" "required" "strict parse-full ratio promotion-trial gate" \
            env \
                PGEN_SV_PARSE_FULL_RATIO_PROMOTION_MODE=1 \
                PGEN_SV_PARSE_FULL_RATIO_PROMOTION_TARGET_MIN_RATIO="$SV_PARSE_FULL_RATIO_PROMOTION_TARGET_MIN_RATIO" \
                PGEN_SV_PARSE_FULL_RATIO_PROMOTION_TRIALS="$SV_PARSE_FULL_RATIO_PROMOTION_TRIALS" \
                PGEN_SV_PARSE_FULL_RATIO_PROMOTION_COUNT="$SV_PARSE_FULL_RATIO_PROMOTION_COUNT" \
                PGEN_SV_PARSE_FULL_RATIO_PROMOTION_SEED_BASE="$SV_PARSE_FULL_RATIO_PROMOTION_SEED_BASE" \
                PGEN_SV_PARSE_FULL_RATIO_PROMOTION_PARSE_FULL_MODE="$SV_PARSE_FULL_RATIO_PROMOTION_PARSE_FULL_MODE" \
                PGEN_SV_PARSE_FULL_RATIO_PROMOTION_SEMANTIC_CLOSURE_MODE="$SV_PARSE_FULL_RATIO_PROMOTION_SEMANTIC_CLOSURE_MODE" \
                PGEN_SV_PARSE_FULL_RATIO_PROMOTION_STIMULI_MODE="$SV_PARSE_FULL_RATIO_PROMOTION_STIMULI_MODE" \
                PGEN_SV_PARSE_FULL_RATIO_PROMOTION_STATE_DIR="$SV_PARSE_FULL_RATIO_PROMOTION_STAGE_STATE_DIR" \
                make -C rust SHELL=/bin/bash sv_parse_full_ratio_promotion_gate
    else
        run_check "sv_parse_full_ratio_promotion_gate" "informational" "report-only parse-full ratio promotion-trial gate" \
            env \
                PGEN_SV_PARSE_FULL_RATIO_PROMOTION_MODE=auto \
                PGEN_SV_PARSE_FULL_RATIO_PROMOTION_TARGET_MIN_RATIO="$SV_PARSE_FULL_RATIO_PROMOTION_TARGET_MIN_RATIO" \
                PGEN_SV_PARSE_FULL_RATIO_PROMOTION_TRIALS="$SV_PARSE_FULL_RATIO_PROMOTION_TRIALS" \
                PGEN_SV_PARSE_FULL_RATIO_PROMOTION_COUNT="$SV_PARSE_FULL_RATIO_PROMOTION_COUNT" \
                PGEN_SV_PARSE_FULL_RATIO_PROMOTION_SEED_BASE="$SV_PARSE_FULL_RATIO_PROMOTION_SEED_BASE" \
                PGEN_SV_PARSE_FULL_RATIO_PROMOTION_PARSE_FULL_MODE="$SV_PARSE_FULL_RATIO_PROMOTION_PARSE_FULL_MODE" \
                PGEN_SV_PARSE_FULL_RATIO_PROMOTION_SEMANTIC_CLOSURE_MODE="$SV_PARSE_FULL_RATIO_PROMOTION_SEMANTIC_CLOSURE_MODE" \
                PGEN_SV_PARSE_FULL_RATIO_PROMOTION_STIMULI_MODE="$SV_PARSE_FULL_RATIO_PROMOTION_STIMULI_MODE" \
                PGEN_SV_PARSE_FULL_RATIO_PROMOTION_STATE_DIR="$SV_PARSE_FULL_RATIO_PROMOTION_STAGE_STATE_DIR" \
                make -C rust SHELL=/bin/bash sv_parse_full_ratio_promotion_gate
    fi

    if [[ -f "$SV_PARSE_FULL_RATIO_PROMOTION_STAGE_REPORT_JSON" ]]; then
        promotion_recommendation="$(jq -er '.recommendation // "unknown"' "$SV_PARSE_FULL_RATIO_PROMOTION_STAGE_REPORT_JSON" 2>/dev/null || echo "unknown")"
        promotion_primary_blocker="$(jq -er '.blockers.primary_non_ratio_blocker // "unknown"' "$SV_PARSE_FULL_RATIO_PROMOTION_STAGE_REPORT_JSON" 2>/dev/null || echo "unknown")"
        promotion_ratio_avg="$(jq -er '.totals.observed_ratio_avg // "unknown"' "$SV_PARSE_FULL_RATIO_PROMOTION_STAGE_REPORT_JSON" 2>/dev/null || echo "unknown")"
        SV_PARSE_FULL_RATIO_PROMOTION_REPORT_JSON="$SV_PARSE_FULL_RATIO_PROMOTION_STAGE_REPORT_JSON"
        SV_PARSE_FULL_RATIO_PROMOTION_RECOMMENDATION="$promotion_recommendation"
        SV_PARSE_FULL_RATIO_PROMOTION_PRIMARY_NON_RATIO_BLOCKER="$promotion_primary_blocker"
        SV_PARSE_FULL_RATIO_PROMOTION_OBSERVED_RATIO_AVG="$promotion_ratio_avg"
    else
        SV_PARSE_FULL_RATIO_PROMOTION_REPORT_JSON="<missing>"
    fi

    echo "sv_parse_full_ratio_promotion_report_json: $SV_PARSE_FULL_RATIO_PROMOTION_REPORT_JSON"
    echo "sv_parse_full_ratio_promotion_recommendation: $SV_PARSE_FULL_RATIO_PROMOTION_RECOMMENDATION"
    echo "sv_parse_full_ratio_promotion_primary_non_ratio_blocker: $SV_PARSE_FULL_RATIO_PROMOTION_PRIMARY_NON_RATIO_BLOCKER"
    echo "sv_parse_full_ratio_promotion_observed_ratio_avg: $SV_PARSE_FULL_RATIO_PROMOTION_OBSERVED_RATIO_AVG"
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
    if [[ "$RUN_SV_DECLARED_SHADOW_PROMOTION" -eq 1 ]]; then
        echo
        echo "Declared-Shadow Promotion Telemetry"
        echo "sv_declared_shadow_promotion_report_json: $SV_DECLARED_SHADOW_PROMOTION_REPORT_JSON"
        echo "sv_declared_shadow_promotion_recommendation: $SV_DECLARED_SHADOW_PROMOTION_RECOMMENDATION"
        echo "sv_declared_shadow_promotion_eligible_for_runtime_enforcement: $SV_DECLARED_SHADOW_PROMOTION_ELIGIBLE"
        echo "sv_declared_shadow_promotion_totals_failed: $SV_DECLARED_SHADOW_PROMOTION_TOTALS_FAILED"
        echo "sv_declared_shadow_promotion_totals_checked: $SV_DECLARED_SHADOW_PROMOTION_TOTALS_CHECKED"
        echo "sv_declared_shadow_promotion_primary_non_shadow_blocker: $SV_DECLARED_SHADOW_PROMOTION_PRIMARY_NON_SHADOW_BLOCKER"
        echo "sv_declared_shadow_promotion_declared_shadow_parseable_only: $SV_DECLARED_SHADOW_PROMOTION_REPORT_DECLARED_SHADOW_PARSEABLE_ONLY"
        echo "sv_declared_shadow_promotion_failed_trial_count: $SV_DECLARED_SHADOW_PROMOTION_FAILED_TRIAL_COUNT"
        echo "sv_declared_shadow_promotion_non_shadow_blocked_trial_count: $SV_DECLARED_SHADOW_PROMOTION_NON_SHADOW_BLOCKED_TRIAL_COUNT"
    fi
    if [[ "$RUN_SV_PARSE_FULL_RATIO_PROMOTION" -eq 1 ]]; then
        echo
        echo "Parse-Full Promotion Telemetry"
        echo "sv_parse_full_ratio_promotion_report_json: $SV_PARSE_FULL_RATIO_PROMOTION_REPORT_JSON"
        echo "sv_parse_full_ratio_promotion_recommendation: $SV_PARSE_FULL_RATIO_PROMOTION_RECOMMENDATION"
        echo "sv_parse_full_ratio_promotion_primary_non_ratio_blocker: $SV_PARSE_FULL_RATIO_PROMOTION_PRIMARY_NON_RATIO_BLOCKER"
        echo "sv_parse_full_ratio_promotion_observed_ratio_avg: $SV_PARSE_FULL_RATIO_PROMOTION_OBSERVED_RATIO_AVG"
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

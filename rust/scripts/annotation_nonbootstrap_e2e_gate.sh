#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
RUST_DIR="$ROOT_DIR/rust"
GRAMMARS_DIR="$ROOT_DIR/grammars"
GENERATED_DIR="$ROOT_DIR/generated"
TOOLS_DIR="$ROOT_DIR/tools"

STATE_DIR="${PGEN_ANNOTATION_NONBOOTSTRAP_STATE_DIR:-$RUST_DIR/target/annotation_nonbootstrap_e2e_gate}"
LOG_DIR="$STATE_DIR/logs"
WORK_DIR="$STATE_DIR/work"

SAMPLE_COUNT="${PGEN_ANNOTATION_NONBOOTSTRAP_COUNT:-24}"
RETURN_SEED="${PGEN_ANNOTATION_NONBOOTSTRAP_RETURN_SEED:-6021}"
SEMANTIC_SEED="${PGEN_ANNOTATION_NONBOOTSTRAP_SEMANTIC_SEED:-6022}"
REGEX_SEED="${PGEN_ANNOTATION_NONBOOTSTRAP_REGEX_SEED:-6023}"

if ! [[ "$SAMPLE_COUNT" =~ ^[0-9]+$ ]] || [[ "$SAMPLE_COUNT" -lt 1 ]]; then
    echo "error: PGEN_ANNOTATION_NONBOOTSTRAP_COUNT must be an integer >= 1" >&2
    exit 2
fi

mkdir -p "$LOG_DIR" "$WORK_DIR"

AST_PIPELINE_BIN="$RUST_DIR/target/debug/ast_pipeline"
EBNF_TO_JSON="$TOOLS_DIR/ebnf_to_json.pl"

RETURN_JSON="$GENERATED_DIR/return_annotation.json"
SEMANTIC_JSON="$GENERATED_DIR/semantic_annotation.json"
REGEX_EBNF="$GRAMMARS_DIR/regex.ebnf"
REGEX_JSON="$WORK_DIR/regex.json"

if [[ ! -f "$EBNF_TO_JSON" ]]; then
    echo "error: missing EBNF frontend at '$EBNF_TO_JSON'" >&2
    exit 1
fi
if [[ ! -f "$RETURN_JSON" ]]; then
    echo "error: missing return annotation JSON at '$RETURN_JSON'" >&2
    exit 1
fi
if [[ ! -f "$SEMANTIC_JSON" ]]; then
    echo "error: missing semantic annotation JSON at '$SEMANTIC_JSON'" >&2
    exit 1
fi
if [[ ! -f "$REGEX_EBNF" ]]; then
    echo "error: missing regex grammar at '$REGEX_EBNF'" >&2
    exit 1
fi

echo "==> Annotation non-bootstrap E2E gate"
echo "state_dir: $STATE_DIR"
echo "sample_count: $SAMPLE_COUNT"
echo "return_seed: $RETURN_SEED"
echo "semantic_seed: $SEMANTIC_SEED"
echo "regex_seed: $REGEX_SEED"

run_logged() {
    local label="$1"
    shift
    local log_file="$LOG_DIR/${label}.log"
    echo "==> ${label}"
    if "$@" >"$log_file" 2>&1; then
        echo "    ok (${log_file})"
    else
        echo "    fail (${log_file})" >&2
        tail -n 40 "$log_file" >&2 || true
        exit 1
    fi
}

run_logged_rust() {
    local label="$1"
    shift
    local log_file="$LOG_DIR/${label}.log"
    echo "==> ${label}"
    if (
        cd "$RUST_DIR"
        "$@"
    ) >"$log_file" 2>&1; then
        echo "    ok (${log_file})"
    else
        echo "    fail (${log_file})" >&2
        tail -n 40 "$log_file" >&2 || true
        exit 1
    fi
}

run_logged_rust "build_generated_ast_pipeline" \
    cargo build --features generated_parsers --bin ast_pipeline

if [[ ! -x "$AST_PIPELINE_BIN" ]]; then
    echo "error: ast_pipeline binary is still missing at '$AST_PIPELINE_BIN' after build" >&2
    exit 1
fi

run_logged "frontend_regex_ebnf_to_json" \
    "$EBNF_TO_JSON" --pretty "$REGEX_EBNF" -o "$REGEX_JSON"

run_logged "nonbootstrap_return_generate_parser" \
    "$AST_PIPELINE_BIN" "$RETURN_JSON" --generate-parser --output "$WORK_DIR/return_annotation_nonbootstrap.rs"
run_logged "nonbootstrap_semantic_generate_parser" \
    "$AST_PIPELINE_BIN" "$SEMANTIC_JSON" --generate-parser --output "$WORK_DIR/semantic_annotation_nonbootstrap.rs"
run_logged "nonbootstrap_regex_generate_parser" \
    "$AST_PIPELINE_BIN" "$REGEX_JSON" --generate-parser --output "$WORK_DIR/regex_nonbootstrap.rs"

run_logged "nonbootstrap_return_stimuli_parseability" \
    "$AST_PIPELINE_BIN" "$RETURN_JSON" \
    --generate-stimuli \
    --count "$SAMPLE_COUNT" \
    --seed "$RETURN_SEED" \
    --validate-parseability \
    --output "$WORK_DIR/return_samples.txt" \
    --coverage-output "$WORK_DIR/return_coverage.json" \
    --gap-report-json "$WORK_DIR/return_gap_report.json"

run_logged "nonbootstrap_semantic_stimuli_parseability" \
    "$AST_PIPELINE_BIN" "$SEMANTIC_JSON" \
    --generate-stimuli \
    --count "$SAMPLE_COUNT" \
    --seed "$SEMANTIC_SEED" \
    --validate-parseability \
    --output "$WORK_DIR/semantic_samples.txt" \
    --coverage-output "$WORK_DIR/semantic_coverage.json" \
    --gap-report-json "$WORK_DIR/semantic_gap_report.json"

run_logged "nonbootstrap_regex_stimuli" \
    "$AST_PIPELINE_BIN" "$REGEX_JSON" \
    --generate-stimuli \
    --count "$SAMPLE_COUNT" \
    --seed "$REGEX_SEED" \
    --output "$WORK_DIR/regex_samples.txt" \
    --coverage-output "$WORK_DIR/regex_coverage.json" \
    --gap-report-json "$WORK_DIR/regex_gap_report.json"

cat <<EOF
✅ Annotation non-bootstrap E2E gate passed.
Logs: $LOG_DIR
Artifacts: $WORK_DIR
EOF

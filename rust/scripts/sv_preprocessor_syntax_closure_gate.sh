#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
RUST_DIR="$ROOT_DIR/rust"
SV_SYNTAX_CLOSURE_GATE="$RUST_DIR/scripts/sv_syntax_closure_gate.sh"

exec env \
    PGEN_SV_SYNTAX_CLOSURE_STATE_DIR="${PGEN_SV_PREPROCESSOR_SYNTAX_CLOSURE_STATE_DIR:-$RUST_DIR/target/sv_preprocessor_syntax_closure_gate}" \
    PGEN_SV_SYNTAX_CLOSURE_CONTRACT="${PGEN_SV_PREPROCESSOR_SYNTAX_CLOSURE_CONTRACT:-$RUST_DIR/test_data/grammar_quality/systemverilog_preprocessor_syntax_closure_contract.json}" \
    "$SV_SYNTAX_CLOSURE_GATE"

#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
RUST_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
ROOT_DIR="$(cd "$RUST_DIR/.." && pwd)"
STATE_DIR="$RUST_DIR/target/clippy_gate"
LOG_DIR="$STATE_DIR/logs"

FORCE_RUN="${PGEN_CLIPPY_FORCE:-0}"
if [[ "${1:-}" == "--force" ]]; then
    FORCE_RUN=1
fi

GENERATED_STRICT="${PGEN_CLIPPY_GENERATED_STRICT:-0}"

mkdir -p "$LOG_DIR"

run_stage() {
    local stage_name="$1"
    shift
    local log_file="$LOG_DIR/${stage_name}.log"
    echo "==> $stage_name"
    if "$@" >"$log_file" 2>&1; then
        echo "    ok ($log_file)"
        return 0
    fi
    echo "    fail ($log_file)" >&2
    tail -n 40 "$log_file" >&2 || true
    return 1
}

should_run=0
if [[ "$FORCE_RUN" -eq 1 ]]; then
    should_run=1
else
    mapfile -t changed_paths < <(
        {
            git -C "$ROOT_DIR" diff --name-only
            git -C "$ROOT_DIR" diff --cached --name-only
            git -C "$ROOT_DIR" ls-files --others --exclude-standard
        } | awk 'NF' | sort -u
    )

    for path in "${changed_paths[@]}"; do
        if [[ "$path" == rust/*.rs || "$path" == generated/*.rs || "$path" == rust/Cargo.toml || "$path" == rust/Cargo.lock ]]; then
            should_run=1
            break
        fi
    done
fi

if [[ "$should_run" -ne 1 ]]; then
    echo "No Rust/generated Rust changes detected; skipping clippy flow."
    exit 0
fi

echo "Running clippy flow (Rust files amended/generated detected)."

run_stage "clippy_source_all_targets" \
    cargo clippy --manifest-path "$RUST_DIR/Cargo.toml" --all-targets

if run_stage "clippy_generated_all_targets" \
    cargo clippy --manifest-path "$RUST_DIR/Cargo.toml" --all-targets --features generated_parsers,ebnf_dual_run; then
    echo "Generated-parser clippy stage: pass"
else
    if [[ "$GENERATED_STRICT" == "1" ]]; then
        echo "Generated-parser clippy stage failed with PGEN_CLIPPY_GENERATED_STRICT=1" >&2
        exit 1
    fi
    echo "Generated-parser clippy stage failed (non-strict mode). See $LOG_DIR/clippy_generated_all_targets.log" >&2
fi

echo "Clippy flow completed."

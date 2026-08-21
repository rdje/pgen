#!/usr/bin/env bash
# Sequential (one cargo lock at a time) rebuild of both instruments after a codegen change.
set -euo pipefail
cd "$(git -C "$(dirname "${BASH_SOURCE[0]}")" rev-parse --show-toplevel)"
echo "=== [1/2] release parseability_probe ==="
( cd rust && cargo build --release --features generated_parsers --bin parseability_probe )
echo "=== [2/2] debug ast_pipeline (generated_parsers + ebnf_dual_run) ==="
( cd rust && cargo build --features "generated_parsers ebnf_dual_run" --bin ast_pipeline )
echo "=== BOTH BINARIES BUILT ==="
ls -la rust/target/release/parseability_probe rust/target/debug/ast_pipeline

#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
RUST_DIR="$(cd "${SCRIPT_DIR}/.." && pwd)"
REPORT_DIR="${PERF_REPORT_DIR:-${RUST_DIR}/target/performance_gate}"
THRESHOLDS_JSON="${PERF_THRESHOLDS_JSON:-${RUST_DIR}/perf/thresholds.json}"
ITERATIONS="${PERF_ITERATIONS:-80}"
WARMUP_ITERATIONS="${PERF_WARMUP_ITERATIONS:-10}"
MAX_CASES="${PERF_MAX_CASES:-64}"
SUITE_FILTER="${PERF_SUITE_FILTER:-}"

mkdir -p "${REPORT_DIR}"

ARGS=(
  --iterations "${ITERATIONS}"
  --warmup-iterations "${WARMUP_ITERATIONS}"
  --max-cases "${MAX_CASES}"
  --thresholds-json "${THRESHOLDS_JSON}"
  --output-json "${REPORT_DIR}/report.json"
  --enforce-thresholds
)

if [[ -n "${SUITE_FILTER}" ]]; then
  ARGS+=(--suite "${SUITE_FILTER}")
fi

cd "${RUST_DIR}"
cargo run --features generated_parsers --bin perf_bench -- "${ARGS[@]}"

echo "✅ Performance gate passed. Report: ${REPORT_DIR}/report.json"

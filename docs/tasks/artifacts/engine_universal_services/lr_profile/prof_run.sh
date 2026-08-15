#!/usr/bin/env bash
# `ENGINE-UNIVERSAL-SERVICES.20` slice 4 / `.21` acceptance (e) — repeat slice 2's profile so its
# single-run numbers gain a spread. One run: bare parse (the FUSED `cascade_*` graph, which is the
# ONLY graph a production parse runs — the deterministic counters all observe the PROTOCOL graph
# instead, TOOLBOX 3.8), sampled at 1 ms.
#
# ⛔ PROMOTED OUT OF `rust/target/audit_scratch/` BY `.21` (e). The 3.5-6 MB reports it writes are
# deliberately NOT tracked; THIS is, so they are regenerable — which is the whole difference between
# a number that can be re-derived and one you have to take on trust.
#
# Usage: prof_run.sh <corpus-file> <out.txt> [sample-seconds]
set -uo pipefail
# lr_profile → engine_universal_services → artifacts → tasks → docs → repository root (5 levels).
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../../../.." && pwd)"; cd "$ROOT" || exit 2
if [ ! -x "./rust/target/release/parseability_probe" ]; then
  echo "prof_run: rust/target/release/parseability_probe is missing — build it first:" >&2
  echo "  (cd rust && cargo build --release --features generated_parsers --bin parseability_probe)" >&2
  exit 2
fi
F="$1"; OUT="$2"; SECS="${3:-6}"
exec ./rust/target/release/parseability_probe --parse systemverilog "$F" --profile sv_2017 \
     >/dev/null 2>&1 &
PID=$!
sleep 0.4
/usr/bin/sample "$PID" "$SECS" 1 -f "$OUT" >/dev/null 2>&1
wait "$PID"
echo "sampled pid=$PID -> $OUT"

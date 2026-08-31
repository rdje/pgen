#!/bin/bash
# `ENGINE-UNIVERSAL-SERVICES.69`(a) — does the SHIPPED profile accept each shape at all?
#
# ⛔ THIS RUNS BEFORE THE CENSUS, NOT AFTER. A shape the parser rejects would send the
# census down the rejection path and report a stack reading about a parse that failed for
# an unrelated reason. It is also a north-star check: SystemVerilog is 100 % LRM-compliant
# BY DEFAULT and over-acceptance is a defect, so a shape that accepts ONLY because the
# parser is too permissive must not be silently promoted into a measurement.
#
# Usage:  bash <this script> <path-to-parseability_probe> [levels]
set -uo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../../../.." && pwd)"; cd "$ROOT" || exit 1
probe="${1:?usage: validate_shapes.sh <path-to-parseability_probe> [levels]}"
levels="${2:-4}"
gen="docs/tasks/artifacts/engine_universal_services/eus69a_overshoot_census/gen_shapes.py"
rc_all=0
for s in $(python3 "$gen" --list); do
  f="$(python3 "$gen" "$s" "$levels" --out-dir tmp/eus69a/shapes)"
  o="$("$probe" --parse systemverilog "$f" --profile sv_2017 2>&1)"
  if printf '%s' "$o" | grep -q 'parse_full passed'; then
    printf '%-12s ACCEPT\n' "$s"
  else
    printf '%-12s REJECT  %s\n' "$s" "$(printf '%s' "$o" | grep -oE 'Parser did not consume[^]]*|rejected sample[^]]*' | head -1 | cut -c1-80)"
    rc_all=1
  fi
done
exit "$rc_all"

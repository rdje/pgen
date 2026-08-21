#!/usr/bin/env bash
cd "$(git -C "$(dirname "${BASH_SOURCE[0]}")" rev-parse --show-toplevel)"
rc_all=0
stage() {
  local name="$1"; shift
  echo "=== STAGE-START $name ==="
  if "$@" > "docs/tasks/artifacts/engine_universal_services/deep_nesting_cliff/logs/stage_${name}.log" 2>&1; then
    echo "=== STAGE-PASS $name ==="
  else
    echo "=== STAGE-FAIL $name (rc=$?) ==="; rc_all=1
  fi
}
# `scratch` is the ONE artifact `regenerate_generated_parsers` does not cover (it is not in
# GENERATED_PARSER_FAMILIES), so a codegen change leaves it stale and GENERATED-REPRODUCIBILITY
# correctly refuses to rebaseline over it.
stage focus_scratch      make -C rust SHELL=/bin/bash focus_scratch
stage repro_rebaseline2  make -C rust SHELL=/bin/bash generated_reproducibility_rebaseline
stage parse_cost_ratchet2 make -C rust SHELL=/bin/bash sv_parse_cost_ratchet
echo "=== REBASELINE2-DONE rc=$rc_all ==="
exit $rc_all

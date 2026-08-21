#!/usr/bin/env bash
# ENGINE-UNIVERSAL-SERVICES.43 — the two doctrine re-derivations any codegen change owes.
# ⛔ These RE-MEASURE. Never edit the numbers they hold.
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
stage parse_cost_ratchet  make -C rust SHELL=/bin/bash sv_parse_cost_ratchet
stage parse_cost_share    make -C rust SHELL=/bin/bash sv_parse_cost_family_share
stage repro_rebaseline    make -C rust SHELL=/bin/bash generated_reproducibility_rebaseline
echo "=== REBASELINE-DONE rc=$rc_all ==="
exit $rc_all

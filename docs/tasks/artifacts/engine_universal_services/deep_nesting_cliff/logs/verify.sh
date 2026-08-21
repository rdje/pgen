#!/usr/bin/env bash
# ENGINE-UNIVERSAL-SERVICES.43 — the verification suite, run sequentially (one cargo lock at a time).
# Each stage prints a STAGE line so a monitor can follow it; nothing is allowed to fail silently.
cd "$(git -C "$(dirname "${BASH_SOURCE[0]}")" rev-parse --show-toplevel)"
rc_all=0
stage() {
  local name="$1"; shift
  echo "=== STAGE-START $name ==="
  if "$@" > "docs/tasks/artifacts/engine_universal_services/deep_nesting_cliff/logs/stage_${name}.log" 2>&1; then
    echo "=== STAGE-PASS $name ==="
  else
    echo "=== STAGE-FAIL $name (rc=$?) ==="
    rc_all=1
  fi
}
stage embedding_tests   bash -c 'cd rust && cargo test --features generated_parsers --lib embedding_api -- --test-threads 1'
stage ast_shape         make -C rust SHELL=/bin/bash ast_shape_contract_gate
stage harness_equiv     make -C rust SHELL=/bin/bash parse_harness_equivalence_gate
stage harness_combinator make -C rust SHELL=/bin/bash parse_harness_combinator_gate
stage sv_corpus_triage  make -C rust SHELL=/bin/bash sv_external_corpus_triage_gate
stage generated_clippy  make -C rust SHELL=/bin/bash generated_clippy_correctness_gate
echo "=== VERIFY-DONE rc=$rc_all ==="
exit $rc_all

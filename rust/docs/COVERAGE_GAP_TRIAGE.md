# Coverage Gap Triage

`coverage_gap_triage` is a local debugging helper for the remaining parser-family closure debt.

It joins three existing artifact surfaces:
- gap-report JSON
- coverage JSON
- grammar raw-AST JSON

This is useful when a family is stuck on a small number of replay targets and the raw gap report no longer gives enough context to choose a safe grammar or generator fix.

## Typical Usage

Verified example:

```bash
cargo run --bin coverage_gap_triage -- \
  --gap-report rust/target/vhdl_stimuli_quality_gate/work/closed_loop_replay_gap.json \
  --coverage rust/target/vhdl_stimuli_quality_gate/work/closed_loop_initial_coverage.json \
  --grammar-ast rust/target/vhdl_stimuli_quality_gate/work/vhdl.json
```

## What It Adds

For each reachable branch-debt item, the tool reports:
- exact branch id, rule, node path, and branch index
- selected/success/deficit counts
- rendered branch text from the grammar AST
- sibling branch counts and renderings
- rule-reference success counts
- a lightweight heuristic such as:
  - `selection_bias_likely`
  - `branch_specific_failure_likely`
  - `shared_dependency_failure_likely`
  - `dependency_rule_debt_likely`

The tool is intentionally read-only. It does not mutate coverage state or attempt target driving by itself.

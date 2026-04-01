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

## Important Limitation

When using `ast_pipeline --generate-stimuli --entry-rule <subrule>`, generated parseability validation is only trustworthy for the grammar's full entry rule today.

- `--entry-rule` affects generation and gap-report reachability as expected.
- `--validate-parseability` still validates samples through the generated parser's full grammar entry, not an arbitrary subrule entry.
- Because of that, subrule triage should use coverage/gap artifacts without `--validate-parseability` unless and until explicit entry-rule parseability support is added.

The CLI now rejects `--validate-parseability` when paired with a non-default entry rule so this limitation fails loudly instead of creating misleading counterexamples.

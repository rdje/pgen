# Generated `try_parse` recursion-name rollback audit

## Source contract

Generated `try_parse` snapshots the recursion stack beside the semantic and
coverage checkpoints. On error it reads the last name frame for
`RollbackLabel::TryParseErr`, then calls the shared paired-stack truncate. The
current emitter lives in `rust/src/ast_pipeline/ast_based_generator.rs` around
the generated speculative-parse rollback path.

The compatible `PGEN-RGX-0078-0188` representation keeps the complete
`rule_id_stack` on generated bare paths and reconstructs a diagnostic name from
the generated parser's static `RULE_NAMES` table when an error path needs one.
That implies three explicit replacement obligations:

1. the saved name-stack length becomes a saved ID-stack length one-for-one;
2. the retained ID stack is still truncated to that saved length; and
3. last-rule capture maps the retained `RuleId` through `RULE_NAMES`.

The classifier credits none of those obligations as removed work. In
particular, it excludes the replacement name lookup entirely because the
preserved binary does not contain that future operation and therefore cannot
price it.

## Exact removable subset

The preserved fused symbols contain 39 generated `try_parse` rollback sites:
12 in `piece` and 27 in the sampled `atom` closure. At every site, machine
dataflow proves the same paired invariant:

- a name-stack current-length load feeds a compare with the saved depth;
- its conditional branch guards one name-stack length store; and
- the same saved depth then feeds the required ID-stack truncate store.

Only the first three PCs—the name compare, conditional branch, and name length
store—are net-removable under the ID-only design. The snapshot load is not
credited because the replacement needs the corresponding ID depth. The ID
truncate and all semantic/coverage rollback remain required. This yields
exactly **39 × 3 = 117 PCs** (39 memory, 78 control).

The subset is disjoint from accepted G1-C, successful thin-memo work, the
`PGEN-RGX-0078-0185` diagnostic union, the `-0186` thin lookup, the `-0187`
telemetry union, and the `-0188` ordinary guard push/pop subset. Public legacy
guard methods, recursion verdicts, rollback labels, error text, and trace text
remain unchanged.

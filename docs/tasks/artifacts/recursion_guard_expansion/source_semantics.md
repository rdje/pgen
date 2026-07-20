# Recursion-guard representation and reader audit

## Which guard is in scope

The shared generated-parser guard is `ast_pipeline::RecursionGuard` in
`rust/src/ast_pipeline/mod.rs:1321-1435`. The similarly named private guard in
`mutual_recursion_handler.rs` and the standalone bootstrap structure emitted by
`ast_code_generator.rs` are different implementations and are out of scope.

C1 (`PGEN-RGX-0078-0118`) added a parallel `rule_id_stack` beside the existing
`parse_stack`:

- `parse_stack`: 24-byte `(&'static str, position)` frames;
- `rule_id_stack`: 16-byte `(RuleId, position)` frames.

Modern generated methods call `enter_id`, so every modern frame already has a
complete rule ID and position. `check_cycle_id` scans that dense ID stack; the
name stack is not the cycle key. Legacy `enter` writes `RuleId::MAX` placeholders
and must retain its existing paired-stack behavior.

## Complete name-stack readers

The repository reader audit finds four semantic consumers of the shared
`parse_stack`:

1. legacy `check_cycle`, which must continue to scan names;
2. `check_cycle_id`'s maximum-depth arm, which builds the public
   `CycleType::MutualRecursive.rules` payload from names;
3. generated `try_parse`, which snapshots the stack length, captures the last
   rule name for `RollbackLabel::TryParseErr`, and truncates both stacks; and
4. generated error/trace paths: the predicate-failure stack log and
   `create_contextual_error`'s complete rule-name vector.

The generated recursion-error match arms use the depth but not the
`MutualRecursive.rules` values. That does not permit changing the public
`check_cycle_id` contract: the method and `CycleType` are public. Likewise,
`parse_stack` and `rule_id_stack` are public fields. Existing `enter_id`,
`exit`, `truncate_stack`, and `check_cycle_id` therefore remain compatibility
surfaces rather than candidates for silent semantic change.

## Concrete additive candidate

The compatible implementation shape is an additive generated/bare API, not a
mutation of the public legacy methods:

- an ID-only enter/exit/truncate family maintains `rule_id_stack` for fused
  bare frames;
- an ID-only cycle check retains the same Infinite/LeftRecursive/depth verdict
  without constructing an unused name payload on the ordinary path;
- generated `try_parse`, contextual errors, and trace diagnostics map retained
  IDs through that parser's static `RULE_NAMES` table when names are actually
  needed; and
- protocol methods and legacy callers keep the existing paired name+ID path.

This preserves recursion protection and all user-visible rule names while
removing duplicate name storage only where the generated parser itself owns a
complete ID-to-name bijection. During a fused internal call, the ID stack can
be deeper than the legacy name stack; the boundary restores the prior depth
before control returns to a protocol frame.

## What this leaf prices

The machine classifier admits only ordinary duplicate name-stack pushes and
pops:

- four inlined paired-push sites in the captured fused symbols;
- the name half of the outlined `RecursionGuard::enter_id`; and
- nine exact inlined paired-pop sites.

It deliberately excludes cycle scans, maximum-depth checks, try-parse
snapshot/truncate and last-rule lookup, contextual-error/trace readers, ABI
prologue/epilogue, and time inside `RawVec::grow_one`/allocator children. The
three interleaved ID-base/capacity carriers at inlined push sites are retained
as required replacement work. Thus the price is a conservative fixed subset,
not the whole name-stack ceiling.

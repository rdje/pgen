# PGEN Semantic Steering Control Matrix (Living)

Last updated: 2026-02-19

## Intent
Semantic annotations are a language-level superset. The Rust AST pipeline is an implementation-level subset at any given time.

This document defines:
1. What parser/stimuli steering controls we need.
2. Which controls are currently supported.
3. Which controls are parse-only/validated-only vs truly steering.
4. The non-negotiable return-annotation completeness contract.

## Built-In vs Annotation Balance (Decision)
Adopt a layered control model:

1. Layer A: built-in invariants (hardcoded, minimal, mandatory)
- Parser/stimuli correctness and determinism guards.
- Safety limits and bounded runtime behavior.
- Stable diagnostics/error-code contracts.
- Return-annotation full feature support (no compromise).

2. Layer B: annotation policy controls (user-authored in EBNF)
- Parser/stimuli steering directives that vary by language/domain/project.
- Defaults exist, but directives are the preferred way to control behavior.

3. Layer C: optional extension hooks (future)
- Project-specific behavior integration without polluting core invariants.

Precedence rule:
- Built-in correctness/safety contracts override everything.
- Then semantic directive policy (when supported/typed).
- Then fallback defaults.

Hard boundary:
- Do not hardcode domain semantics that can be expressed as typed semantic directives.
- Do hardcode only the minimum invariant semantic behavior required for correctness/safety.

## Capability Tiers
- `Tier 0` Parsed only (stored as AST/raw, no validation contract).
- `Tier 1` Parsed + validated (diagnostics), no runtime steering.
- `Tier 2` Parser steering enabled.
- `Tier 3` Parser + stimuli steering enabled.
- `Tier 4` Gate-enforced contract (tests + CI requirements).

## Semantic Steering Controls

| ID | Control | Parser Steering Need | Stimuli Steering Need | Candidate Semantic Construct(s) | Current Status (2026-02-19) | Target Tier | Priority |
|---|---|---|---|---|---|---|---|
| `SC-01` | Canonical terminal transform/coercion | Transform matched terminal into typed parse content | Bias sample shape by target type | `@transform: str::parse::<T>().unwrap_or(default)` | Implemented (canonical parser shared across validator/codegen/stimuli) | Tier 4 | P0 |
| `SC-02` | Raw literal sample hint | None | Allow explicit literal sample override | Raw semantic payload (`"literal"`) | Implemented for stimuli only | Tier 3 | P1 |
| `SC-03` | Name-based directive routing | Select behavior by semantic annotation name | Same | `@sample`, `@weight`, `@recover`, `@token`, `@constraint` | Foundation implemented: typed name routing + unknown-directive policy + transform/literal routing guards; broader directive steering still pending | Tier 3 | P0 |
| `SC-04` | Token-class steering | Choose tokenizer/matcher strategy per atom/rule | Generate samples by token class family | `@token_class`, `@charset`, `@pattern` | Not implemented | Tier 3 | P1 |
| `SC-05` | Precedence/associativity steering | Resolve ambiguity/branching deterministically | Generate precedence-respecting trees | `@precedence`, `@associativity`, `@priority` | Baseline implemented: parser tie-break uses priority+associativity; stimuli branch sampling uses semantic priority/precedence + associativity multipliers | Tier 3 | P0 |
| `SC-06` | Branch weighting and selection policy | Prefer deterministic branch policy where grammar is ambiguous | Coverage-guided weighted generation | `@weight`, `@branch_policy` | External gap-priority exists; semantic-driven route not implemented | Tier 3 | P1 |
| `SC-07` | Error recovery and sync strategy | Production-grade parser recovery hints | Generate recovery-focused stimuli sets | `@recover`, `@sync`, `@panic_until` | Not implemented | Tier 2 | P1 |
| `SC-08` | Value-domain constraints | Enforce value contracts at parse/validation boundaries | Generate in-domain samples | `@range`, `@enum`, `@regex`, `@len` | Not implemented | Tier 3 | P0 |
| `SC-09` | Cross-field/cross-capture constraints | Validate relational constraints between captures | Generate constraint-satisfying combinations | `@constraint`, `@requires`, `@implies` | Not implemented | Tier 3 | P1 |
| `SC-10` | Coverage target hints | Optional parser instrumentation priority tags | Rule/branch target boosting | `@coverage_target`, `@critical_path` | Coverage/gap pipeline exists; semantic hints not implemented | Tier 3 | P1 |
| `SC-11` | Negative case semantics | Emit expected-failure parser paths | Generate invalid/near-invalid stimuli | `@invalid_case`, `@negative` | Not implemented | Tier 3 | P2 |
| `SC-12` | Determinism partitioning hints | Stable behavior partitions for parser modes | Seed partitioning per semantic class | `@seed_group`, `@deterministic_group` | Not implemented | Tier 2 | P2 |

## Interpretation Rules
- Semantic annotations may be accepted before they are steering-capable.
- Non-steering acceptance must be explicit (`Tier 0`/`Tier 1`) and documented.
- Promotion rule for each control: `Tier 0/1 -> Tier 2/3 -> Tier 4` only with tests + gate integration.
- Return annotation behavior must not be weakened to compensate for semantic feature gaps.
- Unknown semantic directives should move toward explicit policy (`warn`/`strict`) rather than silent accept-and-ignore.

## Return Annotation No-Compromise Contract
Return annotation support is mandatory and must be complete in bootstrap + generated paths.

Required construct coverage:
1. Positional references: `$n`.
2. Extraction: `$n::k`, `$n::first`, `$n::last`.
3. Spread: `$n*`, `$n::k*`, nested spread contexts.
4. Property access: `$n.field` (including chained where grammar allows).
5. Array access: `$n[idx]` (including nested/chained access).
6. Array literals with nested expressions/spreads.
7. Object literals with identifier/string keys and nested expressions.
8. Literals: string/number/boolean.
9. Parenthesized/nested expression forms.
10. Structured diagnostics for invalid return constructs.

Required quality bar:
1. No return-annotation construct regressions between bootstrap and generated parsers.
2. Return contract suites and advanced/stress suites must stay green.
3. Differential debt for return annotations should converge to zero before strict SOTA exit.
4. Any temporary mismatch must be explicitly tracked in differential baselines with closure plan.

## Next Implementation Focus (Recommended)
1. Implement `SC-08` value-domain constraints for both parser validation and stimuli generation.
2. Add deterministic directive conflict-resolution contract for multi-directive overlap.
3. Add typed validator checks for invalid `priority/precedence/associativity` payloads.
4. Add dedicated return-annotation closure work to drive differential return mismatches to zero.

## Priority Queue (Balance-Oriented)
- `P0` Keep built-in core minimal and invariant-only (correctness/safety/return completeness).
- `P0` Implement typed semantic directive registry + unknown-directive policy modes (`warn`/`strict`). (Completed 2026-02-19)
- `P0` Promote precedence/associativity and value-domain constraints to parser+stimuli steering. (Precedence/associativity completed 2026-02-19; value-domain pending)
- `P1` Add semantic directive conflict-resolution contract and deterministic precedence rules.
- `P1` Drive return differential mismatch debt to zero and tighten release gate criteria.

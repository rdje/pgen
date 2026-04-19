# PGEN Semantic Steering Control Matrix (Living)

Last updated: 2026-03-26

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

## Rust AST Pipeline Supported Semantic Directives
Authoritative implementation source:
- `rust/src/ast_pipeline/semantic_directive_registry.rs`

The Rust AST pipeline currently registers these semantic annotations explicitly.

Parsed and validated:
- `@type`
- `@category`
- `@effect`
- `@deprecated`
- `@emit_fact`
- `@open_scope`
- `@close_scope`
- `@predicate`

Stimuli steering:
- `@sample`
- `@weight`
- `@literal`
- `@example`

Parser steering:
- `@recover_budget`
- `@recover_parse_budget`
- `@recover_global_budget`

Parser and stimuli steering:
- `@transform`
- `@precedence`
- `@associativity`
- `@priority`
- `@branch_policy`
- `@recover`
- `@sync`
- `@panic_until`
- `@range`
- `@enum`
- `@regex`
- `@len`
- `@constraint`
- `@requires`
- `@implies`
- `@token_class`
- `@charset`
- `@pattern`
- `@coverage_target`
- `@critical_path`
- `@invalid_case`
- `@negative`
- `@seed_group`
- `@deterministic_group`
- `@profiles`

Parsed only:
- none are intentionally registry-listed as parsed-only today

Implementation notes for the capability taxonomy above:
- The registry is the authoritative capability table, but some directives have visible leverage beyond the simple bucket labels.
- `@profiles` is registry-classified as parser-and-stimuli steering, while the currently explicit surfaced leverage is parser-side rule/profile gating in `rust/src/ast_pipeline/ast_based_generator.rs`; a distinct stimuli-side leverage path is not separately surfaced in `rust/src/ast_pipeline/stimuli_generator.rs`.
- `@emit_fact`, `@open_scope`, `@close_scope`, and `@predicate` are registry-classified as parsed-and-validated, but they already drive generated-parser semantic runtime behavior through `rust/src/ast_pipeline/semantic_runtime.rs`.
- `@stimulus` remains a legacy named-routing alias in stimuli generation; it is intentionally not a registry-listed typed directive.

## Capability Tiers
- `Tier 0` Parsed only (stored as AST/raw, no validation contract).
- `Tier 1` Parsed + validated (diagnostics), no runtime steering.
- `Tier 2` Parser steering enabled.
- `Tier 3` Parser + stimuli steering enabled.
- `Tier 4` Gate-enforced contract (tests + CI requirements).

## Semantic Steering Controls

| ID | Control | Parser Steering Need | Stimuli Steering Need | Candidate Semantic Construct(s) | Current Status (2026-03-26) | Target Tier | Priority |
|---|---|---|---|---|---|---|---|
| `SC-01` | Canonical terminal transform/coercion | Transform matched terminal into typed parse content | Bias sample shape by target type | `@transform: str::parse::<T>().unwrap_or(default)` | Implemented Tier 4 contract: shared canonical-transform parser utility + validator canonical acceptance/strict noncanonical rejection + parser codegen transformed-terminal emission + stimuli canonical target-type hints/noncanonical guard + dedicated SC-01 contract slice (`semantic_annotation_sc01_contract`) that tracks the current bootstrap-pass/generated-expected-fail parseability boundary for named canonical transforms; comparable-only differential taxonomy remains mismatch-clean and currently empty | Tier 4 | P0 |
| `SC-02` | Raw literal sample hint | None | Allow explicit literal sample override | Bare or literalish-named string payloads (`"literal"`, `@sample`, `@literal`, `@example`, legacy `@stimulus`) | Implemented Tier 4 contract: focused stimuli contracts cover the bare-string baseline plus named literalish directives across raw/structured string payloads, legacy `@stimulus`, non-literalish guards, non-regex rule overrides, and branch-local OR overrides with preserved branch accounting; dedicated SC-02 shared slice (`semantic_annotation_sc02_contract`) keeps generated-comparable named string forms parity-clean with differential taxonomy checks wired into `annotation_contract_gate`/CI | Tier 4 | P1 |
| `SC-03` | Name-based directive routing | Select behavior by semantic annotation name | Same | `@sample`, `@weight`, `@recover`, `@token_class`, `@constraint` | Gate-hardened routing baseline: typed directive registry with capability taxonomy checks + unknown-directive warn/strict policy contracts + strict-warning policy selectors + parser/stimuli transform/literal named-routing guards across regex, non-regex, and branch-local use sites + dedicated SC-03 contract slice (`semantic_annotation_sc03_contract`) with bootstrap/generated parity + differential taxonomy checks wired into `annotation_contract_gate`/CI; broader directive steering remains tracked under per-control SC items | Tier 4 | P0 |
| `SC-04` | Token-class steering | Choose matcher strategy per regex atom/rule | Generate samples by token family/class | `@token_class`, `@charset`, `@pattern` | Implemented Tier 4 contract: typed payload validators + deterministic precedence (`@pattern > @charset > @token_class`) + parser regex matcher override + stimuli regex sampling override + grammar-aware inactive-steering warning when no regex atom is present + dedicated SC-04 contract slice (`semantic_annotation_sc04_contract`) with bootstrap/generated parity + differential taxonomy checks wired into `annotation_contract_gate`/CI | Tier 4 | P1 |
| `SC-05` | Precedence/associativity steering | Resolve ambiguity/branching deterministically | Generate precedence-respecting trees | `@precedence`, `@associativity`, `@priority` | Implemented Tier 4 contract: typed payload validators (`@priority/@precedence/@associativity`) + deterministic conflict/duplicate contracts (`priority > precedence`, last-wins duplicates) + parser OR tie-break steering contracts (priority + associativity extraction/runtime emission) + stimuli branch steering contracts (priority bias, priority-over-precedence, associativity tie bias) + dedicated SC-05 contract slice (`semantic_annotation_sc05_contract`) with bootstrap/generated parity + differential taxonomy checks wired into `annotation_contract_gate`/CI | Tier 4 | P0 |
| `SC-06` | Branch weighting and selection policy | Prefer deterministic branch policy where grammar is ambiguous | Coverage-guided weighted generation | `@weight`, `@branch_policy` | Implemented Tier 4 contract: typed branch-policy payload validators + parser/stimuli branch-policy steering (`longest_match`, `ordered`, `priority_first`) + deterministic weighted-probability stimuli contracts + dedicated SC-06 contract slice (`semantic_annotation_sc06_contract`) with bootstrap/generated parity + differential taxonomy checks wired into `annotation_contract_gate`/CI | Tier 4 | P1 |
| `SC-07` | Error recovery and sync strategy | Production-grade parser recovery hints | Generate recovery-focused stimuli sets | `@recover`, `@recover_budget`, `@recover_parse_budget`, `@recover_global_budget`, `@sync`, `@panic_until` | Implemented Tier 4 contract: typed validator payload/coherence diagnostics + parser OR-failure recovery hooks + scoped budget enforcement (`rule`/`parse`/`global`) + structured recovery event reporting APIs (`recovery_events`, `take_recovery_events`, `recovery_event_count`, `recovery_parse_count`, `recovery_global_count`) + stimuli OR-failure fallback marker emission (`panic_until` first, then `sync`) + dedicated modes (`recovery_biased`, `near_sync_negative`) + dedicated SC-07 contract slice (`semantic_annotation_sc07_contract`) with bootstrap/generated parity + differential taxonomy checks wired into `annotation_contract_gate`/CI | Tier 4 | P1 |
| `SC-08` | Value-domain constraints | Enforce value contracts at parse/validation boundaries | Generate in-domain samples | `@range`, `@enum`, `@regex`, `@len` | Implemented Tier 4 contract: typed payload validators (`@range/@enum/@regex/@len`) + unsatisfiable intersection diagnostics (`W_SEM_UNSATISFIABLE_VALUE_DOMAIN`) + parser value-guard runtime contracts + stimuli in-domain synthesis contracts (enum/range/len/regex composition) + dedicated SC-08 contract slice (`semantic_annotation_sc08_contract`) with bootstrap/generated parity + differential taxonomy checks wired into `annotation_contract_gate`/CI | Tier 4 | P0 |
| `SC-09` | Cross-field/cross-capture constraints | Validate relational constraints between captures | Generate constraint-satisfying combinations | `@constraint`, `@requires`, `@implies` | Implemented Tier 4 contract: typed payload contracts + relational coherence warning + parser runtime relational enforcement + stimuli relational sequence synthesis retries (`@requires` reference checks, `@constraint` expression evaluation, `@implies` antecedent/consequent gating), including nested named/positional path reference resolution over structured JSON-like captures and non-structured object-like captures (`=/:` pairs, `,`/`;` delimiters, wrapper-aware parsing), plus ranked unsatisfiable-contract diagnostics on retry exhaustion (`relational_failures`, `generation_failures`, `top_violations`, `likely_unsatisfiable`) + dedicated SC-09 contract slice (`semantic_annotation_sc09_contract`) with bootstrap/generated parity + differential taxonomy checks wired into `annotation_contract_gate`/CI | Tier 4 | P1 |
| `SC-10` | Coverage target hints | Optional parser instrumentation priority tags | Rule/branch target boosting | `@coverage_target`, `@critical_path` | Implemented Tier 4 contract: typed payload + coherence validator contracts (`W_SEM_INVALID_COVERAGE_TARGET_PAYLOAD`, `W_SEM_INVALID_CRITICAL_PATH_PAYLOAD`, `W_SEM_CRITICAL_PATH_WITHOUT_COVERAGE_TARGET`) + stimuli coverage steering integration (semantic multipliers in branch guidance + semantic priority bonuses in gap report/target ordering) + parser runtime instrumentation hooks (`CoverageTargetEvent`, selected-branch tracking, rule/branch hit counters + accessors) + dedicated SC-10 contract slice (`semantic_annotation_sc10_contract`) with bootstrap/generated parity + differential taxonomy checks wired into `annotation_contract_gate`/CI | Tier 4 | P1 |
| `SC-11` | Negative case semantics | Emit expected-failure parser paths | Generate invalid/near-invalid stimuli | `@invalid_case`, `@negative` | Implemented Tier 4 contract: typed validator payload/coherence diagnostics (`W_SEM_INVALID_INVALID_CASE_PAYLOAD`, `W_SEM_INVALID_NEGATIVE_PAYLOAD`, `W_SEM_NEGATIVE_WITHOUT_INVALID_CASE`) + generated-parser expected-failure event surface (`NegativeCaseEvent`, per-rule counters/accessors) + stimuli invalid/near-invalid mutation baseline with deterministic negative marker routing + dedicated SC-11 contract slice (`semantic_annotation_sc11_contract`) with bootstrap/generated parity + differential taxonomy checks wired into `annotation_contract_gate`/CI | Tier 4 | P2 |
| `SC-12` | Determinism partitioning hints | Stable behavior partitions for parser modes | Seed partitioning per semantic class | `@seed_group`, `@deterministic_group` | Implemented Tier 4 contract: typed payload/coherence validator contracts (`W_SEM_INVALID_SEED_GROUP_PAYLOAD`, `W_SEM_INVALID_DETERMINISTIC_GROUP_PAYLOAD`, `W_SEM_SEED_GROUP_WITHOUT_DETERMINISTIC_GROUP`) + deterministic parser OR-branch partition steering (group-key-based evaluation offset for ordered-choice behavior) + parser partition telemetry (`DeterministicPartitionEvent`, per-rule counters/accessors) + explicit parser runtime override modes for embedders (`AnnotationDriven`, `ForceEnabled`, `ForceDisabled`) + deterministic stimuli entry seed partition routing with interleaving-order independence + dedicated SC-12 contract slice (`semantic_annotation_sc12_contract`) with bootstrap/generated parity + differential taxonomy checks wired into `annotation_contract_gate`/CI | Tier 4 | P2 |
| `SC-13` | Profile gating and semantic runtime scaffold | Gate rules by active grammar profile and expose typed scope/fact/predicate runtime scaffolding | None | `@profiles`, `@emit_fact`, `@open_scope`, `@close_scope`, `@predicate` | Implemented Tier 4 contract: typed validator payload contracts for `@profiles` and runtime directives + generated-parser profile-guard emission + semantic runtime parsing/state/predicate contracts + dedicated SC-13 contract slice (`semantic_annotation_sc13_contract`) with bootstrap/generated parity + differential taxonomy checks wired into `annotation_contract_gate`/CI | Tier 4 | P1 |

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
1. Keep Tier-4 SC contract slices parity-clean (bootstrap/generated comparable corpus mismatches must stay zero for each SC gate).
2. Expand per-directive capability taxonomy assertions so registry metadata cannot drift from runtime behavior.

## Priority Queue (Balance-Oriented)
- `P0` Keep built-in core minimal and invariant-only (correctness/safety/return completeness).
- `P0` Promote `SC-01` to Tier-4 gate-enforced contract by adding dedicated SC-01 contract slices + canonical-transform utility/codegen/stimuli parity checks in `annotation_contract_gate`. (Completed 2026-03-26)
- `P0` Implement typed semantic directive registry + unknown-directive policy modes (`warn`/`strict`). (Completed 2026-02-19)
- `P0` Harden SC-03 with gate-enforced contract slices and strict-policy coverage (`sc03_contract_gate` wired into `annotation_contract_gate`). (Completed 2026-02-20)
- `P0` Promote precedence/associativity and value-domain constraints to parser+stimuli steering. (Completed 2026-02-19)
- `P0` Promote `SC-05` to Tier-4 gate-enforced contract by adding dedicated SC-05 contract slices + precedence/associativity runtime contracts + differential mismatch taxonomy parity checks in `annotation_contract_gate`. (Completed 2026-02-20)
- `P0` Promote `SC-08` to Tier-4 gate-enforced contract by adding dedicated SC-08 contract slices + value-domain runtime contracts + differential mismatch taxonomy parity checks in `annotation_contract_gate`. (Completed 2026-02-20)
- `P1` Expand conflict diagnostics from directive precedence to unsatisfiable cross-directive constraint intersections. (Completed 2026-02-19)
- `P1` Promote `SC-02` to Tier-4 gate-enforced contract by adding dedicated SC-02 contract slices + literal-sample runtime contracts + differential mismatch taxonomy parity checks in `annotation_contract_gate`. (Completed 2026-03-26)
- `P1` Start `SC-09` typed cross-field/cross-capture validator contract (`@constraint/@requires/@implies`) with stable diagnostics. (Completed 2026-02-20)
- `P1` Start `SC-04` token-class steering (`@token_class/@charset/@pattern`) with typed validator contracts and parser/stimuli runtime precedence baseline. (Completed 2026-02-20)
- `P1` Promote `SC-04` to Tier-4 gate-enforced contract by adding dedicated SC-04 contract slices + differential mismatch taxonomy parity checks in `annotation_contract_gate`. (Completed 2026-02-20)
- `P1` Promote `SC-06` to Tier-4 gate-enforced contract by adding dedicated SC-06 contract slices + branch-policy/weighting runtime contracts + differential mismatch taxonomy parity checks in `annotation_contract_gate`. (Completed 2026-02-20)
- `P1` Promote `SC-07` to Tier-4 gate-enforced contract by adding dedicated SC-07 contract slices + recovery/sync runtime contracts + differential mismatch taxonomy parity checks in `annotation_contract_gate`. (Completed 2026-02-20)
- `P1` Promote `SC-09` to Tier-4 gate-enforced contract by adding dedicated SC-09 contract slices + relational runtime contracts + differential mismatch taxonomy parity checks in `annotation_contract_gate`. (Completed 2026-02-20)
- `P1` Promote `SC-09` to parser runtime relational enforcement (`@requires/@constraint/@implies`). (Completed 2026-02-20)
- `P1` Promote `SC-09` to stimuli runtime relational synthesis steering baseline. (Completed 2026-02-20)
- `P1` Start `SC-10` typed coverage-target semantic hinting (`@coverage_target/@critical_path`) with validator contracts and stimuli coverage/gap steering integration. (Completed 2026-02-20)
- `P1` Extend `SC-10` to parser runtime instrumentation hooks (`CoverageTargetEvent`, selected-branch tagging, rule/branch hit counters + accessors). (Completed 2026-02-20)
- `P1` Promote `SC-10` to Tier-4 gate-enforced contract by adding dedicated SC-10 contract slices + coverage-target runtime contracts + differential mismatch taxonomy parity checks in `annotation_contract_gate`. (Completed 2026-02-20)
- `P1` Promote selected semantic warnings to strict-mode errors under explicit policy controls (`PGEN_STRICT_SEMANTIC_WARNING_CODES`) with strict-default SC-10 invalid payload escalations. (Completed 2026-02-20)
- `P2` Start `SC-11` negative-case semantic steering (`@invalid_case/@negative`) with typed validator contracts and parser/stimuli baseline behavior. (Completed 2026-02-20)
- `P2` Promote `SC-11` to Tier-4 gate-enforced contract by adding dedicated SC-11 contract slices + negative-case runtime contracts + differential mismatch taxonomy parity checks in `annotation_contract_gate`. (Completed 2026-02-20)
- `P2` Start `SC-12` determinism partitioning hints (`@seed_group/@deterministic_group`) with typed validator contracts and deterministic stimuli seed partition routing baseline. (Completed 2026-02-20)
- `P2` Promote SC-12 to parser-side deterministic partition steering baseline (`ordered` OR-branch evaluation offset + typed partition telemetry events/counters). (Completed 2026-02-20)
- `P2` Harden SC-12 embedder controls by adding parser runtime partition mode overrides (`AnnotationDriven`/`ForceEnabled`/`ForceDisabled`) and runtime-effective branch-order/event resolution. (Completed 2026-02-20)
- `P2` Promote `SC-12` to Tier-4 gate-enforced contract by adding dedicated SC-12 contract slices + deterministic partition runtime contracts + differential mismatch taxonomy parity checks in `annotation_contract_gate`. (Completed 2026-02-20)
- `P1` Promote `SC-13` to Tier-4 gate-enforced contract by adding dedicated SC-13 contract slices + profile/runtime-scaffold validator and runtime contracts + differential mismatch taxonomy parity checks in `annotation_contract_gate`. (Completed 2026-03-26)
- `P1` Drive return differential mismatch debt to zero and tighten release gate criteria.

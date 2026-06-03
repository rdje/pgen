# Knowledge Map

> **AUTO-GENERATED — DO NOT EDIT.** Regenerate with `knowledge-map/scripts/gen_knowledge_map.sh`.
> Source of truth = YAML front-matter in: `docs/knowledge docs/decisions`. Edit the fact files, never this map.
> A fact is any `.md` whose front-matter has a non-empty `answers:` list.
> **4** facts · **20** question keys.

## Questions → fact

- "are warnings and errors hidden at low verbosity" -> [stimuli-generation-error-reasons](docs/knowledge/stimuli-generation-error-reasons.md) · 2026-06-03 · reverify: `grep -n "GenerationErrorReason\|classify_generation_error" rust/src/ast_pipeline/stimuli_generator.rs`
- "how do I tell a depth failure from a visit-limit or timeout failure" -> [stimuli-generation-error-reasons](docs/knowledge/stimuli-generation-error-reasons.md) · 2026-06-03 · reverify: `grep -n "GenerationErrorReason\|classify_generation_error" rust/src/ast_pipeline/stimuli_generator.rs`
- "how does the generator steer to a specific coverage target" -> [stimuli-residual-coverage-model](docs/knowledge/stimuli-residual-coverage-model.md) · 2026-06-03 · reverify: `grep -n "StimuliCoverageTarget\|replay_target\|reach_classification" rust/src/ast_pipeline/stimuli_generator.rs`
- "what are never_hit / never_selected / selected_but_failed reasons" -> [stimuli-residual-coverage-model](docs/knowledge/stimuli-residual-coverage-model.md) · 2026-06-03 · reverify: `grep -n "StimuliCoverageTarget\|replay_target\|reach_classification" rust/src/ast_pipeline/stimuli_generator.rs`
- "what do the stimuli generation error reasons mean" -> [stimuli-generation-error-reasons](docs/knowledge/stimuli-generation-error-reasons.md) · 2026-06-03 · reverify: `grep -n "GenerationErrorReason\|classify_generation_error" rust/src/ast_pipeline/stimuli_generator.rs`
- "what does the Stimuli generation depth exceeded message mean" -> [sv-residual-depth-budget-cause](docs/knowledge/sv-residual-depth-budget-cause.md) · 2026-06-03 · reverify: `grep -n "max_depth\|depth exceeded\|DEPTH_EXCEEDED" rust/src/ast_pipeline/stimuli_generator.rs`
- "what does the stimuli coverage gap report measure" -> [stimuli-residual-coverage-model](docs/knowledge/stimuli-residual-coverage-model.md) · 2026-06-03 · reverify: `grep -n "StimuliCoverageTarget\|replay_target\|reach_classification" rust/src/ast_pipeline/stimuli_generator.rs`
- "what is GenerationErrorReason / classify_generation_error" -> [stimuli-generation-error-reasons](docs/knowledge/stimuli-generation-error-reasons.md) · 2026-06-03 · reverify: `grep -n "GenerationErrorReason\|classify_generation_error" rust/src/ast_pipeline/stimuli_generator.rs`
- "what is focused_replay_target_debt_zero / literal-0" -> [stimuli-residual-coverage-model](docs/knowledge/stimuli-residual-coverage-model.md) · 2026-06-03 · reverify: `grep -n "StimuliCoverageTarget\|replay_target\|reach_classification" rust/src/ast_pipeline/stimuli_generator.rs`
- "what is replay_target_count / the stimuli residual" -> [stimuli-residual-coverage-model](docs/knowledge/stimuli-residual-coverage-model.md) · 2026-06-03 · reverify: `grep -n "StimuliCoverageTarget\|replay_target\|reach_classification" rust/src/ast_pipeline/stimuli_generator.rs`
- "what is the root cause of the uncovered SV coverage branches" -> [sv-residual-depth-budget-cause](docs/knowledge/sv-residual-depth-budget-cause.md) · 2026-06-03 · reverify: `grep -n "max_depth\|depth exceeded\|DEPTH_EXCEEDED" rust/src/ast_pipeline/stimuli_generator.rs`
- "what is the two-surface architecture (runtime interpreter vs codegen)" -> [ast-two-surface-construction](docs/knowledge/ast-two-surface-construction.md) · 2026-06-03 · reverify: `grep -n "serde_json::Map::new\|Value::Object" rust/src/ast_pipeline/unified_return_ast.rs`
- "what must I edit to add a sibling key to every AST object" -> [ast-two-surface-construction](docs/knowledge/ast-two-surface-construction.md) · 2026-06-03 · reverify: `grep -n "serde_json::Map::new\|Value::Object" rust/src/ast_pipeline/unified_return_ast.rs`
- "where are depth_exceeded_errors / target_timeout_errors counted" -> [stimuli-generation-error-reasons](docs/knowledge/stimuli-generation-error-reasons.md) · 2026-06-03 · reverify: `grep -n "GenerationErrorReason\|classify_generation_error" rust/src/ast_pipeline/stimuli_generator.rs`
- "where are typed AST objects constructed in PGEN" -> [ast-two-surface-construction](docs/knowledge/ast-two-surface-construction.md) · 2026-06-03 · reverify: `grep -n "serde_json::Map::new\|Value::Object" rust/src/ast_pipeline/unified_return_ast.rs`
- "which files build the AST object map" -> [ast-two-surface-construction](docs/knowledge/ast-two-surface-construction.md) · 2026-06-03 · reverify: `grep -n "serde_json::Map::new\|Value::Object" rust/src/ast_pipeline/unified_return_ast.rs`
- "why can't deeply-nested SV rules be generated from the top entry" -> [sv-residual-depth-budget-cause](docs/knowledge/sv-residual-depth-budget-cause.md) · 2026-06-03 · reverify: `grep -n "max_depth\|depth exceeded\|DEPTH_EXCEEDED" rust/src/ast_pipeline/stimuli_generator.rs`
- "why does replay_target_count plateau (e.g. at 888)" -> [sv-residual-depth-budget-cause](docs/knowledge/sv-residual-depth-budget-cause.md) · 2026-06-03 · reverify: `grep -n "max_depth\|depth exceeded\|DEPTH_EXCEEDED" rust/src/ast_pipeline/stimuli_generator.rs`
- "why doesn't the SystemVerilog stimuli residual reach zero" -> [sv-residual-depth-budget-cause](docs/knowledge/sv-residual-depth-budget-cause.md) · 2026-06-03 · reverify: `grep -n "max_depth\|depth exceeded\|DEPTH_EXCEEDED" rust/src/ast_pipeline/stimuli_generator.rs`
- "why is the _meta carrier (A5) a coordinated multi-surface change" -> [ast-two-surface-construction](docs/knowledge/ast-two-surface-construction.md) · 2026-06-03 · reverify: `grep -n "serde_json::Map::new\|Value::Object" rust/src/ast_pipeline/unified_return_ast.rs`

## Facts (by id)

### ast-two-surface-construction
_PGEN builds typed AST objects on TWO surfaces that must change in lockstep_

- **answers:** where are typed AST objects constructed in PGEN | why is the _meta carrier (A5) a coordinated multi-surface change | what is the two-surface architecture (runtime interpreter vs codegen) | what must I edit to add a sibling key to every AST object | which files build the AST object map
- **date:** 2026-06-03 · **status:** current
- **evidence:** `rust/src/ast_pipeline/unified_return_ast.rs (object build ~:636-:706, serde_json::Map); rust/src/ast_pipeline/return_annotation_handler.rs (emits object-construction Rust into generated parsers, ~:355); docs/tasks/PARSE-SOTA-A5-meta-carrier-design.md`
- **reverify:** `grep -n "serde_json::Map::new\|Value::Object" rust/src/ast_pipeline/unified_return_ast.rs`
- **source:** [`docs/knowledge/ast-two-surface-construction.md`](docs/knowledge/ast-two-surface-construction.md)

### stimuli-generation-error-reasons
_Stimuli generation error-reason taxonomy (and severity is never gated by verbosity)_

- **answers:** what do the stimuli generation error reasons mean | what is GenerationErrorReason / classify_generation_error | how do I tell a depth failure from a visit-limit or timeout failure | where are depth_exceeded_errors / target_timeout_errors counted | are warnings and errors hidden at low verbosity
- **date:** 2026-06-03 · **status:** current
- **evidence:** `rust/src/ast_pipeline/stimuli_generator.rs (GenerationErrorReason, classify_generation_error, *_errors counters, TargetDriveSummary); rust/src/ast_pipeline/mod.rs (Severity, emit_diagnostic, pgen_warn!/error!/fatal!); docs/tasks/DIAG-SEVERITY.md`
- **reverify:** `grep -n "GenerationErrorReason\|classify_generation_error" rust/src/ast_pipeline/stimuli_generator.rs`
- **source:** [`docs/knowledge/stimuli-generation-error-reasons.md`](docs/knowledge/stimuli-generation-error-reasons.md)

### stimuli-residual-coverage-model
_What the stimuli residual / replay_target_count actually is (coverage model)_

- **answers:** what is replay_target_count / the stimuli residual | what does the stimuli coverage gap report measure | what are never_hit / never_selected / selected_but_failed reasons | what is focused_replay_target_debt_zero / literal-0 | how does the generator steer to a specific coverage target
- **date:** 2026-06-03 · **status:** current
- **evidence:** `rust/src/ast_pipeline/stimuli_generator.rs (StimuliCoverageTarget {Rule,Branch}, generate_gap_report, compute_reach_path, forced_or_branch_for_site, set_reach_plan); docs/tasks/SV-EXH-PROOF.md`
- **reverify:** `grep -n "StimuliCoverageTarget\|replay_target\|reach_classification" rust/src/ast_pipeline/stimuli_generator.rs`
- **source:** [`docs/knowledge/stimuli-residual-coverage-model.md`](docs/knowledge/stimuli-residual-coverage-model.md)

### sv-residual-depth-budget-cause
_Why the SV stimuli residual doesn't reach literal zero — depth-budget exhaustion_

- **answers:** why doesn't the SystemVerilog stimuli residual reach zero | why does replay_target_count plateau (e.g. at 888) | what is the root cause of the uncovered SV coverage branches | what does the Stimuli generation depth exceeded message mean | why can't deeply-nested SV rules be generated from the top entry
- **date:** 2026-06-03 · **status:** current
- **evidence:** `rust/src/ast_pipeline/stimuli_generator.rs (depth gate ~:4377, near-limit guard ~:4509/:5413, depth-slack retry ~:4869); docs/tasks/SV-EXH-PROOF.md leaf .7.4.3a (PGEN-SV-EXH-PROOF-0140)`
- **reverify:** `grep -n "max_depth\|depth exceeded\|DEPTH_EXCEEDED" rust/src/ast_pipeline/stimuli_generator.rs`
- **source:** [`docs/knowledge/sv-residual-depth-budget-cause.md`](docs/knowledge/sv-residual-depth-budget-cause.md)

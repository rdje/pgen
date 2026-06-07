# Knowledge Map

> **AUTO-GENERATED — DO NOT EDIT.** Regenerate with `knowledge-map/scripts/gen_knowledge_map.sh`.
> Source of truth = YAML front-matter in: `docs/knowledge docs/decisions`. Edit the fact files, never this map.
> A fact is any `.md` whose front-matter has a non-empty `answers:` list.
> **22** facts · **133** question keys.

## Questions → fact

- "are warnings and errors hidden at low verbosity" -> [stimuli-generation-error-reasons](docs/knowledge/stimuli-generation-error-reasons.md) · 2026-06-03 · reverify: `grep -n "GenerationErrorReason\|classify_generation_error" rust/src/ast_pipeline/stimuli_generator.rs`
- "can we trust the grammar linter 100%" -> [grammar-linter-trustworthiness](docs/knowledge/grammar-linter-trustworthiness.md) · 2026-06-06 · reverify: `grep -n 'verify_unreachability_certificate\\|UnreachabilityCertificate\\|EarlierArmAlwaysSucceeds' rust/src/ast_pipeline/grammar_wellformedness.rs`
- "do I need a bootstrap regen to change the EBNF grammar syntax" -> [ebnf-frontend-architecture](docs/knowledge/ebnf-frontend-architecture.md) · 2026-06-06 · reverify: ``grep -nE "fn parse_ebnf|fn tokenize_rule_expression|has_generated_ebnf_parser|generated_verify_required" rust/src/ebnf_frontend.rs`; `grep -n "fn transform_from_raw_ast" rust/src/ast_pipeline/mod.rs``
- "does PGEN handle left recursion automatically or must the author eliminate it" -> [pgen-parsing-model](docs/knowledge/pgen-parsing-model.md) · 2026-06-03 · reverify: `grep -n "eliminate_left_recursion\|eliminate_left_recursive_patterns" rust/src/ast_pipeline/mod.rs rust/src/main.rs`
- "does packrat memoization guarantee no catastrophic backtracking here" -> [stateful-packrat-not-linear](docs/knowledge/stateful-packrat-not-linear.md) · 2026-06-03 · reverify: `see docs/tasks/PARSE-TERMINATION.md; grep -n "memoiz" rust/src/ast_pipeline/*.rs`
- "does the semantic store affect parse-time complexity" -> [stateful-packrat-not-linear](docs/knowledge/stateful-packrat-not-linear.md) · 2026-06-03 · reverify: `see docs/tasks/PARSE-TERMINATION.md; grep -n "memoiz" rust/src/ast_pipeline/*.rs`
- "does the stimuli generator have all the necessary features" -> [stimuli-generator-capability-gaps](docs/knowledge/stimuli-generator-capability-gaps.md) · 2026-06-03 · reverify: `see docs/tasks/STIMULI-SIGNOFF.md leaves .2-.7`
- "how can the stimuli generator produce strings the parser rejects" -> [ebnf-single-source-of-truth](docs/knowledge/ebnf-single-source-of-truth.md) · 2026-06-07 · reverify: ``./rust/target/debug/parseability_probe --parse regex <(printf '\\\\u{b7a2}')`; `grep -n 'unsupported regex escape' rust/src/regex_compile_validation.rs`; `grep -c 'regex_compile_validation' rust/src/ast_pipeline/stimuli_generator.rs` (expect 0); `grep -nE 'unicode_escape|directive_verb' grammars/regex.ebnf``
- "how do I build a StimuliGenerator programmatically" -> [stimuli-generator-construction-path](docs/knowledge/stimuli-generator-construction-path.md) · 2026-06-06 · reverify: `grep -n 'StimuliGenerator::new\\|generate_gap_report\\|generate_target_witnesses' rust/src/main.rs`
- "how do I debug what the stimuli generator is deriving" -> [ebnf-single-source-of-truth](docs/knowledge/ebnf-single-source-of-truth.md) · 2026-06-07 · reverify: ``./rust/target/debug/parseability_probe --parse regex <(printf '\\\\u{b7a2}')`; `grep -n 'unsupported regex escape' rust/src/regex_compile_validation.rs`; `grep -c 'regex_compile_validation' rust/src/ast_pipeline/stimuli_generator.rs` (expect 0); `grep -nE 'unicode_escape|directive_verb' grammars/regex.ebnf``
- "how do I dump the grammar IR / gen_ast.json used by stimuli generation" -> [ast-pipeline-cli-reference](docs/knowledge/ast-pipeline-cli-reference.md) · 2026-06-04 · reverify: `run `rust/target/release/ast_pipeline --help` (flags can change); for .ebnf input the binary must be built `--features ebnf_dual_run``
- "how do I dump the parsed AST of a grammar rule" -> [ast-pipeline-cli-reference](docs/knowledge/ast-pipeline-cli-reference.md) · 2026-06-04 · reverify: `run `rust/target/release/ast_pipeline --help` (flags can change); for .ebnf input the binary must be built `--features ebnf_dual_run``
- "how do I extend the EBNF meta-grammar syntax (add a new annotation or construct)" -> [ebnf-frontend-architecture](docs/knowledge/ebnf-frontend-architecture.md) · 2026-06-06 · reverify: ``grep -nE "fn parse_ebnf|fn tokenize_rule_expression|has_generated_ebnf_parser|generated_verify_required" rust/src/ebnf_frontend.rs`; `grep -n "fn transform_from_raw_ast" rust/src/ast_pipeline/mod.rs``
- "how do I generate grammar stimuli from the command line" -> [ast-pipeline-cli-reference](docs/knowledge/ast-pipeline-cli-reference.md) · 2026-06-04 · reverify: `run `rust/target/release/ast_pipeline --help` (flags can change); for .ebnf input the binary must be built `--features ebnf_dual_run``
- "how do I lint a grammar for well-formedness from the CLI" -> [ast-pipeline-cli-reference](docs/knowledge/ast-pipeline-cli-reference.md) · 2026-06-04 · reverify: `run `rust/target/release/ast_pipeline --help` (flags can change); for .ebnf input the binary must be built `--features ebnf_dual_run``
- "how do I regenerate a parser from its grammar" -> [makefile-targets-reference](docs/knowledge/makefile-targets-reference.md) · 2026-06-04 · reverify: ``grep -oE "^[a-zA-Z_][a-zA-Z0-9_]*:" rust/Makefile | sort -u` for the live target list; `ls rust/scripts/*.sh` for gates`
- "how do I regenerate the SystemVerilog parser after a grammar edit" -> [makefile-targets-reference](docs/knowledge/makefile-targets-reference.md) · 2026-06-04 · reverify: ``grep -oE "^[a-zA-Z_][a-zA-Z0-9_]*:" rust/Makefile | sort -u` for the live target list; `ls rust/scripts/*.sh` for gates`
- "how do I run clippy the project-sanctioned way" -> [makefile-targets-reference](docs/knowledge/makefile-targets-reference.md) · 2026-06-04 · reverify: ``grep -oE "^[a-zA-Z_][a-zA-Z0-9_]*:" rust/Makefile | sort -u` for the live target list; `ls rust/scripts/*.sh` for gates`
- "how do I run generate_target_witnesses" -> [stimuli-generator-construction-path](docs/knowledge/stimuli-generator-construction-path.md) · 2026-06-06 · reverify: `grep -n 'StimuliGenerator::new\\|generate_gap_report\\|generate_target_witnesses' rust/src/main.rs`
- "how do I run the SV stimuli quality gate or the external-corpus triage" -> [makefile-targets-reference](docs/knowledge/makefile-targets-reference.md) · 2026-06-04 · reverify: ``grep -oE "^[a-zA-Z_][a-zA-Z0-9_]*:" rust/Makefile | sort -u` for the live target list; `ls rust/scripts/*.sh` for gates`
- "how do I run the ast_pipeline binary and what are its modes" -> [ast-pipeline-cli-reference](docs/knowledge/ast-pipeline-cli-reference.md) · 2026-06-04 · reverify: `run `rust/target/release/ast_pipeline --help` (flags can change); for .ebnf input the binary must be built `--features ebnf_dual_run``
- "how do I run the mdbook docs gate" -> [makefile-targets-reference](docs/knowledge/makefile-targets-reference.md) · 2026-06-04 · reverify: ``grep -oE "^[a-zA-Z_][a-zA-Z0-9_]*:" rust/Makefile | sort -u` for the live target list; `ls rust/scripts/*.sh` for gates`
- "how do I run the witness pass (target-report-input) by hand" -> [ast-pipeline-cli-reference](docs/knowledge/ast-pipeline-cli-reference.md) · 2026-06-04 · reverify: `run `rust/target/release/ast_pipeline --help` (flags can change); for .ebnf input the binary must be built `--features ebnf_dual_run``
- "how do I tell a depth failure from a visit-limit or timeout failure" -> [stimuli-generation-error-reasons](docs/knowledge/stimuli-generation-error-reasons.md) · 2026-06-03 · reverify: `grep -n "GenerationErrorReason\|classify_generation_error" rust/src/ast_pipeline/stimuli_generator.rs`
- "how do I write generated stimuli to a file (--output vs positional path)" -> [ast-pipeline-cli-reference](docs/knowledge/ast-pipeline-cli-reference.md) · 2026-06-04 · reverify: `run `rust/target/release/ast_pipeline --help` (flags can change); for .ebnf input the binary must be built `--features ebnf_dual_run``
- "how do we detect mis-parse (accepted but wrong AST)" -> [parse-fidelity-oracles](docs/knowledge/parse-fidelity-oracles.md) · 2026-06-03 · reverify: `see docs/tasks/PARSE-FIDELITY.md literature grounding`
- "how do we prove the parser never rejects valid input" -> [parse-completeness-differential-oracle](docs/knowledge/parse-completeness-differential-oracle.md) · 2026-06-03 · reverify: `see docs/tasks/PARSE-COMPLETENESS.md literature grounding`
- "how does PGEN decide if an uncovered branch is a generator bug or a grammar bug" -> [grammar-linter-trustworthiness](docs/knowledge/grammar-linter-trustworthiness.md) · 2026-06-06 · reverify: `grep -n 'verify_unreachability_certificate\\|UnreachabilityCertificate\\|EarlierArmAlwaysSucceeds' rust/src/ast_pipeline/grammar_wellformedness.rs`
- "how does PGEN parse a .ebnf grammar file" -> [ebnf-frontend-architecture](docs/knowledge/ebnf-frontend-architecture.md) · 2026-06-06 · reverify: ``grep -nE "fn parse_ebnf|fn tokenize_rule_expression|has_generated_ebnf_parser|generated_verify_required" rust/src/ebnf_frontend.rs`; `grep -n "fn transform_from_raw_ast" rust/src/ast_pipeline/mod.rs``
- "how does a .ebnf file flow into the AST pipeline" -> [ebnf-frontend-architecture](docs/knowledge/ebnf-frontend-architecture.md) · 2026-06-06 · reverify: ``grep -nE "fn parse_ebnf|fn tokenize_rule_expression|has_generated_ebnf_parser|generated_verify_required" rust/src/ebnf_frontend.rs`; `grep -n "fn transform_from_raw_ast" rust/src/ast_pipeline/mod.rs``
- "how does main.rs construct the stimuli generator from the ebnf" -> [stimuli-generator-construction-path](docs/knowledge/stimuli-generator-construction-path.md) · 2026-06-06 · reverify: `grep -n 'StimuliGenerator::new\\|generate_gap_report\\|generate_target_witnesses' rust/src/main.rs`
- "how does the AST pipeline work end-to-end" -> [ast-pipeline-architecture](docs/knowledge/ast-pipeline-architecture.md) · 2026-06-06 · reverify: ``grep -nE 'pub enum ASTNode|pub enum ParseContent|pub struct Annotations|fn transform_from_raw_ast|fn extract_rule_annotations' rust/src/ast_pipeline/mod.rs`; `grep -nE 'semantic_annotations\\.get|branch_semantic_annotations\\.get|branch_mid_sequence' rust/src/ast_pipeline/stimuli_generator.rs``
- "how does the generator steer to a specific coverage target" -> [stimuli-residual-coverage-model](docs/knowledge/stimuli-residual-coverage-model.md) · 2026-06-03 · reverify: `grep -n "StimuliCoverageTarget\|replay_target\|reach_classification" rust/src/ast_pipeline/stimuli_generator.rs`
- "how does the witness pass get invoked" -> [stimuli-generator-construction-path](docs/knowledge/stimuli-generator-construction-path.md) · 2026-06-06 · reverify: `grep -n 'StimuliGenerator::new\\|generate_gap_report\\|generate_target_witnesses' rust/src/main.rs`
- "how is a grammar fully certified" -> [grammar-linter-trustworthiness](docs/knowledge/grammar-linter-trustworthiness.md) · 2026-06-06 · reverify: `grep -n 'verify_unreachability_certificate\\|UnreachabilityCertificate\\|EarlierArmAlwaysSucceeds' rust/src/ast_pipeline/grammar_wellformedness.rs`
- "how is a grammar loaded into the generator" -> [stimuli-generator-construction-path](docs/knowledge/stimuli-generator-construction-path.md) · 2026-06-06 · reverify: `grep -n 'StimuliGenerator::new\\|generate_gap_report\\|generate_target_witnesses' rust/src/main.rs`
- "how is parser correctness decomposed / the four pillars" -> [parser-signoff-four-pillars](docs/knowledge/parser-signoff-four-pillars.md) · 2026-06-03 · reverify: `grep -l "parser sign-off pillar" docs/tasks/PARSE-*.md`
- "how is the StimuliGenerator built from a grammar" -> [stimuli-generator-construction-path](docs/knowledge/stimuli-generator-construction-path.md) · 2026-06-06 · reverify: `grep -n 'StimuliGenerator::new\\|generate_gap_report\\|generate_target_witnesses' rust/src/main.rs`
- "how to generate an input that reaches a specific grammar production (directed)" -> [grammar-coverage-and-directed-generation](docs/knowledge/grammar-coverage-and-directed-generation.md) · 2026-06-03 · reverify: `see docs/tasks/SV-EXH-PROOF.md (.7.4.x) + docs/tasks/STIMULI-SIGNOFF.md`
- "how to generate deep witnesses without timeout (.7.4.6)" -> [grammar-coverage-and-directed-generation](docs/knowledge/grammar-coverage-and-directed-generation.md) · 2026-06-03 · reverify: `see docs/tasks/SV-EXH-PROOF.md (.7.4.x) + docs/tasks/STIMULI-SIGNOFF.md`
- "how to make slow deeply-factored SV witnesses converge" -> [sv-witness-purdom-ordering](docs/knowledge/sv-witness-purdom-ordering.md) · 2026-06-03 · reverify: `PGEN_WITNESS_NO_PURDOM=1 vs unset, rerun the witness pass (--target-report-input <sample> --target-max-attempts 0 --target-generation-timeout-ms 7000 --seed 712001) and compare resolved / target_timeout`
- "how to run the SV corpus gate safely without exhausting host RAM" -> [sv-corpus-gate-uvm-memory](docs/knowledge/sv-corpus-gate-uvm-memory.md) · 2026-06-05 · reverify: ``ulimit -v 12582912; target/debug/parseability_probe --parse systemverilog <uvm_pkg.preprocessed.sv> --profile 2017` and watch RSS — slow, and memory climbs super-linearly deeper in the parse`
- "how to speed up the pgen regex parser" -> [rgx-0078-regex-slowness-followup](docs/knowledge/rgx-0078-regex-slowness-followup.md) · 2026-06-03 · reverify: `cat the RGX issue yaml; run the pgen_iteration_flow harness (PCRE2-relative bench)`
- "how to test parser correctness without a reference AST" -> [parse-fidelity-oracles](docs/knowledge/parse-fidelity-oracles.md) · 2026-06-03 · reverify: `see docs/tasks/PARSE-FIDELITY.md literature grounding`
- "how were the slow witness timeouts reduced" -> [sv-witness-purdom-ordering](docs/knowledge/sv-witness-purdom-ordering.md) · 2026-06-03 · reverify: `PGEN_WITNESS_NO_PURDOM=1 vs unset, rerun the witness pass (--target-report-input <sample> --target-max-attempts 0 --target-generation-timeout-ms 7000 --seed 712001) and compare resolved / target_timeout`
- "is PGEN a PEG or a CFG parser generator" -> [pgen-parsing-model](docs/knowledge/pgen-parsing-model.md) · 2026-06-03 · reverify: `grep -n "eliminate_left_recursion\|eliminate_left_recursive_patterns" rust/src/ast_pipeline/mod.rs rust/src/main.rs`
- "is PGEN stateless or stateful packrat" -> [pgen-parsing-model](docs/knowledge/pgen-parsing-model.md) · 2026-06-03 · reverify: `grep -n "eliminate_left_recursion\|eliminate_left_recursive_patterns" rust/src/ast_pipeline/mod.rs rust/src/main.rs`
- "is PGEN's generator behind or ahead of academic grammar fuzzers" -> [stimuli-generator-capability-gaps](docs/knowledge/stimuli-generator-capability-gaps.md) · 2026-06-03 · reverify: `see docs/tasks/STIMULI-SIGNOFF.md leaves .2-.7`
- "is PGEN's parser guaranteed linear time / hang-free" -> [stateful-packrat-not-linear](docs/knowledge/stateful-packrat-not-linear.md) · 2026-06-03 · reverify: `see docs/tasks/PARSE-TERMINATION.md; grep -n "memoiz" rust/src/ast_pipeline/*.rs`
- "is a position-specific (mid-sequence) annotation consumed by the stimuli generator" -> [ast-pipeline-architecture](docs/knowledge/ast-pipeline-architecture.md) · 2026-06-06 · reverify: ``grep -nE 'pub enum ASTNode|pub enum ParseContent|pub struct Annotations|fn transform_from_raw_ast|fn extract_rule_annotations' rust/src/ast_pipeline/mod.rs`; `grep -nE 'semantic_annotations\\.get|branch_semantic_annotations\\.get|branch_mid_sequence' rust/src/ast_pipeline/stimuli_generator.rs``
- "is exact grammar reachability decidable" -> [grammar-linter-trustworthiness](docs/knowledge/grammar-linter-trustworthiness.md) · 2026-06-06 · reverify: `grep -n 'verify_unreachability_certificate\\|UnreachabilityCertificate\\|EarlierArmAlwaysSucceeds' rust/src/ast_pipeline/grammar_wellformedness.rs`
- "is the EBNF parser hand-written or generated" -> [ebnf-frontend-architecture](docs/knowledge/ebnf-frontend-architecture.md) · 2026-06-06 · reverify: ``grep -nE "fn parse_ebnf|fn tokenize_rule_expression|has_generated_ebnf_parser|generated_verify_required" rust/src/ebnf_frontend.rs`; `grep -n "fn transform_from_raw_ast" rust/src/ast_pipeline/mod.rs``
- "is the SV parser slow / memory-heavy on uvm and why" -> [sv-corpus-gate-uvm-memory](docs/knowledge/sv-corpus-gate-uvm-memory.md) · 2026-06-05 · reverify: ``ulimit -v 12582912; target/debug/parseability_probe --parse systemverilog <uvm_pkg.preprocessed.sv> --profile 2017` and watch RSS — slow, and memory climbs super-linearly deeper in the parse`
- "is the SV stateful-packrat fix relevant to regex slowness" -> [rgx-0078-regex-slowness-followup](docs/knowledge/rgx-0078-regex-slowness-followup.md) · 2026-06-03 · reverify: `cat the RGX issue yaml; run the pgen_iteration_flow harness (PCRE2-relative bench)`
- "is the grammar linter trustworthy" -> [grammar-linter-trustworthiness](docs/knowledge/grammar-linter-trustworthiness.md) · 2026-06-06 · reverify: `grep -n 'verify_unreachability_certificate\\|UnreachabilityCertificate\\|EarlierArmAlwaysSucceeds' rust/src/ast_pipeline/grammar_wellformedness.rs`
- "is the grammar linter's reachability answer always correct" -> [grammar-linter-trustworthiness](docs/knowledge/grammar-linter-trustworthiness.md) · 2026-06-06 · reverify: `grep -n 'verify_unreachability_certificate\\|UnreachabilityCertificate\\|EarlierArmAlwaysSucceeds' rust/src/ast_pipeline/grammar_wellformedness.rs`
- "is the parser failing on real inputs or is it the stimuli generator" -> [parser-signoff-four-pillars](docs/knowledge/parser-signoff-four-pillars.md) · 2026-06-03 · reverify: `grep -l "parser sign-off pillar" docs/tasks/PARSE-*.md`
- "is there a published technique for the literal-0 / derivation-directed generation idea" -> [grammar-coverage-and-directed-generation](docs/knowledge/grammar-coverage-and-directed-generation.md) · 2026-06-03 · reverify: `see docs/tasks/SV-EXH-PROOF.md (.7.4.x) + docs/tasks/STIMULI-SIGNOFF.md`
- "is there a ready-made SV compliance test suite" -> [parse-completeness-differential-oracle](docs/knowledge/parse-completeness-differential-oracle.md) · 2026-06-03 · reverify: `see docs/tasks/PARSE-COMPLETENESS.md literature grounding`
- "what Makefile targets exist and which do I use" -> [makefile-targets-reference](docs/knowledge/makefile-targets-reference.md) · 2026-06-04 · reverify: ``grep -oE "^[a-zA-Z_][a-zA-Z0-9_]*:" rust/Makefile | sort -u` for the live target list; `ls rust/scripts/*.sh` for gates`
- "what are never_hit / never_selected / selected_but_failed reasons" -> [stimuli-residual-coverage-model](docs/knowledge/stimuli-residual-coverage-model.md) · 2026-06-03 · reverify: `grep -n "StimuliCoverageTarget\|replay_target\|reach_classification" rust/src/ast_pipeline/stimuli_generator.rs`
- "what are the non-covering branch targets in SV stimuli coverage" -> [sv-literal-0-residual-decomposition](docs/knowledge/sv-literal-0-residual-decomposition.md) · 2026-06-04 · reverify: `run the witness pass (--target-report-input <150-sample> --target-max-attempts 0 --target-generation-timeout-ms 200 --seed 712001 --output /tmp/o.json) and read the "Witness 'target_timeout'-failure samples" + "still-UNRESOLVED samples" stdout lines; or `make sv_stimuli_quality_gate` for closed_loop_replay_targets_total`
- "what are the remaining problem types for the SV parser" -> [parser-signoff-four-pillars](docs/knowledge/parser-signoff-four-pillars.md) · 2026-06-03 · reverify: `grep -l "parser sign-off pillar" docs/tasks/PARSE-*.md`
- "what are the stages from .ebnf to generated parser / stimuli" -> [ast-pipeline-architecture](docs/knowledge/ast-pipeline-architecture.md) · 2026-06-06 · reverify: ``grep -nE 'pub enum ASTNode|pub enum ParseContent|pub struct Annotations|fn transform_from_raw_ast|fn extract_rule_annotations' rust/src/ast_pipeline/mod.rs`; `grep -nE 'semantic_annotations\\.get|branch_semantic_annotations\\.get|branch_mid_sequence' rust/src/ast_pipeline/stimuli_generator.rs``
- "what are the stimuli generator capability gaps" -> [stimuli-generator-capability-gaps](docs/knowledge/stimuli-generator-capability-gaps.md) · 2026-06-03 · reverify: `see docs/tasks/STIMULI-SIGNOFF.md leaves .2-.7`
- "what coverage metric for grammar-based generation (k-path)" -> [grammar-coverage-and-directed-generation](docs/knowledge/grammar-coverage-and-directed-generation.md) · 2026-06-03 · reverify: `see docs/tasks/SV-EXH-PROOF.md (.7.4.x) + docs/tasks/STIMULI-SIGNOFF.md`
- "what do the stimuli generation error reasons mean" -> [stimuli-generation-error-reasons](docs/knowledge/stimuli-generation-error-reasons.md) · 2026-06-03 · reverify: `grep -n "GenerationErrorReason\|classify_generation_error" rust/src/ast_pipeline/stimuli_generator.rs`
- "what does PGEN_WITNESS_NO_PURDOM do" -> [sv-witness-purdom-ordering](docs/knowledge/sv-witness-purdom-ordering.md) · 2026-06-03 · reverify: `PGEN_WITNESS_NO_PURDOM=1 vs unset, rerun the witness pass (--target-report-input <sample> --target-max-attempts 0 --target-generation-timeout-ms 7000 --seed 712001) and compare resolved / target_timeout`
- "what does UNKNOWN mean in the grammar linter" -> [grammar-linter-trustworthiness](docs/knowledge/grammar-linter-trustworthiness.md) · 2026-06-06 · reverify: `grep -n 'verify_unreachability_certificate\\|UnreachabilityCertificate\\|EarlierArmAlwaysSucceeds' rust/src/ast_pipeline/grammar_wellformedness.rs`
- "what does the Annotations struct hold and how is it populated" -> [ast-pipeline-architecture](docs/knowledge/ast-pipeline-architecture.md) · 2026-06-06 · reverify: ``grep -nE 'pub enum ASTNode|pub enum ParseContent|pub struct Annotations|fn transform_from_raw_ast|fn extract_rule_annotations' rust/src/ast_pipeline/mod.rs`; `grep -nE 'semantic_annotations\\.get|branch_semantic_annotations\\.get|branch_mid_sequence' rust/src/ast_pipeline/stimuli_generator.rs``
- "what does the Stimuli generation depth exceeded message mean" -> [sv-residual-depth-budget-cause](docs/knowledge/sv-residual-depth-budget-cause.md) · 2026-06-03 · reverify: `grep -n "max_depth\|depth exceeded\|DEPTH_EXCEEDED" rust/src/ast_pipeline/stimuli_generator.rs`
- "what does the stimuli coverage gap report measure" -> [stimuli-residual-coverage-model](docs/knowledge/stimuli-residual-coverage-model.md) · 2026-06-03 · reverify: `grep -n "StimuliCoverageTarget\|replay_target\|reach_classification" rust/src/ast_pipeline/stimuli_generator.rs`
- "what does the witness construction speedup (Cow) do" -> [sv-literal-0-residual-decomposition](docs/knowledge/sv-literal-0-residual-decomposition.md) · 2026-06-04 · reverify: `run the witness pass (--target-report-input <150-sample> --target-max-attempts 0 --target-generation-timeout-ms 200 --seed 712001 --output /tmp/o.json) and read the "Witness 'target_timeout'-failure samples" + "still-UNRESOLVED samples" stdout lines; or `make sv_stimuli_quality_gate` for closed_loop_replay_targets_total`
- "what env vars tune stimuli/witness generation" -> [ast-pipeline-cli-reference](docs/knowledge/ast-pipeline-cli-reference.md) · 2026-06-04 · reverify: `run `rust/target/release/ast_pipeline --help` (flags can change); for .ebnf input the binary must be built `--features ebnf_dual_run``
- "what happens when the stimuli generator cannot reach a grammar fragment" -> [grammar-linter-trustworthiness](docs/knowledge/grammar-linter-trustworthiness.md) · 2026-06-06 · reverify: `grep -n 'verify_unreachability_certificate\\|UnreachabilityCertificate\\|EarlierArmAlwaysSucceeds' rust/src/ast_pipeline/grammar_wellformedness.rs`
- "what is ASTNode vs ParseContent (grammar IR vs runtime parse result)" -> [ast-pipeline-architecture](docs/knowledge/ast-pipeline-architecture.md) · 2026-06-06 · reverify: ``grep -nE 'pub enum ASTNode|pub enum ParseContent|pub struct Annotations|fn transform_from_raw_ast|fn extract_rule_annotations' rust/src/ast_pipeline/mod.rs`; `grep -nE 'semantic_annotations\\.get|branch_semantic_annotations\\.get|branch_mid_sequence' rust/src/ast_pipeline/stimuli_generator.rs``
- "what is GenerationErrorReason / classify_generation_error" -> [stimuli-generation-error-reasons](docs/knowledge/stimuli-generation-error-reasons.md) · 2026-06-03 · reverify: `grep -n "GenerationErrorReason\|classify_generation_error" rust/src/ast_pipeline/stimuli_generator.rs`
- "what is LoadedGrammar" -> [stimuli-generator-construction-path](docs/knowledge/stimuli-generator-construction-path.md) · 2026-06-06 · reverify: `grep -n 'StimuliGenerator::new\\|generate_gap_report\\|generate_target_witnesses' rust/src/main.rs`
- "what is PEG / what is Packrat / what is a data-dependent grammar" -> [pgen-parsing-model](docs/knowledge/pgen-parsing-model.md) · 2026-06-03 · reverify: `grep -n "eliminate_left_recursion\|eliminate_left_recursive_patterns" rust/src/ast_pipeline/mod.rs rust/src/main.rs`
- "what is PGEN with respect to PEG and Packrat" -> [pgen-parsing-model](docs/knowledge/pgen-parsing-model.md) · 2026-06-03 · reverify: `grep -n "eliminate_left_recursion\|eliminate_left_recursive_patterns" rust/src/ast_pipeline/mod.rs rust/src/main.rs`
- "what is PGEN-RGX-0078 / the regex parser slowness follow-up" -> [rgx-0078-regex-slowness-followup](docs/knowledge/rgx-0078-regex-slowness-followup.md) · 2026-06-03 · reverify: `cat the RGX issue yaml; run the pgen_iteration_flow harness (PCRE2-relative bench)`
- "what is a certifying linter or certificate" -> [grammar-linter-trustworthiness](docs/knowledge/grammar-linter-trustworthiness.md) · 2026-06-06 · reverify: `grep -n 'verify_unreachability_certificate\\|UnreachabilityCertificate\\|EarlierArmAlwaysSucceeds' rust/src/ast_pipeline/grammar_wellformedness.rs`
- "what is a witness vs a proof in the grammar linter" -> [grammar-linter-trustworthiness](docs/knowledge/grammar-linter-trustworthiness.md) · 2026-06-06 · reverify: `grep -n 'verify_unreachability_certificate\\|UnreachabilityCertificate\\|EarlierArmAlwaysSucceeds' rust/src/ast_pipeline/grammar_wellformedness.rs`
- "what is an out-of-band acceptance validator and why is it a defect" -> [ebnf-single-source-of-truth](docs/knowledge/ebnf-single-source-of-truth.md) · 2026-06-07 · reverify: ``./rust/target/debug/parseability_probe --parse regex <(printf '\\\\u{b7a2}')`; `grep -n 'unsupported regex escape' rust/src/regex_compile_validation.rs`; `grep -c 'regex_compile_validation' rust/src/ast_pipeline/stimuli_generator.rs` (expect 0); `grep -nE 'unicode_escape|directive_verb' grammars/regex.ebnf``
- "what is focused_replay_target_debt_zero / literal-0" -> [stimuli-residual-coverage-model](docs/knowledge/stimuli-residual-coverage-model.md) · 2026-06-03 · reverify: `grep -n "StimuliCoverageTarget\|replay_target\|reach_classification" rust/src/ast_pipeline/stimuli_generator.rs`
- "what is invertible syntax / bidirectional parsing pretty-printing" -> [parse-fidelity-oracles](docs/knowledge/parse-fidelity-oracles.md) · 2026-06-03 · reverify: `see docs/tasks/PARSE-FIDELITY.md literature grounding`
- "what is missing from the stimuli generator vs the literature" -> [stimuli-generator-capability-gaps](docs/knowledge/stimuli-generator-capability-gaps.md) · 2026-06-03 · reverify: `see docs/tasks/STIMULI-SIGNOFF.md leaves .2-.7`
- "what is replay_target_count / the stimuli residual" -> [stimuli-residual-coverage-model](docs/knowledge/stimuli-residual-coverage-model.md) · 2026-06-03 · reverify: `grep -n "StimuliCoverageTarget\|replay_target\|reach_classification" rust/src/ast_pipeline/stimuli_generator.rs`
- "what is src/ebnf_frontend.rs vs generated/ebnf.rs vs grammars/ebnf.ebnf" -> [ebnf-frontend-architecture](docs/knowledge/ebnf-frontend-architecture.md) · 2026-06-06 · reverify: ``grep -nE "fn parse_ebnf|fn tokenize_rule_expression|has_generated_ebnf_parser|generated_verify_required" rust/src/ebnf_frontend.rs`; `grep -n "fn transform_from_raw_ast" rust/src/ast_pipeline/mod.rs``
- "what is the attribution rule" -> [grammar-linter-trustworthiness](docs/knowledge/grammar-linter-trustworthiness.md) · 2026-06-06 · reverify: `grep -n 'verify_unreachability_certificate\\|UnreachabilityCertificate\\|EarlierArmAlwaysSucceeds' rust/src/ast_pipeline/grammar_wellformedness.rs`
- "what is the closed-loop replay residual / focused_replay_target_debt_zero" -> [sv-literal-0-residual-decomposition](docs/knowledge/sv-literal-0-residual-decomposition.md) · 2026-06-04 · reverify: `run the witness pass (--target-report-input <150-sample> --target-max-attempts 0 --target-generation-timeout-ms 200 --seed 712001 --output /tmp/o.json) and read the "Witness 'target_timeout'-failure samples" + "still-UNRESOLVED samples" stdout lines; or `make sv_stimuli_quality_gate` for closed_loop_replay_targets_total`
- "what is the closure criterion for regex parse performance" -> [rgx-0078-regex-slowness-followup](docs/knowledge/rgx-0078-regex-slowness-followup.md) · 2026-06-03 · reverify: `cat the RGX issue yaml; run the pgen_iteration_flow harness (PCRE2-relative bench)`
- "what is the difference between parser failures and the stimuli residual" -> [parser-signoff-four-pillars](docs/knowledge/parser-signoff-four-pillars.md) · 2026-06-03 · reverify: `grep -l "parser sign-off pillar" docs/tasks/PARSE-*.md`
- "what is the fix for stateful packrat non-linearity" -> [stateful-packrat-not-linear](docs/knowledge/stateful-packrat-not-linear.md) · 2026-06-03 · reverify: `see docs/tasks/PARSE-TERMINATION.md; grep -n "memoiz" rust/src/ast_pipeline/*.rs`
- "what is the literature for grammar-based test/stimuli generation coverage" -> [grammar-coverage-and-directed-generation](docs/knowledge/grammar-coverage-and-directed-generation.md) · 2026-06-03 · reverify: `see docs/tasks/SV-EXH-PROOF.md (.7.4.x) + docs/tasks/STIMULI-SIGNOFF.md`
- "what is the literature for parser completeness testing" -> [parse-completeness-differential-oracle](docs/knowledge/parse-completeness-differential-oracle.md) · 2026-06-03 · reverify: `see docs/tasks/PARSE-COMPLETENESS.md literature grounding`
- "what is the production stimuli-generation path" -> [stimuli-generator-construction-path](docs/knowledge/stimuli-generator-construction-path.md) · 2026-06-06 · reverify: `grep -n 'StimuliGenerator::new\\|generate_gap_report\\|generate_target_witnesses' rust/src/main.rs`
- "what is the property_expr construction-cost residual" -> [sv-literal-0-residual-decomposition](docs/knowledge/sv-literal-0-residual-decomposition.md) · 2026-06-04 · reverify: `run the witness pass (--target-report-input <150-sample> --target-max-attempts 0 --target-generation-timeout-ms 200 --seed 712001 --output /tmp/o.json) and read the "Witness 'target_timeout'-failure samples" + "still-UNRESOLVED samples" stdout lines; or `make sv_stimuli_quality_gate` for closed_loop_replay_targets_total`
- "what is the root cause of the uncovered SV coverage branches" -> [sv-residual-depth-budget-cause](docs/knowledge/sv-residual-depth-budget-cause.md) · 2026-06-03 · reverify: `grep -n "max_depth\|depth exceeded\|DEPTH_EXCEEDED" rust/src/ast_pipeline/stimuli_generator.rs`
- "what is the test oracle for parser fidelity / correct AST" -> [parse-fidelity-oracles](docs/knowledge/parse-fidelity-oracles.md) · 2026-06-03 · reverify: `see docs/tasks/PARSE-FIDELITY.md literature grounding`
- "what is the two-surface architecture (runtime interpreter vs codegen)" -> [ast-two-surface-construction](docs/knowledge/ast-two-surface-construction.md) · 2026-06-03 · reverify: `grep -n "serde_json::Map::new\|Value::Object" rust/src/ast_pipeline/unified_return_ast.rs`
- "what is witness_mode / the Purdom witness ordering" -> [sv-witness-purdom-ordering](docs/knowledge/sv-witness-purdom-ordering.md) · 2026-06-03 · reverify: `PGEN_WITNESS_NO_PURDOM=1 vs unset, rerun the witness pass (--target-report-input <sample> --target-max-attempts 0 --target-generation-timeout-ms 7000 --seed 712001) and compare resolved / target_timeout`
- "what makes generation and parsing inconsistent in PGEN" -> [ebnf-single-source-of-truth](docs/knowledge/ebnf-single-source-of-truth.md) · 2026-06-07 · reverify: ``./rust/target/debug/parseability_probe --parse regex <(printf '\\\\u{b7a2}')`; `grep -n 'unsupported regex escape' rust/src/regex_compile_validation.rs`; `grep -c 'regex_compile_validation' rust/src/ast_pipeline/stimuli_generator.rs` (expect 0); `grep -nE 'unicode_escape|directive_verb' grammars/regex.ebnf``
- "what must I edit to add a sibling key to every AST object" -> [ast-two-surface-construction](docs/knowledge/ast-two-surface-construction.md) · 2026-06-03 · reverify: `grep -n "serde_json::Map::new\|Value::Object" rust/src/ast_pipeline/unified_return_ast.rs`
- "what reference parsers / corpus to use for SystemVerilog differential testing" -> [parse-completeness-differential-oracle](docs/knowledge/parse-completeness-differential-oracle.md) · 2026-06-03 · reverify: `see docs/tasks/PARSE-COMPLETENESS.md literature grounding`
- "what should a signoff-grade EBNF stimuli generator do that ours doesn't" -> [stimuli-generator-capability-gaps](docs/knowledge/stimuli-generator-capability-gaps.md) · 2026-06-03 · reverify: `see docs/tasks/STIMULI-SIGNOFF.md leaves .2-.7`
- "what to do after the SV main parser reaches Done" -> [rgx-0078-regex-slowness-followup](docs/knowledge/rgx-0078-regex-slowness-followup.md) · 2026-06-03 · reverify: `cat the RGX issue yaml; run the pgen_iteration_flow harness (PCRE2-relative bench)`
- "where are depth_exceeded_errors / target_timeout_errors counted" -> [stimuli-generation-error-reasons](docs/knowledge/stimuli-generation-error-reasons.md) · 2026-06-03 · reverify: `grep -n "GenerationErrorReason\|classify_generation_error" rust/src/ast_pipeline/stimuli_generator.rs`
- "where are grammar_tree and rule_order available in main.rs" -> [stimuli-generator-construction-path](docs/knowledge/stimuli-generator-construction-path.md) · 2026-06-06 · reverify: `grep -n 'StimuliGenerator::new\\|generate_gap_report\\|generate_target_witnesses' rust/src/main.rs`
- "where are the gate shell scripts and how do I run one directly" -> [makefile-targets-reference](docs/knowledge/makefile-targets-reference.md) · 2026-06-04 · reverify: ``grep -oE "^[a-zA-Z_][a-zA-Z0-9_]*:" rust/Makefile | sort -u` for the live target list; `ls rust/scripts/*.sh` for gates`
- "where are typed AST objects constructed in PGEN" -> [ast-two-surface-construction](docs/knowledge/ast-two-surface-construction.md) · 2026-06-03 · reverify: `grep -n "serde_json::Map::new\|Value::Object" rust/src/ast_pipeline/unified_return_ast.rs`
- "where does PGEN stand in the parsing literature" -> [pgen-parsing-model](docs/knowledge/pgen-parsing-model.md) · 2026-06-03 · reverify: `grep -n "eliminate_left_recursion\|eliminate_left_recursive_patterns" rust/src/ast_pipeline/mod.rs rust/src/main.rs`
- "where does regex_compile_validation.rs fit and why is it invisible to the generator" -> [ebnf-single-source-of-truth](docs/knowledge/ebnf-single-source-of-truth.md) · 2026-06-07 · reverify: ``./rust/target/debug/parseability_probe --parse regex <(printf '\\\\u{b7a2}')`; `grep -n 'unsupported regex escape' rust/src/regex_compile_validation.rs`; `grep -c 'regex_compile_validation' rust/src/ast_pipeline/stimuli_generator.rs` (expect 0); `grep -nE 'unicode_escape|directive_verb' grammars/regex.ebnf``
- "where does transform_from_raw_ast / extract_rule_annotations fit" -> [ast-pipeline-architecture](docs/knowledge/ast-pipeline-architecture.md) · 2026-06-06 · reverify: ``grep -nE 'pub enum ASTNode|pub enum ParseContent|pub struct Annotations|fn transform_from_raw_ast|fn extract_rule_annotations' rust/src/ast_pipeline/mod.rs`; `grep -nE 'semantic_annotations\\.get|branch_semantic_annotations\\.get|branch_mid_sequence' rust/src/ast_pipeline/stimuli_generator.rs``
- "where is the EBNF tokenizer and how are optional [ ] / @ annotations tokenized" -> [ebnf-frontend-architecture](docs/knowledge/ebnf-frontend-architecture.md) · 2026-06-06 · reverify: ``grep -nE "fn parse_ebnf|fn tokenize_rule_expression|has_generated_ebnf_parser|generated_verify_required" rust/src/ebnf_frontend.rs`; `grep -n "fn transform_from_raw_ast" rust/src/ast_pipeline/mod.rs``
- "which consumer (codegen vs stimuli generator) reads which annotation granularity" -> [ast-pipeline-architecture](docs/knowledge/ast-pipeline-architecture.md) · 2026-06-06 · reverify: ``grep -nE 'pub enum ASTNode|pub enum ParseContent|pub struct Annotations|fn transform_from_raw_ast|fn extract_rule_annotations' rust/src/ast_pipeline/mod.rs`; `grep -nE 'semantic_annotations\\.get|branch_semantic_annotations\\.get|branch_mid_sequence' rust/src/ast_pipeline/stimuli_generator.rs``
- "which files build the AST object map" -> [ast-two-surface-construction](docs/knowledge/ast-two-surface-construction.md) · 2026-06-03 · reverify: `grep -n "serde_json::Map::new\|Value::Object" rust/src/ast_pipeline/unified_return_ast.rs`
- "which task tree owns parser reject / hang / mis-parse" -> [parser-signoff-four-pillars](docs/knowledge/parser-signoff-four-pillars.md) · 2026-06-03 · reverify: `grep -l "parser sign-off pillar" docs/tasks/PARSE-*.md`
- "why can't deeply-nested SV rules be generated from the top entry" -> [sv-residual-depth-budget-cause](docs/knowledge/sv-residual-depth-budget-cause.md) · 2026-06-03 · reverify: `grep -n "max_depth\|depth exceeded\|DEPTH_EXCEEDED" rust/src/ast_pipeline/stimuli_generator.rs`
- "why do SV witnesses time out instead of erroring" -> [sv-witness-purdom-ordering](docs/knowledge/sv-witness-purdom-ordering.md) · 2026-06-03 · reverify: `PGEN_WITNESS_NO_PURDOM=1 vs unset, rerun the witness pass (--target-report-input <sample> --target-max-attempts 0 --target-generation-timeout-ms 7000 --seed 712001) and compare resolved / target_timeout`
- "why do regex \\u{...} and (*verb) generate but fail to re-parse" -> [ebnf-single-source-of-truth](docs/knowledge/ebnf-single-source-of-truth.md) · 2026-06-07 · reverify: ``./rust/target/debug/parseability_probe --parse regex <(printf '\\\\u{b7a2}')`; `grep -n 'unsupported regex escape' rust/src/regex_compile_validation.rs`; `grep -c 'regex_compile_validation' rust/src/ast_pipeline/stimuli_generator.rs` (expect 0); `grep -nE 'unicode_escape|directive_verb' grammars/regex.ebnf``
- "why does a grammar-driven generator emit parser-rejected output" -> [ebnf-single-source-of-truth](docs/knowledge/ebnf-single-source-of-truth.md) · 2026-06-07 · reverify: ``./rust/target/debug/parseability_probe --parse regex <(printf '\\\\u{b7a2}')`; `grep -n 'unsupported regex escape' rust/src/regex_compile_validation.rs`; `grep -c 'regex_compile_validation' rust/src/ast_pipeline/stimuli_generator.rs` (expect 0); `grep -nE 'unicode_escape|directive_verb' grammars/regex.ebnf``
- "why does ast_pipeline say it requires --features ebnf_dual_run" -> [ast-pipeline-cli-reference](docs/knowledge/ast-pipeline-cli-reference.md) · 2026-06-04 · reverify: `run `rust/target/release/ast_pipeline --help` (flags can change); for .ebnf input the binary must be built `--features ebnf_dual_run``
- "why does parsing uvm_pkg / uvm_compat_pkg use tens of GB of memory" -> [sv-corpus-gate-uvm-memory](docs/knowledge/sv-corpus-gate-uvm-memory.md) · 2026-06-05 · reverify: ``ulimit -v 12582912; target/debug/parseability_probe --parse systemverilog <uvm_pkg.preprocessed.sv> --profile 2017` and watch RSS — slow, and memory climbs super-linearly deeper in the parse`
- "why does replay_target_count plateau (e.g. at 888)" -> [sv-residual-depth-budget-cause](docs/knowledge/sv-residual-depth-budget-cause.md) · 2026-06-03 · reverify: `grep -n "max_depth\|depth exceeded\|DEPTH_EXCEEDED" rust/src/ast_pipeline/stimuli_generator.rs`
- "why does the SV external corpus gate eat huge RAM or seem stuck" -> [sv-corpus-gate-uvm-memory](docs/knowledge/sv-corpus-gate-uvm-memory.md) · 2026-06-05 · reverify: ``ulimit -v 12582912; target/debug/parseability_probe --parse systemverilog <uvm_pkg.preprocessed.sv> --profile 2017` and watch RSS — slow, and memory climbs super-linearly deeper in the parse`
- "why doesn't the SystemVerilog stimuli residual reach zero" -> [sv-residual-depth-budget-cause](docs/knowledge/sv-residual-depth-budget-cause.md) · 2026-06-03 · reverify: `grep -n "max_depth\|depth exceeded\|DEPTH_EXCEEDED" rust/src/ast_pipeline/stimuli_generator.rs`
- "why is literal-0 coverage a theorem not a target" -> [grammar-linter-trustworthiness](docs/knowledge/grammar-linter-trustworthiness.md) · 2026-06-06 · reverify: `grep -n 'verify_unreachability_certificate\\|UnreachabilityCertificate\\|EarlierArmAlwaysSucceeds' rust/src/ast_pipeline/grammar_wellformedness.rs`
- "why is the SV stimuli residual not literal-0" -> [sv-literal-0-residual-decomposition](docs/knowledge/sv-literal-0-residual-decomposition.md) · 2026-06-04 · reverify: `run the witness pass (--target-report-input <150-sample> --target-max-attempts 0 --target-generation-timeout-ms 200 --seed 712001 --output /tmp/o.json) and read the "Witness 'target_timeout'-failure samples" + "still-UNRESOLVED samples" stdout lines; or `make sv_stimuli_quality_gate` for closed_loop_replay_targets_total`
- "why is the SystemVerilog main parser still Mostly Done not Done" -> [sv-literal-0-residual-decomposition](docs/knowledge/sv-literal-0-residual-decomposition.md) · 2026-06-04 · reverify: `run the witness pass (--target-report-input <150-sample> --target-max-attempts 0 --target-generation-timeout-ms 200 --seed 712001 --output /tmp/o.json) and read the "Witness 'target_timeout'-failure samples" + "still-UNRESOLVED samples" stdout lines; or `make sv_stimuli_quality_gate` for closed_loop_replay_targets_total`
- "why is the _meta carrier (A5) a coordinated multi-surface change" -> [ast-two-surface-construction](docs/knowledge/ast-two-surface-construction.md) · 2026-06-03 · reverify: `grep -n "serde_json::Map::new\|Value::Object" rust/src/ast_pipeline/unified_return_ast.rs`
- "why is the grammar linter sound but not complete" -> [grammar-linter-trustworthiness](docs/knowledge/grammar-linter-trustworthiness.md) · 2026-06-06 · reverify: `grep -n 'verify_unreachability_certificate\\|UnreachabilityCertificate\\|EarlierArmAlwaysSucceeds' rust/src/ast_pipeline/grammar_wellformedness.rs`
- "why might the SV parser hang or go exponential" -> [stateful-packrat-not-linear](docs/knowledge/stateful-packrat-not-linear.md) · 2026-06-03 · reverify: `see docs/tasks/PARSE-TERMINATION.md; grep -n "memoiz" rust/src/ast_pipeline/*.rs`

## Facts (by id)

### ast-pipeline-architecture
_The AST pipeline end-to-end — stages, IR data structures, the two consumers, and the annotation consumption matrix_

- **answers:** how does the AST pipeline work end-to-end | what are the stages from .ebnf to generated parser / stimuli | what is ASTNode vs ParseContent (grammar IR vs runtime parse result) | what does the Annotations struct hold and how is it populated | which consumer (codegen vs stimuli generator) reads which annotation granularity | where does transform_from_raw_ast / extract_rule_annotations fit | is a position-specific (mid-sequence) annotation consumed by the stimuli generator
- **date:** 2026-06-06 · **status:** current
- **evidence:** `code-verified — src/ast_pipeline/mod.rs (ASTNode enum :861, ParseContent enum :715, Annotations struct :1074, MidSequenceSemanticAnnotation :1066, transform_from_raw_ast :1355, extract_rule_annotations :2246); src/ebnf_frontend.rs (tokenizer); src/ast_pipeline/ast_based_generator.rs (codegen, mid-sequence at :190); src/ast_pipeline/stimuli_generator.rs (generator annotation reads ~:8150-8248). 5-step transform algorithm: docs/ast_transformation_pipeline.md; codegen layer: docs/AST_GENERATOR_ARCHITECTURE.md`
- **reverify:** ``grep -nE 'pub enum ASTNode|pub enum ParseContent|pub struct Annotations|fn transform_from_raw_ast|fn extract_rule_annotations' rust/src/ast_pipeline/mod.rs`; `grep -nE 'semantic_annotations\\.get|branch_semantic_annotations\\.get|branch_mid_sequence' rust/src/ast_pipeline/stimuli_generator.rs``
- **source:** [`docs/knowledge/ast-pipeline-architecture.md`](docs/knowledge/ast-pipeline-architecture.md)

### ast-pipeline-cli-reference
_ast_pipeline CLI — modes, key flags, env vars, and the gotchas_

- **answers:** how do I run the ast_pipeline binary and what are its modes | how do I generate grammar stimuli from the command line | how do I dump the grammar IR / gen_ast.json used by stimuli generation | why does ast_pipeline say it requires --features ebnf_dual_run | how do I run the witness pass (target-report-input) by hand | what env vars tune stimuli/witness generation | how do I write generated stimuli to a file (--output vs positional path) | how do I lint a grammar for well-formedness from the CLI | how do I dump the parsed AST of a grammar rule
- **date:** 2026-06-04 · **status:** current
- **evidence:** `rust/src/main.rs (arg parsing); `rust/target/release/ast_pipeline --help``
- **reverify:** `run `rust/target/release/ast_pipeline --help` (flags can change); for .ebnf input the binary must be built `--features ebnf_dual_run``
- **source:** [`docs/knowledge/ast-pipeline-cli-reference.md`](docs/knowledge/ast-pipeline-cli-reference.md)

### ast-two-surface-construction
_PGEN builds typed AST objects on TWO surfaces that must change in lockstep_

- **answers:** where are typed AST objects constructed in PGEN | why is the _meta carrier (A5) a coordinated multi-surface change | what is the two-surface architecture (runtime interpreter vs codegen) | what must I edit to add a sibling key to every AST object | which files build the AST object map
- **date:** 2026-06-03 · **status:** current
- **evidence:** `rust/src/ast_pipeline/unified_return_ast.rs (object build ~:636-:706, serde_json::Map); rust/src/ast_pipeline/return_annotation_handler.rs (emits object-construction Rust into generated parsers, ~:355); docs/tasks/PARSE-SOTA-A5-meta-carrier-design.md`
- **reverify:** `grep -n "serde_json::Map::new\|Value::Object" rust/src/ast_pipeline/unified_return_ast.rs`
- **source:** [`docs/knowledge/ast-two-surface-construction.md`](docs/knowledge/ast-two-surface-construction.md)

### ebnf-frontend-architecture
_How PGEN parses .ebnf — the hand-written Rust frontend (authoritative) vs the generated cross-check_

- **answers:** how does PGEN parse a .ebnf grammar file | is the EBNF parser hand-written or generated | what is src/ebnf_frontend.rs vs generated/ebnf.rs vs grammars/ebnf.ebnf | how do I extend the EBNF meta-grammar syntax (add a new annotation or construct) | do I need a bootstrap regen to change the EBNF grammar syntax | how does a .ebnf file flow into the AST pipeline | where is the EBNF tokenizer and how are optional [ ] / @ annotations tokenized
- **date:** 2026-06-06 · **status:** current
- **evidence:** `src/ebnf_frontend.rs (hand-written parser); src/ast_pipeline/mod.rs:1355 transform_from_raw_ast; src/main.rs (load_grammar_bundle / emit_rust_frontend_raw_ast_json call sites); grammars/ebnf.ebnf (documented meta-grammar + seed); rust/Makefile:579-592 (one-time generated/ebnf.rs seed)`
- **reverify:** ``grep -nE "fn parse_ebnf|fn tokenize_rule_expression|has_generated_ebnf_parser|generated_verify_required" rust/src/ebnf_frontend.rs`; `grep -n "fn transform_from_raw_ast" rust/src/ast_pipeline/mod.rs``
- **source:** [`docs/knowledge/ebnf-frontend-architecture.md`](docs/knowledge/ebnf-frontend-architecture.md)

### ebnf-single-source-of-truth
_The EBNF is the single source of truth for the accepted language — out-of-band post-parse validators break the generator⟷parser duality_

- **answers:** how can the stimuli generator produce strings the parser rejects | why does a grammar-driven generator emit parser-rejected output | what makes generation and parsing inconsistent in PGEN | what is an out-of-band acceptance validator and why is it a defect | why do regex \\u{...} and (*verb) generate but fail to re-parse | where does regex_compile_validation.rs fit and why is it invisible to the generator | how do I debug what the stimuli generator is deriving
- **date:** 2026-06-07 · **status:** current
- **evidence:** `tool-backed 2026-06-07 (PGEN-EBNF-SOT-0001). parseability_probe: `\\u{b7a2}` -> 'unsupported regex escape \\u'; `(*xjDD)` -> 'unrecognized PCRE2 verb or start option'; `\\x{b7a2}` / `(?|a)` / `(?P>n)` / `(?(1)a)` PARSE. EBNF structurally accepts the failing forms: grammars/regex.ebnf `unicode_escape = \"u{\" hex_digits \"}\"`, `directive_verb = \"(*\" directive_body \")\"`. Rejection source: rust/src/regex_compile_validation.rs (validate_regex_compile_contract). grep: stimuli_generator.rs references it 0x; regex.ebnf encodes it as @predicate/@generate 0x.`
- **reverify:** ``./rust/target/debug/parseability_probe --parse regex <(printf '\\\\u{b7a2}')`; `grep -n 'unsupported regex escape' rust/src/regex_compile_validation.rs`; `grep -c 'regex_compile_validation' rust/src/ast_pipeline/stimuli_generator.rs` (expect 0); `grep -nE 'unicode_escape|directive_verb' grammars/regex.ebnf``
- **source:** [`docs/knowledge/ebnf-single-source-of-truth.md`](docs/knowledge/ebnf-single-source-of-truth.md)

### grammar-coverage-and-directed-generation
_Don't reinvent stimuli coverage / targeted generation — k-path, Tribble, FDLOOP, Boltzmann_

- **answers:** what is the literature for grammar-based test/stimuli generation coverage | how to generate an input that reaches a specific grammar production (directed) | what coverage metric for grammar-based generation (k-path) | how to generate deep witnesses without timeout (.7.4.6) | is there a published technique for the literal-0 / derivation-directed generation idea
- **date:** 2026-06-03 · **status:** current
- **evidence:** `Purdom 1972 sentence generator; Havrikov & Zeller, Systematically Covering Input Structure (k-paths), ASE 2019 + tool Tribble; Kirschner & Soremekun, Directed Grammar-Based Test Generation (FDLOOP), arXiv 2508.01472 (2025); Boltzmann samplers (Duchon et al. 2004; USAIN BOLTZ); EMI (PLDI 2014)`
- **reverify:** `see docs/tasks/SV-EXH-PROOF.md (.7.4.x) + docs/tasks/STIMULI-SIGNOFF.md`
- **source:** [`docs/knowledge/grammar-coverage-and-directed-generation.md`](docs/knowledge/grammar-coverage-and-directed-generation.md)

### grammar-linter-trustworthiness
_Trusting the grammar linter — sound-not-complete, certifying (witness/proof), attribution rule_

- **answers:** can we trust the grammar linter 100% | is the grammar linter's reachability answer always correct | is the grammar linter trustworthy | is exact grammar reachability decidable | why is the grammar linter sound but not complete | what does UNKNOWN mean in the grammar linter | what happens when the stimuli generator cannot reach a grammar fragment | how does PGEN decide if an uncovered branch is a generator bug or a grammar bug | what is the attribution rule | what is a certifying linter or certificate | what is a witness vs a proof in the grammar linter | how is a grammar fully certified | why is literal-0 coverage a theorem not a target
- **date:** 2026-06-06 · **status:** current
- **evidence:** `docs/decisions/feedback_certifying_linter_trustworthiness.md + feedback_unreachable_target_attribution_rule.md; docs/tasks/GRAMMAR-WELLFORMED.md Phase G/H; rust/src/ast_pipeline/grammar_wellformedness.rs (verify_unreachability_certificate, UnreachabilityCertificate); book docs/book/src/grammar-wellformedness.md`
- **reverify:** `grep -n 'verify_unreachability_certificate\\|UnreachabilityCertificate\\|EarlierArmAlwaysSucceeds' rust/src/ast_pipeline/grammar_wellformedness.rs`
- **source:** [`docs/knowledge/grammar-linter-trustworthiness.md`](docs/knowledge/grammar-linter-trustworthiness.md)

### makefile-targets-reference
_rust/Makefile — the targets you actually reach for (regen, gates, clippy, book)_

- **answers:** what Makefile targets exist and which do I use | how do I regenerate a parser from its grammar | how do I regenerate the SystemVerilog parser after a grammar edit | how do I run the SV stimuli quality gate or the external-corpus triage | how do I run clippy the project-sanctioned way | how do I run the mdbook docs gate | where are the gate shell scripts and how do I run one directly
- **date:** 2026-06-04 · **status:** current
- **evidence:** `rust/Makefile (phony targets); rust/scripts/*.sh (gate implementations)`
- **reverify:** ``grep -oE "^[a-zA-Z_][a-zA-Z0-9_]*:" rust/Makefile | sort -u` for the live target list; `ls rust/scripts/*.sh` for gates`
- **source:** [`docs/knowledge/makefile-targets-reference.md`](docs/knowledge/makefile-targets-reference.md)

### parse-completeness-differential-oracle
_Don't reinvent the no-reject oracle — differential testing + ready-made SV corpus_

- **answers:** how do we prove the parser never rejects valid input | what reference parsers / corpus to use for SystemVerilog differential testing | is there a ready-made SV compliance test suite | what is the literature for parser completeness testing
- **date:** 2026-06-03 · **status:** current
- **evidence:** `McKeeman 1998 Differential Testing; Csmith (Yang et al. PLDI 2011); EMI (Le/Afshari/Su PLDI 2014); Lammel Grammar Testing FASE 2001; Grammar Mutation TOSEM 2025; slang sv-lang.com; Verible; chipsalliance sv-tests`
- **reverify:** `see docs/tasks/PARSE-COMPLETENESS.md literature grounding`
- **source:** [`docs/knowledge/parse-completeness-differential-oracle.md`](docs/knowledge/parse-completeness-differential-oracle.md)

### parse-fidelity-oracles
_Don't reinvent the no-mis-parse oracle — invertible syntax, metamorphic/EMI, round-trip_

- **answers:** how do we detect mis-parse (accepted but wrong AST) | what is the test oracle for parser fidelity / correct AST | what is invertible syntax / bidirectional parsing pretty-printing | how to test parser correctness without a reference AST
- **date:** 2026-06-03 · **status:** current
- **evidence:** `Rendel & Ostermann, Invertible Syntax Descriptions, Haskell 2010 (invertible-syntax/partial-isomorphisms); Metamorphic Testing (Chen/Cheung/Yiu 1998); EMI (Le/Afshari/Su PLDI 2014); PGEN round_trip_tests.rs + ast_shape_contract manifests`
- **reverify:** `see docs/tasks/PARSE-FIDELITY.md literature grounding`
- **source:** [`docs/knowledge/parse-fidelity-oracles.md`](docs/knowledge/parse-fidelity-oracles.md)

### parser-signoff-four-pillars
_The four parser sign-off pillars (reject / hang / mis-parse + coverage)_

- **answers:** what are the remaining problem types for the SV parser | how is parser correctness decomposed / the four pillars | which task tree owns parser reject / hang / mis-parse | is the parser failing on real inputs or is it the stimuli generator | what is the difference between parser failures and the stimuli residual
- **date:** 2026-06-03 · **status:** current
- **evidence:** `docs/tasks/PARSE-COMPLETENESS.md, PARSE-TERMINATION.md, PARSE-FIDELITY.md, STIMULI-SIGNOFF.md, SV-EXH-PROOF.md (.7.4); live SV status = external corpus 14/14, parser clean on tracked input`
- **reverify:** `grep -l "parser sign-off pillar" docs/tasks/PARSE-*.md`
- **source:** [`docs/knowledge/parser-signoff-four-pillars.md`](docs/knowledge/parser-signoff-four-pillars.md)

### pgen-parsing-model
_What PGEN is w.r.t. PEG / Packrat / data-dependent grammars (precise placement)_

- **answers:** what is PGEN with respect to PEG and Packrat | is PGEN a PEG or a CFG parser generator | what is PEG / what is Packrat / what is a data-dependent grammar | does PGEN handle left recursion automatically or must the author eliminate it | is PGEN stateless or stateful packrat | where does PGEN stand in the parsing literature
- **date:** 2026-06-03 · **status:** current
- **evidence:** `docs/tasks/PARSE-SOTA-research-synthesis.md; Ford POPL 2004 (PEG) + ICFP 2002 (packrat); SPEG/Nez; Yakker (data-dependent); Laurent & Mens SLE 2016 (stateful delta); code: ast_pipeline/mod.rs:1488/1523/1588/1640, main.rs:309/900/1828`
- **reverify:** `grep -n "eliminate_left_recursion\|eliminate_left_recursive_patterns" rust/src/ast_pipeline/mod.rs rust/src/main.rs`
- **source:** [`docs/knowledge/pgen-parsing-model.md`](docs/knowledge/pgen-parsing-model.md)

### rgx-0078-regex-slowness-followup
_QUEUED — after SV reaches Done, attempt RGX-0078 (regex parser slowness vs PCRE2)_

- **answers:** what to do after the SV main parser reaches Done | what is PGEN-RGX-0078 / the regex parser slowness follow-up | how to speed up the pgen regex parser | what is the closure criterion for regex parse performance | is the SV stateful-packrat fix relevant to regex slowness
- **date:** 2026-06-03 · **status:** current
- **evidence:** `docs/decisions/project_rgx_0078_regex_slowness_followup.md; /Users/richarddje/Documents/github/rgx/pgen-issues/PGEN-RGX-0078.yaml (issue, in the rgx repo)`
- **reverify:** `cat the RGX issue yaml; run the pgen_iteration_flow harness (PCRE2-relative bench)`
- **source:** [`docs/knowledge/rgx-0078-regex-slowness-followup.md`](docs/knowledge/rgx-0078-regex-slowness-followup.md)

### stateful-packrat-not-linear
_WARNING — PGEN's stateful packrat is NOT guaranteed linear (semantic store = state)_

- **answers:** is PGEN's parser guaranteed linear time / hang-free | does packrat memoization guarantee no catastrophic backtracking here | why might the SV parser hang or go exponential | what is the fix for stateful packrat non-linearity | does the semantic store affect parse-time complexity
- **date:** 2026-06-03 · **status:** current
- **evidence:** `Ford Packrat Parsing ICFP 2002 (linear ONLY for stateless PEG); Chida & Kawakoya, Is Stateful Packrat Parsing Really Linear in Practice?, CC 2020 (counter-example + conditional-memoization fix, 260x/217x)`
- **reverify:** `see docs/tasks/PARSE-TERMINATION.md; grep -n "memoiz" rust/src/ast_pipeline/*.rs`
- **source:** [`docs/knowledge/stateful-packrat-not-linear.md`](docs/knowledge/stateful-packrat-not-linear.md)

### stimuli-generation-error-reasons
_Stimuli generation error-reason taxonomy (and severity is never gated by verbosity)_

- **answers:** what do the stimuli generation error reasons mean | what is GenerationErrorReason / classify_generation_error | how do I tell a depth failure from a visit-limit or timeout failure | where are depth_exceeded_errors / target_timeout_errors counted | are warnings and errors hidden at low verbosity
- **date:** 2026-06-03 · **status:** current
- **evidence:** `rust/src/ast_pipeline/stimuli_generator.rs (GenerationErrorReason, classify_generation_error, *_errors counters, TargetDriveSummary); rust/src/ast_pipeline/mod.rs (Severity, emit_diagnostic, pgen_warn!/error!/fatal!); docs/tasks/DIAG-SEVERITY.md`
- **reverify:** `grep -n "GenerationErrorReason\|classify_generation_error" rust/src/ast_pipeline/stimuli_generator.rs`
- **source:** [`docs/knowledge/stimuli-generation-error-reasons.md`](docs/knowledge/stimuli-generation-error-reasons.md)

### stimuli-generator-capability-gaps
_Stimuli generator — what it has vs the literature signoff bar (6 gaps)_

- **answers:** does the stimuli generator have all the necessary features | what is missing from the stimuli generator vs the literature | what are the stimuli generator capability gaps | is PGEN's generator behind or ahead of academic grammar fuzzers | what should a signoff-grade EBNF stimuli generator do that ours doesn't
- **date:** 2026-06-03 · **status:** current
- **evidence:** `grep stimuli_generator.rs (reach_plan, StimuliCoverageTarget, min_terminal/purdom, {mutation,constraint,negative,recovery}_mode, shrink); literature sweep 2026-06-03; docs/tasks/STIMULI-SIGNOFF.md (.1 audit)`
- **reverify:** `see docs/tasks/STIMULI-SIGNOFF.md leaves .2-.7`
- **source:** [`docs/knowledge/stimuli-generator-capability-gaps.md`](docs/knowledge/stimuli-generator-capability-gaps.md)

### stimuli-generator-construction-path
_How main.rs builds the StimuliGenerator from a grammar + runs the witness pass (production path)_

- **answers:** how does main.rs construct the stimuli generator from the ebnf | how is the StimuliGenerator built from a grammar | what is the production stimuli-generation path | how do I build a StimuliGenerator programmatically | how is a grammar loaded into the generator | what is LoadedGrammar | how does the witness pass get invoked | how do I run generate_target_witnesses | where are grammar_tree and rule_order available in main.rs
- **date:** 2026-06-06 · **status:** current
- **evidence:** `rust/src/main.rs (StimuliGenerator::new at :1083/:1187/:1261/:2194; generate_gap_report at :1210/:1661; generate_target_witnesses at :1474); rust/src/ast_pipeline/stimuli_generator.rs (StimuliGenerator::new, generate_gap_report, generate_target_witnesses, witness_certificates)`
- **reverify:** `grep -n 'StimuliGenerator::new\\|generate_gap_report\\|generate_target_witnesses' rust/src/main.rs`
- **source:** [`docs/knowledge/stimuli-generator-construction-path.md`](docs/knowledge/stimuli-generator-construction-path.md)

### stimuli-residual-coverage-model
_What the stimuli residual / replay_target_count actually is (coverage model)_

- **answers:** what is replay_target_count / the stimuli residual | what does the stimuli coverage gap report measure | what are never_hit / never_selected / selected_but_failed reasons | what is focused_replay_target_debt_zero / literal-0 | how does the generator steer to a specific coverage target
- **date:** 2026-06-03 · **status:** current
- **evidence:** `rust/src/ast_pipeline/stimuli_generator.rs (StimuliCoverageTarget {Rule,Branch}, generate_gap_report, compute_reach_path, forced_or_branch_for_site, set_reach_plan); docs/tasks/SV-EXH-PROOF.md`
- **reverify:** `grep -n "StimuliCoverageTarget\|replay_target\|reach_classification" rust/src/ast_pipeline/stimuli_generator.rs`
- **source:** [`docs/knowledge/stimuli-residual-coverage-model.md`](docs/knowledge/stimuli-residual-coverage-model.md)

### sv-corpus-gate-uvm-memory
_The SV corpus gate's uvm parse can consume ~26 GB RAM and appear to hang — cap it_

- **answers:** why does the SV external corpus gate eat huge RAM or seem stuck | why does parsing uvm_pkg / uvm_compat_pkg use tens of GB of memory | is the SV parser slow / memory-heavy on uvm and why | how to run the SV corpus gate safely without exhausting host RAM
- **date:** 2026-06-05 · **status:** current
- **evidence:** `docs/tasks/PARSE-TERMINATION.md leaf .3.2 (PGEN-PARSE-TERMINATION-0005); .1 (N^1.66 super-linear) + .3 (O(N^2) SemanticRuntimeState clone)`
- **reverify:** ``ulimit -v 12582912; target/debug/parseability_probe --parse systemverilog <uvm_pkg.preprocessed.sv> --profile 2017` and watch RSS — slow, and memory climbs super-linearly deeper in the parse`
- **source:** [`docs/knowledge/sv-corpus-gate-uvm-memory.md`](docs/knowledge/sv-corpus-gate-uvm-memory.md)

### sv-literal-0-residual-decomposition
_The SV literal-0 residual — what it is, why it is not yet 0, and the two named causes_

- **answers:** why is the SystemVerilog main parser still Mostly Done not Done | what is the closed-loop replay residual / focused_replay_target_debt_zero | why is the SV stimuli residual not literal-0 | what is the property_expr construction-cost residual | what are the non-covering branch targets in SV stimuli coverage | what does the witness construction speedup (Cow) do
- **date:** 2026-06-04 · **status:** current
- **evidence:** `rust/src/ast_pipeline/stimuli_generator.rs (generate_target_witnesses construct_mode + strip_probability_prefix Cow + timeout_failure_samples/unresolved_after_samples); docs/tasks/SV-EXH-PROOF.md leaves .7.4.6.3/.4/.5/.6 (PGEN-SV-EXH-PROOF-0148..0151)`
- **reverify:** `run the witness pass (--target-report-input <150-sample> --target-max-attempts 0 --target-generation-timeout-ms 200 --seed 712001 --output /tmp/o.json) and read the "Witness 'target_timeout'-failure samples" + "still-UNRESOLVED samples" stdout lines; or `make sv_stimuli_quality_gate` for closed_loop_replay_targets_total`
- **source:** [`docs/knowledge/sv-literal-0-residual-decomposition.md`](docs/knowledge/sv-literal-0-residual-decomposition.md)

### sv-residual-depth-budget-cause
_Why the SV stimuli residual doesn't reach literal zero — depth-budget exhaustion_

- **answers:** why doesn't the SystemVerilog stimuli residual reach zero | why does replay_target_count plateau (e.g. at 888) | what is the root cause of the uncovered SV coverage branches | what does the Stimuli generation depth exceeded message mean | why can't deeply-nested SV rules be generated from the top entry
- **date:** 2026-06-03 · **status:** current
- **evidence:** `rust/src/ast_pipeline/stimuli_generator.rs (depth gate ~:4377, near-limit guard ~:4509/:5413, depth-slack retry ~:4869); docs/tasks/SV-EXH-PROOF.md leaf .7.4.3a (PGEN-SV-EXH-PROOF-0140)`
- **reverify:** `grep -n "max_depth\|depth exceeded\|DEPTH_EXCEEDED" rust/src/ast_pipeline/stimuli_generator.rs`
- **source:** [`docs/knowledge/sv-residual-depth-budget-cause.md`](docs/knowledge/sv-residual-depth-budget-cause.md)

### sv-witness-purdom-ordering
_The SV residual tail is slow witness generation — Purdom ordering (witness-mode) fixes it_

- **answers:** why do SV witnesses time out instead of erroring | what is witness_mode / the Purdom witness ordering | how were the slow witness timeouts reduced | what does PGEN_WITNESS_NO_PURDOM do | how to make slow deeply-factored SV witnesses converge
- **date:** 2026-06-03 · **status:** current
- **evidence:** `rust/src/ast_pipeline/stimuli_generator.rs (generate_or witness arm + generate_target_witnesses witness_mode/witness_min_terminal_lengths); docs/tasks/SV-EXH-PROOF.md leaf .7.4.4 (PGEN-SV-EXH-PROOF-0143)`
- **reverify:** `PGEN_WITNESS_NO_PURDOM=1 vs unset, rerun the witness pass (--target-report-input <sample> --target-max-attempts 0 --target-generation-timeout-ms 7000 --seed 712001) and compare resolved / target_timeout`
- **source:** [`docs/knowledge/sv-witness-purdom-ordering.md`](docs/knowledge/sv-witness-purdom-ordering.md)

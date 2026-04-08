# Source Map

This chapter maps the current authoritative docs into the book structure so future migration work stays intentional.

## Book Chapter To Source-Doc Map

### Platform overview

- `README.md`

### Getting started

- `README.md`
- `QUICKSTART_AI_ONBOARDING.md`
- `SESSION_BOOTSTRAP.md`
- `PGEN_USER_GUIDE.md`

### User-facing surfaces

- `PGEN_USER_GUIDE.md`
- `rust/docs/EMBEDDING_API_CONTRACT.md`

### Parser families and contracts

- `docs/contracts/PGEN_PARSER_INTEGRATION_CONTRACTS.md`
- `docs/contracts/PGEN_REGEX_PARSER_INTEGRATION_CONTRACT.md`
- `docs/contracts/PGEN_SYSTEMVERILOG_PARSER_INTEGRATION_CONTRACT.md`
- `docs/contracts/PGEN_SYSTEMVERILOG_PREPROCESSOR_PARSER_INTEGRATION_CONTRACT.md`
- `docs/contracts/PGEN_VHDL_PARSER_INTEGRATION_CONTRACT.md`
- `docs/contracts/PGEN_RETURN_ANNOTATION_PARSER_INTEGRATION_CONTRACT.md`
- `docs/contracts/PGEN_SEMANTIC_ANNOTATION_PARSER_INTEGRATION_CONTRACT.md`
- `docs/contracts/PGEN_PARSER_ISSUE_REPORTING_PROTOCOL.md`
- `docs/contracts/PGEN_RELEASED_PARSER_BUG_LEDGER.md`

### Stimuli and quality

- `docs/reference/PGEN_STIMULI_MODULE_NORMATIVE_SPEC.md`
- `docs/reference/PGEN_SEMANTIC_STEERING_CONTROL_MATRIX.md`
- `docs/reference/STRESS_TEST_STANDARDIZATION.md`
- `regex_corpus_bundle/README.md`

### Developer architecture

- `docs/reference/RUST_CODEBASE_ANALYSIS.md`
- `docs/AST_GENERATOR_ARCHITECTURE.md`
- `docs/ast_transformation_pipeline.md`
- `docs/BOOTSTRAP_MODE_SPECIFICATION.md`
- `docs/EBNF_INCLUDE_SYSTEM.md`
- `docs/parser_architecture_evolution.md`
- `docs/RETURN_ANNOTATIONS_REFERENCE.md`
- `docs/TEST_INFRASTRUCTURE.md`

### Operations and governance

- `LIVE_ACHIEVEMENT_STATUS.md`
- `CHANGES.md`
- `DEVELOPMENT_NOTES.md`
- `MEMORY.md`
- `SESSION_BOOTSTRAP.md`
- `COMMIT.md`
- `docs/reference/PGEN_RELEASE_POLICY.md`
- `docs/reference/PGEN_SOTA_IMPLEMENTATION_ROADMAP.md`

## Migration Intention

This map is the first live bridge between the current markdown surface and a curated book surface.

As the project evolves, the book can absorb more material directly, but it should stay curated and readable. The goal is not to dump the whole repository into a navigation tree; the goal is to give users and developers a layered path to mastery.

# Source Map

This chapter maps the current authoritative docs into the book structure so future migration work stays intentional.

## Book Chapter To Source-Doc Map

### Documentation model

- `README.md`
- `COMMIT.md`
- `SESSION_BOOTSTRAP.md`
- `docs/book/src/how-to-use-this-book.md`
- `docs/book/src/operations-and-governance.md`

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

### CLI and workflows

- `PGEN_USER_GUIDE.md`
- `README.md`
- `rust/docs/CLI_REFERENCE.md`
- `rust/scripts/ci_workflow_local_gate.sh`

### Annotation system

- `docs/reference/PGEN_ANNOTATION_NORMATIVE_SPEC.md`
- `docs/RETURN_ANNOTATIONS_REFERENCE.md`
- `docs/contracts/PGEN_RETURN_ANNOTATION_PARSER_INTEGRATION_CONTRACT.md`
- `docs/contracts/PGEN_SEMANTIC_ANNOTATION_PARSER_INTEGRATION_CONTRACT.md`

### Embedding and downstream integration

- `rust/docs/EMBEDDING_API_CONTRACT.md`
- `docs/contracts/PGEN_PARSER_INTEGRATION_CONTRACTS.md`
- `docs/contracts/PGEN_PARSER_ISSUE_REPORTING_PROTOCOL.md`
- `docs/contracts/PGEN_RELEASED_PARSER_BUG_LEDGER.md`
- `docs/contracts/PGEN_REGEX_PARSER_INTEGRATION_CONTRACT.md`

### Roadmap and live status

- `LIVE_ACHIEVEMENT_STATUS.md`
- `docs/reference/PGEN_SOTA_IMPLEMENTATION_ROADMAP.md`
- `README.md`

### Quality and closure model

- `README.md`
- `LIVE_ACHIEVEMENT_STATUS.md`
- `docs/reference/PGEN_SOTA_IMPLEMENTATION_ROADMAP.md`
- `docs/contracts/PGEN_PARSER_INTEGRATION_CONTRACTS.md`

### Parser families and contracts

- `PGEN_USER_GUIDE.md`
- `docs/reference/REGEX_BOOTSTRAP_ARCHITECTURE.md`
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

This map is the live bridge between the current markdown surface and the public book surface.

As the project evolves, the book should absorb more of the real project surface directly, while staying readable and structured. The goal is not to dump the whole repository into a navigation tree; the goal is to make the book comprehensive enough that external readers can understand PGEN there without having to mine internal continuity docs.

That also means the source map is not only a migration checklist. It is a coverage checklist for the public documentation system: if an important subsystem or doctrine still has no clear chapter home, the book is still missing part of the platform.

# Parser Families

PGEN applies one quality doctrine across all EBNF-based parser families. The live tracker differs by landed proof depth, not by quality bar.

## Mature Or Near-Mature Families

### Regex

- active downstream consumer: RGX
- strong published contract surface
- repeated maintenance releases and bug-response workflow

Primary sources:

- `docs/contracts/PGEN_REGEX_PARSER_INTEGRATION_CONTRACT.md`
- `docs/contracts/PGEN_RELEASED_PARSER_BUG_LEDGER.md`
- `regex_corpus_bundle/README.md`

### VHDL

- tracked as a closed parser family in the live status view
- important regression sentinel for cross-family stimuli work

Primary source:

- `docs/contracts/PGEN_VHDL_PARSER_INTEGRATION_CONTRACT.md`

### SystemVerilog

- still one of the deepest proof surfaces in the repository
- main parser remains an active closure target
- preprocessor parser is tracked as done

Primary sources:

- `docs/contracts/PGEN_SYSTEMVERILOG_PARSER_INTEGRATION_CONTRACT.md`
- `docs/contracts/PGEN_SYSTEMVERILOG_PREPROCESSOR_PARSER_INTEGRATION_CONTRACT.md`
- `docs/reference/SV_GRAMMAR_COVERAGE_MATRIX.md`

## Annotation Families

These are core platform grammars, not side utilities:

- return annotation parser
- semantic annotation parser

Primary sources:

- `docs/contracts/PGEN_RETURN_ANNOTATION_PARSER_INTEGRATION_CONTRACT.md`
- `docs/contracts/PGEN_SEMANTIC_ANNOTATION_PARSER_INTEGRATION_CONTRACT.md`
- `docs/reference/PGEN_ANNOTATION_NORMATIVE_SPEC.md`
- `docs/RETURN_ANNOTATIONS_REFERENCE.md`

## Phase S Families

Ongoing Phase S work currently centers around:

- `rtl_const_expr`
- `rtl_frontend`

These matter because they push PGEN from parsing into more elaboration-oriented RTL front-end territory while staying inside the same proof-first doctrine.

The `rtl_frontend` generated-parser proof surface is still not fully closed, but it is no longer just a toy syntax lane. Its curated generated contract now covers representative generate/dataflow cases, mixed procedural/dataflow member-path cases, procedural/dataflow ternary assignment values plus richer plain-`always @(*)`, `always_latch`, sequential/procedural/dataflow ranged and concatenated assignment targets, generated `always_ff` blocking-assignment rejection, `always_latch` event-control rejection, ranged/concatenated assignment-target near-miss rejects, an instance-array/wildcard-port case, ordered parameter/port actuals with repeat-concatenation values, deeper ordered actuals with comma-bearing repeat-concatenation member ranges, ordered/named parameter overrides and ordered/named port actuals with ternary/binary expressions, named parameter overrides and named port actuals with repeat-concatenation range expressions, ternary and repeat/list near-miss rejects, and homogeneous named/ordered override and port-list rejects in addition to earlier reduced syntax samples.

For exact current status, always check:

- `LIVE_ACHIEVEMENT_STATUS.md`
- `docs/reference/PGEN_SOTA_IMPLEMENTATION_ROADMAP.md`

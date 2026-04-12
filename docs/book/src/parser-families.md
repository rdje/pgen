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

The `rtl_frontend` generated-parser proof surface is still not fully closed, but it is no longer just a toy syntax lane. Its curated generated contract now covers representative generate/dataflow cases including empty no-port multi-module declarations, module-local parameter/localparam items, generate-if/else local net declarations, generate-for local net declarations, generate if/else dataflow, mixed procedural/dataflow member-path cases, single- and multi-module file-scope, local, package-qualified, wildcard-imported, and named-imported struct typedef surfaces, local enum/union typedef surfaces, inline struct typed nets, handwritten-baseline `byte` union field-name surfaces, builtin integral typed nets and inline enum base-type forms, header-imported struct/enum/union typedef ports, package-qualified/header-wildcard-imported/module-named-imported constant parameter and range flows, procedural/dataflow ternary assignment values plus labeled parameter-expression `always_comb` procedural blocks, packed multi-net declarations, scalar `always @(*)` if/else blocks, scalar nonblocking `always_latch` blocks plus syntax-only unknown-body-identifier latch parsing, isolated `always_ff` struct-member bit-select nonblocking targets, isolated `always_ff` struct-member concatenation values, syntax-only `always_ff` event-control identifier parse surfaces, isolated `always_comb` struct-member concatenated assignment targets, syntax-only unknown-member continuous assignment target/value and concatenated-target parse surfaces, isolated continuous struct-member bit-select assignment targets, isolated continuous struct-member concatenated assignment targets, isolated continuous struct-member concatenation values, and richer plain-`always @(*)`, `always_latch`, sequential/procedural/dataflow ranged and concatenated assignment targets, generated isolated scalar and ranged/member `always_ff` blocking-assignment rejections, `always_latch` event-control rejection, ranged/concatenated assignment-target near-miss rejects including lane-local plain-`always @(*)` / `always_latch` ranged/member and concatenated-target rejects, scalar named-parameter-override/named-port module instantiations, scalar wildcard-port module instantiations, named-port, parameterized named-port, and wildcard-port instance-array cases, named-port union-member actuals plus syntax-only unknown union-member actuals, named-port bit-select/concatenation actuals, named-port member bit-select/repetition actuals, ordered parameter/port actuals with repeat-concatenation values, deeper ordered actuals with comma-bearing repeat-concatenation member ranges, ordered/named parameter overrides and ordered/named port actuals with ternary/binary expressions including named-port member-path ternaries, named parameter overrides and named port actuals with repeat-concatenation range expressions, ternary and repeat/list near-miss rejects, and homogeneous named/ordered override and port-list rejects in addition to earlier reduced syntax samples.

It now also retains syntax-only unindexed unpacked-array, unknown typedef-backed struct-member, unknown parent-identifier named-port actuals, and inline plus typedef-backed packed-union field-width mismatch declarations. Those lanes prove parser acceptance for `child u_child (.a(cfgs.data), .y(y));`, `child u_child (.a(cfg.missing), .y(y));`, `child u_child (.a(missing_signal), .y(y));`, `union packed { logic [7:0] data; logic [15:0] word; } payload;`, and `typedef union packed { logic [7:0] data; logic [15:0] word; } payload_t; payload_t payload;` while keeping the corresponding semantic rejections in elaboration.

For exact current status, always check:

- `LIVE_ACHIEVEMENT_STATUS.md`
- `docs/reference/PGEN_SOTA_IMPLEMENTATION_ROADMAP.md`

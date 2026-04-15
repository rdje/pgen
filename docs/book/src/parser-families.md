# Parser Families

PGEN applies one quality doctrine across all EBNF-based parser families. The live tracker differs by landed proof depth, not by quality bar.

## Mature Or Near-Mature Families

### Regex

- active downstream consumer: RGX
- strong published contract surface
- repeated maintenance releases and bug-response workflow
- PCRE2-conformance work is source-of-truth driven: prose docs explain intent, `pcre2_compile.c` resolves edge cases, and upstream `testdata/testinput*` provides the executable regression oracle
- the current maintenance track includes generated-host depth resilience for legal PCRE2 inputs with deep capture nesting, backreference depth, and grammar-like recursive named-group patterns
- the regex surface is intentionally split between `regex.ebnf` syntax and generated-host compile-contract checks, so PCRE2 source-derived compile rules such as malformed short Unicode property escapes, empty quoted class regions, unbounded lookbehind, malformed named references, invalid verb usage, forbidden class escapes, `\K` in lookarounds, and scan-substring reference existence are documented and gated without forcing them into grammar productions that would overfit the implementation
- the current public handoff is regex parser release `1.1.23` / integration contract `1.1.25`
- the current public handoff includes short PCRE2 Unicode property escapes such as `\pL` / `\PN`, quoted class literals such as `[z\Qa-d]\E]`, bounded variable-length lookbehind such as `(?<=a{1,3})b`, PCRE2 control verbs inside lookbehind such as `(?<=a(*ACCEPT)b)c`, Unicode capture names such as `(?'ABáC'...)\g{ABáC}`, and orphan class `\E` as a zero-width `stray_class_end_quote`; these are retained as explicit AST/contract shapes rather than downstream adapters having to recover them from generic escape/literal fallback
- the regex family proof still computes `Done`; the latest target-drive refresh closes `734 -> 0` targets after `5759` target-drive attempts with a documented `stimuli_target_max_attempts=10000` budget, preserving the rule that status closure requires zero final target debt

Primary sources:

- `docs/contracts/PGEN_REGEX_PARSER_INTEGRATION_CONTRACT.md`
- `docs/contracts/PGEN_RELEASED_PARSER_BUG_LEDGER.md`
- `PGEN_USER_GUIDE.md`
- `docs/reference/REGEX_BOOTSTRAP_ARCHITECTURE.md`
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

The `rtl_frontend` generated-parser proof surface is still not fully closed, but it is no longer just a toy syntax lane. Its curated generated contract now covers representative generate/dataflow cases including empty no-port multi-module declarations, module-local parameter/localparam items, generate-if/else local net declarations, generate-for local net declarations with exact retained `generate_region` and `generate_body` spans, generate if/else dataflow with exact retained `generate_region`, `generate_if`, and branch `generate_body` spans, mixed procedural/dataflow member-path cases, single- and multi-module file-scope, local, package-qualified, wildcard-imported, and named-imported struct typedef surfaces, local enum/union typedef surfaces, inline struct typed nets, handwritten-baseline `byte` union field-name surfaces, builtin integral typed nets and inline enum base-type forms, header-imported struct/enum/union typedef ports, package-qualified/header-wildcard-imported/module-named-imported constant parameter and range flows, subset-retained-text hierarchy locks for package-backed constant and generate-contained instantiations, subset-retained-text procedural/dataflow/instance-actual ternary and binary expression spans, text-locked concat-member assignment values plus labeled parameter-expression `always_comb` procedural blocks, packed multi-net declarations, scalar `always @(*)` if/else blocks, scalar nonblocking `always_latch` blocks plus syntax-only unknown-body-identifier latch parsing, item-level single- and dual-edge `always_ff` event-control evidence, isolated `always_ff` struct-member bit-select nonblocking targets, isolated `always_ff` struct-member concatenation values, item-level syntax-only `always_ff` event-control identifier parse surfaces, isolated `always_comb` struct-member concatenated assignment targets, syntax-only unknown-member continuous assignment target/value and concatenated-target parse surfaces, isolated continuous struct-member bit-select assignment targets, isolated continuous struct-member concatenated assignment targets, isolated continuous struct-member concatenation values, and text-locked richer plain-`always @(*)`, `always_latch`, sequential/procedural/dataflow ranged and concatenated assignment targets, generated isolated scalar and ranged/member `always_ff` blocking-assignment rejections, `always_latch` event-control rejection, ranged/concatenated assignment-target near-miss rejects including lane-local plain-`always @(*)` / `always_latch` ranged/member and concatenated-target rejects, scalar named-parameter-override/named-port module instantiations, scalar wildcard-port module instantiations, named-port, parameterized named-port, and wildcard-port instance-array cases, named-port union-member actuals plus syntax-only unknown union-member actuals, text-locked named-port bit-select/concatenation actuals, text-locked named-port member bit-select/repetition actuals, plain unpacked-array element actuals with malformed empty-index rejection, unpacked-array struct-member bit-select actuals, ordered parameter/port actuals with repeat-concatenation values, deeper ordered actuals with comma-bearing repeat-concatenation member ranges, ordered/named parameter overrides and ordered/named port actuals with ternary/binary expressions including named-port member-path ternaries, named parameter overrides and named port actuals with repeat-concatenation range expressions, ternary and repeat/list near-miss rejects, and homogeneous named/ordered override and port-list rejects in addition to earlier reduced syntax samples.

It now also retains syntax-only unindexed unpacked-array, plain indexed unpacked-array element actuals with malformed empty-index rejection, module-local/file-scope/body-wildcard-import/body-named-import/header-named-import known and inline/module-local-typedef unknown struct-member, unknown parent-identifier named-port actuals, and inline, builtin-integral, plus typedef-backed packed-union field-width mismatch declarations. Those lanes prove parser acceptance for `child u_child (.a(cfgs.data), .y(y));`, `child u_child (.a(banks[IDX]), .y(y));`, `child u_child (.a(cfg.data), .y(y));`, `child u_child (.a(cfg.missing), .y(y));`, `child u_child (.a(missing_signal), .y(y));`, `union packed { logic [7:0] data; logic [15:0] word; } payload;`, `union packed { byte data; shortint word; } payload;`, and `typedef union packed { logic [7:0] data; logic [15:0] word; } payload_t; payload_t payload;` while keeping the corresponding semantic acceptance/rejection decisions in elaboration.

For exact current status, always check:

- `LIVE_ACHIEVEMENT_STATUS.md`
- `docs/reference/PGEN_SOTA_IMPLEMENTATION_ROADMAP.md`

## Future Downstream Parser Requests

PNR has requested future parser support for LEF, DEF, Liberty, SDC, structural Verilog netlists, and SPEF, but those parser-family releases are not shipped yet.

Important source notes:

- `docs/contracts/PGEN_PNR_PARSER_INTEGRATION_CONTRACT.md` records the downstream request, crate-shape target, and authoritative-source acquisition policy.
- `docs/tcl/md/tcl.md` is a local Tcl syntax reference note for the future SDC lane, because SDC is Tcl-shaped at the tokenization/quoting/substitution layer.

Do not treat the Tcl note as a complete SDC grammar or as an implemented parser family. It is reference input for future EBNF work once the authoritative Synopsys SDC source is acquired.

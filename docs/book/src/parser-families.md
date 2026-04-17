# Parser Families

PGEN applies one quality doctrine across all EBNF-based parser families. The live tracker differs by landed proof depth, not by quality bar.

## Mature Or Near-Mature Families

### Regex

- active downstream consumer: RGX
- strong published contract surface
- repeated maintenance releases and bug-response workflow
- PCRE2-conformance work is source-of-truth driven: prose docs explain intent, `pcre2_compile.c` resolves edge cases, and upstream `testdata/testinput*` provides the executable regression oracle
- the current maintenance track includes generated-host depth resilience for legal PCRE2 inputs with deep capture nesting, backreference depth, and grammar-like recursive named-group patterns
- the regex surface is intentionally split between `regex.ebnf` syntax and generated-host compile-contract checks, so PCRE2 source-derived compile rules such as malformed short Unicode property escapes, empty quoted class regions, unbounded lookbehind, malformed named references, invalid verb usage, forbidden class escapes, `\K` in lookarounds, and scan-substring reference existence against the whole pattern are documented and gated without forcing them into grammar productions that would overfit the implementation
- the current public handoff is regex parser release `1.1.26` / integration contract `1.1.28`
- the current public handoff includes short PCRE2 Unicode property escapes such as `\pL` / `\PN`, quoted class literals such as `[z\Qa-d]\E]`, bounded variable-length lookbehind such as `(?<=a{1,3})b`, PCRE2 control verbs inside lookbehind such as `(?<=a(*ACCEPT)b)c`, Unicode capture names such as `(?'ABáC'...)\g{ABáC}`, orphan class `\E` as a zero-width `stray_class_end_quote`, single-code-unit escape `\C` as `single_byte_escape`, callout-prefixed conditional assertions such as `^(?(?C25)(?=abc)abcd|xyz)`, UTF width start-option aliases such as `(*UTF8)`, and scan-substring forward capture references such as `(*scs:(1)a)(a)|x`; these are retained as explicit AST/contract shapes rather than downstream adapters having to recover them from generic escape/literal fallback
- the regex family proof still computes `Done`; the latest target-drive refresh closes `758 -> 0` targets after `5825` target-drive attempts with a documented `stimuli_target_max_attempts=10000` budget, preserving the rule that status closure requires zero final target debt

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

The `rtl_frontend` generated-parser proof surface is still not fully closed, but it is no longer just a toy syntax lane. Its curated generated contract now covers representative generate/dataflow cases including empty no-port multi-module declarations with exact module keyword/name/endkeyword locks, module-local parameter/localparam items with exact statement retained-text locks, unpacked-array port/net dimensional declarations with exact retained text, an integrated handwritten-baseline arithmetic/procedural/generate flow that combines dependent parameters, ANSI port ranges, local parameter statements, continuous ternary dataflow, labeled `always_comb`, generate `if/else`, and generate `for`, an integrated child/generate hierarchy flow that combines direct, generate-if, and generate-for child instantiations with named parameter overrides and named port connections, generate-if/else local net declarations, generate-for local net declarations with exact retained `generate_region` and `generate_body` spans including symbolic-limit non-unit stride syntax, generate if/else dataflow with exact retained `generate_region`, `generate_if`, and branch `generate_body` spans, single-branch generate-if named instantiation with exact retained `generate_region`, `generate_if`, `generate_body`, and full `module_instantiation` spans, mixed procedural/dataflow member-path cases, single- and multi-module file-scope, local, package-qualified, wildcard-imported, and named-imported struct typedef surfaces with exact local/file-scope/module/package typedef, port, and net shell locks, local enum/union typedef surfaces, inline struct/enum/union typed nets with exact aggregate type, datatype/range, module/port, and net-declaration locks, typedef-backed aggregate net declarations with exact named-use, typedef body, datatype/range, module/port, and net-declaration locks, handwritten-baseline `byte` union field-name surfaces, builtin integral typed nets with exact datatype/keyword retained-text locks, inline enum logic base/range retained-text locks, inline enum byte-base retained-text locks, and inline enum base-type forms, header-imported struct/enum/union typedef ports with exact named-type port-shell locks, package-qualified/header-wildcard-imported/module-named-imported constant parameter and range flows with exact statement retained-text locks, exact retained-text hierarchy locks for package-backed constant, unpacked-array struct-member, and generate-contained instantiations, subset-retained-text procedural/dataflow/instance-actual ternary and binary expression spans, text-locked concat-member assignment values plus labeled parameter-expression `always_comb` procedural blocks, packed multi-net declarations, scalar `always @(*)` if/else blocks, scalar nonblocking `always_latch` blocks plus syntax-only unknown-body-identifier latch parsing, item-level single- and dual-edge `always_ff` event-control evidence, isolated `always_ff` struct-member bit-select nonblocking targets, isolated `always_ff` struct-member concatenation values, item-level syntax-only `always_ff` event-control identifier parse surfaces, isolated `always_comb` struct-member concatenated assignment targets, syntax-only unknown-member continuous assignment target/value and concatenated-target parse surfaces, isolated continuous struct-member bit-select assignment targets, isolated continuous struct-member concatenated assignment targets, isolated continuous struct-member concatenation values, and text-locked richer plain-`always @(*)`, `always_latch`, sequential/procedural/dataflow ranged and concatenated assignment targets, generated isolated scalar and ranged/member `always_ff` blocking-assignment rejections, `always_latch` event-control rejection, ranged/concatenated assignment-target near-miss rejects including lane-local plain-`always @(*)` / `always_latch` ranged/member and concatenated-target rejects, scalar named-parameter-override/named-port module instantiations, scalar wildcard-port module instantiations, named-port, parameterized named-port symbolic-range, and wildcard-port instance-array cases, named-port union-member actuals plus syntax-only unknown union-member actuals, text-locked named-port bit-select/concatenation actuals, text-locked named-port member bit-select/repetition actuals, plain unpacked-array element actuals with malformed empty-index rejection, unpacked-array struct-member bit-select actuals, ordered parameter/port actuals with repeat-concatenation values, deeper ordered actuals with comma-bearing repeat-concatenation member ranges, ordered/named parameter overrides and ordered/named port actuals with ternary/binary expressions including named-port member-path ternaries, named parameter overrides and named port actuals with repeat-concatenation range expressions, ternary and repeat/list near-miss rejects, and homogeneous named/ordered override and port-list rejects in addition to earlier reduced syntax samples.

It now also retains syntax-only unindexed unpacked-array, plain indexed unpacked-array element actuals with malformed empty-index rejection, module-local/file-scope/body-wildcard-import/body-named-import/header-named-import known and inline/module-local-typedef unknown struct-member actuals with exact typedef/body locks for the typedef-backed forms, unknown parent-identifier named-port actuals, and inline, builtin-integral, plus typedef-backed packed-union field-width mismatch declarations with exact union-body locks, exact datatype/range or builtin-keyword locks where applicable, and exact typedef locks for the typedef-backed form. Those lanes prove parser acceptance for `child u_child (.a(cfgs.data), .y(y));`, `child u_child (.a(banks[IDX]), .y(y));`, `child u_child (.a(cfg.data), .y(y));`, `child u_child (.a(cfg.missing), .y(y));`, `child u_child (.a(missing_signal), .y(y));`, `union packed { logic [7:0] data; logic [15:0] word; } payload;`, `union packed { byte data; shortint word; } payload;`, and `typedef union packed { logic [7:0] data; logic [15:0] word; } payload_t; payload_t payload;` while keeping the corresponding semantic acceptance/rejection decisions in elaboration.

The named-port union-member actual lane now exact-locks the inline packed-union body and `payload` declaration for both known and unknown member parse surfaces; the known-member sample also exact-locks `payload.data` signal-reference text. This is still generated-parser retained-text proof, not a claim that generated elaboration has closed all union-member legality decisions.

The inline struct-member actual lane now also subset-locks the inline `struct packed { ... }` body for both unindexed unpacked-array and unknown inline-member parse surfaces. The unindexed unpacked-array sample additionally locks `cfgs.data` and `[0:1]`, while the unknown inline-member sample locks `struct packed { ... } cfg;`; these remain parser-retained-text proofs, not semantic legality or elaboration closure claims.

The indexed unpacked-array actual lane now further locks `unpacked_array_struct_member_actual`, `unpacked_array_element_actual`, and `unpacked_array_struct_member_bitselect_actual`: the contract proves member-path actuals such as `cfgs[IDX].data` and `cfgs[IDX].data[BIT]`, the inline struct body, the array declaration `logic [7:0] banks [0:DEPTH-1];`, and the key `[7:0]` / `[0:1]` dimension text. This keeps indexed-array and parameter semantics in elaboration while making the generated parser's retained syntax evidence sharper.

The named-port actual expression lane now tightens `named_port_bitselect_and_concat_actuals`, `named_port_member_bitselect_and_repeat_actuals`, and `named_port_actual_ternary_member_paths` by proving declaration and parameter context around actuals such as `bus[IDX]`, `{a, b}`, `cfg.data[IDX]`, `{LANES{a}}`, and `SEL ? cfg.data : backup.data`. This is still syntax-retention proof; actual expression typing, member legality, and parameter evaluation remain elaboration work.

The continuous struct-member assignment lane now tightens retained evidence around bit-select targets, unknown-member target/value parse surfaces, concatenated targets, and concatenation values. Those samples now prove the inline `struct packed { ... } cfg;` declaration, the simple `input logic d` port context, and `parameter BIT = 1` where present, while preserving the existing exact locks for `assign ...` text, assignment targets, concatenations, and signal references.

The same continuous struct-member assignment lane also now exact-locks the inline field declarations `logic [7:0] data;` and `logic valid;` across the bit-select, unknown-member, concatenated-target, and concatenation-value samples. This gives the generated parser proof a cleaner bridge from aggregate field syntax to member-path assignment syntax.

The generate/dataflow retained-context lane now also proves local declaration and parameter/port/range context around representative generate samples. `generate_if_with_dataflow_and_named_instantiation`, `generate_if_else_with_dataflow`, and `generate_if_else_with_local_net_declarations` exact-lock `logic [7:0] mid;`, `logic mid;`, `[TOTAL-1:0]`, `parameter WIDTH = 8,\n    parameter TOTAL = WIDTH * 2`, and `output logic y`, while subset-locking the ternary dataflow expression `en ? {a[3:0], b[3:0]} : {a[3:0], a[3:0]}` without claiming semantic generate evaluation or elaboration closure.

The symbolic non-unit generate-for lane now exact-locks the retained `generate`, `for`, and `genvar` keyword spans plus `parameter LIMIT = 5` as both the parameter declaration sequence and group. This tightens the parser proof around `for (genvar i = 0; i < LIMIT; i = i + 2)` while leaving generate unrolling and parameter evaluation to elaboration.

The labeled `always_comb` retained-context lane now exact-locks `begin`, `if`, and `else` keyword spans, local parameter/net/range/port context, and selected recursive expression evidence around `labeled_always_comb_block` and `labeled_always_comb_parameter_exprs_and_packed_multi_nets`. This sharpens the parser-retained syntax bridge around labeled procedural blocks without claiming expression typing or elaboration closure.

The module-local parameter/localparam lane now proves the retained parameter-sequence shape for both header parameters and module-body parameter statements. `module_local_parameter_and_localparam_items` exact-locks the header sequence, body `parameter EXTRA = DEPTH + 1`, body `localparam TOTAL = WIDTH * 2`, retained `parameter` / `localparam` keywords, and `output logic [DEPTH-1:0] y`, while subset-locking the relevant arithmetic and ternary expression text without claiming parameter evaluation or expression typing.

The unpacked-array actual lane now carries tighter parameter context around existing array actual proofs. `unpacked_array_struct_member_actual` exact-locks `parameter IDX = 1`, and `unpacked_array_element_actual` exact-locks `parameter DEPTH = 2,\n    parameter IDX = 1` as both parameter sequence and group text, while preserving the existing `cfgs[IDX].data` and `banks[IDX]` retained actual evidence without claiming semantic array elaboration.

For exact current status, always check:

- `LIVE_ACHIEVEMENT_STATUS.md`
- `docs/reference/PGEN_SOTA_IMPLEMENTATION_ROADMAP.md`

## Future Downstream Parser Requests

PNR has requested future parser support for LEF, DEF, Liberty, SDC, structural Verilog netlists, and SPEF, but those parser-family releases are not shipped yet.

Important source notes:

- `docs/contracts/PGEN_PNR_PARSER_INTEGRATION_CONTRACT.md` records the downstream request, crate-shape target, and authoritative-source acquisition policy.
- `docs/tcl/md/tcl.md` is a local Tcl syntax reference note for the future SDC lane, because SDC is Tcl-shaped at the tokenization/quoting/substitution layer.

Do not treat the Tcl note as a complete SDC grammar or as an implemented parser family. It is reference input for future EBNF work once the authoritative Synopsys SDC source is acquired.

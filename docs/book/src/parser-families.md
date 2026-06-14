# Parser Families

PGEN applies one quality doctrine across all EBNF-based parser families. The live tracker differs by landed proof depth, not by quality bar.

## Per-Parser Integration Reference Books

Alongside this platform mastery book, **every PGEN parser has its own live mdBook** — the canonical
AST-integration reference for that parser (envelope shape, worked examples, build recipe, and — for the
shipped families — a per-release changelog). Both the `src/*.md` source and the rendered `*-html/` are
tracked in git, so each book is browsable directly on GitHub without an mdbook install. The `json` book is
explicit that json is a *simplified built-in* grammar, not a conforming JSON parser. Start at each book's
**Welcome** page:

| Parser family | Per-parser book (source) | Rendered HTML | Repo-standard gate |
| --- | --- | --- | --- |
| regex | [PGEN Regex Parser — Integration Reference](../../regex_parser_book/src/welcome.md) | [`docs/regex_parser_book-html/`](../../regex_parser_book-html/welcome.html) | `make -C rust SHELL=/bin/bash regex_parser_book_gate` |
| systemverilog | [PGEN SystemVerilog Parser — Integration Reference](../../systemverilog_parser_book/src/welcome.md) | [`docs/systemverilog_parser_book-html/`](../../systemverilog_parser_book-html/welcome.html) | `make -C rust SHELL=/bin/bash systemverilog_parser_book_gate` |
| systemverilog_preprocessor | [PGEN SystemVerilog Preprocessor Parser — Integration Reference](../../systemverilog_preprocessor_parser_book/src/welcome.md) | [`docs/systemverilog_preprocessor_parser_book-html/`](../../systemverilog_preprocessor_parser_book-html/welcome.html) | `make -C rust SHELL=/bin/bash systemverilog_preprocessor_parser_book_gate` |
| vhdl | [PGEN VHDL Parser — Integration Reference](../../vhdl_parser_book/src/welcome.md) | [`docs/vhdl_parser_book-html/`](../../vhdl_parser_book-html/welcome.html) | `make -C rust SHELL=/bin/bash vhdl_parser_book_gate` |
| rtl_frontend | [PGEN rtl_frontend Parser — Integration Reference](../../rtl_frontend_parser_book/src/welcome.md) | [`docs/rtl_frontend_parser_book-html/`](../../rtl_frontend_parser_book-html/welcome.html) | `make -C rust SHELL=/bin/bash rtl_frontend_parser_book_gate` |
| rtl_const_expr | [PGEN rtl_const_expr Parser — Integration Reference](../../rtl_const_expr_parser_book/src/welcome.md) | [`docs/rtl_const_expr_parser_book-html/`](../../rtl_const_expr_parser_book-html/welcome.html) | `make -C rust SHELL=/bin/bash rtl_const_expr_parser_book_gate` |
| json (built-in, simplified) | [PGEN JSON Parser — Integration Reference](../../json_parser_book/src/welcome.md) | [`docs/json_parser_book-html/`](../../json_parser_book-html/welcome.html) | `make -C rust SHELL=/bin/bash json_parser_book_gate` |

Each per-parser book is paired with the matching downstream **integration contract** under
`docs/contracts/` (the deep authoritative surface) and the family's AST shape-contract manifest under
`rust/test_data/ast_shape_contract/`. The per-family sections below link the relevant book again next to
that family's primary sources.

## Mature Or Near-Mature Families

### Regex

- active downstream consumer: RGX
- strong published contract surface
- repeated maintenance releases and bug-response workflow
- PCRE2-conformance work is source-of-truth driven: prose docs explain intent, `pcre2_compile.c` resolves edge cases, and upstream `testdata/testinput*` provides the executable regression oracle
- the current maintenance track includes generated-host depth resilience for legal PCRE2 inputs with deep capture nesting, backreference depth, and grammar-like recursive named-group patterns
- bare `\NN…` octal-vs-backreference disambiguation is now PCRE2-compliant **at parse time** (downstream report `PGEN-RGX-0084`): single-digit `\1`…`\9` are always numeric back references (unchanged); a two+-digit `\NN…` is a back reference only when ≥ N capturing groups (plain or named) have been opened *up to that source position*, otherwise it re-splits to an octal/literal escape. This is expressed grammar-level via PGEN's parser-agnostic semantic-annotation mechanism (`@emit_fact` capture-group-open markers + a generic `fact_count_at_least` `@predicate`), not a downstream post-parse concern; the regex parser book's escapes chapter has the verified worked-family table
- the `[89]`-leading multi-digit sub-family the above did not cover is now PCRE2-faithful too (downstream report `PGEN-RGX-0087`, family-linked residual of `PGEN-RGX-0084` which stays closed/correct): `\8`/`\9` are not octal digits, so an `\8N`/`\9N` (`N≥10`) run that is not a valid full back reference is neither a valid back reference nor a valid octal escape — PCRE2 (oracle `pcre2test` 10.47) **rejects it at compile**, and PGEN now likewise **hard-rejects** it at parse instead of degrade-resplitting to a single-digit back reference + literals. `[1-7]`-leading runs still degrade to octal (RGX-0084 byte-identical); single-digit `N<10` unchanged. Two negative-lookahead guards in `regex.ebnf` (the proven RGX-0079 idiom), parser-agnostic; accept-set tightening + one corrected classification (`\199`@0-groups), no new shape ⇒ schema stays `1`
- the regex surface is intentionally split between `regex.ebnf` syntax and generated-host compile-contract checks, so PCRE2 source-derived compile rules such as malformed short Unicode property escapes, empty quoted class regions, unbounded lookbehind, malformed named references, invalid verb usage, forbidden class escapes, `\K` in lookarounds, scan-substring reference existence against the whole pattern, plain class `\N`, nonliteral class range endpoints, and decoded escaped endpoint ordering are documented and gated without forcing them into grammar productions that would overfit the implementation
- the current public handoff is regex parser release `1.1.81` / integration contract `1.1.83` (post PGEN-RGX-0081/0082 typed-shape fixes → `1.1.75/1.1.77`, PGEN-RGX-0084 `\NN…` octal-vs-backref disambiguation → `1.1.76/1.1.78`, PGEN-RGX-0085 parenthesis-nesting ceiling → `1.1.77/1.1.79`, PGEN-RGX-0087 `[89]`-leading multi-digit hard-reject → `1.1.78/1.1.80`, PGEN-RGX-0087 **FIX2** scoping that hard-reject to non-character-class context → `1.1.79/1.1.81`, and PGEN-RGX-0087 **FIX2.3** octal `>\377` overflow now rejecting (PCRE2-faithful, both contexts) → `1.1.80/1.1.82` — **`PGEN-RGX-0087` is now fully resolved & closed** (all FIX2 sub-leaves done; RGX PCRE2 ratchet at the report's full target 12,807/3); see the regex parser mdBook's changelog index for the per-release shape changelog. `embedding_api.rs`'s `REGEX_PARSER_RELEASE_VERSION`/`…CONTRACT_VERSION` constants — formerly stale-drifted (`1.1.29`/`1.1.31`), the `PGEN-RGX-0086` report — are kept synced to this ledger-authoritative pair by a ledger-derived drift gate (`regex_parser_pgen_rgx_0086_embedding_version_consts_match_ledger`) that fails deterministically if any future ledger row forgets the const bump; and PGEN-RGX-0088 reverting FIX2.3's mode-blind octal-`>0o377` parse-reject (octal range is mode-dependent; PGEN mode-agnostically emits the octal atom, the mode-aware consumer range-checks) → `1.1.81/1.1.83`; now `1.1.81`/`1.1.83`, ledger `REGEX-0087`)
- REGEX-PCRE2-FIDELITY.3.1 (2026-06-07, surface-neutral — handoff stays `1.1.81`/`1.1.83`) begins migrating the host compile-contract checks INTO `regex.ebnf` so the EBNF is the single source of truth (composes with the EBNF-source-of-truth invariant and the PCRE2-faithful-by-default + `relaxed`-opt-out directive). The first migrated check: the six PCRE2-unsupported escape letters `\i \F \l \L \u \U` (and braced `\u{…}`) — formerly rejected by the out-of-band `find_invalid_escape_i` validator, now rejected by the grammar's strict (default `pcre2`) escape catch-alls in every context (atom / class / class-range) and re-admitted by a new opt-out **`relaxed`** profile (CLI `--grammar-profile relaxed`; the embedding API still exposes only strict `regex_default`). Default-mode accept/reject is byte-identical (`regex_pcre2_compile_oracle_gate` stash-baseline proven); AST shapes unchanged; the rejection diagnostic stays code `E_PARSE_FAILURE` (message text changed — consumers should match on the code). The remaining validator sub-checks + the broader unrecognized-escape whitelist (`\I`, `\J`, …) are tracked as further `REGEX-PCRE2-FIDELITY.3.x` leaves. REGEX-PCRE2-FIDELITY.3.2 (`PGEN-REGEX-PCRE2-0008`, also surface-neutral) then migrated `(*verb)` NAME acceptance the same way: `directive_name` split into a strict default ordered-choice of exactly the recognized PCRE2 verb (`MARK ACCEPT F FAIL COMMIT PRUNE SKIP THEN`) + 26 start-option names (case-sensitive) and a `relaxed` catch-all; the validator's unrecognized-name reject moved into the grammar (its structural checks — MARK-arg, start-option position, `=value`, quantified-ACCEPT — stay). Default `(*FOO)`/`(*MARKX)` rejection is conformance-neutral (oracle byte-identical)
- the current public handoff includes short PCRE2 Unicode property escapes such as `\pL` / `\PN`, quoted class literals such as `[z\Qa-d]\E]`, quoted class range endpoints such as `^[\Qa\E-\Qz\E]+`, bounded variable-length lookbehind such as `(?<=a{1,3})b`, PCRE2 control verbs inside lookbehind such as `(?<=a(*ACCEPT)b)c`, Unicode capture names such as `(?'ABáC'...)\g{ABáC}`, orphan class `\E` as a zero-width `stray_class_end_quote`, single-code-unit escape `\C` as `single_byte_escape`, callout-prefixed conditional assertions such as `^(?(?C25)(?=abc)abcd|xyz)`, UTF width start-option aliases such as `(*UTF8)`, scan-substring forward capture references such as `(*scs:(1)a)(a)|x`, and quoted literals such as `\Qabc\$xyz\E` with retained literal backslash content; these are retained as explicit AST/contract shapes rather than downstream adapters having to recover them from generic escape/literal fallback
- the generated-host compile contract now also keeps `[z-\x{100}]`, `[\000-\037]`, `[a-\377]`, and mixed bare-octal/hex ranges accepted by comparing class range endpoints by decoded codepoint value, while retaining `[\x{100}-z]` and `[\x1f-\0]` as descending-range failures
- the regex family proof still computes `Done`; the latest target-drive refresh closes `804 -> 0` targets after `6526` target-drive attempts with parser-backed stimuli `5911/5197/714` and a documented `stimuli_target_max_attempts=10000` budget, preserving the rule that status closure requires zero final target debt

Primary sources:

- **Per-parser book:** [PGEN Regex Parser — Integration Reference](../../regex_parser_book/src/welcome.md)
- `docs/contracts/PGEN_REGEX_PARSER_INTEGRATION_CONTRACT.md`
- `docs/contracts/PGEN_RELEASED_PARSER_BUG_LEDGER.md`
- `PGEN_USER_GUIDE.md`
- `docs/reference/REGEX_BOOTSTRAP_ARCHITECTURE.md`
- `regex_corpus_bundle/README.md`

### VHDL

- tracked as a closed parser family in the live status view
- important regression sentinel for cross-family stimuli work

Primary sources:

- **Per-parser book:** [PGEN VHDL Parser — Integration Reference](../../vhdl_parser_book/src/welcome.md)
- `docs/contracts/PGEN_VHDL_PARSER_INTEGRATION_CONTRACT.md`

### SystemVerilog

- still one of the deepest proof surfaces in the repository
- main parser remains an active closure target
- preprocessor parser is tracked as done
- the retained focused adapter-backed `sv_2017` and `sv_2023` direct probes now both accept `179/179` targeted samples with `0` parser rejections on the current narrow `timeunits_declaration` plus `line_comment` seam
- the next retained focused replay slice now also uses parser-proven branch-local sample steering rather than regex-only hinting:
  - `sv_2017`: `180/181` accepted, `1` parser rejection, `319/2613` targets resolved in the retained 200-attempt loop
  - `sv_2023`: `179/180` accepted, `1` parser rejection, `387/2393` targets resolved in the retained 200-attempt loop
  - the seeded branches cover selected `assignment_pattern`, `case_statement`, `clocking_declaration`, `conditional_statement`, struct/enum `data_type` surfaces, simple function/task bodies, and `nettype` forms
  - the surviving rejects are still small comment/attribute seams, not the new seeded constructs
- the live row stays conservative until the heavier `sv_stimuli_quality_gate` proof surface is refreshed, so the current story is "focused direct-lane closure on this seam" rather than a full family-status promotion

Primary sources:

- **Per-parser books:** [PGEN SystemVerilog Parser — Integration Reference](../../systemverilog_parser_book/src/welcome.md) · [PGEN SystemVerilog Preprocessor Parser — Integration Reference](../../systemverilog_preprocessor_parser_book/src/welcome.md)
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

## JSON (built-in, deliberately simplified)

`grammars/json.ebnf` is a small **built-in** grammar used for examples and cross-family stimuli work. It
is a **deliberately simplified subset** of JSON, *not* a conforming RFC 8259 / ECMA-404 parser — and PGEN
says so out loud, with evidence. Per the **external-corpus doctrine** (every parser is proven by BOTH the
internal stimuli generator AND an officially-recognized external corpus), the json parser is characterized
against **JSONTestSuite** (Nicolas Seriot's *"Parsing JSON is a Minefield"*, MIT) vendored under
`json_corpus_bundle/`:

- the EBNF-internal duality is clean — json is the first grammar to report `fully_certified=true` from the
  certificate-coverage gate (see [Grammar Well-Formedness](./grammar-wellformedness.md)); but
- the **external** corpus shows the simplified grammar diverges from the standard: of the recognized
  test files it accepts **81/95** must-accept (`y_`) and rejects **158/188** must-reject (`n_`), and **3
  deep-nesting files crash** the recursive-descent parser. The divergences root-cause to the grammar
  lacking number **exponents** and string **escapes**, allowing **leading zeros**, and tolerating loose
  **trailing/whitespace** — grammar-scope limits, not engine bugs — plus a missing recursion/stack guard.

This is the whole point of pairing the two oracles: the generator can only manufacture what the grammar
already describes, so only an independently-authored external corpus exposes the gap between the grammar
and the real language. The full root-caused report and the RFC-8259 upgrade plan live in
`json_corpus_bundle/results/characterization.md`.

Primary sources:

- **Per-parser book:** [PGEN JSON Parser — Integration Reference](../../json_parser_book/src/welcome.md)
- `json_corpus_bundle/README.md` and `json_corpus_bundle/results/characterization.md`
- `grammars/json.ebnf`

## Phase S Families

Ongoing Phase S work currently centers around:

- `rtl_const_expr` — per-parser book: [PGEN rtl_const_expr Parser — Integration Reference](../../rtl_const_expr_parser_book/src/welcome.md)
- `rtl_frontend` — per-parser book: [PGEN rtl_frontend Parser — Integration Reference](../../rtl_frontend_parser_book/src/welcome.md)

These matter because they push PGEN from parsing into more elaboration-oriented RTL front-end territory while staying inside the same proof-first doctrine.

The `rtl_frontend` generated-parser proof surface has now reached its per-family PGEN closure bar — certificate-coverage `fully_certified=true` (`total=169 proof=1 witness=168 UNKNOWN=0`, deterministic at seeds 0/7/42, zero sample-parse failures) plus the green `rtl_frontend_generated_contract_gate` — and its LIVE row is `Done` (leaf `RTL-FE-CLOSURE.8`). It is the sixth fully-certified grammar (after json, regex, rtl_const_expr, systemverilog_preprocessor, and vhdl). Its curated generated contract (typed-AST-era `0.2.0`; parser release `1.0.5` / AST-dump schema `3`) proves, over one 130-sample manifest:

- **parse-acceptance** for all 130 curated samples (98 accepts / 32 rejects) spanning module/port/net declaration shells (including bare ANSI ports `input R` / `output R` / `inout R`, accepted since the `1.0.4` `RTL-FE-CLOSURE.9` fix that gave `port_group` a no-type branch, and keyword-prefixed identifiers such as `input_data` / `reg_file` / `format`, accepted since the `1.0.5` `RTL-FE-CLOSURE.10` keyword word-boundary (`\b`) fix), typed aggregate (struct/enum/union) surfaces, procedural `always_ff` / `always_comb` / `always @(*)` / `always_latch` lanes, generate `if`/`else`/`for` structures, hierarchy/instance-array/parameter-override and package-qualified constant flows, unpacked-array and member-path actuals, and curated near-miss rejects;
- **rule participation** for every accepted parse via the generated parser's transactional coverage record (`required_rule_names` / `forbidden_rule_names`) — the parser's own entry testimony, sound under PEG backtracking and complete under return-annotation folding;
- **retained-text evidence** as 232 curated locks expressed as exact string values of the released schema-3 typed JSON carrier (`required_typed_string_values`).

The `0.1.0` raw-envelope checks (walking the dumped AST for `rule_name`/`span` keys) were retired in the `0.2.0` migration: they predated the rtl_frontend typing campaign and became structurally unsatisfiable once return annotations folded the structural rules into the typed carrier. The manifest itself is the authoritative per-sample detail; the per-lock prose formerly mirrored here described the retired layer and was removed with it.

The same gate also replays that manifest through the handwritten `rtl_frontend` baseline. The current 130-sample manifest has no remaining `expected_handwritten_parse_ok` divergence overrides: generated-parser acceptance/rejection and handwritten `parse_design` acceptance/rejection agree across the curated contract (the handwritten baseline always accepted bare ANSI ports and keyword-prefixed identifiers — it was the intent oracle that flagged the `1.0.4` and `1.0.5` generated-parser acceptance gaps). A ratcheted elaboration replay layer lives in the same manifest through optional `expected_elaboration` entries — 59 curated semantic samples (46 accepts / 13 rejects) covering procedural blocks, hierarchy/instance-array flows, package constants, aggregate member actuals, union-width checks, unknown event/member diagnostics, parameter-override accept and reject lanes, and generate `if`/`else`/`for` elaboration replay — with ratcheted minimums on top-parameter, child-path, child-parameter, and child-port-binding checks, so accepted hierarchy cases prove more than "it elaborated" and non-constant override forms prove they fail in the expected way.

The handwritten baseline is tightened whenever a divergence is really a parse-boundary mismatch rather than a deliberate generated/bootstrap split: mixed positional/named parameter-override and port-connection lists reject in `parse_design`; scalar blocking assignments inside `always_ff` reject before elaboration (matching the generated grammar's nonblocking boundary); and a syntax-only lane parses selector/concat-rich expression text (`a[HI:LO]`, `cfgs[IDX].data[BIT]`, ternaries, concatenations, repeat-concatenations) without pretending those shapes are elaboration-time constants, while malformed ternary/repeat/list samples remain rejected by the shared manifest.

For exact current status, always check:

- `LIVE_ACHIEVEMENT_STATUS.md`
- `docs/reference/PGEN_SOTA_IMPLEMENTATION_ROADMAP.md`

## Future Downstream Parser Requests

PNR has requested future parser support for LEF, DEF, Liberty, SDC, structural Verilog netlists, and SPEF, but those parser-family releases are not shipped yet.

Important source notes:

- `docs/contracts/PGEN_PNR_PARSER_INTEGRATION_CONTRACT.md` records the downstream request, crate-shape target, and authoritative-source acquisition policy.
- `docs/tcl/md/tcl.md` is a local Tcl syntax reference note for the future SDC lane, because SDC is Tcl-shaped at the tokenization/quoting/substitution layer.

Do not treat the Tcl note as a complete SDC grammar or as an implemented parser family. It is reference input for future EBNF work once the authoritative Synopsys SDC source is acquired.

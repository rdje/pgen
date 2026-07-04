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
| return_annotation | [PGEN Return-Annotation Parser — Integration Reference](../../return_annotation_parser_book/src/welcome.md) | [`docs/return_annotation_parser_book-html/`](../../return_annotation_parser_book-html/welcome.html) | `make -C rust SHELL=/bin/bash return_annotation_parser_book_gate` |
| semantic_annotation | [PGEN Semantic-Annotation Parser — Integration Reference](../../semantic_annotation_parser_book/src/welcome.md) | [`docs/semantic_annotation_parser_book-html/`](../../semantic_annotation_parser_book-html/welcome.html) | `make -C rust SHELL=/bin/bash semantic_annotation_parser_book_gate` |
| ebnf (meta-grammar) | [PGEN EBNF — Grammar-Author's Reference](../../ebnf_parser_book/src/welcome.md) | [`docs/ebnf_parser_book-html/`](../../ebnf_parser_book-html/welcome.html) | `make -C rust SHELL=/bin/bash ebnf_parser_book_gate` |

The every-parser-book directive is now **complete**: every PGEN grammar — the nine parser/annotation
families above plus the `ebnf` meta-grammar — has its own live, gated mdBook. The `ebnf` book is the
grammar-author's reference (the EBNF *input* language), where the others document each parser's AST
*output*.

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
- the SV grammar is **dialect-profiled**: alongside the two IEEE 1800 SystemVerilog profiles (`sv_2017`, `sv_2023`) it now also carries a strict **`verilog_2005`** (IEEE 1364-2005 Verilog) profile, selected the same way (`--profile verilog_2005`, aliases `1364-2005` / `ieee1364-2005`; embedding API `1.3.0`). Because Verilog-2005 is a syntactic subset of IEEE 1800, the profile *rides the `sv_2017` baseline* of the shared core constructs and gates the SystemVerilog-only surface out. The profile has been **built to wellformedness coherence** — `--lint-grammar` reports `0` `verilog_2005` profile-orphans — via three declarative axes on the one grammar: (1) the Verilog-2005 core (declarations, statements, expressions, parameters, tasks/functions, UDPs, gate/path constructs) is admitted to the `sv_2017` baseline variants; (2) the SystemVerilog-only surface is whole-rule gated out (classes, packages/imports, interfaces/modports, programs, immediate + concurrent assertions, properties/sequences, covergroups, constraints, clocking, checkers, DPI, `randcase`/`randsequence`, `typedef`/type parameters, `++`/`--` and `return`/`break`/`continue`, `final` blocks, SV-only integer atom types like `int`, `unique`/`priority` qualifiers, `case … matches` patterns); and (3) keyword reservation is profile-faithful — the full IEEE 1364-2005 Annex B keyword set is reserved under `verilog_2005` (so a gated construct cannot re-parse as identifiers), while the ~48 SV-only reserved words (`logic`, `class`, `typedef`, `int`, `do`, …) parse as ordinary Verilog-2005 identifiers. Worked examples: `module m; endmodule`, a realistic parameterized counter design, and `wire logic;` (a net named `logic` — legal Verilog-2005) parse under `--profile verilog_2005`, while `class C; endclass`, `package p; endpackage`, `import p::*;`, `typedef integer t;`, `int x;`, `logic x;`, `i++`, `initial assert (1);`, `final $display("bye");`, `timeunit 1ns;`, `always_comb q = 1'b0;`, `do … while (…);`, `foreach (a[i]) …`, `shortreal r;`, `interconnect w;`, `wait fork;`, `task t (ref integer a);`, `function void f; … endfunction` and `void'(f());` (the `void` return type and `void'(…)` cast — SystemVerilog-only, gated by `SV-0032`), and `-> #5 e;` (the delayed event-trigger form — SystemVerilog-only; plain `-> e;` still parses) are all **rejected** (each still accepted under `sv_2017` / `sv_2023`). The bare-keyword SystemVerilog-only branch alternatives *inside* shared core rules are gated via shape-preserving named lifts (e.g. `always_keyword_sv_only`, `integer_vector_type_sv_only`), so the `sv_2017` / `sv_2023` ASTs are byte-identical to before. The profile now has its **repo-standard machine-checkable conformance surface**: `make -C rust SHELL=/bin/bash verilog_2005_conformance_gate` (script `rust/scripts/verilog_2005_conformance_gate.sh`) asserts, against the tracked contract `rust/test_data/grammar_quality/verilog_2005_conformance_contract_v0.json`, (a) the `--lint-grammar` **0 `verilog_2005` profile-orphan lock** (the build-to-coherence invariant cannot silently regress), (b) the curated conformance corpus (`rust/test_data/grammar_quality/verilog_2005_conformance/`, 15 accept + 51 reject files) as a **full per-file × per-profile matrix** — 198 accept/reject checks (66 cases) across `verilog_2005`/`sv_2017`/`sv_2023`, with spec-derived notes on every non-obvious row and two profile-alias normalization probes — and (c) the **profiled certificate-coverage baseline** `--grammar-profile verilog_2005`: `total=1147 proof=4 witness=816 UNKNOWN=327 (sample_parse_failures=0)`, deterministic across seeds 0/7/42 (re-pinned from `826/310` to `817/319` when the `SV-0025`/`SV-0027` leak fixes removed 9 *false* witnesses that had been earned through the leaked SV-only surface, then to `809/327` when the `SV-0026` `$unit`-surface gates removed 8 more — witness dropping on a leak fix is the honest direction — to `1147/819/326` when the `SV-0030` digit/compare restoration added its 9 rules all-witnessed AND earned the `scalar_constant` witness itself, and to the current `1147` total / `4` proofs / `816` witnesses / `327` UNKNOWN when the `SV-0029` wave-3 anchor restoration landed — the `$root`/`$unit` tokens upgrade witness→PROOF (the strict profile now PROVES the SystemVerilog-only anchors unreachable instead of witnessing them through mangled identifier-ish text) and the bare-`$` token joins the NO-reach set by design). Read that baseline honestly: 296 of the 327 `UNKNOWN` are NO-reach-path dead-rule candidates *under this profile* — the gated SystemVerilog-only surface being profile-unreachable **by design** — so the pin is a regression lock and ratchet floor, not a closure claim. The downstream write-up lives in `docs/contracts/PGEN_SYSTEMVERILOG_PARSER_INTEGRATION_CONTRACT.md` § "Dialect Profile — `verilog_2005`". The profile's tracker status is `Mostly Done`: three open grammar defects bound it today, tracked in the released-parser bug ledger (`SV-0023`: the `->>` nonblocking event trigger is unsupported and the delay form is mis-attached to `->`; `SV-0024`: un-braced multi-identifier port expressions such as `module m (a b);` are wrongly accepted under every profile — so the bare `module m (interconnect w);` form still accepts under `verilog_2005`, and the corpus pins the ANSI form until `SV-0024` lands; `SV-0028`: a stray top-level `;` is accepted under every profile; `SV-0029` is now **FIXED in full as of release `1.0.161`** (fix tree `SV-DOLLAR-LRM-FIDELITY`; schema `14`→`15`): nineteen keyword-like `$` tokens had matched mangled `sv_dollar_*` literal text instead of the IEEE `$` spellings — the 12 specify timing-check tokens landed at `1.0.159` (LRM `$setup(d, posedge clk, 1);` and its 11 siblings, core Verilog-2005 §15 surface, accept under every profile; locked by two conformance-corpus rows), and the SystemVerilog-only group landed at `1.0.161`: `$unit::name` scopes, module-level severity tasks (`$fatal;` …), `$root.`-anchored names and calls (now carrying an explicit root anchor in the AST instead of mis-routing through the generic system-TF token), and the bare-`$` primary all accept under `sv_2017`/`sv_2023` and — being SystemVerilog-only surfaces — reject under strict `verilog_2005`, locked by four more conformance-corpus rows and six AST-shape samples. The mangled spellings are ordinary identifier text and keep their identifier readings byte-identically). A fifth, `SV-0030`, is now **FIXED as of release `1.0.160`** (`SV-DOLLAR-LRM-FIDELITY.3`, schema `13`→`14`): the `scalar_constant` and UDP `init_val` rules had lost their LRM trailing digits — a sequential-UDP initializer `initial q = 1'b0;` was wrongly **rejected** under every profile while the digit-less nonsense `initial q = 1'b;` was accepted — and the specify timing-check compare forms (`expr == scalar_constant` and the `===`/`!=`/`!==` siblings) were doubly dead: shadowed by the plain-expression first branch AND starved by their own greedy full-expression lhs (a pure reorder would have been inert). The fix restores the ten IEEE digit alternatives at both sites and makes the compare branches reachable through a precedence-restricted lhs (only the operators binding tighter than `==` per IEEE 1800-2017 Table 11-2) plus a follow-guard, so `e == 1'b0` now emits the typed `eq` shape while operator-continued conditions like `e == 1'b0 && f` keep their precedence-correct flat parse byte-identically — locked by 4 conformance-corpus rows and 4 AST-shape-contract samples. The three strict-subset boundary leaks found by adjudicating the profiled certificate-coverage residual itself are all **FIXED** (`SV-0025`: `wire #1step w;` / `wire #10ns w;` delay forms and `SV-0027`: `10ns` / `'0` expression literals — the SV-only alternatives of `delay_value` and `primary_literal` profile-gated via shape-preserving named lifts with 4 corpus reject-locks; and `SV-0026`, the widest — SystemVerilog's `$unit` compilation-unit surface, both carriers (`description`'s `package_item` alternative AND `source_text_item`'s direct top-level `localparam`/`parameter` alternatives) gated via `description_unit_item_sv_only` / `source_text_item_unit_sv_only` with 4 more reject-locks, so bare top-level `wire w;` / `reg r;` / `localparam p = 1;` / `parameter p = 1;` now correctly reject under `verilog_2005` while staying legal IEEE 1800 `$unit` items under the SV profiles; landing it required first fixing an engine witness-planner defect the reshape exposed — the armed name-prelude integrity fix described in the grammar-wellformedness chapter — so the recognized-union certificate invariant held). A later machine residual-classification pass (the read-only `PGEN_CERT_RESIDUAL_CLASSIFICATION` analysis) surfaced three more `verilog_2005`-only over-acceptances, of which `SV-0032` (the `void` function return type and `void'(…)` cast — IEEE 1800 only) is now **FIXED** (`data_type_or_void_sv_only` / `void_cast_statement_sv_only` shape-preserving named lifts with 2 corpus reject-locks; the v2005 cert stays `1147/4/816/327` byte-identical and the SV profiles are AST-identical); `SV-0033` (the dynamic-array `[]` unsized dimension `reg q [];`) is now **FIXED** too (`VERILOG-2005-PROFILE.6.9`: a single whole-rule `@profiles: ["sv_2017","sv_2023"]` gate on the entirely-SV-only `unsized_dimension` rule, cf. `uniqueness_constraint` — zero new rules, sv_2017/sv_2023 AST byte-identical, v2005 cert re-pinned `1147/4/816/327` → `1146/4/815/327` as the rule leaves the profile universe and drops its one leak-earned false witness, union gate byte-identical), and `SV-0031` (the `::` scope-resolution surface — `p::f()` / `p::C::f()` / `p::X` / `import p::*;`) is now **FIXED** too (`VERILOG-2005-PROFILE.6.11`, 2026-07-04 — the LAST `verilog_2005`-only leak): IEEE 1364-2005 has no `::` token anywhere in Annex A, so the whole `::` class (package/class scopes, scoped calls, package import/export) is IEEE 1800 only. Fixed by whole-rule `@profiles: ["sv_2017","sv_2023"]` gates on **27 rules** — the `::` token root `scope_resolution` (the single `::` producer) plus the 26 `::`-only consumer rules the multi-profile `--lint-grammar` orphan-coherence check *derived* as newly-unsatisfiable once the token was gated (the well-formedness contract deriving the complete `::`-surface gate, reaching the 0-orphan fixpoint in one wave); ZERO new rules, `sv_2017`/`sv_2023` AST byte-identical, v2005 cert re-pinned `1144/4/813/327` → `1117/4/773/340` (all 27 leave the profile universe and the token de-witnesses the `::`-reachable SV-only surface it had falsely witnessed through the leak — every de-witnessed rule is SV-only `::` surface, zero core-Verilog-2005 constructs), union byte-identical. `SV-0034` — the SV-only associative-array (`[*]` / `[data_type]`) and queue (`[$]`) dimension alternatives of the same `variable_dimension` rule, surfaced tools-first while fixing `SV-0033` — was **FIXED** at `VERILOG-2005-PROFILE.6.10`: two more whole-rule `@profiles` gates on `associative_dimension`/`queue_dimension`; `reg q [*];` and `reg q [$];` now reject under `verilog_2005`; v2005 cert re-pinned `1146/4/815/327` → `1144/4/813/327`, union byte-identical. Fixing `SV-0034` in turn surfaced a *distinct* leak `SV-0035`: a reserved *type* keyword such as `integer` parsed as a bare primary expression under **every** profile (`assign w = integer;` / `initial x = integer;` / `localparam p = integer + 1;` wrongly accepted). `SV-0035` is now **FIXED** at release `1.0.162` (`SV-KEYWORD-PRIMARY-FIDELITY.2`, 2026-07-04, schema `15` unchanged): the three expression-primary bare-`identifier` carriers (`specparam_identifier`/`genvar_identifier` in `constant_primary`, and the trailing name of `hierarchical_identifier` in `primary`) now route through the keyword-excluding `non_keyword_identifier`, so the whole reserved-type family (`integer`/`real`/`time`/`reg`/`logic`/`int`/`bit`/`byte`/…) rejects as a primary under the profiles where it is reserved — shape-preserving (`{body:X}` unchanged), all no-regression gates byte-identical (canonical cert `1343/2/1321/20`, v2005 `1117/4/773/340`, conformance 219/0, union canonical=20/union=1, external corpus 14/14). The residual `localparam p = integer;` accept is the LRM-faithful `constant_param_expression ::= … | data_type` route, not a keyword-as-primary. `SV-0035`'s moved-leak re-probe surfaced a new, narrower defect **`SV-0036`** (`Root Caused`, owned by `SV-KEYWORD-PRIMARY-FIDELITY.3`): 35 net-type/gate/structural SV keywords (`wire`/`and`/`always`/…) that are reserved in SV but absent from `reserved_non_keyword_identifier_sv` still leak as primaries under `sv_2017`/`sv_2023` only (a reserved-list-completeness residual; `verilog_2005` is unaffected — its reserved list is the complete Annex B). The SV parser book's public-API page lists the current profile strings
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

- return annotation parser — has its own per-parser book:
  [PGEN Return-Annotation Parser — Integration Reference](../../return_annotation_parser_book/src/welcome.md)
  (the AST-shaping `-> …` language: references, literals, objects/arrays, the extraction/spread/access
  operators, the parsed annotation envelope, and the Bootstrap-vs-Generated backend split)
- semantic annotation parser — has its own per-parser book:
  [PGEN Semantic-Annotation Parser — Integration Reference](../../semantic_annotation_parser_book/src/welcome.md)
  (the steering `@…` language: the `@name: value` value language, the catalog of steering directives
  `@predicate`/`@emit_fact`/`@profiles`/`@transform`/…, the semantic-store lifecycle, the parsed
  envelope, and the Bootstrap-vs-Generated backend split)

Primary sources:

- **Per-parser book (return_annotation):** [PGEN Return-Annotation Parser — Integration Reference](../../return_annotation_parser_book/src/welcome.md)
- **Per-parser book (semantic_annotation):** [PGEN Semantic-Annotation Parser — Integration Reference](../../semantic_annotation_parser_book/src/welcome.md)
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

The `rtl_const_expr` family's canonical certificate-coverage baseline — `total=48 proof=0 witness=48 UNKNOWN=0 fully_certified=true`, deterministic at seeds 0/7/42 with zero sample-parse failures (the **default** entry rule, `--max-depth 32`, `--count 40`, the default diverse generation step-budget) — is oracle-locked by `make -C rust SHELL=/bin/bash rtl_const_expr_cert_gate` against the tracked contract `rust/test_data/grammar_quality/rtl_const_expr_cert_contract.json`. Diagnostic sub-entry probe configurations (for example `--entry-rule conditional_expr`) are *not* this baseline: a sub-entry lane structurally reports the grammar's root rule as `UNKNOWN=1` (an entry-relative artifact), which is exactly the wording confusion that once mislabeled a probe lane as canonical — the standing gate makes that failure mode mechanical instead of narrative.

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

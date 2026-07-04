# Changelog Index

This chapter is an index — pointers into other docs that carry the full changelog detail. Use it to find what changed in a given release.

## Where the canonical changelogs live

| Source | Granularity | Purpose |
|---|---|---|
| `docs/contracts/PGEN_SYSTEMVERILOG_PARSER_INTEGRATION_CONTRACT.md` | Per-release shape change | The authoritative contract. Each release's section lists the AST shape changes consumers care about. |
| `docs/contracts/PGEN_RELEASED_PARSER_BUG_LEDGER.md` | Per-bug | When a bug is fixed in a release, the ledger entry records the input/output shape change. |
| `CHANGES.md` (root) | Per-release | Human-readable summary of all changes. |
| Git tags + commit log | Commit-by-commit | The most granular source. |

When investigating "what changed and why," start with the contract document, drop down to the ledger for specific bugs, fall back to git for diffs.

## Releases relevant to this book

This book is **live** and tracks current main HEAD. Versioning summary:

- The most recent **published** parser-release section in the contract is **1.0.0 / Contract 1.0.0** (foundation baseline).

### 1.0.165 / Contract 1.0.165 — SV-LRM-SHAPE-FIDELITY.1 (`PGEN-SV-LRM-SHAPE-FIDELITY-0001`, 2026-07-04), ledger `SV-0024` (`Released`): **UN-BRACED MULTI-IDENTIFIER PORT LISTS NO LONGER OVER-ACCEPT; BRACED CONCATENATION PORTS NOW ACCEPTED (schema 15 unchanged)** — a whitespace-separated multi-identifier port with no comma and no concatenation braces (`module m (a b);`) previously accepted under EVERY profile. IEEE `port_expression ::= port_reference | { port_reference {, port_reference} }` — the multi-reference form requires LITERAL concatenation braces. Fixed grammar-only: `port_expression`'s second alternative (`grammars/systemverilog.ebnf:4092`) changed from the un-braced `( port_reference ( comma port_reference )* )*` to `lbrace port_reference ( comma port_reference )* rbrace -> {kind:"list", body:[$2, $3::2*]}` (the `assignment_pattern` concatenation-list idiom); ZERO new rules (census `1465`). **What consumers see:** `module m (a b);` flips **ACCEPT → REJECT** under `verilog_2005` (its only reading is the non-ANSI `port_expression`); it still ACCEPTS under `sv_2017`/`sv_2023` as a legitimate ANSI interface port (`a`=interface type, `b`=port name — valid IEEE 1800 syntax, an elaboration-time check); and the braced concatenation port `module m ({a, b});` flips **REJECT → ACCEPT** under EVERY profile (the LRM's valid multi-reference form, previously unsupported because the list alternative was written un-braced), emitting `port_expression {kind:"list", body:[{name,select}, …]}`. Single / comma-separated / named `.p(a)` / ANSI declaration ports are unchanged. **Why schema stays 15:** the `{kind:"list"}` body is still typed-JSON — the braced-concat shape is newly *reachable*, not a new envelope shape. Proof (all seeds 0/7/42): `sv_cert_recognized_union_gate` GREEN (canonical `1343/2/1321/20` byte-identical, union UNKNOWN=1 residual `context_member_method_call`), `verilog_2005_conformance_gate` GREEN (orphans 0, matrix 225/0 with 2 new locks, cert `1117/4/773/340` byte-identical), ast_shape 18/18, external corpus 14/14, the six fully-certified grammars byte-identical, clippy source strict-clean (grammar-only). Full matrix in ledger row `SV-0024` and the contract's § "Release 1.0.165".

#### 1.0.165 (same release — no version bump) — SV-LRM-SHAPE-FIDELITY.2 (`PGEN-SV-LRM-SHAPE-FIDELITY-0002`, 2026-07-04), ledger `SV-0028` (`Released`): **STRAY TOP-LEVEL `;` NO LONGER OVER-ACCEPTS UNDER `verilog_2005` (schema 15 unchanged)** — a bare top-level `;` with no declaration previously accepted under `verilog_2005`. IEEE 1364-2005 A.1.2 (`source_text ::= { description }`, `description ::= module_declaration | udp_declaration | config_declaration`) admits no null/empty top-level item, so a stray `;` is invalid Verilog-2005; it accepted only via `source_text_item`'s LRM-less direct `| semi -> {kind:"semi"}` alternative (`grammars/systemverilog.ebnf:231`), now removed (ZERO rule census change — a branch, not a rule; census `1465`). **Tools-first re-adjudication** (AST dumps + IEEE LRM markdown) corrected an earlier "all-profile" framing to **`verilog_2005`-only**: under `sv_2017`/`sv_2023` a stray `;` is the **legitimate LRM empty package-or-generate item** (`description ::= … | { attribute_instance } package_item`, `package_or_generate_item_declaration ::= … | ;` — IEEE 1800-2017 §26 / 2023 Annex A) and correctly stays ACCEPT (the PEG-first `description` path shadows `:231` under the SV profiles, so removing it changes nothing there). **What consumers see:** `;` / `;;` / `module m; endmodule\n;` flip **ACCEPT → REJECT** under `verilog_2005`; unchanged ACCEPT under `sv_2017`/`sv_2023` with the empty-item AST byte-shape-identical. **No version bump** (SV profiles byte-invariant — the `SV-0026`/`SV-0031`..`SV-0034` precedent). Proof (all seeds 0/7/42): `sv_cert_recognized_union_gate` GREEN (canonical `1343/2/1321/20` byte-identical, union UNKNOWN=1), `verilog_2005_conformance_gate` GREEN (orphans 0, matrix 228/0 with 1 new lock `reject/stray_top_level_semi.sv`, cert `1117/4/773/340` byte-identical), ast_shape 18/18, external corpus 14/14, the six fully-certified grammars byte-identical, clippy source strict-clean (grammar-only). Full detail in ledger row `SV-0028`. **This closes the `SV-LRM-SHAPE-FIDELITY` tree** (both all-profile over-acceptance leaks `SV-0024` + `SV-0028` fixed).

### 1.0.164 / Contract 1.0.164 — SV-KEYWORD-PRIMARY-FIDELITY.3.2 (`PGEN-SV-KEYWORD-PRIMARY-FIDELITY-0005`, 2026-07-04), ledger `SV-0036` (`Released`): **NET/GATE/STRUCTURAL RESERVED KEYWORDS NO LONGER PARSE AS BARE EXPRESSION PRIMARIES (schema 15 unchanged)** — 91 keywords that SystemVerilog reserves but that were absent from `reserved_non_keyword_identifier_sv` (net types `wire`/`wand`/`tri`/`supply0`/…, gate primitives `and`/`or`/`nand`/`buf`/`nmos`/…, structural `always`/`assign`/`initial`/`posedge`/`genvar`/`specparam`/…) could match as a bare `primary` (`assign w = wire;`, `initial x = and;`) under `sv_2017`/`sv_2023`. IEEE 1800 Annex B reserves these; they are never an operand. Fixed grammar-only: the `_v2005 \ _sv` delta (91 words) was appended to `reserved_non_keyword_identifier_sv` (`grammars/systemverilog.ebnf:386`, `_sv` 81→172 words, `_sv` ⊇ `_v2005`). **What consumers see:** `assign w = <reserved-kw>;` and operand forms flip **ACCEPT → REJECT** under `sv_2017`/`sv_2023`; keyword-**prefixed** identifiers (`wire_en`/`input_data`/`and_gate`) still parse (the `\b` word boundary). `verilog_2005` unchanged (complete Annex B list). **Why schema stays 15:** shape-preserving (`{body:X}` unchanged — only reserved keywords change acceptance); ZERO new rules (census `1465` unchanged). A co-landed **generator prerequisite** (`rust/src/ast_pipeline/stimuli_generator.rs`) extended the RTL-FE-CLOSURE.6 keyword-exclusion guard from the fixed-literal `!kw_A…!kw_Z` shape to SystemVerilog's `!reserved_non_keyword_identifier identifier` (regex whole-word-list) shape so the stimuli generator stops out-generating its own parser (canonical `sample_parse_failures` 1→0 at seed 0; six fully-certified grammars byte-identical) — a generation-side fix with no parser-output-shape change. Proof: canonical cert `1343/2/1321/20` byte-identical, `sv_cert_recognized_union_gate` GREEN (union UNKNOWN=1, residual `context_member_method_call`), v2005 conformance orphans 0 / matrix 219/0 / cert `1117/4/773/340`, ast_shape 18/18, external corpus 14/14, clippy source strict-clean (all seeds 0/7/42). Full matrix in ledger row `SV-0036` and the contract's § "Release 1.0.164". Closes the all-profile reserved-keyword-as-primary family (`SV-0035`+`SV-0036`).

### 1.0.163 / Contract 1.0.163 — SV-KEYWORD-PRIMARY-FIDELITY.3.1 (`PGEN-SV-KEYWORD-PRIMARY-FIDELITY-0004`, 2026-07-04), ledger `SV-0037` (`Released`): **IMPLICIT-TYPE ANSI PORTS PARSE CORRECTLY (schema 15 unchanged)** — an implicit-type ANSI port such as `module m(input a);` (and `module m(input [7:0] a);`) was mis-parsed as a non-ANSI port list (`{kind:"nonansi"}`) instead of the correct ANSI port (`{kind:"ansi"}`, direction=`input`, implicit type, name=`a`). Root cause: `net_port_type_sv_2017`/`_sv_2023` alt 1 used the ungated `net_type_identifier`/`nettype_identifier`, and under the grammar's default `branch_policy=longest_match` that bare identifier out-consumed alt 0's empty implicit-type match and ate the port name. Fix: both alt-1 carriers routed to the store-gated `checked_nettype_identifier` (`grammars/systemverilog.ebnf:3537`), so only a DECLARED nettype matches there. `module m(input a);` now realizes `{kind:"ansi"}` under all profiles; declared-nettype ports are byte-identical. Consumers keying off the module `kind` for implicit-type ANSI ports should expect the corrected `ansi` shape. No-regression: canonical cert `1343/2/1321/20` byte-identical, ast_shape 18/18, union canonical=20/union=1, v2005 conformance orphans 0 / matrix 219/0 / cert `1117/4/773/340`, external corpus 14/14 (all seeds 0/7/42). Unblocks `SV-0036`/`.3.2`.

### 1.0.162 / Contract 1.0.162 — SV-KEYWORD-PRIMARY-FIDELITY.2 (`PGEN-SV-KEYWORD-PRIMARY-FIDELITY-0002`, 2026-07-04), ledger `SV-0035` (`Released`): **RESERVED TYPE KEYWORDS NO LONGER PARSE AS BARE EXPRESSION PRIMARIES (schema 15 unchanged)** — a reserved *type* keyword (`integer`/`real`/`time`/`reg`/`logic`/`int`/`bit`/`byte`/`shortint`/`longint`/`shortreal`/`string`/`chandle`/`event`/…) could match as a bare `primary` (`assign w = integer;`, `initial x = integer;`, `localparam p = integer + 1;`) under every profile at every release `1.0.0`–`1.0.161`. IEEE 1364-2005 / IEEE 1800 Annex B reserve these words as data-type keywords only — never a `primary`/operand. Fixed grammar-only: the three expression-primary bare-`identifier` carriers — `specparam_identifier` (`grammars/systemverilog.ebnf:5083`) + `genvar_identifier` (`:2375`) in `constant_primary_sv_2017`/`_sv_2023`, and the trailing name of `hierarchical_identifier` (`:2412`) in `primary_sv_2017` — now route through the keyword-excluding `non_keyword_identifier` (`:359`). **What consumers see:** `assign w = <reserved-type-kw>;` and operand forms flip **ACCEPT → REJECT** under `sv_2017`/`sv_2023` (and the reserved family under `verilog_2005`). The residual `localparam p = integer;` still ACCEPTs — that is the LRM-faithful `constant_param_expression ::= … | data_type | $` route (`:1380`), not a keyword-as-primary; likewise `reg q [integer];` under the SV profiles is the legitimate `associative_dimension ::= [ data_type ]`. **Why schema stays 15:** shape-preserving by construction (`identifier` and `non_keyword_identifier` both emit `{body:X}`; a legal name's AST is byte-identical, only reserved keywords change acceptance); ZERO new rules (census `1465` unchanged). Proof: canonical cert `1343/2/1321/UNKNOWN=20` + `verilog_2005` cert `1117/4/773/340` (seeds 0/7/42, byte-identical); `verilog_2005_conformance_gate` GREEN (orphans 0, matrix 219/0); `sv_cert_recognized_union_gate` GREEN (canonical UNKNOWN=20, union UNKNOWN=1, residual `context_member_method_call`); `ast_shape_contract_gate` 18/18; `sv_external_corpus_triage_gate` 14/14; `clippy_on_rust_change` source strict-clean; json `9/0` + regex `198/0` fully_certified. **Known residual (`SV-0036`, OPEN, owned by `.3`):** 35 net-type/gate/structural SV keywords (`wire`/`and`/`always`/…) that are reserved in SV but absent from `reserved_non_keyword_identifier_sv` still leak as primaries under `sv_2017`/`sv_2023` (a reserved-list-completeness defect); `verilog_2005` is unaffected (complete Annex B list). Full matrix in ledger row `SV-0035` and the contract's § "Release 1.0.162".

### 1.0.161 / Contract 1.0.161 — SV-DOLLAR-LRM-FIDELITY.4 (`PGEN-SV-DOLLAR-LRM-FIDELITY-0004`, 2026-07-03), ledger `SV-0029` (`Released`): **THE SV-ONLY `$` ANCHORS RESTORED — `$root` / `$unit` / severity / bare `$` (schema 14 → 15)** — the remaining 7 mangled `sv_dollar_*` keyword-token literals now carry their IEEE `$` spellings, closing the `SV-0029` family (wave 1 = the 12 timing checks at `1.0.159`). What consumers see: IEEE 1800 `$unit::name` scopes (`{kind:"dollar_unit", name:{body:"$unit"}}` inside `package_scope`), module-level elaboration severity tasks (`$fatal;` … → `{kind:"fatal"|"error"|"warning"|"info", args}`), `$root.`-anchored hierarchical names (`root: {kind:"root"}` marker — previously the whole name mis-routed as generic `kind:"system_tf"`) and `$root.`-anchored calls (the NEW `{kind:"rooted_tf", target, attributes, args}` carrier), and the bare-`$` primary (`{kind:"system_dollar"}`) all ACCEPT under `sv_2017`/`sv_2023` and REJECT under strict `verilog_2005`. Mechanics: two inline steal guards on `system_tf_call` (`!( $root . )`, `!( $unit :: )` — the generic token had been PEG-committing on the anchors), three profile-gated `_sv_only` lifts, the `!( identifier )`-guarded `rooted_tf_call_sv_only` carrier (needed because `scoped_or_hierarchical_tf_identifier`'s branch predicate is runtime-broadcast to all branches and hard-rejects rooted results — recorded as an open engine question), and a load-bearing `name:{body:"$unit"}` literal that keeps consumer typedef-exclusion predicates resolvable. The mangled `sv_dollar_*` spellings are ordinary identifiers and keep their identifier readings byte-identically (24/24 cmp-proven). Proof: the 28-probe matrix; canonical cert `1341/2/1319/UNKNOWN=20` `spf=0` seeds 0/7/42 (residual set-identical); `verilog_2005` cert `1147/4/816/327` (the `$root`/`$unit` tokens upgrade witness→PROOF; the bare-`$` token joins NO-reach-by-design); conformance 192 checks/0 mismatches (4 new locks); union re-pinned; 6 new shape samples; clippy source-clean.

### 1.0.160 / Contract 1.0.160 — SV-DOLLAR-LRM-FIDELITY.3 (`PGEN-SV-DOLLAR-LRM-FIDELITY-0003`, 2026-07-02), ledger `SV-0030` (`Released`): **LRM DIGIT + TIMING-CHECK COMPARE-SHAPE RESTORATION (schema 13 → 14, both defect sites)** — the LRM-extraction prefix-merge had truncated the IEEE digit alternatives at TWO sites at every release `1.0.0`–`1.0.159`: `scalar_constant` (the ten IEEE A.7.5.3 alternatives had become 6 digit-less branches, so the rule was dead-in-practice) and UDP `init_val` (IEEE A.5.1 — **sequential-UDP initializers `initial q = 1'b0;` / `1'b1;` REJECTED outright** while the nonsense digit-less `initial q = 1'b;` ACCEPTED); additionally the `scalar_timing_check_condition` eq/case_eq/ne/case_ne compare branches were doubly dead (listed after the bare-`expression` branch AND their greedy full-`expression` lhs consumed `== 1'b0` into its own operand chain — a pure reorder would have been inert), so `e == 1'b0` emitted a flat `{kind:"expression"}`. Fixed grammar-only: the ten digit-carrying alternatives restored at both sites (new `\b`-guarded tokens `1'b0`/`1'b1`/`1'B0`/`1'B1` + `'b0`/`'b1`/`'B0`/`'B1`; digit-less tokens removed with their last sites; `!tick` follow-guards on the bare `1`/`0` branches), and the compare branches made reachable — ordered first with the precedence-restricted `scalar_timing_check_compare_lhs` (operator tier `binary_operator_above_equality` = IEEE 1800-2017 Table 11-2 rows 3–7 only) and a `!binary_operator` follow-guard so operator-continued conditions (`e == 1'b0 && f`) and mixed-precedence forms (`a & b == 1'b0`) deliberately keep the precedence-correct flat parse (9 fallback-probe ASTs byte-identical pre↔post). Schema `13 → 14` (digit-carrying kinds; `{kind:"eq"|"case_eq"|"ne"|"case_ne", lhs, rhs}` now realizable). VERIFIED: UDP digits REJECT→ACCEPT + digit-less ACCEPT→REJECT under both dialects; canonical cert `1337/2/1315/UNKNOWN=20` `spf=0` seeds 0/7/42 (total +9 = exactly the census delta; the 20-rule residual set-identical pre↔post); `verilog_2005` cert `1147/2/819/326` — **`scalar_constant` itself leaves the UNKNOWN residual** (set-diff-proven sole change; the `VERILOG-2005-PROFILE.6.4`-adjudicated not-earnable class-D witness now genuinely earned); shape-contract 18/18 (4 new samples incl. the compare-lhs revert guard); 4 new conformance-corpus locks; external corpus green; clippy source-clean.

### 1.0.159 / Contract 1.0.159 — SV-DOLLAR-LRM-FIDELITY.2 (`PGEN-SV-DOLLAR-LRM-FIDELITY-0002`, 2026-07-02), ledger `SV-0029` (wave 1, `Fix In Progress`): **THE 12 SPECIFY TIMING-CHECK `$` SPELLINGS RESTORED** — the timing-check keyword tokens matched the mangled literal text `sv_dollar_setup` … `sv_dollar_nochange` (an LRM-extraction name-canonicalization leak) at every release `1.0.0`–`1.0.158`, so LRM specify timing checks like `$setup(d, posedge clk, 1);` — core IEEE 1364-2005 §15 / 1800-2017 §31 surface — wrongly REJECTED under every profile while the nonsense spellings accepted. Fixed grammar-only (12 token literals → `/\$setup\b/` …; token names kept): 12/12 LRM spellings now ACCEPT and 12/12 mangled spellings now REJECT under `sv_2017`, `sv_2023`, AND `verilog_2005`; procedural `$…(…)` system-TF calls unaffected (no collision — `specify_item` has no generic system-TF route). Schema stays `13` (timing-check typed kinds unchanged). Cert pins unchanged and re-verified: canonical `1328/2/1306/UNKNOWN=20` (residual set-identical), union `1325/1`, `verilog_2005` `1138/2/809/327`, seeds 0/7/42; corpus green; shape 18/18; 2 new conformance-corpus locks (accept + reject). Still open in the same ledger row: the SV-only `$root`/`$unit`/severity/bare-`$` tokens (leaf `.4`) and `SV-0030` (`scalar_constant`/`init_val` digit loss, leaf `.3`).

### 1.0.158 / Contract 1.0.158 — SV-AST-SHAPE-FIDELITY.2.7 (`PGEN-SV-AST-SHAPE-FIDELITY-0008`, 2026-07-01), ledger `SV-0020`: **CLASS-SCOPED-CALL-PREFIX AST-SHAPE CORRUPTION FIX** — a package-class-scoped static method call (`pkg::C::foo()`, reachable in statement + assignment-RHS context) emitted 2 `<invalid_sequence_access>` in the `class_scoped_call_prefix` `params`/`scope_chain` sub-shapes at every release `1.0.0`–`1.0.157`. Fixed grammar-only via the named-lift `class_scoped_call_prefix_head` (mirroring the in-file `class_scope_type_head`); `params`/`scope_chain` corrupted → clean `[]`, both profiles 2 → 0. `SV-0013` gated the alt's branches but did not lift it — `.2.7` completes that rule. Schema `12 → 13` (candidate #21 of the `SV-AST-SHAPE-FIDELITY.2` sweep, tool-proven reachable in `.2.4`; the LAST of the three reachable-corrupt candidates). Cert `total 1311→1312 / witness 1290→1291 / UNKNOWN=20` unchanged (identical residual; `class_scoped_call_prefix_head` witnessed); SV external corpus 14/14; the 6 fully-certified grammars byte-identical; `cargo test --lib` 739/0 (`systemverilog_ast_shape_contract` PASS, 2 new samples).

### 1.0.157 / Contract 1.0.157 — SV-AST-SHAPE-FIDELITY.2.6 (`PGEN-SV-AST-SHAPE-FIDELITY-0007`, 2026-07-01), ledger `SV-0019`: **NETTYPE WITH-CLAUSE AST-SHAPE CORRUPTION FIX** — both nettype rules (`net_type_declaration_sv_2017` + sv_2023 twin `nettype_declaration_sv_2023`) bound a double-nested inline alt `( non_typedef_package_scope | class_scope )?` inside the br0 `with`-clause group; `nettype logic n with pkg::f;` (a nettype with a package-scoped resolution function) emitted 3 `<invalid_sequence_access>` in the `with_clause` inner scope element at every release `1.0.0`–`1.0.156`. Fixed grammar-only via the shared named-lift `nettype_scope_prefix` (order preserved vs the reversed `scoped_type_scope_prefix`); `with_clause` inner scope corrupted → clean, both profiles 3 → 0 (br1 already clean). Schema `11 → 12` (candidate #14 of the `SV-AST-SHAPE-FIDELITY.2` sweep, tool-proven reachable in `.2.4`). Cert `total 1310→1311 / witness 1289→1290 / UNKNOWN=20` unchanged; SV external corpus 14/14; the 6 fully-certified grammars byte-identical; `cargo test --lib` 739/0 (`systemverilog_ast_shape_contract` PASS, 2 new samples).

### 1.0.156 / Contract 1.0.156 — SV-AST-SHAPE-FIDELITY.2.5 (`PGEN-SV-AST-SHAPE-FIDELITY-0006`, 2026-07-01), ledger `SV-0018`: **INTERFACE-CLASS-TYPE AST-SHAPE CORRUPTION FIX** — a class implementing a package-scoped interface class (`class C implements pkg::IC;`, common in UVM) emitted 1 `<invalid_sequence_access>` in the `interface_class_type` `params` sub-shape at every release `1.0.0`–`1.0.155`. Fixed grammar-only via the named-lift `interface_class_type_head` (mirroring the in-file `base_class_type_head`); `params` corrupted → clean `[]`, both profiles 1 → 0. Schema `10 → 11` (candidate #8 of the `SV-AST-SHAPE-FIDELITY.2` sweep, tool-proven reachable in `.2.4`). Cert `total 1309→1310 / witness 1288→1289 / UNKNOWN=20` unchanged (identical residual; `interface_class_type_head` witnessed); SV external corpus 14/14; the 6 fully-certified grammars byte-identical; `cargo test --lib` 739/0 (`systemverilog_ast_shape_contract` PASS, 2 new samples).

### 1.0.155 / Contract 1.0.155 — SV-AST-SHAPE-FIDELITY.2.3 (`PGEN-SV-AST-SHAPE-FIDELITY-0004`, 2026-07-01), ledger `SV-0017`: **SCOPED-TYPE-REFERENCE AST-SHAPE CORRUPTION FIX — a package/class-scoped type reference now emits clean nested sub-shapes (schema 9 → 10).** The single ubiquitous repro `module m; C::D::E v; endmodule` (a package/class-scoped type reference on a variable declaration) emitted **4 `<invalid_sequence_access>` sentinels** from TWO nested inline-alternation-`$N` corruptions (`ast_return_transform.rs:193/438`, the `SV-0014`/`SV-0015`/`SV-0016`/`SVPP-0001`/`RTL-FE-0002` class): (a) `class_scope_type` (`grammars/systemverilog.ebnf:1067`) bound `head: $1` to an inline 4-way alternation → `params`/`scope_chain` corrupted when the self-annotated `scoped_class_scope_identifier` branch won (the un-lifted twin of `base_class_type`/`class_type`); (b) `scoped_block_type_identifier` (`:1642`) and the byte-identical `scoped_data_type_identifier` (`:1649`) bound `scope: $1` to the shared inline alt `( class_scope | non_typedef_package_scope )` → `type`/`dims` corrupted when the self-annotated `class_scope` branch won. Found by the `SV-AST-SHAPE-FIDELITY.2` grammar-wide inline-alt-`$N` sweep (cluster #1/#5/#6). **Fix (grammar-only, the `SV-0014`/`SV-0015`/`SV-0016` named-lift idiom mirroring the in-file `class_type_head`):** TWO new un-annotated named rules `class_scope_type_head` (lifts the `class_scope_type` alt) + `scoped_type_scope_prefix` (lifts the shared alt, which occurs ONLY at the two `scoped_*_type_identifier` rules), each referenced by its parent (annotation text + positions unchanged); one shared prefix fixes both byte-identical `scoped_*_type_identifier` rules. **Schema 9 → 10:** the `class_scope_type` `params`/`scope_chain` and `scoped_data_type_identifier` `type`/`dims` sub-shapes change consumer-visibly from the corrupted `<invalid_sequence_access>` sentinels to the clean shapes. VERIFIED: `C::D::E v;` `<invalid_sequence_access>` **4 → 0** both profiles (`parse_full` still passes); SV cert `total 1307→1309 / witness 1286→1288 / UNKNOWN=20` unchanged (identical residual; both new rules witnessed; zero newly-unknown) `spf=0` seeds 0/7/42; **SV external corpus 14/14**; `--lint-grammar` clean (1428→1430 rules; `always_matches=8` unchanged); 6 fully-certified grammars byte-identical; `ast_shape_contract` GREEN (4 new scoped-type samples); `cargo test --lib` 739/0; clippy source-clean. Full matrix in ledger row `SV-0017` and the contract's § "AST-Shape Corrections — 1.0.155".
### 1.0.154 / Contract 1.0.154 — SV-AST-SHAPE-FIDELITY.2.2 (`PGEN-SV-AST-SHAPE-FIDELITY-0003`, 2026-07-01), ledger `SV-0016`: **BASE-CLASS-TYPE AST-SHAPE CORRUPTION FIX — a package-scoped base class now emits clean `params`/`scope_chain` (schema 8 → 9).** `base_class_type` (`grammars/systemverilog.ebnf:1094`) bound `head: $1` to an inline 3-way alternation `( scoped_base_class_type_identifier | known_unscoped_base_class_type_identifier | known_unscoped_base_class_type_parameter_identifier )` — the systemic inline-alternation-`$N` positional-model corruption (`ast_return_transform.rs:193/438`, the `SV-0014`/`SV-0015`/`SVPP-0001`/`RTL-FE-0002` class). A derived class whose base is package-scoped (`class D extends pkg::B;` — ubiquitous in SV/UVM, e.g. `extends uvm_pkg::uvm_driver`) emitted **2 `<invalid_sequence_access>` sentinels** in its `base_class_type` `params`/`scope_chain` sub-shapes (when the self-annotated `scoped_base_class_type_identifier` branch won, the rule's own `{head:$1, params:$2, scope_chain:$3}` annotation mis-recursed onto the following `$2`/`$3` slots); the declared-bare `extends B;` was clean. The sibling `class_type` (`:1099`) was already correct because it lifts its head into the named `class_type_head` (`:1102`) — `base_class_type` was the un-lifted twin. Found by the `SV-AST-SHAPE-FIDELITY.2` grammar-wide inline-alt-`$N` sweep (candidate #2). **Fix (grammar-only, the `SV-0014`/`SV-0015` named-lift idiom):** lifted the inline alternation into ONE new un-annotated named rule `base_class_type_head`, referenced by `base_class_type` as its head (annotation text + positions unchanged); one shared rule fixes both profiles (`class_declaration_sv_2017:955` / `_sv_2023:959` share `base_class_type` via `extends`). **Schema 8 → 9:** the `base_class_type` `params`/`scope_chain` change consumer-visibly from the corrupted `<invalid_sequence_access>` sentinels to the clean empty arrays `[]`. VERIFIED: `class D extends pkg::B;` `<invalid_sequence_access>` **2 → 0** both profiles (`parse_full` still passes), declared-bare stays clean; SV cert `total 1306→1307 / witness 1285→1286 / UNKNOWN=20` unchanged (identical residual; `base_class_type_head` witnessed; zero newly-unknown) `spf=0` seeds 0/7/42; **SV external corpus 14/14**; `--lint-grammar` clean (1427→1428 rules; `always_matches=8` unchanged); 6 fully-certified grammars byte-identical; `ast_shape_contract` 11/11 (new `base_class_type_head_scoped` + `base_class_type_scoped` samples); `cargo test --lib` 739/0; clippy source-clean. Full matrix in ledger row `SV-0016` and the contract's § "AST-Shape Corrections — 1.0.154".
### 1.0.153 / Contract 1.0.153 — SV-AST-SHAPE-FIDELITY.2.1 (`PGEN-SV-AST-SHAPE-FIDELITY-0002`, 2026-07-01), ledger `SV-0015`: **FORWARD-TYPEDEF KEYWORD AST-SHAPE CORRUPTION FIX — a forward typedef with a keyword now emits a clean `keyword` (schema 7 → 8).** `type_declaration_sv_2017` (`grammars/systemverilog.ebnf:5209`) and `type_declaration_sv_2023` (`:5225`) branch 6 bound `keyword: $2` to an inline alternation inside an optional group `( kw_enum | kw_struct | kw_union )?` — the systemic inline-alternation-`$N` positional-model corruption (`ast_return_transform.rs:193/438`, the `SV-0014`/`SVPP-0001`/`RTL-FE-0002` class). A forward typedef with a keyword (`typedef enum e_t;` / `struct` / `union` — a common cycle-breaking idiom) emitted **3–4 `<invalid_sequence_access>` sentinels** in its `keyword` sub-shape (branch-1's `class_alias` annotation mis-recursed onto the `keyword` slot); the keyword-less `typedef e_t;` was clean. Found by the `SV-AST-SHAPE-FIDELITY.2` grammar-wide inline-alt-`$N` sweep (candidates #19+#20). **Fix (grammar-only, the `SV-0014` named-lift idiom):** lifted the shared inline alternation into ONE new un-annotated named rule `forward_type_keyword := kw_enum_e338e8e3 | kw_struct_d118e5a3 | kw_union_67ad5a07`, referenced by both br6 as `forward_type_keyword?` (annotation text + positions unchanged); one shared rule fixes both profiles. **Schema 7 → 8:** the forward node's `keyword` changes consumer-visibly from the corrupted `{kind:"class_alias", …}` object to the clean fused-token `[trivia, "enum"]`. VERIFIED: `typedef enum/struct/union e_t;` `<invalid_sequence_access>` **3/4 → 0** both profiles; SV cert `total 1305→1306 / witness 1284→1285 / UNKNOWN=20` unchanged (identical residual; `forward_type_keyword` witnessed; zero newly-unknown) `spf=0` seeds 0/7/42; **SV external corpus 14/14**; `--lint-grammar` clean (1426→1427 rules; `always_matches=8` unchanged); 6 fully-certified grammars byte-identical; `ast_shape_contract` 18/18 (new `forward_typedef_keyword` + `forward_typedef_enum` samples); `cargo test --lib` 739/0; clippy source-clean. Full matrix in ledger row `SV-0015` and the contract's § "AST-Shape Corrections — 1.0.153".
### 1.0.152 / Contract 1.0.152 — SV-AST-SHAPE-FIDELITY.1 (`PGEN-SV-AST-SHAPE-FIDELITY-0001`, 2026-07-01), ledger `SV-0014`: **ANSI-PORT AST-SHAPE CORRUPTION FIX — a typed ANSI module port's `header` now emits the clean `{direction, port_type}` (schema 6 → 7).** `ansi_port_declaration` (`grammars/systemverilog.ebnf:519-520`) bound `header: $1` to an inline alternation inside an optional group `( net_port_header | interface_port_header )?` — the systemic inline-alternation-`$N` positional-model corruption (`ast_return_transform.rs:193/438`, the `SVPP-0001`/`RTL-FE-0002` class). Every typed ANSI module port (`input logic a`) emitted **3 `<invalid_sequence_access>` sentinels** in its `header` sub-shape (`net_port_header`'s clean `{direction, port_type}` mis-nested as `header.header`, the out-of-range `dims`/`default`/`name` slots corrupted); POST-SV-AUDIT did not cover this (it swept only iteration-lead `{first/rest:$N}` list rules). **Fix (grammar-only, the `pp_if_keyword`/`event_separator` named-lift idiom):** lifted the inline alternation into the new un-annotated named rule `ansi_port_header := net_port_header | interface_port_header`; `header: $1` now binds the clean named rule (annotation text unchanged, parse behavior byte-identical). **Schema 6 → 7:** `port.header` changes consumer-visibly from the corrupted sentinel-bearing object to the clean `net_port_header` `{direction, port_type}` (or `interface_port_header` `{kind:"named", name, modport}`). VERIFIED: `module m (input logic a); endmodule` `<invalid_sequence_access>` **6 → 0** (`parse_full` still passes); SV cert `total 1304→1305 / witness 1283→1284 / UNKNOWN=20` unchanged (identical residual; `ansi_port_header` witnessed; zero newly-unknown) `spf=0` seeds 0/7/42; **SV external corpus 14/14**; `--lint-grammar ordered_choice_shadowing=0` (1426 rules; `always_matches 7→8` = the now-visible pre-existing `net_port_header`-always-matches shadow, behavior-preserving); 6 fully-certified grammars byte-identical; `ast_shape_contract` GREEN (new `ansi_port_header_typed` + `ansi_port_typed` samples); `cargo test --lib` 739/0; clippy source-clean. The first member of the reachable inline-alternation-`$N` AST-shape corruption sweep (`SV-AST-SHAPE-FIDELITY`). Full matrix in ledger row `SV-0014` and the contract's § "AST-Shape Corrections — 1.0.152".

### 1.0.151 / Contract 1.0.151 — GRAMMAR-WELLFORMED.H.12.8.3 (`PGEN-GRAMMAR-WELLFORMED-0146`, 2026-06-30), ledger `SV-0013`: **CLASS-SCOPED-CALL REACH-GAP CLOSURE — an interface_class / type_parameter-scoped call now emits `class_scoped_tf` (schema stays 6, acceptance-preserving).** A class-scoped subroutine call whose head is a declared interface class or type parameter (e.g. `\foo ::\foo_0 ()` after `typedef interface class \foo ;`) previously routed through the `package_scope`/`tf` path instead of the dedicated `class_scoped_tf` path, so the `known_unscoped_class_scoped_call_interface_class_identifier` / `…type_parameter_identifier` cousin branches were reachable-but-never-witnessed (cert `UNKNOWN`). The fix (two AND-stacked `lacks_fact_attribute_equals` predicates on the branch-1 `<pkg>::<class>` prefix, the `known_unscoped_block_class_type` template) restricts branch-1 longest-match to a genuine PACKAGE scope so an interface_class / type_parameter head falls through to its dedicated cousin. **Why schema stays 6:** no new node kinds and the strings still parse — the change re-routes an existing call to the correct `class_scoped_tf` node (an acceptance-preserving AST-shape correction, the `SV-0013` category). **Consumer impact:** a class-scoped call over an interface-class / type-parameter head now emits `class_scoped_tf` (was `package_scope`/`tf`); `<pkg>::<class>::call()` over a real package is unchanged. Surfaced grammar-first by the cert-coverage `UNKNOWN`→0 / reach-gap drive (`H.12.8.3`). Full matrix in ledger row `SV-0013` and the contract's 1.0.151 schema-version note.

### 1.0.150 / Contract 1.0.150 — GRAMMAR-WELLFORMED.H.12.8.4.4 (`PGEN-GRAMMAR-WELLFORMED-0143`, 2026-06-29), ledger `SV-0012`: **LIBRARY-COHORT LRM-FIDELITY GRAMMAR FIX — `include` / `library` productions now accept real IEEE 1800 §33 file paths (schema stays 6).** The `file_path_spec` lexeme (`kw_file_path_spec_c26c9dc9 := trivia /file_path_spec\b/`) was a **literal-keyword extraction artifact**: it matched only the literal word `file_path_spec`, so the `include`/`library` library-map productions **rejected every real file path** (`include ../rtl/cpu.v;`, `library mylib /path/to/*.sv;`, `library rtl ./src/*.sv, ./pkg/*.sv -incdir ./inc;` were all rejected at releases ≤ `1.0.149`) and accepted only the literal token text. IEEE 1800-2017 §33.3.1 / Annex A.1.1 define `file_path_spec` as a file-system *path* token (absolute/relative, wildcards `?`/`*`/`...`, `/` separators) — not a keyword. The fix replaces the lexeme with the LRM-faithful path char-class `/[A-Za-z0-9_.\/?*~$+]+/`, keeping the rule's **name and arity** so the typed carrier and AST-dump schema are byte-identical. **Why schema stays 6:** the rule is a single token at the same position — every typed AST shape is unchanged; the change is strictly-more-permissive (the `SV-0008`/`SV-0010`/`SV-0011` category). **Consumer impact:** if you parse SV **library map files** (the `library_text` analysis entry), `include`/`library` statements with real paths now parse (were rejected before); the `systemverilog_file` embedding entry is unaffected — the library cohort is unreachable from it, so embedding consumers see no change. **Documented limitation:** a hyphenated filename (`uvm-1.2`) is not accepted — `-` is excluded from the class so the `-incdir` flag boundary (`minus`+`kw_incdir`) stays separable (no §33 example uses a hyphenated path; a future negative-lookahead extension could admit it). VERIFIED: the three real-path forms REJECT→PASS and the literal `include file_path_spec;` still PASS; SV canonical cert `UNKNOWN=22` + 4-config union `UNKNOWN=3` (`witness=1300`) byte-identical seeds 0/7/42 (`spf=0`, library cohort incl. `kw_incdir` still witnesses); the 6 fully-certified grammars byte-identical `fully_certified=true`; **SV external corpus 14/14** (`parse_fail_total=0`); `ast_shape_contract` 18/18; `--lint-grammar` clean (`1425` rules, pre-existing `always_matches=8` A2 backlog unchanged); clippy source-clean; `cargo test --lib` 771 passed. Surfaced grammar-first by the cert-coverage `UNKNOWN`→0 / library-cohort LRM-fidelity drive (`H.12.8.4`). Full matrix in ledger row `SV-0012` and the contract's 1.0.150 schema-version note.

### 1.0.149 / Contract 1.0.149 — GRAMMAR-WELLFORMED.H.12.5.8.3.2 (`PGEN-GRAMMAR-WELLFORMED-0133`, 2026-06-25), ledger `SV-0011`: **SVA PROPERTY PRECEDENCE-CASCADE GRAMMAR FIX — the IEEE 1800 §16 property infix operators `until` / `s_until` / `until_with` / `s_until_with` / `iff` now parse (schema stays 6).** This is the **property-layer dual** of the 1.0.148 sequence cascade. `property_expr_sv_2017` / `property_expr_sv_2023` were flat **directly-left-recursive** definitions (`property_expr OP property_expr` for `or` / `and` / `iff` / the `until`-family / `implies_binary`); PGEN's left-recursion eliminator handles only the *indirect* bare-reference wrapper-chain shape, not direct inline LR, so the runtime cycle-breaker blocked every left-recursive infix branch. The four `until`-family operators are **property-only** (no other use site), so `a until b` / `a s_until b` / `a until_with b` / `a s_until_with b` (and property-level `a iff b`, and `or`/`and` over genuine property operands like `(always a) or (always b)`) were **rejected at the operator** on every release ≤ `1.0.148` — and the four `until`-family keyword rules were the SV certificate-coverage residual. The fix restructures both profiles into the IEEE 1800-2017 §16 (Table 16-3) **binary-infix precedence cascade** — `property_expr_sv_<profile> := prop_until_sv_<profile> -> $1` over 5 new `prop_*` rules per profile, loosest→tightest `prop_until > prop_iff > prop_or > prop_and > prop_primary` (all right-associative inline `head op self`, so no separate tail rules). **`prop_primary` keeps every non-binary-infix branch byte-identical** — `sequence` / `strong` / `weak` / `paren` / `not` / `nexttime*` / `always*` / `eventually*` / `accept_on`-family / `if` / `case` / `imp_minus` (`#-#`) / `imp_assign` (`#=#`) / `sequence_or_assign` (`|=`) / `instance` / `clocking` — with their bodies still referencing the `property_expr` dispatch. **What you get now:** SVA property infix operators parse with a precedence-nested AST — e.g. `a until b iff c` → `until(a, iff(b,c))` (until is looser than iff per Table 16-3); `a and b or c` → `or(and(a,b), c)`. **Why schema stays 6:** every previously-VALID property parse keeps its exact `{kind:"sv_2017"/"sv_2023"}`-wrapped carrier (the cascade unwraps single-operand levels via `-> $1`, and `prop_primary` is byte-identical — e.g. `not not a` is unchanged); the newly-parsing infix forms **reuse the existing documented `{kind, lhs, rhs}` carriers** (`until` / `s_until` / `until_with` / `s_until_with` / `iff` / `or` / `and`), so there are **no carrier changes at all** (strictly-more-permissive — the `SV-0008` / `SV-0010` category). **Consumer impact:** if you parse SVA properties with infix `until`-family / `iff` operators (or property-operand `or`/`and`), they now succeed (were rejected before); every other property form is byte-identical. **Documented limitation:** the *tight* prefix operators `not` / `nexttime` / `s_nexttime` (Table 16-3 level 6) keep their pre-fix dispatch-body precedence (they bind their whole rhs), so `not a until b` parses as `not (a until b)` rather than the strict-Table-16-3 `(not a) until b` — this is unchanged from the pre-fix behavior (the form simply rejected before), preserves schema 6, and these forms parenthesize in practice. VERIFIED: SV cert `UNKNOWN 26 → 22` (the 4 `until`-family keywords witnessed; `total 1299→1304` (+5 cascade rules), `witness 1272→1281`; deterministic seeds 0/7/42; `spf=0`; zero newly-unknown — set-diff = exactly the 4 `until`-family keywords); the previously-rejected property infix matrix REJECT→PASS with correct precedence, previously-working `not not a` / `always a` / `a #-# b` / `accept_on(a) b` byte-identical; the 6 fully-certified grammars BYTE-IDENTICAL (grammar-only, SV-isolated); **SV external corpus 14/14** (`parse_fail_total=0`); `ast_shape_contract` 18/18; `--lint-grammar` clean (`1425` rules, +10; `non_terminating=0`, `unreachable_rules=0`, `ordered_choice_shadowing=0`, `profile_orphans=0`); clippy source-clean. This completes the `H.12.5.8` SVA infix left-recursion class (sequence layer `SV-0010` + property layer `SV-0011`). Full matrix in ledger row `SV-0011` and the contract's 1.0.149 schema-version note.

### 1.0.148 / Contract 1.0.148 — GRAMMAR-WELLFORMED.H.12.5.8.3.1.1 (`PGEN-GRAMMAR-WELLFORMED-0132`, 2026-06-25), ledger `SV-0010`: **SVA SEQUENCE PRECEDENCE-CASCADE GRAMMAR FIX — the IEEE 1800 §16 sequence infix operators `and` / `or` / `intersect` / `within` now parse (schema stays 6).** The `sequence_expr` rule was a flat **directly-left-recursive** definition (`A := A op A | …`); PGEN's left-recursion eliminator handles only the *indirect* bare-reference wrapper-chain shape, not direct inline LR, so the runtime cycle-breaker blocked every left-recursive infix branch and `a and b` / `a or b` / `a intersect b` / `a within b` (and chained / mixed-precedence forms `a ##1 b ##2 c`, `a or b and c`) were **rejected at the operator** on every release ≤ `1.0.147`. The fix restructures `sequence_expr` into the IEEE 1800-2017 §16 (Table 16-3) **precedence cascade** — `sequence_expr := seq_or_expr -> $1` over 11 new `seq_*` rules, loosest→tightest `or > and > intersect > within > throughout > ## > unary`, each operator layer a `head tail tail*` chain with a named tail rule and a `-> $1` passthrough on bare operands. A companion parser-agnostic generator change (a reach-site passthrough preference in the cert-coverage reach planner) lets the cascade's deep operands witness cleanly. **What you get now:** SVA sequence infix operators parse with a precedence-nested AST — e.g. `a or b and c` → `{kind:"or", lhs:…, rest:[{kind:"and", …}]}`. **Why schema stays 6:** every previously-VALID sequence parse keeps its carrier — the cascade unwraps single-element levels via the `-> $1` passthroughs, so a simple `a [*2]` / `first_match(a)` / `x throughout a` is byte-identical; only the previously-REJECTED infix forms gain new `{kind:"or"/"and"/"intersect"/"within", lhs, rest}` shapes (strictly-more-permissive — the `SV-0008` category, distinct from the carrier-changing `SV-0009`). **Consumer impact:** if you parse SVA sequences with infix `and`/`or`/`intersect`/`within`, they now succeed (were rejected before); the simple sequence forms are unchanged. VERIFIED: SV cert `UNKNOWN 28 → 26` (`kw_intersect`/`kw_within` witnessed; `total 1288→1299` (+11 cascade rules), `witness 1259→1272`; deterministic seeds 0/7/42; `spf=0`; zero newly-unknown); `a and b`/`a or b`/`a intersect b`/`a within b`/`a ##1 b ##2 c`/`a or b and c`/`a intersect b within c` REJECT→PASS, previously-working `first_match(a)`/`a[*2]`/`x throughout a` still PASS; the 6 fully-certified grammars BYTE-IDENTICAL (the engine change inert); **SV external corpus 14/14** (`parse_fail_total=0`); `ast_shape_contract` 18/18; `--lint-grammar` clean (`1415` rules, +11; `non_terminating=0`, `unreachable_rules=0`, `ordered_choice_shadowing=0`); clippy source-clean. The `until`-family **property**-layer operators remain (tracked `H.12.5.8.3.2`, the property cascade — same shape, reuses this engine fix). Full matrix in ledger row `SV-0010` and the contract's 1.0.148 schema-version note.

### 1.0.147 / Contract 1.0.147 — GRAMMAR-WELLFORMED.H.14.3 (`PGEN-GRAMMAR-WELLFORMED-0130`, 2026-06-25), ledger `SV-0009`: **UDP TRUTH-TABLE ENTRY SHAPE RESTORE + EBNF self-hosting 12/12 — schema `5` → `6`.** The `combinational_entry` (`-> {inputs, output}`) and `sequential_entry` (`-> {inputs, current_state, next_state}`) typed carriers, documented in this book's [JSON carrier reference](json-carrier.md) since SV-Slice-18 (`1.0.18`), had **silently regressed to the raw `Sequence` envelope at SV-Slice-66 (`1.0.66`)** — a duplicate identical `->` return annotation was added on each rule (`grammars/systemverilog.ebnf:1168` / `:4639`), and because a single EBNF rule carries only ONE rule-level `return_annotation?`, the generated parser's return-transform compiler could not parse the doubled text and fell back to raw. Deleting the redundant 2nd `->` restores the documented typed shape. **What you get now:** parsing a UDP truth-table row yields the typed object, e.g. `combinational_entry → {"inputs":[{"kind":"0"},{"kind":"0"}],"output":{"kind":"0"}}` (the `:` and `;` tokens are folded away), instead of the raw token sequence. **Why schema bumped 5 → 6:** a previously-VALID UDP parse's carrier changes — the same criterion as the covergroup `SV-0006`/`SV-0007` carrier change (distinct from the SVA `SV-0008` case, which only affected previously-invalid input). **Consumer impact:** if you wrote code against the raw `Sequence` shape these rules emitted at releases `1.0.66`–`1.0.146`, switch to walking the typed `{inputs, output}` / `{inputs, current_state, next_state}` object documented here. This fix also closes `systemverilog.ebnf`'s last EBNF self-parse gap (`error_position=72476`), bringing the generated ebnf meta-parser to **12/12** self-hosting across the tracked grammar set. VERIFIED: SV cert seeds 0/7/42 `total=1288 witness=1259 UNKNOWN=28 spf=0` (unchanged); `ast_shape_contract` 18/18; **SV external corpus 14/14** (`parse_fail_total=0`); clippy source-clean; generated-parser byte-diff is 97 lines confined to exactly the two rules. Full matrix in ledger row `SV-0009` and the contract's 1.0.147 schema-version note.

### 1.0.146 / Contract 1.0.146 — SV-SVA-PROPERTY-FIDELITY.1 (`PGEN-SV-SVA-PROPERTY-FIDELITY-0001`, 2026-06-24), ledger `SV-0008`: **LRM DELIMITER-RESTORE GRAMMAR FIX — the six SVA bounded-property operators now require their IEEE 1800 §16 / Annex A.2.10 `[ ]` brackets (schema stays 5).** Extraction had dropped the literal `[ ]` on the six bounded-property operators in BOTH profiles — `nexttime [constant_expression]`, `s_nexttime [constant_expression]`, `always [cycle_delay_const_range_expression]`, `s_always [constant_range]`, `eventually [constant_range]`, `s_eventually [cycle_delay_const_range_expression]` — each was bracket-less `( <inner> )?`, so without the brackets the optional inner range/expr could not be delimited from the following `property_expr` and the LRM bracket forms were unparseable. The `sv_2017` `nexttime` was further mis-tokenized: the LRM nonterminal `constant_expression` became the literal keyword `constant` (`kw_constant_d810ca96 := trivia /constant\b/`) + a bare `expression`, so the dead `kw_constant_d810ca96` token was reachable only via that mis-synthesis (cert `UNKNOWN`). The fix restores the `[ ]` using the proven in-grammar `lbrack … rbrack` idiom (`const_expr:$3` / `range:$3`, `body:$5`) and deletes the dead token. **Schema stays 5** — the `{kind, const_expr/range, body}` shape is byte-identical (the brackets are parsed then folded away by the return annotation); the `sv_2017` `nexttime_const.const_expr` carrier is corrected from a bogus-only `(kw_constant expression)` group to `constant_expression`, now matching `sv_2023` + LRM (affecting only previously-invalid input). **Consumer notice:** LRM-bracketed bounded-property assertions `nexttime [N] p` / `s_nexttime [N] p` / `always [R] p` / `s_always [R] p` / `eventually [R] p` / `s_eventually [R] p` now parse (were rejected at all releases ≤ `1.0.145`); the non-LRM `nexttime constant N p` is now correctly rejected; the bare `nexttime p` / `always p` forms are unchanged. VERIFIED: SV cert-coverage `UNKNOWN 30 → 29` (the `kw_constant_d810ca96` token removed; `total 1289 → 1288`; witness `1258`; deterministic seeds 0/7/42; `spf=0`; zero newly-unknown — `comm -13` empty); the six LRM forms REJECT→PASS (both profiles), bare PASS, `nexttime constant 3 a` REJECT; `--lint-grammar` clean (pre-existing `always_matches=8` A2 backlog unchanged; `1404` rules, −1); **SV external corpus 14/14** (`parse_fail_total=0`); SV shape-contract GREEN (18/0); `cargo test --lib` 768/0; 6 fully-certified grammars inert (regex `198/198`); clippy source-clean. The property sibling of the covergroup `SV-0006`/`SV-0007` (and `SV-0004`/`SV-0002`) delimiter-drop class. Surfaced + adjudicated grammar-first by the cert-coverage `UNKNOWN`→0 drive (`STORE-AWARE-GEN.4b.15` 9A-ii). Full matrix in ledger row `SV-0008` and the contract's 1.0.146 schema-version note.

### 1.0.145 / Contract 1.0.145 — SV-COVERGROUP-FIDELITY.2 (`PGEN-SV-COVERGROUP-FIDELITY-0002`, 2026-06-24), ledger `SV-0007`: **LRM BINS-SET BRACE GRAMMAR FIX — the covergroup `bins_or_options` range-list form now requires its IEEE 1800 §A.2.11 braces `= { covergroup_range_list }` (schema 4 → 5).** Extraction had dropped the mandatory `{ }` on the bins range-list (`bins_or_options:643` was bare `assign covergroup_range_list*`). Because `covergroup_range_list → covergroup_value_range → covergroup_expression → expression` and the SV expression grammar parses `with(x)` as a `plain_tf` subroutine call, the range-list's first expression swallowed the entire `with(…)` (AST `kind:"call"/"plain_tf", name:"with"`) and the dedicated `( with … with_covergroup_expression … )?` clause stayed empty (`with_expr:[]`) → `with_covergroup_expression` was reachable-but-never-witnessed (cert `UNKNOWN`). The bins sibling of the `SV-0006` (covergroup trans-repeat), `SV-0004` (sequence-repetition) and `SV-0002` (streaming) delimiter-drop class. The fix restores the braces (`… assign lbrace covergroup_range_list rbrace ( with … )? …`, `ranges:$7, with_expr:$9, iff:$10`), so `with(…)` after `}` is the dedicated clause. **Schema 4 → 5** — the typed `range_list` carrier changes: `ranges` collapses one nesting level (`[[…]]` → `[…]`, now individual `covergroup_value_range`s, not one brace-concatenation) and `with_expr` becomes populatable (was always `[]`), the same carrier-shape class as the `SV-0002` bump. **Consumer notice:** the LRM braced bins range-list `bins b = {…} [with (…)]` now parses with the correct structure (and `with(…)` witnesses `with_covergroup_expression`); a bare single value `bins b = v` still parses (via the `set` branch) and `bins b = id with(e)` via `cover_point_with`; the non-LRM bare comma-list `bins b = 1,2,3` is now correctly rejected per IEEE 1800 §A.2.11 (must be `{1,2,3}`). Consumers reading the `range_list` bins AST must adopt the schema-`5` `ranges` (flattened) / `with_expr` (populatable) shape. VERIFIED: SV cert-coverage `UNKNOWN 31 → 30` (`with_covergroup_expression` witnessed; witness `1257 → 1258`; deterministic seeds 0/7/42; `spf=0`; zero newly-unknown — `comm -13` empty; `total=1289`); braced `bins b = {3} with(e)` populates `with_expr`, `bins b = {[0:63],65}` parses the embedded range, bare/`set`/`cover_point_with` forms parse, bare comma-list rejected; **SV external corpus 14/14** (`parse_fail_total=0` — no regression; no SV corpus/test exercises bins); SV shape-contract GREEN (18/0); `cargo test --lib` 768/0; 6 fully-certified grammars inert (regex `198/198`); clippy source-clean. Surfaced + adjudicated grammar-first by the cert-coverage `UNKNOWN`→0 drive (`SV-COVERGROUP-FIDELITY.2`). Full matrix in ledger row `SV-0007` and the contract's 1.0.145 schema-version note.

### 1.0.144 / Contract 1.0.144 — SV-COVERGROUP-FIDELITY.1 (`PGEN-SV-COVERGROUP-FIDELITY-0001`, 2026-06-24), ledger `SV-0006`: **LRM DELIMITER-RESTORE GRAMMAR FIX — the covergroup `trans_range_list` repeat forms now require their IEEE 1800 §A.2.11 `[ ]` brackets (schema stays 4).** Extraction had dropped the literal `[ ]` on the covergroup trans-repeat forms — `[* repeat_range]` (consecutive), `[-> repeat_range]` (goto), `[= repeat_range]` (nonconsecutive) — the direct covergroup sibling of the `SV-0004` sequence-repetition family (and `SV-0002` streaming). Pre-fix the bracket-less `trans_item ( star repeat_range )?` / `( implies … )?` / `( assign … )?` (`star := "*"`, `implies := "->"`, `assign := "="`) collided with the multiply/arrow/eq operators, so `trans_item`'s greedy `expression` parsed `a*b` as one multiplication — the optional `repeat_range` never fired → `repeat_range` was operator-shadowed (never witnessed → cert `UNKNOWN`) and the bracketed LRM forms `(1[*2])`/`(1[->2])`/`(1[=2])`/`(1=>2[*3])` were unparseable. The fix restores the `[ ]` using the proven in-grammar `lbrack … rbrack` idiom (`range: $4`). **Schema stays 4** — the emitted `{kind, range}` shape is byte-identical (the brackets are parsed then folded away by the return annotation). **Consumer notice:** LRM-bracketed covergroup trans repetitions `(1[*N])`/`(1[->N])`/`(1[=N])` now parse (were rejected at all releases ≤ `1.0.143`); a bare `(1*2)` still parses as a `simple` multiply trans_item (no regression). VERIFIED: SV cert-coverage `UNKNOWN 32 → 31` (`repeat_range` witnessed; witness `1256 → 1257`; deterministic seeds 0/7/42; `spf=0`; zero newly-unknown; `total=1289`); `(1[*2])`/`(1[->2])`/`(1[=2])`/`(1=>2[*3])`/`([1:2][*3])` REJECT→PASS; bare `(1*2)`/`(1)`/`(1=>2)`/`(1,2)` still PASS; `--lint-grammar` clean (pre-existing `always_matches=8` A2 backlog unchanged); **SV external corpus 14/14** (`parse_fail_total=0` — no regression); SV shape-contract GREEN (18/0; no SV sample exercises trans/bins); `cargo test --lib` 768/0; 6 fully-certified grammars inert (regex `198/198`); clippy source-clean. Surfaced + adjudicated grammar-first by the cert-coverage `UNKNOWN`→0 drive (`STORE-AWARE-GEN.4b.14` 9C-iii — the delimiter-drop class, exactly as `SV-0004`/`SV-0002`). Full matrix in ledger row `SV-0006` and the contract's 1.0.144 schema-version note. (The companion `with_covergroup_expression` bins-set-brace fidelity — LRM `= { covergroup_range_list }` — is tracked as `SV-COVERGROUP-FIDELITY.2`, a separate heavier accepted-language change.)

### 1.0.143 / Contract 1.0.143 — GRAMMAR-WELLFORMED.H.12.6.1 (`PGEN-GRAMMAR-WELLFORMED-0111`, 2026-06-17), ledger `SV-0005`: **PRODUCER LEFT-RECURSION GRAMMAR FIX (AST-shape + reachability) — ternary module-path conditions now emit a clean `module_path_conditional_expression` node, and the last `no_path` budget case is retired (schema stays 4).** `module_path_conditional_expression` (IEEE 1800 Annex A.8.3) was stranded by an indirect **left-recursion** in its producer `module_path_expression` (mpce's condition referenced `module_path_expression`, whose first branch was mpce). The left-recursion eliminator rewrote the producer into `module_path_expression_lr_base`/`_lr_suffix` and left `module_path_conditional_expression` as an **unreferenced** rewritten seed (its generated parse fn defined-but-never-called → `no_path`), while the `_lr_suffix` reconstruction leaked raw internal `wrapper_specs` metadata into the ternary module-path AST. The fix makes mpce's condition the non-left-recursive `module_path_expression_operand` chain (same accepted language — the right-associative ternary reading), so `module_path_expression` is natively non-left-recursive (no elimination; the `…_lr_base`/`…_lr_suffix` synthetic rules vanish), mpce is positively reachable + **witnessed**, and the `wrapper_specs` leak is gone. **Schema stays 4** — the conditional module-path's prior `wrapper_specs`-leaking form was never a realized clean shape a consumer could build against, and the common (non-conditional) chain module-path is byte-identical. **Consumer notice:** ternary module-path conditions (`a ? b : c` in `specify` state-dependent paths) now emit a clean `{kind:"conditional", body:{condition, attributes, then_expr, else_expr}}` node instead of leaking raw `wrapper_specs`; the accepted language is unchanged (the ternary parsed before too); non-conditional module paths are byte-identical. VERIFIED: SV cert-coverage `UNKNOWN 86 → 84` (`module_path_conditional_expression` witnessed + `module_path_expression_lr_suffix` removed; witness `1204`; total `1291 → 1289`; deterministic seeds 0/7/42; `spf=0`); `no_path 20 → 19`; the genuineness oracle emits a clean `conditional` node with `0` `wrapper_specs` (was `0`/`3`); syntax-closure `unreachable_rules=0` (`max_unreachable_rules` budget `1 → 0`, contract v6 — the linter's "no unreachable rules" is now literally true for SV); **SV external corpus 14/14** (`parse_fail_total=0`); `stimuli_cross_family_platform_gate` PASS; SV shape-contract GREEN; clippy source-clean. Surfaced + adjudicated grammar-first by the cert-coverage `UNKNOWN`→0 drive (`GRAMMAR-WELLFORMED.H.12.6` `no_path` re-audit) — a producer-wiring flaw FIXED, never deleted. Full matrix in ledger row `SV-0005` and the contract's 1.0.143 schema-version note.

### 1.0.142 / Contract 1.0.142 — GRAMMAR-WELLFORMED.H.12.5.5.3.3.3 (`PGEN-GRAMMAR-WELLFORMED-0105`, 2026-06-17), ledger `SV-0004`: **LRM DELIMITER-RESTORE GRAMMAR FIX — the `boolean_abbrev` sequence-repetition family now requires its IEEE 1800 §A.8.1 `[ ]` brackets (schema stays 4).** Extraction had dropped the literal `[ ]` on `consecutive_repetition` (`[* N]` / `[*]` / `[+]`), `goto_repetition` (`[-> N]`), and `non_consecutive_repetition` (`[= N]`, both profiles), so the bracket-less forms collided with the `*` / `->` / `=` operators: the grammar **REJECTED valid SV** sequence repetitions (`a[*3]`, `a[->2]`, `a[=2]`) and wrongly accepted the bracket-less spellings; `goto_repetition` was operator-shadowed (never witnessed → cert `UNKNOWN`). The fix restores the `[ ]` using the proven in-grammar `lbrack … rbrack` idiom (`range: $3`). **Schema stays 4** — the emitted `{kind, range}` / `{range}` shape is byte-identical (the brackets are parsed then folded away by the return annotation). **Consumer notice:** LRM-bracketed sequence repetitions `[*N]`/`[*]`/`[+]`/`[->N]`/`[=N]` now parse (were rejected at all releases ≤ `1.0.141`); the bare `*N`/`->N`/`=N` sequence-repetition spellings are now correctly rejected per the LRM; a bare `a*3` still parses as a multiplication expression (no regression). VERIFIED: SV cert-coverage `UNKNOWN 87 → 86` (`goto_repetition` closed; witness `1203 → 1204`; deterministic seeds 0/7/42; `spf=0`; zero newly-unknown; 1291 rules); `--lint-grammar` clean (pre-existing `always_matches=6` A2 backlog unchanged); **SV external corpus 14/14** (`parse_fail_total=0`, both uvm cases — no regression); SV shape-contract GREEN (samples=3 aligned=3 drift=0); clippy source-clean. Surfaced + adjudicated grammar-first by the cert-coverage `UNKNOWN`→0 drive (the attribution rule's delimiter-drop class, same family as `SV-0002`). Full matrix in ledger row `SV-0004` and the contract's 1.0.142 schema-version note.

### 1.0.141 / Contract 1.0.141 — GRAMMAR-WELLFORMED.H.12.5.5.3.3.4.2.2.1 (`PGEN-GRAMMAR-WELLFORMED-0100`, 2026-06-16), ledger `SV-0003`: **STORE-GATE GRAMMAR FIX — a module-scope class-handle declaration `C a;` is no longer mis-parsed as a `net_declaration` (schema stays 4).** `checked_nettype_identifier`'s `@predicate` is tightened from `has_fact(type_name, $body)` to `fact_attribute_equals(type_name, $body, declaration_family, nettype)`, so the `net_declaration` user-nettype branch resolves ONLY to a declared nettype. A declared **class** is also a `type_name`, so the bare gate wrongly consumed `C a;` (C a declared class) as a net declaration — emitting no `variable_binding` fact and blocking class-handle member-method chains (`C a; … a.b[0].c()` was REJECTED). Post-fix `C a;` routes to `data_declaration` (a class variable — per IEEE 1800 a class is not a `net_type`) and the chain parses and witnesses `context_member_method`. **Schema stays 4** — no new shape vocabulary; `C a;` re-routes between the two *existing* `net_declaration`/`data_declaration` shapes. **Consumer notice:** acceptance + categorisation correction — a module-scope class-handle declaration now parses as `data_declaration` (was `net_declaration`) and module-scope class-handle member-method chains now parse; built-in nets (`wire a;`) and real user nettypes (`nettype logic NT; NT a;`) are unaffected. VERIFIED: SV cert-coverage byte-identical (`total=1292 witness=1203 UNKNOWN=88 spf=0`, seeds 0/7/42); `--lint-grammar` clean; SV external corpus 14/14; `stimuli_cross_family_platform_gate` PASS; SV shape-contract GREEN (+ the `systemverilog_class_handle_decl_is_data_declaration_not_net` lock). Surfaced by the cert-coverage `UNKNOWN`→0 drive as a bug-finding oracle (the `-0099` class-handle WHY+WHERE). Full matrix in ledger row `SV-0003` and the contract's 1.0.141 schema-version note.

### 1.0.140 / Contract 1.0.140 — GRAMMAR-WELLFORMED.H.12.5.5.3.3.5 (`PGEN-GRAMMAR-WELLFORMED-0096`, 2026-06-16), ledger `SV-0002`: **GRAMMAR FIX — restores the LRM stream-concatenation delimiters dropped in extraction (schema 3 → 4).** `stream_concatenation` regains its mandatory inner `{ }` (and `body` becomes a clean array of stream-expressions), `stream_expression` regains the literal `[ ]` around its `with`-clause range, and `streaming_concatenation`'s optional `slice_size` is guarded `( slice_size &lbrace )?` so it no longer greedily eats the brace-concatenation that IS the mandatory stream-concatenation. Effect: the LRM with-bracket form `{<< 4 {a with [3:0]}}` now parses (was REJECTED at every shipped release), the canonical brace forms `{>> {data}}` / `{<< 8 {a, b}}` parse with the *correct* `stream_concatenation` structure, and a bare `with X` without brackets is correctly rejected per IEEE 1800 §A.8.1. **Schema 3 → 4** — `stream_concatenation.body` changes from a raw quantified node to an array (`[$2, $3::2*]`); consumers reading streaming-concatenation ASTs must adopt the new shape (inputs with no streaming concatenation are byte-identical). Closes the certificate-coverage `array_range_expression` `UNKNOWN` (SV cert `UNKNOWN 89 → 88`, seeds 0/7/42, `spf=0`). VERIFIED: SV external corpus 14/14 (uvm's own `{>>{i}}` / `{<<{v}}` streaming forms parse — no regression); `--lint-grammar` clean; `stimuli_cross_family_platform_gate` PASS. Full matrix in ledger row `SV-0002` and the contract's 1.0.140 Highlights.

### 1.0.139 / Contract 1.0.139 — GRAMMAR-WELLFORMED.H.11.5-FIX (`PGEN-GRAMMAR-WELLFORMED-0075`, 2026-06-11), ledger `SV-0001`: **ENGINE FIX — the generated layout skipper's `#`-to-EOL comment arm no longer exists in the SV parser**, so it can no longer steal a real `#` token (delays, `#( )` parameter lists) during speculative comment attempts and memo-poison the parse. Grammar-valid `#`+comment shapes that were rejected on every shipped release now parse. Release bump, schema stays 3; acceptance widening only for grammar-valid inputs (every previously-parsed input yields a byte-identical AST). Full matrix in the ledger row.

### 1.0.136 / Contract 1.0.136 — SV-EXH-PROOF.5.2.4.1 (`PGEN-SV-EXH-PROOF-0107`, 2026-05-30): **BEHAVIOUR-TIGHTENING grammar fix — a scoped type reference `Head::member` where `Head` is a plain (non-class) `typedef` is now correctly REJECTED in `data_type` position.** Release bump, schema stays 3.

> SV-EXH-PROOF.5.2.4.1 (`PGEN-SV-EXH-PROOF-0107`, 2026-05-30). Surfaced by the `sv_semantic_scope_contract` gate's `parameter_typedef_fail` case: `module m; typedef int T; localparam int Y = T::P; endmodule` was wrongly ACCEPTED. `T` is a plain `typedef int` (not a class/package), so the scoped form `T::P` is invalid per IEEE 1800 (a non-class type has no scope members). **Root cause (tools-first, `--parse-dump-ast-pretty`):** the value `T::P` parsed via `constant_param_expression`'s `data_type` branch → `data_type`'s `provisional_unscoped_block_class_type` branch, which was gated ONLY by `has_fact(type_name, head)`; since `typedef int T` emits a `type_name` fact, `T` satisfied that gate even with the `::P` scope_chain present. **Fix:** `data_type`'s class-type branch now references the typedef-EXCLUDING sibling `known_unscoped_block_class_type` (gated `has_fact(type_name, head)` **and** `lacks_fact_attribute_equals(type_name, head, declaration_family, typedef)`) instead of the permissive `provisional_*`. AST shape UNCHANGED (same `{head, params, scope_chain}` body, same `provisional_class_type` AST kind), so schema stays 3. `block_data_type` (class-internal scoped types) keeps the permissive `provisional_*` branch (minimal blast radius). **Consumer notice (behaviour-tightening, same category as 1.0.127):** inputs that accidentally relied on `Head::member` being accepted for a non-class typedef head are now rejected; valid forms — bare typedef-as-datatype `T x;`, package scope `pkg::C`, real class scope `Class::member` — are unaffected. **EMPIRICAL VERIFICATION:** minimal repro now rejected (furthest_position=52, does not consume full input); the three valid sanity forms still pass; `sv_semantic_scope_contract` gate **20/20** (was 19/20); **SV external corpus 14/14** (no uvm regression — uvm uses scope-resolution heavily); SV shape-contract GREEN; lib (--features generated_parsers, `--lib`) **609/609**. NO-WORKAROUNDS level 1 (existing semantic-annotation gate, per the store-consultation principle).

### 1.0.135 / Contract 1.0.135 — SV-EXH-PROOF.3.3.4.b.6.2.PP.1 (`PGEN-SV-EXH-PROOF-0099`, 2026-05-28): **SV PREPROCESSOR FIX — macro string-substitution `\`"ARG[+]\`"` now handles literal text between the macro-arg and the closing `\`"`** — uvm_pkg ×{2017,2023} flip FAIL → PASS, **SV external corpus 10/14 → 12/14**! Release bump, schema stays 3, strictly-more-permissive.

> SV-EXH-PROOF.3.3.4.b.6.2.PP.1 (`PGEN-SV-EXH-PROOF-0099`, 2026-05-28). The `.PP.0` investigation (also this slice; pure-docs framing absorbed into the fix slice since the fix turned out to be ~50 lines) pinned that `.37.10`'s final 20K-bytes-from-end position was UNEXPANDED UVM `\`uvm_field_*` macro template residue. Per LRM §22.5.3.2, `\`"` opens a "macro string with substitution" region terminated by a matching `\`"`. Pre-fix `substitute_function_macro_body` in `rust/src/sv_preprocessor.rs:1714-1763` assumed the region contained ONLY a single macro-arg identifier — so UVM's `\`m_uvm_field_qda_int`'s `\`"ARG[+]\`"` pattern was mis-expanded: `[+]` was emitted bare and the closing `\`"` became literal `\`"`. Fix: scan forward to the matching closing `\`"`, substitute macro args within the entire region content, emit a single `"..."` string literal. Backwards-compat preserved via fallthrough to the legacy single-ident form when no matching close is found. **uvm_pkg ×{2017,2023} now PASS** (were the deepest residual since the SV-EXH-PROOF tree began). Tools-first per Recipe 2 + a focused source-read of the preprocessor function. **EMPIRICAL VERIFICATION:** minimal repro `\`define t(ARG) push_back(\`"ARG[+]\`")` / `\`t(abc)` produces `push_back("abc[+]")` post-fix (was `push_back("abc"[+]\`")`); lib (--features generated_parsers) **609/609 PASS**; lib (no-features) **548/548 PASS**; SV preprocessor unit tests 22/22 PASS; clippy ✅; **SV external corpus triage gate 12 PASS / 2 FAIL / 0 TIMEOUT** (was 10/4/0 — uvm_pkg ×{2017,2023} flipped FAIL → PASS). Residual 2 FAIL = uvm_compat_pkg ×{2017,2023} which is H1 territory (cross-package extends, depends on uvm_pkg producing a library artifact — now unblocked since uvm_pkg parses).

### 1.0.134 / Contract 1.0.134 — SV-EXH-PROOF.3.3.4.b.6.2.37.10 (`PGEN-SV-EXH-PROOF-0097`, 2026-05-27): SV grammar fix — new `implicit_class_rooted_method_chain` rule + integration into `call_primary` + `method_call_initial` so `this.member[i].method()` / `super.member[i].method()` 3-level chains parse. Release bump, schema stays 3, strictly-more-permissive. uvm_pkg furthest 2,229,367 → 3,006,234 — **~99.3% through, 20K bytes from the end**.

> SV-EXH-PROOF.3.3.4.b.6.2.37.10 (`PGEN-SV-EXH-PROOF-0097`, 2026-05-27). Surfaced by `.37.9`'s 2,229,367 furthest reaching `this.m_update_thread[element].kill();` at uvm_pkg line 66850. Per IEEE 1800 §A.6.6+§A.8.4 `class_qualifier` includes `implicit_class_handle .` (where `implicit_class_handle` covers `this` / `super` / `this.super`) as a primary-prefix, so 3+level chains rooted at `this`/`super` are LRM-valid. **Structural complement of `.37.5`'s `context_member_method_call` fix:** that slice handled IDENTIFIER-rooted 3+level chains (`a.b[i].method()`) via the `has_fact(variable_binding, $head)`-gated rule; this slice handles IMPLICIT-CLASS-HANDLE-rooted chains (`this.b[i].method()`) via a new parallel rule `implicit_class_rooted_method_chain` (no predicate gate needed — `this`/`super` are keywords, unambiguous). The new rule has the same `constant_bit_select &dot` member-loop shape from `.37.5` so indexed members parse, plus a `constant_bit_select` on the head identifier so the FIRST member can also be indexed. Integrated into BOTH `call_primary` (expression context) AND `method_call_initial` (statement context — `subroutine_call` path used by `this.x.kill();`-as-statement). Tools-first per Recipe 2 — byte 2,229,367 → line 66850 col 42 → minimal repro `this.arr[0].kill();` FAILed immediately. **Release bump, NO schema bump** — strictly-more-permissive (SVPP-0002/REGEX-0083). VERIFIED: 5-way repro all PASS — `arr[0].kill()` (was PASS) / `this.arr[0].kill()` (was FAIL) / `this.arr.kill()` (was PASS) / `super.arr[0].kill()` (was FAIL) / `this.x[i].sub[j].run()` (was FAIL); lib (--features generated_parsers) **609/609 PASS**; lib (no-features) **548/548 PASS**; clippy ✅; SV external corpus triage gate **10 PASS / 4 FAIL / 0 TIMEOUT** — binary stable; **uvm_pkg furthest_position 2,229,367 → 3,006,234** (+776,867 bytes deeper, +35% more; reaches ~3.006M of ~3.026M preprocessed bytes = **~99.3% through uvm_pkg, only 20K bytes from the end**). uvm_compat_pkg unchanged at 116752 (H1 territory).

### 1.0.133 / Contract 1.0.133 — SV-EXH-PROOF.3.3.4.b.6.2.37.9 (`PGEN-SV-EXH-PROOF-0096`, 2026-05-27): SV grammar fix — `base_class_type` accepts type-parameter heads so `class C #(type T) extends T;` parses. Release bump, schema stays 3, strictly-more-permissive.

> SV-EXH-PROOF.3.3.4.b.6.2.37.9 (`PGEN-SV-EXH-PROOF-0096`, 2026-05-27). Surfaced by `.37.8`'s 1,582,112 furthest reaching `uvm_port_base`'s `virtual class uvm_port_base #(type IF=uvm_void) extends IF;`. Per IEEE 1800 §A.1.2 a class's `extends class_type` accepts any `class_type` — including one whose head is a type-parameter. Pre-fix `base_class_type` (line 1090) accepted only `class` and `pkg::class` heads (gated by `fact_attribute_equals(type_name, $body, declaration_family, class)` for the unscoped branch) — missing the type-parameter alternative (gated by `declaration_family=type_parameter`). **Fix:** new sibling rule `known_unscoped_base_class_type_parameter_identifier` (gated by the type_parameter predicate) added as a third alternative in `base_class_type`'s head choice (alongside `scoped_base_class_type_identifier` and `known_unscoped_base_class_type_identifier`). Pure grammar LRM alignment; existing predicate primitive reused. Tools-first per Recipe 2 from `docs/book/src/parseability-probe-debug.md` — pinned the defect in ~2 min via furthest_position byte → line+col → minimal repro. **Release bump, NO schema bump** — strictly-more-permissive (SVPP-0002/REGEX-0083). VERIFIED: 4-way extends repro all PASS (`extends concrete` / `extends concrete#(int)` / `extends IF` / `extends IF#(int)`); lib (--features generated_parsers) **609/609 PASS**; lib (no-features) **548/548 PASS**; clippy ✅; SV external corpus triage gate **10 PASS / 4 FAIL / 0 TIMEOUT** — binary stable; **uvm_pkg furthest_position 1,582,112 → 2,229,367** (+647K bytes deeper, ~41% more — biggest single-slice gain since `.37.2`; reaches ~2.23M of ~3.0M preprocessed bytes = ~74% through uvm_pkg). uvm_compat_pkg unchanged at 116752 (H1 territory).

### 1.0.132 / Contract 1.0.132 — SV-EXH-PROOF.3.3.4.b.6.2.37.8 (`PGEN-SV-EXH-PROOF-0095`, 2026-05-27): SV grammar LRM-extraction fix — `empty_unpacked_array_concatenation` now requires the LRM literal `'{ }` (tick + braces) instead of the prior `lbrace epsilon rbrace` (which was missing the tick AND referenced an undefined `epsilon` rule, so the construct was never parseable). Release bump, schema stays 3, strictly-more-permissive.

> SV-EXH-PROOF.3.3.4.b.6.2.37.8 (`PGEN-SV-EXH-PROOF-0095`, 2026-05-27). Surfaced by `.37.7`'s 1,508,106 furthest reaching `uvm_cache::get`'s `return '{};`. Per LRM §A.6.7: `empty_unpacked_array_concatenation ::= '{ }`. pgen's rule at line 1937 was `lbrace epsilon rbrace` with TWO defects: (1) missing the leading `tick` (apostrophe), (2) `epsilon` is not a defined rule in the grammar (no `epsilon :=` exists), so the rule referenced an undefined symbol and could never match at all. Fix: replace with `tick lbrace rbrace` per LRM literal. Pure grammar LRM alignment; no engine change. Per [[feedback_tools_first_no_guessing]] — used the documented Recipe 2 from `docs/book/src/parseability-probe-debug.md` (furthest_position byte → line + col → minimal repro of the visible syntax) to pin the defect in 2 minutes without bisecting. **Release bump, NO schema bump** — strictly-more-permissive (SVPP-0002/REGEX-0083 category; previously-undefinable `'{}` now parses). VERIFIED: minimal repros `q = '{};` + `q = '{1, 2};` PASS; lib (--features generated_parsers) **609/609 PASS**; lib (no-features) **548/548 PASS**; clippy ✅; SV external corpus triage gate **10 PASS / 4 FAIL / 0 TIMEOUT** — binary stable; **uvm_pkg furthest_position 1,508,106 → 1,582,112** (+74K bytes deeper, ~5% more; reaches ~1.58M of ~3.0M preprocessed bytes = ~53% through uvm_pkg). uvm_compat_pkg unchanged at 116752 (H1 territory).

### 1.0.131 / Contract 1.0.131 — SV-EXH-PROOF.3.3.4.b.6.2.37.7 (`PGEN-SV-EXH-PROOF-0094`, 2026-05-26): SV grammar fix — `expression_or_cond_pattern` gains `inside_expression` sibling branch so `if (x inside {...})` parses (release bump, schema stays 3, strictly-more-permissive)

> SV-EXH-PROOF.3.3.4.b.6.2.37.7 (`PGEN-SV-EXH-PROOF-0094`, 2026-05-26). Surfaced by `.37.6`'s 1,278,335 furthest_position reaching `uvm_root::die` where `if (get_core_state() inside {UVM_CORE_PRE_ABORT, UVM_CORE_ABORTED})` was unparseable — `inside_expression` reachable from `expression` (`expression := expression_base | inside_expression`) but NOT from `expression_or_cond_pattern` (which used `expression_base` directly). The `cond_predicate` path used by `if`/`while`/`for` conditions therefore bypassed `inside_expression`. **Fix:** add `inside_expression` as the first sibling alternative in `expression_or_cond_pattern` (keeping `expression_base` second and `cond_pattern` third so the existing parse priority is preserved). The naive `expression_base → expression` replacement caused a uvm_pkg furthest regression 1,278,335 → 191,248 during in-slice verification (probably PEG ambiguity in deep nested expressions); the sibling-branch alternative avoids that regression entirely. **Release bump, NO schema bump** — strictly-more-permissive (previously-unparseable `if (x inside {…})` now parses); SVPP-0002/REGEX-0083 category. VERIFIED: 6-way minimal repro all PASS (`if`/`while` condition + `inside` with literal-set / range / identifier-set); lib (--features generated_parsers) **609/609 PASS**; lib (no-features) **548/548 PASS**; clippy ✅; SV external corpus triage gate **10 PASS / 4 FAIL / 0 TIMEOUT** — binary stable; **uvm_pkg furthest_position 1,278,335 → 1,508,106** (+230K bytes deeper, ~18% more; reaches ~1.51M of ~3.0M preprocessed bytes = ~50% through uvm_pkg). uvm_compat_pkg unchanged at 116752 (H1 territory).

### 1.0.130 / Contract 1.0.130 — SV-EXH-PROOF.3.3.4.b.6.2.37.6 (`PGEN-SV-EXH-PROOF-0093`, 2026-05-26): SV grammar LRM-extraction fix — `inside_expression` + `value_range` now require LRM brackets (`inside { ... }` braces around the range list, and `[ expr : expr ]` brackets around the range alternative) — release bump, schema stays 3, strictly-more-permissive

> SV-EXH-PROOF.3.3.4.b.6.2.37.6 (`PGEN-SV-EXH-PROOF-0093`, 2026-05-26). Surfaced by `.37.5`'s 1,121,290 furthest_position reaching `uvm_phase::jump`'s `phases = phases.find(item) with (item.get_state() inside {[UVM_PHASE_STARTED:UVM_PHASE_CLEANUP]});`. Two LRM-extraction defects (one rule each, both in `grammars/systemverilog.ebnf`):
>
> **(a) `value_range_sv_2017` (line 5313) + `value_range_sv_2023` (line 5317).** Per IEEE 1800 §A.8.3 the LRM defines `value_range ::= expression | [ expression : expression ]`. The pgen rule had `( expression colon expression )?` — missing the brackets AND wrapped in `?` which made the rule silently match empty as a fallback. Fix: require `lbrack expression colon expression rbrack` literally; emit `{kind: "range", lo: $2, hi: $4}` (the `lo`/`hi` convention from line 538's `constant_range_expression`). sv_2023 same fix for all 4 bracketed range variants (range, dollar_lo, dollar_hi, tolerance — each now wrapped in `lbrack ... rbrack`).
>
> **(b) `inside_expression_sv_2017` (line 2374) + `inside_expression_sv_2023` (line 2378).** Per LRM §A.8.3 the `inside_expression ::= expression inside { open_range_list }` literal `{` and `}` braces. The pgen rule had `kw_inside open_range_list*` — missing the braces AND wrapped in `*` which made the rule silently succeed without consuming the list. Fix: require `lbrace open_range_list rbrace` literally (matching the LRM exactly).
>
> Both fixes are strictly-more-permissive (previously-unparseable `inside {[1:5]}` now parses; previously-PASSING `inside {1, 2}` still parses through the same path, just now via the explicit braces). NO new annotation, NO engine change — pure grammar LRM alignment. VERIFIED: 5-way inside-variant repro all PASS (`inside {1, 2}`, `inside {[1:5]}`, `inside {[1:5], 10}`, `inside {[$:5]}`, `inside {[1:$]}`); lib (--features generated_parsers) **609/609 PASS**; lib (no-features) **548/548 PASS**; clippy ✅; SV external corpus triage gate **10 PASS / 4 FAIL / 0 TIMEOUT** — binary stable; **uvm_pkg furthest_position 1,121,290 → 1,278,335** (+157K bytes deeper, ~14% more; reaches ~1.28M of ~3.0M preprocessed bytes = ~42% through uvm_pkg). uvm_compat_pkg unchanged at 116752 (H1 territory).

### 1.0.129 / Contract 1.0.129 — SV-EXH-PROOF.3.3.4.b.6.2.37.5 (`PGEN-SV-EXH-PROOF-0092`, 2026-05-26): SV grammar fix — `context_member_method_call`'s member-loop gains `constant_bit_select` after each identifier so `obj.member[i].method(args)` 3+level chains with indexed members parse cleanly (release bump, schema stays 3, strictly-more-permissive)

> SV-EXH-PROOF.3.3.4.b.6.2.37.5 (`PGEN-SV-EXH-PROOF-0092`, 2026-05-26). Surfaced by `.37.3`'s 866292 furthest_position which reached uvm_pkg's `uvm_report_message_element_container::do_copy` foreach body: `elements.push_back(urme_container.elements[i].clone())` — the argument `c.elements[i].clone()` is a 3-level chain `instance . indexed_member . method_call`. Pre-fix the `context_member_method_call` rule (added by `.b.6.2` for identifier-rooted chains like `a.b.c(x)`) used `( dot identifier &dot )+` for intermediate members — bare identifier only, no bit-select. So `dot elements [i]` failed the `&dot` lookahead (next char is `[` not `.`) and the whole rule was rejected. **Fix (one rule edit, 1 token addition):** the member loop becomes `( dot identifier constant_bit_select &dot )+` — `constant_bit_select` is `( lbrack constant_expression rbrack )*` so matches empty when there's no `[...]`. Backwards-compatible: `a.b.c.method()` parses byte-identically to pre-fix (no shape change on the no-index path); `a.b[i].c.method()` (was unparseable) now parses with the same `{head, members, method, chain}` shape. **Release bump, NO schema bump** — strictly-more-permissive (previously-unparseable inputs now parse; no shape change); SVPP-0002/REGEX-0083 category. VERIFIED: 6-way minimal repro all PASS (`c.elements.clone()`, `c.elements[i].clone()`, `c.elements[0].clone()`, `c.a.b.c.method()`, `c.a.b[i].c.method()`, `c.a[i].b.method()`); lib (--features generated_parsers) **609/609 PASS**; lib (no-features) **548/548 PASS**; clippy ✅; SV external corpus triage gate **10 PASS / 4 FAIL / 0 TIMEOUT** — binary stable; **uvm_pkg furthest_position 866292 → 1,121,290** (+255K bytes deeper, ~30% more); uvm_compat_pkg unchanged at 116752 (H1 territory). uvm_pkg furthest now ~1.12M of ~3.0M preprocessed bytes.

### 1.0.128 / Contract 1.0.128 — SV-EXH-PROOF.3.3.4.b.6.2.{36.4, 36.5, 37.0, 37.1, 37.2, 37.3} bundled release (`PGEN-SV-EXH-PROOF-{0085, 0086, 0087, 0088, 0089, 0090}`, 2026-05-25/26): combined landing of (`.36.4`) engine memoization-delta replay + (`.36.5`) bootstrap `**` → FlattenSpread fix + (`.37.0/.37.1/.37.2`) H2 SV-stdlib auto-load (`process`/`semaphore`/`mailbox` as built-in classes via `parser_libs/sv_<profile>_std/`) + (`.37.3`) SV grammar `Class::member` as primary expression + lvalue (`primary_hier_scope_prefix` + `variable_lvalue_scope` + `nonrange_variable_lvalue` all gain a `class_scope` branch). uvm_pkg furthest_position arc 162162 → 866292 (~5.3× deeper than `.35.1`'s starting point). Corpus binary 10/14 stable (residual = uvm × 4). Release bump, schema stays 3 (strictly-more-permissive throughout).

> **Why a bundle.** `.36.4 / .36.5 / .37.2` each individually said "release-bump candidate, deferred to a combined landing"; `.37.0` was investigation; `.37.1` was data-only; `.37.3` is the natural break that lands them all together with a single 1.0.128 bump. AST shape vocabulary is unchanged across the entire bundle. The contract version moves once. The book changelog entry below documents what changed for consumers.

### 1.0.127 / Contract 1.0.127 — SV-EXH-PROOF.3.3.4.b.6.2.35.1: SV grammar fix — gate `provisional_unscoped_block_class_type` with `@predicate has_fact(type_name, $head.body)` (Slice-64 redux, lands clean now; release bump, schema stays 3)

> Slice-64 (`9375c069`, 2026-05-24) originally added this exact one-line `@predicate` annotation but was reverted (`8eae9eba`) under the (now-retired) "fact persistence" framing. Slice-69 (`.b.6.2.30`, 2026-05-25) re-diagnosed the implicit-type port-list defect using the new `--trace-rules` tool and proved the root cause is **structural**, not persistence: when parsing `function f(string a, b="");`, every gated type-identifier alternative correctly **rejected** the undeclared `b` (`has_fact(type_name, b) → false`); only the **ungated** `provisional_unscoped_block_class_type` accepted it. The semantic store was always working — the grammar was just bypassing it. This slice (`.b.6.2.35.1`) reapplies Slice-64's exact one-line annotation as the first code sub-leaf of the `.b.6.2.35` umbrella that systematically remediates the ungated-identifier-categorisation defect class (1 + 9 + 3 + 3 audit hits across `systemverilog.ebnf`). Under the current engine state (Slice-67's C3-B in place, persistence framing retired by Slice-69), the gate lands without an adjacent producer fix — exactly the outcome Slice-69's evidence predicted. **Fix (one line):** `@predicate: { name: has_fact, args: [type_name, $head.body], phase: post, view: shaped }` added directly above `provisional_unscoped_block_class_type` at `grammars/systemverilog.ebnf:1638`. The sibling `known_unscoped_block_class_type` already carries this exact predicate plus a `lacks_fact_attribute_equals(... typedef)` companion; the `provisional_*` variant differs by now ACCEPTING typedef'd type names where the `known_*` variant rejects them, but both now require the identifier to be a known type. The catch-all-for-unknowns role is gone — but that role was masking real defects. **Test adjustment** (one test, `ast_shape_contract.rs::systemverilog_context_gated_method_chain_handles_negated_and_uvm_shape`): the "uvm-shaped function" case had `uvm_seed_map seed_map;` with `uvm_seed_map` never declared. Pre-gate the catch-all accepted any bare identifier as a type; post-gate undeclared types are correctly rejected per Slice-64's stated semantics ("UnknownGarbage → rejected → forces user to fix their code"). Added `typedef class uvm_seed_map;` to the test input — a real-UVM-style forward declaration — so the gate sees a `type_name` fact for it; test purpose (3-level method-chain parsing) preserved. **Release bump, NO schema bump** — AST shape vocabulary unchanged (`{head, params, scope_chain}` shape is the same; only the acceptance criterion changes). This is a **behaviour-tightening** correctness fix (a previously-accepted incorrect parse path — undeclared types — is now correctly rejected per LRM); consumers whose inputs accidentally depended on the over-permissive acceptance will see those inputs correctly rejected. Different category from SVPP-0002/REGEX-0083 (strictly-more-permissive) — closest precedent is RGX-0087's `\89`-leading hard-reject (correctness tightening). **NO-WORKAROUNDS HIERARCHY:** this fix is **level 1** — use existing semantic-annotation primitive. No new annotation, store, or engine machinery needed. The proven `.3.3.1`/`.3.3.2`/`.3.3.3` declared-id idiom applied to one more rule. Sign-off-quality at the lowest layer in the fix hierarchy per [[feedback_no_workarounds_fix_hierarchy]]. **Architectural principle reinforced** ([[feedback_grammar_rules_must_consult_store]], user-set 2026-05-25): rules that categorise bare identifiers into tracked categories (type/class/interface/package/...) SHALL consult the store via has_fact / fact_attribute_equals / lacks_fact. This slice is the canonical exemplar; the umbrella `.b.6.2.35.{2..16}` will apply the same principle to the remaining 9 + 3 + 3 audit hits. VERIFIED: iso5/iso4 minimal repros (`module m; function bit f(string a, b="");` / `..., b=""` — the implicit-type 2nd-port-with-default that drove Slice-69's diagnosis) **PASS** (were FAIL); iso1/iso2/iso3/iso6 controls PASS (no regression on minimal repros); forward-declared `typedef class Bar; Bar x; class Bar; endclass`, declared-then-used `class MyClass; endclass MyClass obj;`, plain module, `module m #(parameter int W=8) (input logic [W-1:0] a);` parameterized header all PASS; **SV external corpus 10 PASS / 4 FAIL / 0 TIMEOUT** — no regression on the 10 known-PASS files (scr1 ×4, friscv ×4, veer ×2 with bootstrap-chained `--lib-in`); residual 4 FAIL = uvm_pkg ×{2017,2023} + uvm_compat_pkg ×{2017,2023}; **uvm_pkg deep-position advance: surface_position 113637 (F1 PEG-furthest-position-tooling-artefact, unchanged) but Slice-59's `furthest_position` tracker now shows 162162 out of ~180K** (pre-fix tracked deepest was 19378 at Slice-54 — the parser is now reaching ~90% of uvm_pkg before backtracking exhaustion; ~8.4× deeper); uvm_compat_pkg furthest_position 114993 → 116752 (+1759); **lib no-features 548/548 PASS; lib --features generated_parsers 609/609 PASS**; SV shape-contract GREEN; **regex broader corpus / RGX conformance ✅ 44/0** (non-semantic grammar, unaffected by the SV-only change). The H1 cross-file-import class (uvm-extends-chain) is a separate fix path (H1-α library-import chain or H1-β import-emits-permissive-fact) and is what gates uvm_pkg moving from FAIL to PASS; this slice clears the structural defect that was nibbling at every implicit-type-port construct in uvm-style code.

### 1.0.126 / Contract 1.0.126 — SV-EXH-PROOF.3.3.4.b.3: parser-agnostic ENGINE Layer 0 — unified quantifier engine; uniform per-iteration atomicity for `?` / `*` / `+` / `{N}` / `{N,M}` / `{N,}` / `{,M}` (release bump, schema stays 3)

> **Layer 0** — symmetric, atomic-at-quantifier-boundary semantics across every repetition operator in EBNF. The prior codegen in `rust/src/ast_pipeline/ast_based_generator.rs::generate_quantified_logic` (and `ast_code_generator.rs::generate_quantified_content`) special-cased exactly three operators (`*`, `+`, `?`) with three independently-written code paths, with an asymmetric defect: the FIRST iteration of `+` was emitted INLINE without `try_parse` wrapping. Bounded forms (`{N}` / `{N,M}` / `{N,}` / `{,M}`) declared in `grammars/ebnf.ebnf:149-189` were never implemented — the codegen's `_ => Err("Unknown quantifier")` fallthrough. User strategic line (2026-05-21): "After Layer 0, all the repetition in the EBNF shall have the right behavior." **Fix:** new `parse_quantifier_bounds(quantifier: &str) -> Option<(usize, Option<usize>)>` helper in `rust/src/ast_pipeline/mod.rs` mapping every surface form to its `(min, max)` bounds (`?` → `(0, Some(1))`, `*` → `(0, None)`, `+` → `(1, None)`, `{N}` → `(N, Some(N))`, `{N,M}` → `(N, Some(M))`, `{N,}` → `(N, None)`, `{,M}` → `(0, Some(M))`); 4 unit tests cover happy path + whitespace + invalid forms. Both codegen functions rewritten as ONE `loop` with EVERY iteration wrapped in `try_parse` (per-iteration atomicity is uniform; the prior asymmetric inline-first-iter pattern for `+` is gone). Quantifier-level atomicity layered on top: when `min > 0`, the codegen emits `let quantifier_start_position = parser.position;` before the loop and on min-failure restores `parser.position = quantifier_start_position;` before signalling `Err(Backtrack)` — the quantifier is atomic at its own boundary, so when iteration N+1 fails after N successful iters and N is insufficient for `min`, the partial successes are undone and the quantifier reports failure as if it was never tried. For `min == 0` (`*` / `?`) the bind + check are elided to avoid always-false comparisons and unused-variable warnings. Max-count enforcement (`{N}` / `{N,M}` / `{,M}`) emits `if iteration_count >= #m_lit { break; }`. Zero-length-match guard and SAFETY_LIMIT (10_000) are preserved. **Release bump, NO schema bump** — for `?` / `*` / `+` the unified codegen is BEHAVIOR-EQUIVALENT on success paths and BEHAVIOR-EQUIVALENT-OR-MORE-CORRECT on failure paths (the prior `+` first-iter inline path left the cursor where partial parse stopped before the surrounding `try_parse` rolled it back; the unified path rolls back at the quantifier boundary too). Bounded forms `{N}` / `{N,M}` / `{N,}` / `{,M}` were unparseable pre-fix; Layer 0 makes them parser-agnostic infrastructure available for any future grammar (no current grammar uses them); strictly-more-permissive (SVPP-0002/REGEX-0083 category). VERIFIED: lib tests **465/465** PASS no-features (was 461 → +4 `parse_quantifier_bounds` unit tests); SV smoke (`module m; endmodule` / `if (1) $display("ok");` / else-if chain / for-loop) PASS via fresh release `parseability_probe`; SV shape-contract test GREEN. **RGX broader corpus / conformance ✅ 44/0** via `cargo run --features generated_parsers --release --bin test_runner -- --parser regex` after a `make focus_regex` regen (proves the unified codegen produces a regex parser with identical behavior across the entire conformance suite). USER SEMANTICS, CONFIRMED IMPLEMENTED: "the current value of the cursor shall be saved at the start of an iteration, if the iteration fails we put the cursor back to the position before starting that iteration, and we check if the repetition is fulfilled or not" — per-iter cursor save+restore is done by `try_parse`; the loop continues until either an iteration fails (break) or the max bound is reached; the min check at the end determines whether the quantifier as a whole succeeds. "For `{N}`, the group must match exactly N times; if it doesn't, the cursor is left at the position it had before trying to match `<group>{N}`" — exactly what the `quantifier_start_position` save+restore on min-failure delivers. **SV external corpus `8/14 → 10/14`** — full triage gate fresh 2026-05-21 09:55: **`friscv_rv32i_core ×{2017,2023}` now PASS** (previously categorized as the `.3.3.6` statement-level residual). HONEST ROOT-CAUSE UPDATE — the friscv_rv32i blocker was NOT a separate statement-level grammar defect; it was the prior codegen's asymmetric `+`-first-iter-not-wrapped-in-`try_parse` defect. Layer 0 wraps the first iter of `+` uniformly with every other iter, so the cursor rolls back cleanly on first-iter failure — and friscv_rv32i was triggering exactly that path inside its statement parsing. THIS is why .b.3 is a corpus-mover even though it claimed to be "behavior-equivalent for `*`/`+`/`?` on success paths": the BEHAVIORAL DIFFERENCE on the prior `+` first-iter-FAILURE path was real and friscv_rv32i was hitting it. Residual 4 fails: `uvm_pkg`/`uvm_compat_pkg ×{2017,2023}` STILL fail (uvm_pkg_2017 STILL at byte 113637 — identical to `.b.1` baseline, Layer 0 preserves uvm behavior; blocked by the next LRM-extraction-defect class). The `.3.3.6` task entry can now be CLOSED — friscv_rv32i is PASS without needing a separate grammar fix.

### 1.0.125 / Contract 1.0.125 — SV-EXH-PROOF.3.3.4.b.1: first LRM-extraction-defect fix — `conditional_statement` `[ else statement_or_null ]` now truly optional per IEEE 1800 §A.6.6 (release bump, schema stays 3)

> The opening fix of a longer campaign: the SV grammar was auto-extracted from the IEEE 1800 LRM, and the extractor introduced a class of defects where LRM `[ ]` optional clauses were encoded as mandatory. `conditional_statement` at `grammars/systemverilog.ebnf:1111` was the first case identified — and per the defect-class-by-defect-class strategy, it's a class of one (the only `&X`-positive-lookahead-mandates-optional in the entire 1,448-rule grammar). Root cause verified against the authoritative IEEE 1800-2017 PDF page 1164 AND IEEE 1800-2023 PDF page 1201 — both editions specify `conditional_statement ::= [ unique_priority ] if ( cond_predicate ) statement_or_null { else if ( cond_predicate ) statement_or_null } [ else statement_or_null ]`. Square brackets = OPTIONAL; the EBNF had encoded the trailing else as mandatory via `&kw_else kw_else conditional_else_branch` with no `|` no-else alternative. Fix: helper rule `conditional_else_clause := kw_else_ae050f5b conditional_else_branch -> $2` (passes through the branch); `conditional_statement` rewritten to `( unique_priority )? kw_if lparen cond_predicate rparen statement_or_null ( conditional_else_clause )?` with return annotation `else_body: $7` (was `$9`, now nullable when else absent). Removed dead mis-attached line 1116 (a `| @sample: "if (a) ;" ... !kw_else` alternative the original author reached for as the no-else case via PEG negative lookahead, but the blank line at 1113 parented it to `conditional_else_branch` and made it unreachable). Else-if chain `{ else if (cond) stmt }` continues to be handled by the existing recursive `conditional_else_branch := conditional_statement | statement_or_null` — once the outer else is optional, the chain naturally produces zero-or-more recursively. **Release bump, NO schema bump** — strictly-more-permissive (SVPP-0002/REGEX-0083 category): no_else inputs were 100% unparseable so no AST was ever emitted; previously-parseable inputs are byte-identical; the `else_body` field is the same `conditional_else_branch` ParseNode when present, just additionally nullable when absent. Inventory **2299 → 2300** (+1: the new `conditional_else_clause` helper rule). VERIFIED: minimal failing repro `module m; task t(); if (1) $display("ok"); endtask endmodule` PASSES (was FAIL); pre-existing `if (a) ; else ;` STILL PASSES; else-if chain PASSES; `.3.3.3` minimal repro STILL PASSES; lib `461/461` no-features (+1 from `.3.3.4.a.2`'s baseline) and `516/517` with `--features generated_parsers` (only fail = pre-existing rgx_0077, `.3.3.5`-class). **regex broader corpus / RGX conformance ✅ 44/0** via `make regex_broader_corpus_proof_gate`. SV shape-contract GREEN. **SV external corpus stays 8/14** — scr1 ×4 + friscv_pipeline ×2 + veer_el2_lsu ×2 all still PASS; uvm_pkg_2017 STILL fails at the SAME byte position 113637 post-fix (with parse time 60s→14min as the parser now walks deeper into the 89K-line body, exposing OTHER LRM-extraction defects further in). This proves `conditional_statement` was NOT the blocker for uvm_pkg — the conditional_statement fix is correct on its own merits per the LRM oracle, but more LRM-extraction defects remain on the uvm/uvm_compat/friscv_rv32i paths and will need their own leaves. **Lesson learned and recorded in cross-session memory** (`feedback_verify_rule_correctness_before_runtime_hypotheses`): when a parse failure is tied to a specific rule, the FIRST diagnostic step is to read the rule against the LRM/spec — NOT to form runtime/scope hypotheses. The original `.3.3.4.b` framing as "intra-file scope tracking" cost ~30 min of binary-search effort; the rule-vs-LRM read would have revealed the defect in 30 seconds.

### 1.0.124 / Contract 1.0.124 — SV-EXH-PROOF.3.3.4.a.2: parser-agnostic ENGINE extension — non-negative integer indexed-access (`[N]`) in semantic-annotation rule references (release bump, schema stays 3)

> Companion to `.3.3.4.a.1` (which added dotted property-access `$name.body`). Real authoring concern that `.3.3.4.a.1` only half-addressed: when a rule's shaped output is `{items: [{name, body}, …]}` or `{matrix: [[…], …]}`, directives still couldn't reference array elements directly — there was no `$items[0].name` or `$matrix[0][1]` form. `.3.3.4.a.2` closes that gap with a strict, well-defined subset: dotted property + non-negative integer indexed-access only. NOT full JSONPath (no filters `[?(@.foo)]`, no wildcards `*`, no recursive descent `..`). **Engine (parser-agnostic, both surfaces in lockstep):** (1) `unified_semantic_ast.rs::parse_rule_reference` chain loop now accepts an arbitrary mix of `.<ident>` and `[<digits>]` segments via a `loop { match peek_char() { Some('.') => …, Some('[') => …, _ => break } }` form; strict-bracket policy rolls back on malformed `[…]`. (2) EBNF surface `grammars/semantic_annotation.ebnf::rule_reference_name` regex extended in lockstep — `/([a-zA-Z_][a-zA-Z0-9_]*(\.[a-zA-Z_][a-zA-Z0-9_]*|\[[0-9]+\])*|[0-9]+(\.[a-zA-Z_][a-zA-Z0-9_]*|\[[0-9]+\])*)/`. Bootstrap `semantic_annotation_parser.rs` regenerated. (3) Runtime resolver (`ast_based_generator.rs`): three new free-standing helpers emitted into every generated parser — `lex_semantic_reference_segments_suffix`, `lex_semantic_reference_segments_named`, and `parse_bracketed_index`. The two resolvers (positional + named) use the lexers and dispatch per segment: for shaped-JSON content, `serde_json::Value::get` polymorphically accepts both `&str` and `usize`; for raw-tree content (rules without `->`), a new `find_semantic_indexed_child` companion picks the N'th element of a `Sequence`/`Quantified` (or the wrapped node of an `Alternative` when N==0). **DURABLE NO-DEPTH-LIMIT GUARANTEE extends to mixed dotted+indexed depth**: structurally unbounded at every layer (EBNF `*` quantifier; hand-rolled `loop` with no max-iteration cap; lexer `while !remaining.is_empty()` iterates until input exhausted; resolver iterates over the lexed Vec). Locked by `bootstrap_semantic_indexed_rule_reference_depth_is_structurally_unbounded` test exercising **32 mixed `.<ident>` + 32 `[N]` segments** (64 total) and asserting verbatim retention of both kinds of segments (33 dot-separated chunks, 32 bracketed segments). Pairs with the `.3.3.4.a.1` dotted-only test — together they pin the guarantee across both shapes. **No SV grammar changes** — purely a language-surface expressiveness improvement; the SV directive payloads in 1.0.123 (`$name.body`, `$package.body`) continue to work byte-identically. **Release bump, NO schema bump** — strictly additive language extension; no shape changes anywhere; no annotated rules changed (the new helpers are infrastructure emitted into every generated parser, not annotated grammar rules); SVPP-0002/REGEX-0083 category. VERIFIED: lib 461/461 no-features (was 460 → +1 new indexed-access regression test); 516/517 with `--features generated_parsers` (only fail = pre-existing `rgx_0077` from `.3.3.5`-class); cross-file synthetic repro PASSES with `--lib-in` and FAILS without (the `.3.3.4.a.1` cleanup behaviour preserved); `.3.3.3` minimal repro still PASSES; both depth-unbounded regression tests (dotted-only + mixed). **regex broader corpus / RGX conformance ✅ 44/0** via `make regex_broader_corpus_proof_gate`. SV shape-contract GREEN. **SV external corpus stays 8/14** (verified: triage gate fresh 2026-05-20; veer_el2_lsu ×{2017,2023} still PASS via the existing `$name.body` directives — no SV directive uses `[N]` yet, so the new feature is dead-code in the SV path). Surfaced during this slice: a generator-area test (`generated_parser_runtime_contract_emits_post_predicate_content_flow`) asserted the literal string `current.get(segment)` from the resolver's `for segment in reference.split('.')` loop; my refactor changed the iteration to `for segment in &lexed_segments` (segments are now `&&str`), so the literal became `current.get(*segment)` — assertion updated to track the new shape while staying semantic (`current.get(`). NEXT LEAF (queued): `.3.3.4.b` — uvm self-contained same-file-package path (intra-file scope tracking, not artifact-on-disk; the cross-file MVP-0 model doesn't apply when package + uses share a file).

### 1.0.123 / Contract 1.0.123 — SV-EXH-PROOF.3.3.4.a.1: parser-agnostic ENGINE cleanup — dotted property-access in semantic-annotation rule references; revert of the `.3.3.4.a` `body:$N.body` SV shape workarounds (release bump, schema stays 3)

> Companion to `.3.3.4.a`. `.3.3.4.a` had to carry a SV shape workaround (`body: $4.body` on `package_declaration`, `body: $1.body` on both `package_import_item` branches) because the **directive-payload parser** — the hand-rolled `StructuredSemanticValueParser::parse_rule_reference` in `rust/src/ast_pipeline/unified_semantic_ast.rs` — only accepted single-identifier / single-positional refs (`$name` / `$1`), no `.` allowed. The runtime resolver already walked dotted paths via `resolve_named_semantic_reference` / `parse_semantic_reference_segments` (`ast_based_generator.rs`), splitting on `.` and traversing either the shaped JSON object-key path (rules with `->`) or the raw sub-rule-named descendant tree (rules without `->`). The gap was purely at the bootstrap-parser surface. `.3.3.4.a.1` closes it: `parse_rule_reference` gains a `while peek_char() == Some('.')` chain that accepts an arbitrary number of `.<ident>` segments. DURABLE NO-DEPTH-LIMIT GUARANTEE: depth is structurally unbounded at every layer — EBNF `(\.<ident>)*` regex (strict-math `*`), hand-rolled `while` with no max-iteration cap, resolver `for segment in split('.')` iterator-based walk. Locked by `bootstrap_semantic_dotted_rule_reference_depth_is_structurally_unbounded` test that exercises a 64-segment reference (1 head + 64 `.seg<i>` segments) and asserts the full chain text is retained verbatim — any future proposal to cap depth has to fire this test, justify the cap in its own leaf, or back off (the `feedback_recursion_ceiling_must_bound_real_stack` discipline applied to a non-stack resource). `.3.3.4.a.1` also keeps the EBNF surface consistent: `grammars/semantic_annotation.ebnf::rule_reference_name` regex extended in lockstep so the language definition mirrors the runtime; the bootstrap `semantic_annotation_parser.rs` is regenerated, though that generated parser is the EBNF-language surface for parsing semantic-annotation source — NOT the runtime path for grammar directive payloads, which goes through `unified_semantic_ast.rs::parse_bootstrap`. Keeping both surfaces consistent prevents future drift. **SV grammar revert:** `body: $4.body` removed from `package_declaration`; `body: $1.body` removed from both `package_import_item` branches; the two directives now reference the nested scalars directly — `@export_to_library: {kind:package, name_from:$name.body}` and `@import_from_library: {kind:package, name_from:$package.body}`. **Release bump, NO schema bump** — the `body` fields were a one-slice workaround carrier (added in 1.0.122, removed here); they had no consumer pre-1.0.122 and have none post-`.3.3.4.a.1` (cross-file imports still resolve, verified end-to-end). Versus 1.0.121 this is a no-op shape change. VERIFIED: end-to-end synthetic cross-file repro PASSES with `--lib-in` and FAILS without (the library is still the deciding factor — cleanup is behaviour-preserving); `.3.3.3` minimal repro still PASSES; lib tests **460/460** no-features (was 459 → +1 unbounded-depth regression test) and **515/516** with `--features generated_parsers` (only fail is the pre-existing `rgx_0077` confirmed at `.3.3.4.a` via decisive baseline → `.3.3.5`-class). **regex broader corpus / RGX conformance ✅ 44/0** via `make regex_broader_corpus_proof_gate`. SV shape-contract GREEN. **SV external corpus stays `8/14`** — `veer_el2_lsu` ×{2017,2023} still PASS via the new dotted-refs path (fresh triage gate 2026-05-20 15:41; identical to `.3.3.4.a`'s shape-workaround result). NEXT LEAF: `.3.3.4.a.2` will extend the same surfaces with `[<digits>]` indexed access (`$items[0].name`, `$matrix[0][1]`, mixed `$a.b[0].c[1].d`) under the same unbounded-depth guarantee, own leaf, separate commit.

### 1.0.122 / Contract 1.0.122 — SV-EXH-PROOF.3.3.4.a: parser-agnostic ENGINE feature — per-compilation-artifact library (MVP-0); cross-file `import pkg::*` type-name visibility; `veer_el2_lsu` ×{2017,2023} now parses (release bump, schema stays 3)

> **A second deliberate parser-agnostic ENGINE feature in the SV-EXH-PROOF tree** — and like `.3.3.3` it benefits every grammar. PGEN parsed single SV files in isolation, but real HDL is multi-file: `el2_lsu.sv` does `import el2_pkg::*;` to reference types declared in `el2_def.sv` (a separate file). The use-site `@predicate has_fact(type_name, X)` correctly fires (`.3.3.3` made it reliable) but correctly evaluates false because the type-emitting file was never parsed in the same session. The architecturally-correct fix — matching every commercial HDL tool — is **per-file compilation that writes a compact on-disk artifact** containing the exported facts of each scope-creating entity (package/module/interface/…) with **on-demand library lookup** when an importer references that entity. **MVP-0 (this slice)** narrows to packages only, JSON artifacts, user-supplied file order; future increments grow the kind set and add `$unit`/CU semantics under separate leaves. **TWO new parser-agnostic ENGINE annotations:** `@export_to_library: { kind:<static-kind>, name_from:<expr> }` fires on a rule's successful commit (atomic write of the rule's emitted-fact delta to `<lib-dir>/<kind>/<name>.facts.json`); `@import_from_library: { kind:<static-kind>, name_from:<expr> }` fires after the rule's body parses (merges the artifact's facts into the in-progress semantic-runtime transaction's state, so subsequent sibling `has_fact` checks see them). Both go through the `.3.3.3` IIFE so I/O failures `Err`-propagate and the manual restore fires cleanly — same exception-safety invariant. **TWO new parser CLI flags** on `parseability_probe`: `--lib-in DIR` (where artifacts are read) and `--lib-out DIR` (where artifacts are written). Both default off; single-file parses are byte-identical to today. **SV grammar (composite):** `@export_to_library: {kind:package, name_from:$body}` on `package_declaration` (with a new top-level `body: $4.body` shaped field, since the semantic-annotation language only accepts simple `$name` refs, not `$x.y` — the package's scalar name is surfaced at the shape's top); same idiom on `package_import_item` with `@import_from_library: {kind:package, name_from:$body}` and `body: $1.body` for both branches (explicit + wildcard). **Triage gate (composite):** the SV external-corpus triage manifest gains an optional per-case `bootstrap_files: [...]` ordered array (commercial-tool convention — the user supplies the transitive-dependency order); the gate preprocesses + parses each bootstrap file with `--lib-out <per-case-lib-dir>` THEN parses the main case with `--lib-in <per-case-lib-dir>`. `veer_el2_lsu` declares `bootstrap_files: ["…/include/el2_def.sv"]`. **Release bump, NO schema bump** — the two new top-level `body` fields are additive on shapes whose cross-file form was unparseable (no consumer had a realized AST to depend on); previously-parseable single-file inputs are byte-identical (success paths unchanged); only previously-erroring cross-file inputs now succeed (strictly-more-permissive, SVPP-0002/REGEX-0083 category). VERIFIED: end-to-end synthetic repro (`my_pkg` with two typedefs → artifact written; module that imports `my_pkg::*` and uses the types PASSES with `--lib-in`, FAILS without — proves the library path is the deciding factor); minimal `.3.3.3` repro `module m; typedef int my_t; my_t [3:0] x; endmodule` STILL PASSES; lib tests `514/515 PASS` (the only fail — `regex_parser_pgen_rgx_0077_quoted_run_quantified_pieces_flat_in_concatenation` — is confirmed PRE-EXISTING via decisive baseline: `git stash`ed all MVP-0 sources, rebuilt baseline driver from HEAD, regenerated regex+bootstrap parsers, SAME failure → tracked as `.3.3.5`); 6 new directive-parser unit tests + 4 library-module unit tests all GREEN. **regex broader corpus / RGX conformance ✅ 44/0** (the critical downstream is unaffected — non-semantic grammars hit the fast-path and the library code is dead-code-eliminated when neither annotation appears). **SV external corpus `6/14 → 8/14`** — `veer_el2_lsu` ×{2017,2023} now PASS via the bootstrap-files+library-artifact path, exactly as projected. Residual 6 fails: `uvm_pkg`/`uvm_compat_pkg` ×{2017,2023} are SELF-CONTAINED files (no separate `import` declarations the way veer does) — they fail at a deeper `.3.3.3` use-site `known_unscoped_*` residual; `friscv_rv32i_core` ×{2017,2023} is statement-level (`.3.3.6`).

### 1.0.121 / Contract 1.0.121 — SV-EXH-PROOF.3.3.3: parser-agnostic ENGINE exception-safety fix (`?`-bypasses-cleanup in `with_semantic_runtime_rule_transaction`) + SV grammar wrapper (`checked_type_identifier`); class/typedef/parameter use-site `has_fact` now works (release bump, schema stays 3)

> **The first deliberate ENGINE change in the SV-EXH-PROOF tree** — and it is parser-agnostic, benefiting every grammar. Root cause was a classic Rust `?`-bypasses-cleanup bug in the generator-emitted `with_semantic_runtime_rule_transaction` (`rust/src/ast_pipeline/ast_based_generator.rs`): it `std::mem::take`-s `self.semantic_runtime_state` (leaving `Default == new()` valid-but-EMPTY) and then performs `?`-fallible calls (`f(self)?`, `apply_semantic_runtime_effect_directive?`, `resolve_semantic_predicate_spec_against_content?`); a `?` early-return propagated out of the ENTIRE function, JUMPING OVER the trailing `if result.is_err() { self.semantic_runtime_state = original }` restore — so `self.semantic_runtime_state` was left EMPTY, silently destroying every fact emitted by prior COMMITTED sibling rules (e.g. a typedef's `type_name` fact). Every later sibling `@predicate has_fact(…)` then saw nothing. Definitively SEMTRACE-pinned (`RESTORE` fired **0×** across 1664 trace lines while `took post-body state (self.state→EMPTY)` fired **9×**) after three prior fix hypotheses (try_parse semantic checkpoint, grammar wrapper alone, memoization-cache) were each implemented, tested, and disproven. **FIX (engine, in the generator, all parsers benefit):** wrap the fallible body in an immediately-invoked closure ("try-block" emulation) so every `?` returns into `result` and the restore runs on every non-commit exit. Zero `unsafe`; zero behaviour change on the success path. (RAII Drop-guard considered for strict panic-safety; would need `*mut`+`unsafe` due to `&self`-method borrow conflicts; deferred since IIFE+Default-validity is already panic-robust and the parser never `catch_unwind`s mid-parse.) **FIX (SV grammar composite half):** added `checked_type_identifier := type_identifier -> {body:$1.body}` + `@predicate has_fact args:[type_name,$body] phase:post`, routed `known_unscoped_block/data_type_identifier` through it — the proven `.3.3.1`/`.3.3.2` declared-id idiom (required because a sub-rule-name predicate ref on a `->`-bearing rule does NOT resolve and dotted predicate args are `@predicate`-compiler-rejected). **Release bump, NO schema bump** — declarations whose post-predicate emit/restore was leaked were silently unparseable, so no such AST was ever emitted; previously-parseable inputs byte-identical (success paths unchanged); only previously-erroring inputs now succeed (strictly-more-permissive, SVPP-0002/REGEX-0083 category). VERIFIED: minimal repro `module m; typedef int my_t; my_t [3:0] x;` PASSES (unfixable through three prior attempts); SEMTRACE confirmed RESTORE fires 110× (was 0); use-site `has_fact[type_name, my_t]` resolves and returns true; class/typedef/let/`#(parameter…)`/localparam/builtin-dim no-regression (memo ON, normal config). Inventory **2297 → 2299** (+2: the new wrapper rule's `@predicate` + return-shape). SV shape-contract **GREEN**. **regex broader corpus / RGX conformance ✅ 44/0** (the critical downstream is unaffected — non-semantic grammars hit the fast-path and the IIFE region is dead code there). **external-corpus parse stays `6/14`** — `.3.3.3` is the **foundation engine fix** (the `?`-bypass state-leak was masking every downstream semantic-fact-based fix); the remaining 8 cases are blocked by DISTINCT defects tracked separately: `.3.3.4` (cross-package `import pkg::*` type-name visibility — the actual corpus-mover, ~6 cases) and `.3.3.6` (statement-level residual, friscv_rv32i ×2). Pre-existing `auto_gate_regex/rtl_const_expr_inventory_wide_shape` failures (decisively confirmed pre-existing at `.3.3.2` via git-stash baseline check) tracked as `.3.3.5`.

### 1.0.120 / Contract 1.0.120 — SV-EXH-PROOF.3.3.2: package declaration directive fixed (`declared_package_identifier`); `package … endpackage` now parses (release bump, schema stays 3)

> `package_declaration` was the **only** declaration not using the proven `declared_X_identifier := X_identifier -> {body:$1.body}` + `@emit_fact {name:$body}` idiom (every other declared-id — class/typedef/parameter/etc. — has its `declared_X_identifier`). It put `@emit_fact: { kind: package_name, name: $package_identifier }` directly on the big multi-element `package_declaration` rule, and `package_identifier` occurs twice (the declaration name + the trailing `endpackage : label`), so the named reference resolved to the `{body:…}` object / was unresolvable ⇒ *"Semantic runtime could not resolve fact name"* — **package declarations unparseable**. `PGEN-SV-EXH-PROOF-0023` adds `declared_package_identifier := package_identifier -> { body: $1.body }` with `@emit_fact: { kind: package_name, name: $body }` (mirrors `declared_class_identifier`; **emit-only — package's original directive had NO `@predicate has_fact` since a package is not a type; that behavior is preserved**) and routes the declaration-site through it; the trailing `( colon package_identifier )?` stays the plain label use-site (exactly as `class`/`module` do `( colon class_identifier )?`). **Grammar-only fix — the engine is untouched.** **Release bump, NO schema bump** — package declarations were unparseable so no such AST was ever emitted; previously-parseable inputs byte-identical; strictly-more-permissive (SVPP-0002/REGEX-0083 category). SV shape-contract **GREEN** (samples=3 aligned=3 drift=0 regression_lock=0). `package pp; endpackage`, `package pp; endpackage : pp`, `package pp; localparam int W=8; endpackage` now parse; class/typedef/localparam/let/`.3.3.1` no-regression. **external-corpus parse stays `6/14`** — honest: the minimal package repro now parses (was 100% unparseable), but `uvm_pkg`/`uvm_compat_pkg` ×{2017,2023} fail *deeper* at the `.3.3.3` use-site `known_unscoped_*` type-id defect (~`118257`) — `.3.3.2` peeled one onion layer (forward progress; the aggregate count is gated by the deeper defect). Evidence refined the categorization: `.3.3.3` (use-site type-id resolution) is now the dominant residual (veer + uvm ×4 + uvm_compat ×2).

### 1.0.119 / Contract 1.0.119 — SV-EXH-PROOF.3.3.1: declaration-site identifier directive fixed (`non_keyword_identifier`); class/package/typedef/parameter/localparam declarations now parse (release bump, schema stays 3)

> `non_keyword_identifier` is the foundation rule for every declaration-site name (class / package / typedef / parameter / localparam, via `declaration_identifier`). It returned the raw `-> $2` value, which the positional-extraction codegen leaves as an `Alternative`-wrapped node (implicit-passthrough codegen unwraps it). So every `declared_*_identifier -> {body:$1.body}` `@emit_fact`/`@predicate has_fact` directive routed through it failed *"Semantic runtime could not resolve fact name"*, making **all such declarations 100% unparseable** — a pre-existing defect masked by the `.3.2` number bug (the parser never reached declarations before). `PGEN-SV-EXH-PROOF-0022` restores the engine-proven object form `non_keyword_identifier := !reserved_non_keyword_identifier identifier -> {body: $2.body}` (mirrors `identifier := simple_identifier -> {body:$1}`). **Grammar-only fix — the codegen/runtime engine is untouched** (it is stable; the fix lives entirely in the EBNF). **Release bump, NO schema bump** — those declarations were 100% unparseable so no such AST was ever emitted; previously-parseable inputs byte-identical; only previously-erroring declarations now succeed (strictly-more-permissive — SVPP-0002/REGEX-0083 category). SV shape-contract **GREEN** (samples=3 aligned=3 drift=0 regression_lock=0 — the rule's value is consumed internally by directives, the emitted AST envelope is unchanged); annotation inventory **unchanged** (`-> $2` → `-> {body:$2.body}` is a shape change of an already-counted entry). `.3.2` number oracle (13/13) + `.3.1` declared-id family no-regression (interface/program now parse too). **external-corpus parse `4/14 → 6/14`** (`friscv_pipeline` ×{2017,2023} now fully parse). Residual uvm/friscv_rv32i/veer categorized into sub-leaves `.3.3.2` (package `@emit_fact` name resolution) / `.3.3.3` (use-site `known_unscoped_*` type-id resolution) / `.3.3.4` (statement-level `conditional_statement`/labeled `seq_block`).

### 1.0.118 / Contract 1.0.118 — SV-EXH-PROOF.3.2: number-rule decomposition restored to the generator's clean IEEE-1800 lexical form (release bump, schema stays 3)

> The 115-slice campaign + POST-SV-AUDIT had decomposed the number rules into a per-digit-token tree; the generator's blanket `\b`-append on every single-char alnum token (digits, base letters `h/b/d/o/s`, `x/z`) + a `__SV_RULE__`-sentinel-mangled `_` separator made **every** multi-digit / underscore / sized / based SV number unparseable — the external-corpus `0/14` root cause. `PGEN-SV-EXH-PROOF-0021` restores `integral_number`/`real_number`/`unsigned_number`/`decimal_number`/`binary_number`/`octal_number`/`hex_number`/`size`/`fixed_point_number` to the generator's clean single-regex IEEE-1800 (A.8.7) form (sized-first for PEG, size optional for unsized-based), keeping the typed top `number := real_number -> {kind:"real",body:$1} | integral_number -> {kind:"integral",body:$1}` (now clean Terminals ⇒ first realized number shape = flat `{kind,body:"<number>"}`). **Release bump, NO schema bump** — numbers were 100% unparseable so no number AST was ever emitted (no prior realized shape / no consumer); previously-parseable inputs byte-identical; only previously-erroring numbers now succeed (strictly-more-permissive — SVPP-0002/REGEX-0083 category). Full LRM number-form oracle (decimal/`1_000`/`8'h09`/`4'b1010`/`16'd42`/`8'o17`/`'hFF`/`8'shFF`/`4'b1x0z`/`12'hDE_AD`/`1.5`/`1.0e6`/`1e-3`/veer `2294'h…`) + `.3.1` 13-family no-regression + SV shape-contract GREEN + SV lib 8/0. **external-corpus parse `0/14 → 4/14`** (scr1 family now fully parses — first non-zero corpus parse since the SV-EXH-PROOF tree began); residual uvm/friscv/veer (10) pinned as `.3.3`. Resolves the long-DEFERRED `*_value`/`unsigned_number` shaping note.

### 1.0.117 / Contract 1.0.117 — POST-SV-AUDIT.2.4b: 11 structured-per-iteration Category-A misuses corrected via factored record rules (schema 2 → 3)

`PGEN-POST-SV-AUDIT-0006`, leaf POST-SV-AUDIT.2.4b (2026-05-17).
**AST-dump schema bumped `2 → 3`** (the affected list / branch shapes
change in a consumer-visible way).

The 11 structured-per-iteration Category-A rules in
`grammars/systemverilog.ebnf` were corrected. Each repeated
**multi-field** unit was factored into a **new annotated record rule**
and the list / branch became an extraction-spread over it. Field names
of the prior `first` record are **preserved**.

- 5 `list_of_*_identifiers`: `{first: {name, dims[, init]}, rest}`
  (raw `[[comma, id, [dim…]], …]` envelope) → clean flat record array
  via new `interface_identifier_decl` / `port_identifier_decl` /
  `variable_identifier_decl` (`-> {name, dims}`) and
  `tf_variable_identifier_decl` / `variable_port_identifier_decl`
  (`-> {name, dims, init}`); list now `[$1, $2::2*]`.
- `let_` / `property_` / `sequence_list_of_arguments`: new
  `*_named_arg := dot identifier lparen ( <x>_actual_arg )? rparen
  -> {name, value}`; **mixed** branch `{kind:"mixed", head,
  ordered_tail:[…], named_tail:[…]}`; **named_only** branch
  `{kind:"named_only", items:[…]}` (was `{kind:"named_only",
  first_name, first_value, rest}`).
- `parameter_port_list` type_only: `{kind:"type_only", first, rest}` →
  `{kind:"type_only", items:[$4, $5::3*]}` (clean `[type_assignment]`).
- `assignment_pattern` named (both `pattern_sv_2017` and
  `pattern_sv_2023` occurrences): new shared `assignment_pattern_entry
  := member_identifier colon pattern -> {name, pattern}`; branch
  `{kind:"named", entries:[$3, $4::2*]}` (was `{kind:"named",
  entries:{first:{name,pattern},rest}}`).

Annotation inventory: **2299** (was 2290, **+9 — a DELIBERATE count
change, NOT "unchanged"**). The 9 new factored record rules
(`interface_identifier_decl`, `port_identifier_decl`,
`variable_identifier_decl`, `tf_variable_identifier_decl`,
`variable_port_identifier_decl`, `let_named_arg`, `property_named_arg`,
`sequence_named_arg`, `assignment_pattern_entry`) **are annotated** —
unlike the pure-Cat-A `[$1, $2::2*]` rewrites that add no annotations;
this is the same kind of deliberate +N as SVPP-0001's `pp_if_keyword`.
**1008** distinct annotated rules (was 999, +9). Same accept set. The
reachable `list_of_*_identifiers` path is probe-verified on
`module m; wire a, b, c; logic x, y, z; endmodule` (clean `{name,
dims, init}` record list; 0 `<invalid_sequence_access>`; 0 raw `[[],
","]` separator-envelope leaks). The `*_list_of_arguments`,
`parameter_port_list` type_only, and `assignment_pattern` named
branches are likely unreachable via the strict SV root (pre-existing,
out-of-scope coverage limitation) — defensively-correct-by-construction
(identical proven idiom), **not** claimed as a fresh probe and **NOT a
bug-ledger entry** (clean Category-A, no `<invalid_sequence_access>`
corruption). Full detail: contract §
"AST-Shape Corrections — 1.0.117 (POST-SV-AUDIT)" and
`docs/POST_SV_AUDIT_LEDGER.md`.

### 1.0.116 / Contract 1.0.116 — POST-SV-AUDIT.2.4a: net_alias Cat-A raw-envelope correction + 5-number-rule defensive structural fix (schema 1 → 2)

`PGEN-POST-SV-AUDIT-0005`, leaf POST-SV-AUDIT.2.4a (2026-05-17).
**AST-dump schema bumped `1 → 2`** (the reachable, consumer-visible
`net_alias` shape change drives it).

- `net_alias` Category-A raw-envelope misuse corrected:
  `{first, second, rest}` (raw `[[assign, net_lvalue], …]` `rest`
  envelope) → clean flat `{lvalues: [$2, $4, $5::2*]}` list. Stays
  `return_object`, new `normalized_text` only. Probe-verified on
  `module m; wire a, b, c; alias a = b = c; endmodule` →
  `{"lvalues":[{…a…},{…b…},{…c…}]}`.
- Defensive structural correction of 5 number rules
  (`unsigned_number` / `non_zero_unsigned_number` / `binary_value` /
  `octal_value` / `hex_value`): the inline-alternation iteration lead
  `( kw_sv_rule_c82a06f6 | <digit> )` lifted into new **un-annotated**
  named `*_tail` rules so bare `$2` binds cleanly; the
  `{first, rest}` annotation text is **unchanged**. The corruption is
  **structurally present but NOT consumer-reproducible** (SV
  `systemverilog_file` root rejects every numeric-bearing top-level
  construct in all profiles — pre-existing, out-of-scope root coverage
  limitation), so this is a **defensive/latent** fix and is **NOT a
  bug-ledger entry**.

Annotation inventory: **2290** (UNCHANGED — `net_alias` is a text-only
`normalized_text` change; the number-rule annotation text is unchanged;
the 5 `*_tail` rules are un-annotated). **999** distinct rules
(UNCHANGED). Same accept set. Full detail: contract §
"AST-Shape Corrections — 1.0.116 (POST-SV-AUDIT)" and
`docs/POST_SV_AUDIT_LEDGER.md`.

### 1.0.115 / Contract 1.0.115 — SV-Slice-115 batch: Pattern-B ps_type_identifier_sv_2017/2023 typed (2 rules / 2 annotations)

Annotation inventory: **2290** (was 2288, +2). Same accept set.

### 1.0.114 / Contract 1.0.114 — SV-Slice-114 batch: Pattern-A number-value sequence rules typed (5 rules / 5 annotations)

Annotation inventory: **2288** (was 2283, +5). Same accept set.

### 1.0.113 / Contract 1.0.113 — SV-Slice-113 batch: method_call_receiver_sv_2017/2023 per-branch typed (2 rules / 26 annotations after codegen drops 2)

Annotation inventory: **2256** (was 2231, +25). Same accept set.

### 1.0.112 / Contract 1.0.112 — SV-Slice-112 batch: hierarchical_tf_identifier + ansi_port_declaration typed (2 rules / 4 annotations after codegen drops)

Annotation inventory: **2231** (was 2227, +4). Same accept set.

### 1.0.111 / Contract 1.0.111 — SV-Slice-111 batch: delay_sv_2017 + delay_sv_2023 typed (2 rules / 8 annotations)

Annotation inventory: **2227** (was 2219, +8). Same accept set.

### 1.0.110 / Contract 1.0.110 — SV-Slice-110 batch: drive_strength + init_val + scalar_constant typed (3 rules / 26 annotations)

Annotation inventory: **2219** (was 2193, +26). Same accept set.

### 1.0.109 / Contract 1.0.109 — SV-Slice-109 batch: pulldown_strength + pullup_strength + net_type typed (3 rules / 18 annotations)

Annotation inventory: **2193** (was 2175, +18). Same accept set.

### 1.0.108 / Contract 1.0.108 — SV-Slice-108 batch: duplicate-branch leaf rules typed (3 rules / 11 annotations)

Annotation inventory: **2175** (was 2164, +11). Same accept set.

### 1.0.107 / Contract 1.0.107 — SV-Slice-107: provisional_unscoped_block_class_type typed (1 rule / 1 annotation)

Annotation inventory: **2164** (was 2163, +1). Same accept set.

### 1.0.106 / Contract 1.0.106 — SV-Slice-106 batch: type-identifier + block_class_type chain rules typed (5 rules / 4 annotations)

Annotation inventory: **2163** (was 2159, +4). Same accept set.

### 1.0.105 / Contract 1.0.105 — SV-Slice-105 batch: scoped_X passthrough identifier rules typed (11 rules / 11 annotations)

Annotation inventory: **2159** (was 2148, +11). Same accept set.

### 1.0.104 / Contract 1.0.104 — SV-Slice-104 batch: identifier-routing wrappers typed (15 rules / 35 annotations)

Annotation inventory: **2148** (was 2113, +35). Same accept set.

### 1.0.103 / Contract 1.0.103 — SV-Slice-103 batch: operator/punctuation leaves typed (69 rules / 69 annotations)

Annotation inventory: **2113** (was 2044, +69). Same accept set.

### 1.0.102 / Contract 1.0.102 — SV-Slice-102 batch: number-leaf family typed — 12 Or rules with per-branch kind discriminators (63 annotations)

Annotation inventory: **2044** (was 1981, +63). Same accept set.

### 1.0.101 / Contract 1.0.101 — SV-Slice-101 batch: comment_only + timing_check_limit + trans_item + remaining t*_path + type wrappers + variable_port typed (15 rules / 15 annotations)

Annotation inventory: **1981** (was 1966, +15). Same accept set.

### 1.0.100 / Contract 1.0.100 — SV-Slice-100 batch: sign + statement_item + structure_pattern_key + t*_path_delay_expression + tf_port + threshold + timecheck + timeunits wrappers typed (16 rules / 26 annotations) — crosses 100-slice milestone

Annotation inventory: **1966** (was 1940, +26). **Crosses 100-slice milestone.** Same accept set.

### 1.0.99 / Contract 1.0.99 — SV-Slice-99 batch: package_scope + parameter_declaration + parameter_value_assignment + parameter_override + pattern + port_declaration + program_generate_item + property_expr + pulse_control + randomize + ref_declaration + select_condition wrappers typed (25 rules / 50 annotations)

Annotation inventory: **1940** (was 1890, +50). Same accept set.

### 1.0.98 / Contract 1.0.98 — SV-Slice-98 batch: default_skew + dynamic_override + forward_type + module_instantiation + operator_assignment + package_or_generate wrappers typed (24 rules / 39 annotations)

Annotation inventory: **1890** (was 1851, +39). Same accept set.

### 1.0.97 / Contract 1.0.97 — SV-Slice-97 batch: final/initial_construct + method_call internals + identifier_list + interface_instantiation + module_common_item wrappers typed (19 rules / 26 annotations)

Annotation inventory: **1851** (was 1825, +26). Same accept set.

### 1.0.96 / Contract 1.0.96 — SV-Slice-96 batch: constraint/covergroup/data_declaration + design + dist + elaboration + event wrappers typed (17 rules / 28 annotations)

Annotation inventory: **1825** (was 1797, +28). Same accept set.

### 1.0.95 / Contract 1.0.95 — SV-Slice-95 batch: sv_multi_entry_root + comment_only + bit_select + case + clocking + constant_* wrappers typed (12 rules / 18 annotations)

Annotation inventory: **1797** (was 1779, +18). Same accept set.

### 1.0.94 / Contract 1.0.94 — SV-Slice-94 batch: dimension family + integer_covergroup_expression typed (8 rules / 15 annotations)

Annotation inventory: **1779** (was 1764, +15). Same accept set.

### 1.0.93 / Contract 1.0.93 — SV-Slice-93 batch: anonymous_program_item + assignment_pattern + array + block_event + built_in_method + class_item wrappers typed (16 rules / 40 annotations)

Annotation inventory: **1764** (was 1724, +40). Same accept set.

### 1.0.92 / Contract 1.0.92 — SV-Slice-92 batch: terminal + cross_set + weight_specification + passthroughs typed (8 rules / 8 annotations)

Annotation inventory: **1724** (was 1716, +8). Same accept set.

### 1.0.91 / Contract 1.0.91 — SV-Slice-91 batch: rs_ + value_range wrappers typed (9 rules / 16 annotations)

Annotation inventory: **1716** (was 1700, +16). Same accept set.

### 1.0.90 / Contract 1.0.90 — SV-Slice-90 batch: production + udp_declaration + range_list wrappers typed (4 rules / 5 annotations)

Annotation inventory: **1700** (was 1695, +5). Same accept set.

### 1.0.89 / Contract 1.0.89 — SV-Slice-89 batch: profile-router wrappers typed (9 rules / 18 annotations)

Annotation inventory: **1695** (was 1677, +18). Same accept set.

### 1.0.88 / Contract 1.0.88 — SV-Slice-88 batch: constant_primary + primary wrappers typed (2 rules / 4 annotations)

Annotation inventory: **1677** (was 1673, +4). Same accept set.

### 1.0.87 / Contract 1.0.87 — SV-Slice-87 batch: module_path_operators + level_input_list typed (4 rules / 20 annotations)

Annotation inventory: **1673** (was 1653, +20). Same accept set.

### 1.0.86 / Contract 1.0.86 — SV-Slice-86 batch: let_declaration + final/initial specifiers + named_port_connection + nonconsec_rep + time_unit typed (6 rules / 13 annotations)

Annotation inventory: **1653** (was 1640, +13). Same accept set.

### 1.0.85 / Contract 1.0.85 — SV-Slice-85 batch: type_declaration + type_identifier_or_class_type + type_reference + net_alias + net_declaration typed (7 rules / 19 annotations)

Annotation inventory: **1640** (was 1621, +19). Same accept set.

### 1.0.84 / Contract 1.0.84 — SV-Slice-84 batch: net_assignment + param_expression + struct_union + inst_name typed (5 rules / 10 annotations)

Annotation inventory: **1621** (was 1611, +10). Same accept set.

### 1.0.83 / Contract 1.0.83 — SV-Slice-83 batch: block_data_declaration + base_class_type + misc typed (8 rules / 15 annotations)

Annotation inventory: **1611** (was 1596, +15). Same accept set.

### 1.0.82 / Contract 1.0.82 — SV-Slice-82 batch: dynamic_override + incomplete_class + var_data_type + timing leaves typed (10 rules / 14 annotations)

Annotation inventory: **1596** (was 1582, +14). Same accept set.

### 1.0.81 / Contract 1.0.81 — SV-Slice-81 batch: config_rule + library + hierarchical_identifier + severity typed (13 rules / 25 annotations)

Closes LRM A.1.7 config/library walk paths + various leaf rules. Annotation inventory: **1582** (was 1557, +25). Same accept set.

### 1.0.80 / Contract 1.0.80 — SV-Slice-80 batch: boolean_abbrev + repetition + elaboration + repeat_range typed (8 rules / 19 annotations)

Closes LRM A.2.10 boolean_abbrev, A.6.5 elaboration_system_task, repeat_range, and repetition leaves. Annotation inventory: **1557** (was 1538, +19). Same accept set.

### 1.0.79 / Contract 1.0.79 — SV-Slice-79 batch: event + local/type parameter + mintypmax + nettype family typed (12 rules / 18 annotations)

Closes event_expression, event_trigger, local_parameter, mintypmax, nettype_declaration, type_assignment + type_parameter_declaration sub-trees. Annotation inventory: **1538** (was 1520, +18). Same accept set.

### 1.0.78 / Contract 1.0.78 — SV-Slice-78 batch: class_constructor wrappers + let + for + named_port + parameter_port typed (21 rules / 48 annotations)

**What changed:** Closes LRM A.1.10 class_constructor wrappers, A.6.8 let, A.6.5 for-loop, A.6.5 named/ordered checker/port connections, A.1.3 parameter_port.

Annotation inventory: **1520** (was 1472, +48). Same accept set.

### 1.0.77 / Contract 1.0.77 — SV-Slice-77 batch: module_path + constraint internals + uniqueness + misc typed (16 rules / 29 annotations)

**What changed:** Closes LRM A.8.3 module_path expression sub-tree, constraint primary scope, cycle_delay_range, extern_tf, inst_clause, solve_before list, and uniqueness_constraint family.

Annotation inventory: **1472** (was 1443, +29). Same accept set.

### 1.0.76 / Contract 1.0.76 — SV-Slice-76 batch: class_scope + method_call + tf_call family typed (14 rules / 27 annotations)

**What changed:** Closes LRM A.8.4 class-scope + method-call + tf-call walk paths.

```ebnf
class_scope_type / class_scope / implicit_class_handle (3 kinds) /
method_call / method_call_initial (5 kinds) / method_call_body (3 kinds) /
method_call_receiver_sv_2017 / method_call_receiver (2 kinds) /
method_call_root (2 kinds) / plain_tf_call_with_args / tf_call_with_args /
tf_call (4 kinds) / class_scoped_call_prefix /
class_scoped_tf_call_with_args / class_scoped_tf_call (2 kinds)
```

DEFERRED: `method_call_receiver_sv_2023` (~14 kinds with parens-grouped-Or sub-expressions — task #38 risk).

Annotation inventory: **1443** (was 1416, +27). Same accept set.

### 1.0.75 / Contract 1.0.75 — SV-Slice-75 batch: net_port_type + I/O declarations + genvar typed (10 rules / 17 annotations)

**What changed:** Closes LRM A.2.1.2 net_port_type / nonansi I/O declarations / LRM A.2.1.3 genvar sub-trees.

```ebnf
genvar_declaration / genvar_expression / inout_declaration /
input_declaration (2 kinds) / output_declaration (2 kinds) /
net_port_header / net_port_type_sv_2017/2023 (3+3 kinds) /
net_port_type (2 kinds) / net_type_declaration_sv_2017
```

DEFERRED: `net_type` (duplicate-branch grammar bug).

Annotation inventory: **1416** (was 1399, +17). Same accept set.

### 1.0.74 / Contract 1.0.74 — SV-Slice-74 batch: dpi + extern_constraint + interface_class + param family typed (18 rules / 32 annotations)

**What changed:** Closes LRM A.2.6 DPI sub-tree, LRM A.1.4 extern_constraint, LRM A.1.9 interface_class, and LRM A.2.1.1 param_assignment sub-trees.

DPI: `dpi_function_import_property` (2 kinds), `dpi_function_proto`, `dpi_import_export` (4 kinds), `dpi_spec_string` (2 kinds), `dpi_task_import_property`, `dpi_task_proto`. Extern constraint: `extern_constraint_declaration_sv_2017/2023`, `extern_constraint_declaration` (2 kinds). Interface class: `interface_class_item` (5 kinds), `interface_class_method`, `interface_class_type`, `interface_port_declaration` (2 kinds), `interface_port_header` (2 kinds). Param: `ordered_parameter_assignment`, `param_assignment_sv_2017/2023` (each 2 kinds), `param_assignment` (2 kinds).

Annotation inventory: **1399** (was 1367, +32). Same accept set.

### 1.0.73 / Contract 1.0.73 — SV-Slice-73 batch: checker family typed (10 rules / 33 annotations)

**What changed:** Closes LRM A.2.2.2 checker-declaration walk path.

```ebnf
checker_declaration                    -> {name, ports, items, end_label}
checker_generate_item_sv_2017/2023     -> 4 kinds each
checker_generate_item                  -> 2 kinds
checker_instantiation                  -> {name, instance, connections}
checker_or_generate_item               -> 7 kinds
checker_or_generate_item_declaration   -> 10 kinds
checker_port_direction                 -> 2 kinds
checker_port_item                      -> {attributes, direction, formal_type, name, dims, default}
checker_port_list                      -> [$1, $2::2*]
```

Annotation inventory: **1367** (was 1334, +33). Same accept set.

### 1.0.72 / Contract 1.0.72 — SV-Slice-72 batch: sequence family typed (16 rules / 39 annotations)

**What changed:** Closes LRM A.2.10 sequence sub-tree referenced from `property_expr.kind == "sequence".body`.

```ebnf
seq_input_list / sequence_abbrev / sequence_actual_arg_sv_2017/2023 (2+3 kinds) /
sequence_actual_arg / sequence_declaration / sequence_expr (12 kinds) /
sequence_formal_type (3 kinds) / sequence_instance / sequence_list_of_arguments (2 kinds) /
sequence_lvar_port_direction (3 kinds bare) / sequence_match_item (3 kinds) /
sequence_method_call / sequence_port_item / sequence_port_list / with_covergroup_expression
```

Annotation inventory: **1334** (was 1295, +39). Same accept set.

### 1.0.71 / Contract 1.0.71 — SV-Slice-71 batch: property_expr family typed (2 rules / 72 annotations)

**What changed:** Closes LRM A.2.10 property expression sub-tree.

`property_expr_sv_2017` and `property_expr_sv_2023` each get 36 kinds covering all property operators (sequence / strong / weak / paren / not / or / and / sequence_dup / implies_unary / sequence_or_assign / if / case / imp_minus / imp_assign / nexttime / nexttime_const / s_nexttime / s_nexttime_const / always / always_range / s_always / s_eventually / eventually / s_eventually_range / until / s_until / until_with / s_until_with / implies_binary / iff / accept_on / reject_on / sync_accept_on / sync_reject_on / instance / clocking).

Annotation inventory: **1295** (was 1223, +72). Same accept set.

### 1.0.70 / Contract 1.0.70 — SV-Slice-70 batch: property family (excluding property_expr) typed (12 rules / 16 annotations)

**What changed:** Closes the property-declaration walk path.

```ebnf
assertion_variable_declaration -> {data_type, items}
let_list_of_arguments          -> 2 kinds (mixed / named_only)
property_actual_arg            -> 2 kinds
property_case_item             -> {expressions, body}
property_declaration           -> {name, ports, declarations, spec, end_label}
property_formal_type           -> 2 kinds
property_instance              -> {name, args}
property_list_of_arguments     -> 2 kinds
property_lvar_port_direction   -> 1 kind bare
property_port_item             -> {attributes, local_direction, formal_type, name, dims, default}
property_port_list             -> [$1, $2::2*]
property_spec                  -> {clocking, disable_iff, body}
```

DEFERRED: `property_expr_sv_2017/2023` (~30 kinds each) — slice 71.

Annotation inventory: **1223** (was 1207, +16). Same accept set.

### 1.0.69 / Contract 1.0.69 — SV-Slice-69 batch: cover_cross + trans + select_expression typed (12 rules / 29 annotations)

**What changed:** Closes LRM A.2.11 cross-cover and trans-list walk paths.

```ebnf
cover_cross               -> {label, items, condition, body}
cross_body_sv_2017/2023   -> 2 kinds each (block / empty)
cross_body                -> 2 kinds (sv_2017 / sv_2023)
cross_body_item_sv_2017/2023 -> 2 kinds each (function_decl / selection_or_option)
cross_body_item           -> 2 kinds
cross_item                -> 2 kinds (cover_point / variable)
trans_list                -> [$2, $4::3*]
trans_range_list          -> 4 kinds (simple / star / implies / assign)
trans_set                 -> [$1, $2::2*]
select_expression         -> 8 kinds (condition / not / and / or / paren / with_matches / cross / cross_set)
```

Annotation inventory: **1207** (was 1178, +29). Same accept set.

### 1.0.68 / Contract 1.0.68 — SV-Slice-68 batch: bins family typed (5 rules / 14 annotations)

**What changed:** Closes LRM A.2.11 bin-declaration sub-tree referenced from `cover_point.bins`.

```ebnf
bins_expression           -> 2 kinds (variable / cover_point)
bins_or_empty             -> 2 kinds (block / empty)
bins_or_options           -> 7 kinds (coverage_option / range_list / cover_point_with / set / trans_list / default / default_sequence)
bins_selection            -> {keyword, name, select, iff}
bins_selection_or_option  -> 2 kinds (option / selection)
```

Annotation inventory: **1178** (was 1164, +14). Same accept set.

### 1.0.67 / Contract 1.0.67 — SV-Slice-67 batch: covergroup declaration + coverage_event family typed (12 rules / 26 annotations)

**What changed:** Closes LRM A.2.11 covergroup-declaration walk path.

```ebnf
bins_keyword                    -> 3 kinds bare (bins / illegal_bins / ignore_bins)
covergroup_declaration_sv_2017  -> {name, ports, event, items, end_label}
covergroup_declaration_sv_2023  -> 2 kinds (single / extends)
coverage_event                  -> 3 kinds (clocking / sample_function / block_event)
coverage_option                 -> 2 kinds (option / type_option)
coverage_spec                   -> 2 kinds (point / cross)
coverage_spec_or_option         -> 2 kinds (spec / option)
cover_point                     -> {label, expression, condition, bins}
covergroup_range_list           -> [$1, $2::2*]
covergroup_value_range_sv_2017  -> 2 kinds (expression / range)
covergroup_value_range_sv_2023  -> 5 kinds (expression / range / dollar_lo / dollar_hi / tolerance)
covergroup_value_range          -> 2 kinds (sv_2017 / sv_2023)
```

DEFERRED: bins_or_empty / bins_or_options / bins_selection / bins_selection_or_option (deep multi-branch — slice 68).

Annotation inventory: **1164** (was 1138, +26). Same accept set.

### 1.0.66 / Contract 1.0.66 — SV-Slice-66 batch: UDP body/entry + udp_instance family typed (9 rules / 19 annotations)

**What changed:** Closes LRM A.5 UDP body/instance walk paths.

```ebnf
combinational_entry  -> {inputs, output}
sequential_entry     -> {inputs, current_state, next_state}
level_symbol         -> 7 kinds bare (0 / 1 / x / X / ? / b / B)
output_symbol        -> 4 kinds bare (0 / 1 / x / X)
next_state           -> 2 kinds (symbol / minus)
udp_instance         -> {name, output, inputs: [$5, $6::2*]}
udp_instantiation    -> {name, drive_strength, delay, instances: [$4, $5::2*]}
udp_port_declaration -> 3 kinds (output / input / reg)
udp_reg_declaration  -> {attributes, name}
```

DEFERRED: `init_val` (duplicate-branch grammar bug, same family as drive_strength).

Annotation inventory: **1138** (was 1119, +19). Same accept set.

### 1.0.65 / Contract 1.0.65 — SV-Slice-65 batch: timing_check internals + scalar_timing_check_condition typed (16 rules / 22 annotations)

**What changed:** Closes the body field of every `system_timing_check.kind` from slice 64.

```ebnf
scalar_timing_check_condition -> 6 kinds (expression / not / eq / case_eq / ne / case_ne)
delayed_data                  -> 2 kinds (simple / with_expr)
delayed_reference             -> 2 kinds (simple / with_expr)
12 sv_dollar_*_timing_check    -> field-typed top-level args + tail slot for nested optional envelope
```

Annotation inventory: **1119** (was 1097, +22). Same accept set.

### 1.0.64 / Contract 1.0.64 — SV-Slice-64 batch: edge + timing_check family typed (10 rules / 37 annotations)

**What changed:** Closes LRM A.7.5.3 / A.7.6 timing check sub-trees (referenced from `specify_item.kind == "system_timing".body`) and the LRM A.5 UDP edge-input descriptors.

```ebnf
edge_descriptor              -> 4 kinds (01 / 10 / z_or_x_first / digit_first)
edge_indicator               -> 2 kinds (pair / symbol)
edge_symbol                  -> 9 kinds bare (r / R / f / F / p / P / n / N / star)
edge_control_specifier       -> {descriptors}
edge_input_list              -> {leading_levels, indicator, trailing_levels}
system_timing_check          -> 12 kinds (setup / hold / setuphold / recovery / removal / recrem / skew / timeskew / fullskew / period / width / nochange)
timing_check_condition       -> 2 kinds (scalar / paren)
timing_check_event           -> {control, descriptor, condition}
timing_check_event_control   -> 4 kinds (posedge / negedge / edge / edge_control)
controlled_timing_check_event-> {control, descriptor, condition}
```

Annotation inventory: **1097** (was 1060, +37). Same accept set.

### 1.0.63 / Contract 1.0.63 — SV-Slice-63 batch: path_declaration family typed (14 rules / 25 annotations)

**What changed:** Closes LRM A.7.2 / A.7.4 path declarations (referenced from `specify_item.kind == "path".body`).

```ebnf
path_declaration                      -> 3 kinds (simple / edge_sensitive / state_dependent)
simple_path_declaration               -> 2 kinds (parallel / full)
edge_sensitive_path_declaration       -> 2 kinds (parallel / full)
state_dependent_path_declaration      -> 3 kinds (if_simple / if_edge_sensitive / ifnone)
parallel_path_description             -> {input, polarity, output}
full_path_description                 -> {inputs, polarity, outputs}
parallel_edge_sensitive_path_description_sv_2017  -> {edge, input, in_polarity, output, out_polarity, data_source}
parallel_edge_sensitive_path_description_sv_2023  -> 2 kinds (with_data_source / simple)
full_edge_sensitive_path_description_sv_2017      -> {edge, inputs, in_polarity, outputs, out_polarity, data_source}
full_edge_sensitive_path_description_sv_2023      -> 2 kinds
path_delay_value                      -> 2 kinds (bare / paren)
list_of_path_delay_expressions        -> {body}
pulsestyle_declaration                -> 2 kinds (onevent / ondetect)
showcancelled_declaration             -> 2 kinds (showcancelled / noshowcancelled)
```

Annotation inventory: **1060** (was 1035, +25). Same accept set.

### 1.0.62 / Contract 1.0.62 — SV-Slice-62 batch: specify family typed (9 rules / 15 annotations)

**What changed:** Closes LRM A.7 specify-block walk path (referenced from `non_port_module_item.kind == "specify".body`) and the LRM A.7.5.1 specparam declaration sub-tree.

```ebnf
specify_block                        -> {items}
specify_item                         -> 5 kinds (specparam / pulsestyle / showcancelled / path / system_timing)
specify_input_terminal_descriptor    -> {name, range}
specify_output_terminal_descriptor   -> {name, range}
specify_terminal_descriptor          -> 2 kinds (input / output)
specparam_assignment                 -> 2 kinds (simple {name, value} / pulse {body})
specparam_declaration                -> {dims, items}
polarity_operator                    -> 2 kinds bare (plus / minus)
```

Annotation inventory: **1035** (was 1020, +15). Same accept set.

### 1.0.61 / Contract 1.0.61 — SV-Slice-61 batch: gate_instantiation family typed (16 rules / 43 annotations — crosses 1000-annotation milestone)

**What changed:** Closes LRM A.3.1 gate instantiation walk path (referenced from `module_or_generate_item.kind == "gate_instantiation".body`).

```ebnf
gate_instantiation_sv_2017/2023  -> 9 kinds each (cmos / enable / mos / n_input / n_output / pass_en / pass / pulldown / pullup)
cmos_switchtype                  -> 2 kinds bare (cmos / rcmos)
mos_switchtype                   -> 4 kinds bare (nmos / pmos / rnmos / rpmos)
n_input_gatetype                 -> 6 kinds bare (and / nand / or / nor / xor / xnor)
n_output_gatetype                -> 2 kinds bare (buf / not)
pass_switchtype                  -> 2 kinds bare (tran / rtran)
name_of_instance                 -> {name, dims}
cmos_switch_instance             -> {name, output, input, ncontrol, pcontrol}
enable_gate_instance             -> {name, output, input, enable}
mos_switch_instance              -> {name, output, input, enable}
n_input_gate_instance            -> {name, output, inputs: [$5, $6::2*]}
n_output_gate_instance           -> {name, outputs: [$3, $4::2*], input}
pass_enable_switch_instance      -> {name, in1, in2, enable}
pass_switch_instance             -> {name, in1, in2}
pull_gate_instance               -> {name, output}
```

DEFERRED: `enable_gatetype` / `pass_en_switchtype` / `pulldown_strength` / `pullup_strength` all have duplicate-branch grammar bug (same family as drive_strength / unique_priority / delay_sv_2017/2023).

Annotation inventory: **1020** (was 977, +43). **Crosses 1000-annotation milestone.** Same accept set.

### 1.0.60 / Contract 1.0.60 — SV-Slice-60 batch: number + literal family typed (10 rules / 19 annotations)

**What changed:** Closes LRM A.8.7 number sub-tree (referenced from `primary_literal.kind == "number" / "time_literal" / "string_literal".body`).

```ebnf
number              -> 2 kinds (real / integral)
integral_number     -> 4 kinds (decimal / octal / binary / hex)
real_number         -> 2 kinds (fixed_point / exponential {mantissa, fraction, sign, exponent})
binary_number       -> {size, base, value}
octal_number        -> {size, base, value}
hex_number          -> {size, base, value}
decimal_number      -> 4 kinds (unsized / sized / x_digit / z_digit)
fixed_point_number  -> {whole, fractional}
time_literal        -> {value, unit}
string_literal      -> 2 kinds (triple_quoted / double_quoted)
```

Annotation inventory: **977** (was 958, +19). Same accept set.

### 1.0.59 / Contract 1.0.59 — SV-Slice-59 batch: always + modport family typed (11 rules / 19 annotations)

**What changed:** Closes LRM A.1.4 always-construct dispatch and LRM A.2.9 modport family.

```ebnf
always_construct        -> {keyword, body}
always_keyword          -> 4 kinds bare (always / always_comb / always_latch / always_ff)
import_export           -> 2 kinds bare (import / export)
modport_simple_ports_declaration -> {direction, port}
modport_clocking_declaration     -> {name}
modport_declaration              -> {items: [$2, $3::2*]}
modport_item                     -> {name, ports: [$3, $4::2*]}
modport_ports_declaration        -> 3 kinds (simple / tf / clocking; each {kind, attributes, body})
modport_simple_port              -> 2 kinds (identifier {name} / explicit {name, expression})
modport_tf_port                  -> 2 kinds (method_prototype {body} / tf_identifier {name})
modport_tf_ports_declaration     -> {import_export, ports: [$2, $3::2*]}
```

All new pure-list patterns use `[$N, $M::2*]` from the start per slice 58 audit conclusion.

**Calibration:** parses minimal_module.sv unchanged plus a 5-line interface-with-modport sample. Annotation inventory: **958** (was 939, +19). Same accept set.

### 1.0.58 / Contract 1.0.58 — SV-Slice-58 audit: horizontal `{first, rest}` → `[$N, $M::2*]` extraction-spread fix across 49 grammar locations

**What changed:** Horizontal correctness audit (not a typing slice). Replace `{first: $N, rest: $M}` (raw `[sep, item]` envelope) with `[$N, $M::2*]` (clean flat array of items, separators dropped) for every Category A rule (pure `X (sep X)*` with single payload per iteration). Annotation count unchanged at **939**.

**Why:** Earlier slices defaulted to `{first, rest}` which exposed `[[sep, item], ...]` to consumers. The annotation language has a first-class extraction-spread operator `$N::M*` (defined in `grammars/return_annotation.ebnf` lines 50-58 and used by the bootstrap parser itself at line 158). For pure-list patterns it gives consumers a flat item array — no envelope walking required. This audit was triggered by user feedback after slice 57's `tf_port_list` was flagged.

**Consumer-visible diff** (e.g. `tf_port_list := tf_port_item ( comma tf_port_item )*`):

```
Before  →  { first: <tf_port_item>, rest: [[<comma>, <tf_port_item>], ...] }
After   →  [ <tf_port_item>, <tf_port_item>, ... ]
```

**Affected rules:** assignment_pattern, attribute_instance, bind_target_instance_list, case_generate_item, case_item, class_constructor_arg_list_sv_2023, concatenation, cond_predicate, constant_concatenation, data_type (enum names), dist_list, let_port_list, list_of_arguments_{mixed,ordered,named}, list_of_checker_port_connections, list_of_clocking_decl_assign, list_of_cross_items (also flattened first/second/rest into a single array), list_of_defparam_assignments, list_of_genvar_identifiers, list_of_net_assignments, list_of_net_decl_assignments, list_of_param_assignments, list_of_parameter_assignments_sv_2017, list_of_parameter_value_assignments_sv_2023, list_of_path_inputs, list_of_path_outputs, list_of_port_connections, list_of_ports, list_of_specparam_assignments, list_of_type_assignments, list_of_udp_port_identifiers, list_of_variable_assignments, list_of_variable_decl_assignments, net_lvalue (concat), open_range_list_sv_2017, package_export_declaration (explicit), package_import_declaration, pattern_sv_2017/2023 (ordered), production_sv_2017, range_list_sv_2023, rs_case_item_sv_2017/2023, rs_production_sv_2023, tf_port_list, udp_declaration_port_list, udp_port_list, variable_lvalue (concat), wait_order_statement (events).

**Deferred to SV-Slice-59** (Category B — multi-payload per iteration, requires helper-rule extractions): constant_expression op-chain, expression operand_chain, list_of_*_identifiers families with dims/init payloads, pattern named-branch entries.

**Deferred to post-campaign holistic review** (Category C — `X X*` with no separator, currently `{first, rest}` works correctly but verbosely).

**Calibration:** parses minimal_module.sv unchanged. Annotation inventory at 939 (unchanged). Same accept set.

### 1.0.57 / Contract 1.0.57 — SV-Slice-57 batch: tf_port + prototypes + lvalue/decl_assignment family typed (12 rules / 23 annotations + 1 new helper rule with 2 annotations)

**What changed:** Closes the LRM A.2.7 task/function port-list family + prototype rules + LRM A.8.1 lvalue family.

```ebnf
tf_port_list                         -> {first, rest}
tf_port_item                         -> {attributes, direction, var_keyword, data_type, port_spec}
tf_port_direction_sv_2017            -> 2 kinds (port_direction / const_ref)
tf_port_direction_sv_2023            -> 2 kinds (port_direction / ref with optional const + static)
function_prototype_sv_2017           -> {return_type, name, ports}
function_prototype_sv_2023           -> {dynamic_override, return_type, name, ports}
task_prototype_sv_2017/2023          -> parallel shape
let_port_item                        -> {attributes, type, name, dims, init}
let_port_list                        -> {first, rest}
net_decl_assignment                  -> {name, dims, init}
variable_decl_assignment             -> 3 kinds (variable / dynamic_array / class)
net_lvalue                           -> 3 kinds (name / concatenation / pattern)
variable_lvalue                      -> 4 kinds (name / concatenation / pattern / streaming_concatenation)
variable_lvalue_scope (NEW)          -> 2 kinds (instance / package_scope)
```

13th use of helper-rule extraction pattern.

**Annotation inventory:** 939 entries (was 914). +25 in this batch.

**Schema version:** stays at `1`.

**Contract section:** [`docs/contracts/PGEN_SYSTEMVERILOG_PARSER_INTEGRATION_CONTRACT.md`](../../contracts/PGEN_SYSTEMVERILOG_PARSER_INTEGRATION_CONTRACT.md) → "Release 1.0.57 / Contract 1.0.57 Highlights".

### 1.0.56 / Contract 1.0.56 — SV-Slice-56 batch: class_constructor_declaration family typed (4 rules / 5 annotations + 1 new helper rule with 2 annotations)

**What changed:** Closes the class constructor declaration walks for both LRM 1800-2017 and 2023 profiles.

```ebnf
class_constructor_arg_sv_2023            -> 2 kinds (tf_port_item / default)
class_constructor_arg_list_sv_2023       -> {first, rest}
class_constructor_declaration_sv_2017    -> {class_scope, ports, decls, super_call, statements, end_label}
class_constructor_declaration_sv_2023    -> parallel shape
class_constructor_super_args (NEW)       -> 2 kinds (args / default)
```

12th use of helper-rule extraction pattern (now used in 13 places total).

**Annotation inventory:** 914 entries (was 907). +7 in this batch.

**Schema version:** stays at `1`.

**Contract section:** [`docs/contracts/PGEN_SYSTEMVERILOG_PARSER_INTEGRATION_CONTRACT.md`](../../contracts/PGEN_SYSTEMVERILOG_PARSER_INTEGRATION_CONTRACT.md) → "Release 1.0.56 / Contract 1.0.56 Highlights".

### 1.0.55 / Contract 1.0.55 — SV-Slice-55 batch: clocking + class_constructor_prototype + edge_identifier + method_prototype typed (10 rules / 22 annotations — crosses 900-annotation milestone)

**What changed:** Closes the LRM A.6.10 clocking declaration sub-tree.

```ebnf
class_constructor_prototype_sv_2017/2023 -> {ports}
clocking_decl_assign                     -> {name, value}
clocking_declaration                     -> {default_keyword, name, event, items, end_label}
clocking_direction                       -> 4 kinds (input / output / input_output / inout)
clocking_event_sv_2017                   -> {body}
clocking_event_sv_2023                   -> 3 kinds (ps / hierarchical / expression)
clocking_item                            -> 3 kinds (default_skew / direction / assertion)
clocking_skew                            -> 2 kinds (edge / delay)
edge_identifier                          -> 3 kinds bare (posedge / negedge / edge)
method_prototype                         -> 2 kinds (task / function)
```

**Annotation inventory:** 907 entries (was 885). +22 in this batch — crosses the 900-annotation milestone.

**Schema version:** stays at `1`.

**Contract section:** [`docs/contracts/PGEN_SYSTEMVERILOG_PARSER_INTEGRATION_CONTRACT.md`](../../contracts/PGEN_SYSTEMVERILOG_PARSER_INTEGRATION_CONTRACT.md) → "Release 1.0.55 / Contract 1.0.55 Highlights".

### 1.0.54 / Contract 1.0.54 — SV-Slice-54 batch: delay/event/strength leaves typed (10 rules / 33 annotations)

**What changed:** Closes the LRM A.6.5 / A.6.4 timing-control / event-control / strength leaves.

```ebnf
charge_strength                       -> 3 kinds bare (small / medium / large)
cycle_delay                           -> 3 kinds (number / identifier / expression)
cycle_delay_const_range_expression    -> 2 kinds (range / dollar_hi)
delay_control                         -> 2 kinds (value / mintypmax)
delay_or_event_control                -> 3 kinds (delay / event / repeat)
delay_value                           -> 5 kinds (unsigned_number / real_number / ps_identifier / time_literal / step)
event_control_sv_2017                 -> 5 kinds (event / expression / wildcard / wildcard_alt / sequence)
event_control_sv_2023                 -> 3 kinds (clocking / wildcard / wildcard_paren)
event_expression_primary              -> 3 kinds (expression / sequence / paren)
strength                              -> 4 kinds bare (supply / strong / pull / weak)
```

**DEFERRED:** drive_strength, delay_sv_2017/2023 (grammar duplicate-branch bug), event_expression (parens-Or in Quantified — task #38 risk).

**Annotation inventory:** 885 entries (was 852). +33 in this batch.

**Schema version:** stays at `1`.

**Contract section:** [`docs/contracts/PGEN_SYSTEMVERILOG_PARSER_INTEGRATION_CONTRACT.md`](../../contracts/PGEN_SYSTEMVERILOG_PARSER_INTEGRATION_CONTRACT.md) → "Release 1.0.54 / Contract 1.0.54 Highlights".

### 1.0.53 / Contract 1.0.53 — SV-Slice-53 batch: array/stream/class_new/join leaf cleanup typed (9 rules / 18 annotations)

**What changed:** Closes pervasive leaf rules used across primary / streaming-concat / par_block / dynamic-array contexts.

```ebnf
array_method_name                  -> 5 kinds (method_identifier / unique / and / or / xor)
class_new                          -> 2 kinds (constructor / copy)
dynamic_array_new                  -> {size, init}
empty_unpacked_array_concatenation -> bare {kind}
join_keyword                       -> 3 kinds bare (join / join_any / join_none)
slice_size                         -> 2 kinds (simple_type / constant_expression)
stream_concatenation               -> {body}
stream_expression                  -> {expr, with_clause}
stream_operator                    -> 2 kinds bare (shift_right / shift_left)
```

**Annotation inventory:** 852 entries (was 834). +18 in this batch.

**Schema version:** stays at `1`.

**Contract section:** [`docs/contracts/PGEN_SYSTEMVERILOG_PARSER_INTEGRATION_CONTRACT.md`](../../contracts/PGEN_SYSTEMVERILOG_PARSER_INTEGRATION_CONTRACT.md) → "Release 1.0.53 / Contract 1.0.53 Highlights".

### 1.0.52 / Contract 1.0.52 — SV-Slice-52 batch: simple_type + range/dist family typed (14 rules / 29 annotations)

**What changed:** Closes the simple_type / range_expression / part_select_range / dist_* / range_list / value_range walk paths.

```ebnf
simple_type                    -> 4 kinds (integer / non_integer / ps_type / ps_parameter)
range_expression               -> 2 kinds (expression / part_select_range)
part_select_range              -> 2 kinds (range / indexed_range)
constant_part_select_range     -> 2 kinds (parallel)
indexed_range                  -> 2 kinds (plus_indexed [base+:width] / minus_indexed [base-:width])
constant_indexed_range         -> 2 kinds (parallel)
dist_list                      -> {first, rest}
dist_item_sv_2017              -> {value, weight}
dist_item_sv_2023              -> 2 kinds (value / default)
dist_weight                    -> 2 kinds (equal := / proportional :/)
range_list_sv_2023             -> {first, rest}
open_range_list_sv_2017        -> {first, rest}
value_range_sv_2017            -> 2 kinds (expression / range)
value_range_sv_2023            -> 5 kinds (expression / range / dollar_lo / dollar_hi / tolerance)
```

**Annotation inventory:** 834 entries (was 805). +29 in this batch.

**Schema version:** stays at `1`.

**Contract section:** [`docs/contracts/PGEN_SYSTEMVERILOG_PARSER_INTEGRATION_CONTRACT.md`](../../contracts/PGEN_SYSTEMVERILOG_PARSER_INTEGRATION_CONTRACT.md) → "Release 1.0.52 / Contract 1.0.52 Highlights".

### 1.0.51 / Contract 1.0.51 — SV-Slice-51 batch: select + constant_select + constant_range typed (4 rules / 5 annotations + 2 new helper rules with 4 annotations — crosses 800-annotation milestone)

**What changed:** Closes the select / constant_select referent used pervasively across primary's hierarchical-name suffix.

```ebnf
select                       -> {member_chain, tail}
select_tail (NEW)            -> 2 kinds (part_range / bit_select)
constant_select              -> {member_chain, tail}
constant_select_tail (NEW)   -> 2 kinds (part_range / bit_select)
constant_range               -> {lo, hi}
constant_range_expression    -> 2 kinds (expression / part_select_range)
```

**Annotation inventory:** 805 entries (was 796). +9 in this batch — crosses the 800-annotation milestone.

**Schema version:** stays at `1`.

**Contract section:** [`docs/contracts/PGEN_SYSTEMVERILOG_PARSER_INTEGRATION_CONTRACT.md`](../../contracts/PGEN_SYSTEMVERILOG_PARSER_INTEGRATION_CONTRACT.md) → "Release 1.0.51 / Contract 1.0.51 Highlights".

### 1.0.50 / Contract 1.0.50 — SV-Slice-50 batch: casting_type + bit_select + system_tf_call typed (3 rules / 9 annotations)

**What changed:** Closes the cast.type field referent and system-task-call dispatch.

```ebnf
casting_type      -> 5 kinds (simple_type / constant_primary / signing / string / const)
bit_select        -> {body}
system_tf_call    -> 3 kinds (args / data_type / expr_clocking)
```

**Annotation inventory:** 796 entries (was 787). +9 in this batch.

**Schema version:** stays at `1`.

**Contract section:** [`docs/contracts/PGEN_SYSTEMVERILOG_PARSER_INTEGRATION_CONTRACT.md`](../../contracts/PGEN_SYSTEMVERILOG_PARSER_INTEGRATION_CONTRACT.md) → "Release 1.0.50 / Contract 1.0.50 Highlights".

### 1.0.49 / Contract 1.0.49 — SV-Slice-49 batch: concat / cast / call_primary / attr_spec typed (9 rules / 14 annotations)

**What changed:** Closes the leaf rules used pervasively across primary_sv_2017/2023.

```ebnf
attr_spec                          -> {name, value}
cast / constant_cast               -> {type, body}
concatenation / constant_concatenation -> {first, rest}
multiple_concatenation / constant_multiple_concatenation -> {count, body}
streaming_concatenation            -> {op, slice_size, body}
call_primary                       -> 6 kinds (split_direct_callable_method / class_scoped_tf / plain_tf / tf / direct_callable_method / system_tf)
```

**Annotation inventory:** 787 entries (was 773). +14 in this batch.

**Schema version:** stays at `1`.

**Contract section:** [`docs/contracts/PGEN_SYSTEMVERILOG_PARSER_INTEGRATION_CONTRACT.md`](../../contracts/PGEN_SYSTEMVERILOG_PARSER_INTEGRATION_CONTRACT.md) → "Release 1.0.49 / Contract 1.0.49 Highlights".

### 1.0.48 / Contract 1.0.48 — SV-Slice-48 batch: primary_sv_2023 + constant_primary_sv_2023 typed (2 rules / 31 annotations)

**What changed:** Completes the parallel sv_2023 forms. Both sv_2017 and sv_2023 primary expression dispatch fully typed end-to-end.

```ebnf
primary_sv_2023           -> 15 kinds (sv_2017 kinds + LRM 2023's call select extension)
constant_primary_sv_2023  -> 16 kinds (sv_2017's 15 + empty_array_concat per LRM 2023)
```

Profile differences from sv_2017: `call` and `function_call` add optional `select` per LRM 2023, and constant_primary adds `empty_array_concat`. Reuses the 3 helper rules from SV-Slice-47.

**Annotation inventory:** 773 entries (was 742). +31 in this batch.

**Schema version:** stays at `1`.

**Contract section:** [`docs/contracts/PGEN_SYSTEMVERILOG_PARSER_INTEGRATION_CONTRACT.md`](../../contracts/PGEN_SYSTEMVERILOG_PARSER_INTEGRATION_CONTRACT.md) → "Release 1.0.48 / Contract 1.0.48 Highlights".

### 1.0.47 / Contract 1.0.47 — SV-Slice-47 batch: primary_sv_2017 + constant_primary_sv_2017 typed (2 rules / 30 annotations + 3 new helper rules with 6 annotations)

**What changed:** Closes the sv_2017 primary expression dispatch reachable from expression_operand.kind=="primary" and constant_expression_operand.kind=="primary".

```ebnf
primary_sv_2017               -> 15 kinds (literal / call / hierarchical / empty_array_concat / multiple_concat / concat / let / paren / cast / assign_pattern / streaming_concat / sequence_method / this / system_dollar / null_class_assign)
constant_primary_sv_2017      -> 15 kinds (literal / ps_parameter / specparam / genvar / formal_port / enum / multiple_concat / concat / function_call / let / paren / cast / assign_pattern / type_reference / null)
primary_hier_scope_prefix (NEW) -> 2 kinds (class_qualifier / package_scope)
instance_or_class_scope (NEW)   -> 2 kinds (instance / class_scope)
enum_id_scope_prefix (NEW)      -> 2 kinds (package_scope / class_scope)
```

**Helper-rule extraction** (7th, 8th, 9th uses of pattern). Now used in 10 places total.

**DEFERRED:** primary_sv_2023 and constant_primary_sv_2023 (parallel structures — to be applied in a follow-up slice).

**Annotation inventory:** 742 entries (was 706). +36 in this batch.

**Schema version:** stays at `1`.

**Contract section:** [`docs/contracts/PGEN_SYSTEMVERILOG_PARSER_INTEGRATION_CONTRACT.md`](../../contracts/PGEN_SYSTEMVERILOG_PARSER_INTEGRATION_CONTRACT.md) → "Release 1.0.47 / Contract 1.0.47 Highlights".

### 1.0.46 / Contract 1.0.46 — SV-Slice-46 batch: expression family typed (14 rules / 62 annotations — crosses 700-annotation milestone)

**What changed:** Single largest impact slice — `expression`, `constant_expression`, and their operand/operator/literal sub-rules underlie every expression-typed field across the grammar.

```ebnf
expression                       -> 3 kinds (base / inside / conditional)
expression_base                  -> 3 kinds (tagged_union / operand_chain / paren_op_assign)
expression_operand               -> 3 kinds (unary / inc_or_dec / primary)
expression_or_dist               -> {expr, dist}
constant_expression              -> {first, rest, ternary}
constant_expression_operand      -> 2 kinds (unary / primary)
inside_expression_sv_2017/2023   -> {expr, ranges}
conditional_expression           -> {predicate, attributes, then_expr, else_expr}
tagged_union_expression_sv_2017/2023 -> {name, value}
primary_literal                  -> 4 kinds (number / time_literal / unbased_unsized_literal / string_literal)
binary_operator                  -> 29 kinds bare {kind}
unary_operator                   -> 11 kinds bare {kind}
```

**Annotation inventory:** 706 entries (was 644). +62 in this batch — crosses the 700-annotation milestone.

**Schema version:** stays at `1`.

**Contract section:** [`docs/contracts/PGEN_SYSTEMVERILOG_PARSER_INTEGRATION_CONTRACT.md`](../../contracts/PGEN_SYSTEMVERILOG_PARSER_INTEGRATION_CONTRACT.md) → "Release 1.0.46 / Contract 1.0.46 Highlights".

### 1.0.45 / Contract 1.0.45 — SV-Slice-45 batch: pattern + cond_predicate family typed (6 rules / 18 annotations)

**What changed:** Closes the LRM A.6.7.1 pattern-matching walk path used by case_pattern_item, conditional_statement.condition (via cond_predicate), and constraint_expression's various forms.

```ebnf
cond_predicate              -> {first, rest}
cond_pattern                -> {expression, pattern}
expression_or_cond_pattern  -> 2 kinds (expression / cond_pattern)
pattern_sv_2017             -> 6 kinds (variable_capture / wildcard / expression / tagged / ordered / named)
pattern_sv_2023             -> 7 kinds (same 6 + parenthesized per LRM 2023)
assignment_pattern          -> {exprs: {first, rest}}
```

**Annotation inventory:** 644 entries (was 626). +18 in this batch.

**Schema version:** stays at `1`.

**Contract section:** [`docs/contracts/PGEN_SYSTEMVERILOG_PARSER_INTEGRATION_CONTRACT.md`](../../contracts/PGEN_SYSTEMVERILOG_PARSER_INTEGRATION_CONTRACT.md) → "Release 1.0.45 / Contract 1.0.45 Highlights".

### 1.0.44 / Contract 1.0.44 — SV-Slice-44 batch: list_of_* family typed (20 rules / 22 annotations)

**What changed:** Uniform mini-mixed-array pattern across the small declaration-list rules. Every list_of_* rule referenced from typed parents now exposes `{first, rest}` shape.

- 12 simple `{first, rest}` rules (clocking_decl_assign / defparam / genvar / net / param / path_inputs / path_outputs / specparam / type / variable, plus 2 variable-assignment forms).
- 3 `{first: {name, dims}, rest}` rules (interface / port / variable identifiers — with trailing dimension list per item).
- 2 `{first: {name, dims, init}, rest}` rules (tf_variable / variable_port — with optional initializer per item).
- 1 `{first, second, rest}` rule (cross_items — LRM requires at least 2).
- 2 2-kind dispatches (checker_port_connections, port_connections — ordered/named).

**Annotation inventory:** 626 entries (was 604). +22 in this batch.

**Schema version:** stays at `1`.

**Contract section:** [`docs/contracts/PGEN_SYSTEMVERILOG_PARSER_INTEGRATION_CONTRACT.md`](../../contracts/PGEN_SYSTEMVERILOG_PARSER_INTEGRATION_CONTRACT.md) → "Release 1.0.44 / Contract 1.0.44 Highlights".

### 1.0.43 / Contract 1.0.43 — SV-Slice-43 batch: parameter_value_assignment + arguments family typed (10 rules / 16 annotations — crosses 600-annotation milestone)

**What changed:** Closes the function/task/method-call argument and parameter-instance walks.

```ebnf
parameter_value_assignment_sv_2017/2023       -> {params}
list_of_parameter_assignments_sv_2017         -> 2 kinds (ordered / named)
list_of_parameter_value_assignments_sv_2023   -> 2 kinds (parallel)
named_parameter_assignment                    -> {name, value}
named_argument                                -> {name, value}
list_of_arguments                             -> 3 kinds (ordered / named / mixed)
list_of_arguments_ordered                     -> {first, rest}
list_of_arguments_named                       -> {first, rest}
list_of_arguments_mixed                       -> {head, named: {first, rest}}
list_of_arguments_mixed_head                  -> 2 kinds (single / chain — recursive)
```

**Annotation inventory:** 604 entries (was 588). +16 in this batch — crosses the 600-annotation milestone.

**Schema version:** stays at `1`.

**Contract section:** [`docs/contracts/PGEN_SYSTEMVERILOG_PARSER_INTEGRATION_CONTRACT.md`](../../contracts/PGEN_SYSTEMVERILOG_PARSER_INTEGRATION_CONTRACT.md) → "Release 1.0.43 / Contract 1.0.43 Highlights".

### 1.0.42 / Contract 1.0.42 — SV-Slice-42 batch: signing + struct_union + enum + type_reference + class_type internals typed (9 rules / 21 annotations + 2 new helper rules with 5 annotations)

**What changed:** Closes data_type field structural-content walks.

```ebnf
signing                          -> 2 kinds bare (signed / unsigned)
struct_union_sv_2017             -> 2 kinds (struct / union with optional tagged)
struct_union_sv_2023             -> 2 kinds (struct / union with union_modifier helper)
union_modifier (NEW)             -> 2 kinds bare (soft / tagged)
struct_union_member              -> {attributes, random_qualifier, data_type, decls}
enum_base_type                   -> 3 kinds (atom / vector / type_alias)
enum_name_declaration            -> {name, range, value}
type_reference_sv_2017/2023      -> 2 kinds each (expression / data_type[_or_incomplete_class])
class_type                       -> {head, params, suffix}
class_type_head (NEW)            -> 3 kinds (scoped / class / interface_class)
```

**Helper-rule extraction** (5th and 6th uses): `union_modifier` from `( kw_soft | kw_tagged )?`, `class_type_head` from leading 3-way parens-Or in class_type.

**Annotation inventory:** 588 entries (was 567). +21 in this batch.

**Schema version:** stays at `1`.

**Contract section:** [`docs/contracts/PGEN_SYSTEMVERILOG_PARSER_INTEGRATION_CONTRACT.md`](../../contracts/PGEN_SYSTEMVERILOG_PARSER_INTEGRATION_CONTRACT.md) → "Release 1.0.42 / Contract 1.0.42 Highlights".

### 1.0.41 / Contract 1.0.41 — SV-Slice-41 batch: data_type family typed (8 rules / 36 annotations)

**What changed:** Pervasive impact — `data_type` fields appear across the entire grammar.

```ebnf
data_type                                         -> 15 kinds (integer_vector / integer_atom / non_integer / struct_union / enum / string / chandle / virtual_interface / scoped_data_type / known_unscoped_data_type / class_type / provisional_class_type / event / covergroup / type_reference)
data_type_or_implicit                             -> 2 kinds (data_type / implicit)
data_type_or_void                                 -> 2 kinds (data_type / void)
data_type_or_incomplete_class_scoped_type_sv_2023 -> 2 kinds
implicit_data_type                                -> {signing, dims}
integer_atom_type                                 -> 6 kinds bare (byte / shortint / int / longint / integer / time)
integer_vector_type                               -> 3 kinds bare (bit / logic / reg)
non_integer_type                                  -> 3 kinds bare (shortreal / real / realtime)
integer_type                                      -> 2 kinds (vector / atom)
```

**Annotation inventory:** 567 entries (was 531). +36 in this batch.

**Schema version:** stays at `1`.

**Contract section:** [`docs/contracts/PGEN_SYSTEMVERILOG_PARSER_INTEGRATION_CONTRACT.md`](../../contracts/PGEN_SYSTEMVERILOG_PARSER_INTEGRATION_CONTRACT.md) → "Release 1.0.41 / Contract 1.0.41 Highlights".

### 1.0.40 / Contract 1.0.40 — SV-Slice-40 batch: simple immediate assertions + inc_or_dec + weight_specification typed (6 rules / 11 annotations)

**What changed:** Closes immediate_assertion_statement.kind=="simple", inc_or_dec_expression internals, and weight_specification_sv_2017.

```ebnf
simple_immediate_assertion_statement -> 3 kinds (assert / assume / cover)
simple_immediate_assert_statement    -> {condition, action}
simple_immediate_assume_statement    -> {condition, action}
simple_immediate_cover_statement     -> {condition, statement}
inc_or_dec_expression                -> 2 kinds (prefix {op, attributes, lvalue} / postfix {lvalue, attributes, op})
weight_specification_sv_2017         -> 3 kinds (number / identifier / expression)
```

**Annotation inventory:** 531 entries (was 520). +11 in this batch.

**Schema version:** stays at `1`.

**Contract section:** [`docs/contracts/PGEN_SYSTEMVERILOG_PARSER_INTEGRATION_CONTRACT.md`](../../contracts/PGEN_SYSTEMVERILOG_PARSER_INTEGRATION_CONTRACT.md) → "Release 1.0.40 / Contract 1.0.40 Highlights".

### 1.0.39 / Contract 1.0.39 — SV-Slice-39 batch: rs_* family typed (17 rules / 31 annotations — crosses 500-annotation milestone)

**What changed:** Closes the random-sequence walk path end-to-end. After this slice, every reachable randsequence_statement → production → rules.{first,rest} → rs_rule → rs_production_list → rs_prod → ... resolves through typed shapes.

```ebnf
rs_case                         -> {expr, items: {first, rest}}
rs_case_item_sv_2017/2023       -> 2 kinds (expr_list / default)
rs_code_block                   -> {body}
rs_if_else_sv_2017/2023         -> {condition, then_body, else_body}
rs_prod_sv_2017/2023            -> 5 kinds (production_item / code_block / if_else / repeat / case)
rs_production_sv_2023           -> {return_type, name, ports, rules: {first, rest}}
rs_production_item_sv_2023      -> {name, args}
rs_production_list_sv_2017/2023 -> 2 kinds (productions / rand_join)
rs_repeat_sv_2017/2023          -> {count, body}
rs_rule_sv_2017/2023            -> {productions, weight}
rs_weight_specification_sv_2023 -> 3 kinds (number / identifier / expression)
```

**Annotation inventory:** 520 entries (was 489). +31 in this batch — crosses the 500-annotation milestone.

**Schema version:** stays at `1`.

**Contract section:** [`docs/contracts/PGEN_SYSTEMVERILOG_PARSER_INTEGRATION_CONTRACT.md`](../../contracts/PGEN_SYSTEMVERILOG_PARSER_INTEGRATION_CONTRACT.md) → "Release 1.0.39 / Contract 1.0.39 Highlights".

### 1.0.38 / Contract 1.0.38 — SV-Slice-38 batch: randsequence top-level + production typed (4 rules / 4 annotations)

**What changed:** Closes the last raw-envelope statement_item kind.

```ebnf
randsequence_statement_sv_2017 -> {start, productions: {first, rest}}
randsequence_statement_sv_2023 -> {start, productions: {first, rest}}  (uses rs_production per LRM 2023)
production_sv_2017             -> {return_type, name, ports, rules: {first, rest}}
production_item_sv_2017        -> {name, args}
```

**DEFERRED:** rs_* family internals (rs_rule, rs_prod, rs_case, rs_if_else, rs_repeat, rs_code_block) — referenced from production.rules.

**Annotation inventory:** 489 entries (was 485). +4 in this batch.

**Schema version:** stays at `1`.

**Contract section:** [`docs/contracts/PGEN_SYSTEMVERILOG_PARSER_INTEGRATION_CONTRACT.md`](../../contracts/PGEN_SYSTEMVERILOG_PARSER_INTEGRATION_CONTRACT.md) → "Release 1.0.38 / Contract 1.0.38 Highlights".

### 1.0.37 / Contract 1.0.37 — SV-Slice-37 batch: blocking_assignment typed via helper-rule extraction (3 rules / 12 annotations + 1 new helper rule with 3 annotations)

**What changed:** Closes the last DEFERRED statement_item kind. After this slice, **all 20 (sv_2017) / 19 (sv_2023) statement_item kinds expose typed dispatch end-to-end**.

```ebnf
blocking_assignment_sv_2017     -> 4 kinds (delay_assign / dynamic_array_new / class_new / operator)
blocking_assignment_sv_2023     -> 5 kinds (same 4 + inc_or_dec per LRM 2023)
class_or_package_scope (NEW)    -> 3 kinds (instance / class_scope / package_scope)
```

**Helper-rule extraction (4th use of the pattern).** `class_or_package_scope` extracted from `( implicit_class_handle dot | class_scope | package_scope )?` — same pattern as if_generate_else_clause / net_strength / net_vector_scalar / conditional_else_branch.

**Annotation inventory:** 485 entries (was 473). +12 in this batch.

**Schema version:** stays at `1`.

**Contract section:** [`docs/contracts/PGEN_SYSTEMVERILOG_PARSER_INTEGRATION_CONTRACT.md`](../../contracts/PGEN_SYSTEMVERILOG_PARSER_INTEGRATION_CONTRACT.md) → "Release 1.0.37 / Contract 1.0.37 Highlights" with full annotation source + helper-rule pattern table + 100% statement_item dispatch coverage.

### 1.0.36 / Contract 1.0.36 — SV-Slice-36 batch: assignments + procedural assertions + randcase typed (8 rules / 16 annotations)

**What changed:** Closes 4 more statement_item kinds. After this slice, 19 of statement_item's 19/20 kinds expose typed dispatch end-to-end (only blocking_assignment remains DEFERRED).

```ebnf
nonblocking_assignment           -> {lvalue, control, value}
procedural_continuous_assignment -> 6 kinds (assign / deassign / force_variable / force_net / release_variable / release_net)
clocking_drive                   -> {lvalue, cycle_delay, value}
randcase_statement               -> {items: {first, rest}}
randcase_item                    -> {weight, body}
procedural_assertion_statement   -> 3 kinds (concurrent / immediate / checker_instantiation)
immediate_assertion_statement    -> 2 kinds (simple / deferred)
variable_assignment              -> {lvalue, value}
```

**DEFERRED:** `blocking_assignment_sv_2017/2023` (parens-Or workaround needed, 4th use of helper-rule pattern).

**Annotation inventory:** 473 entries (was 457). +16 in this batch.

**Schema version:** stays at `1`.

**Contract section:** [`docs/contracts/PGEN_SYSTEMVERILOG_PARSER_INTEGRATION_CONTRACT.md`](../../contracts/PGEN_SYSTEMVERILOG_PARSER_INTEGRATION_CONTRACT.md) → "Release 1.0.36 / Contract 1.0.36 Highlights" with full annotation source + statement_item dispatch coverage table.

### 1.0.35 / Contract 1.0.35 — SV-Slice-35 batch: conditional_statement typed via helper-rule extraction (1 rule / 1 annotation + 1 new helper rule with 2 annotations)

**What changed:** Closes the SV-Slice-34 DEFERRED `conditional_statement` typing using the helper-rule extraction pattern (third use after if_generate_else_clause and net_strength/net_vector_scalar).

```ebnf
conditional_statement      -> {unique_priority, condition, then_body, else_body}
conditional_else_branch (NEW) -> 2 kinds (elseif {body} / else {body})
```

**Annotation inventory:** 457 entries (was 454). +3 in this batch.

**Schema version:** stays at `1`.

**Contract section:** [`docs/contracts/PGEN_SYSTEMVERILOG_PARSER_INTEGRATION_CONTRACT.md`](../../contracts/PGEN_SYSTEMVERILOG_PARSER_INTEGRATION_CONTRACT.md) → "Release 1.0.35 / Contract 1.0.35 Highlights".

### 1.0.34 / Contract 1.0.34 — SV-Slice-34 batch: case + loop families typed (7 rules / 18 annotations)

**What changed:** Closes case-statement and loop-statement walks.

```ebnf
case_statement           -> {unique_priority, keyword, expr, items: {first, rest}}
case_keyword             -> 3 kinds bare (case / casez / casex)
case_item                -> 2 kinds (expr_list / default)
case_pattern_item        -> 2 kinds (pattern {pattern, condition, body} / default)
case_inside_item_sv_2017 -> 2 kinds (range_list using open_range_list / default)
case_inside_item_sv_2023 -> 2 kinds (range_list using range_list per LRM 2023 / default)
loop_statement           -> 6 kinds (forever / repeat / while / for / do_while / foreach)
```

**DEFERRED:** unique_priority (grammar duplicate-branch bug), conditional_statement (parens-Or with lookahead — needs helper rule).

**Annotation inventory:** 454 entries (was 436). +18 in this batch.

**Schema version:** stays at `1`.

**Contract section:** [`docs/contracts/PGEN_SYSTEMVERILOG_PARSER_INTEGRATION_CONTRACT.md`](../../contracts/PGEN_SYSTEMVERILOG_PARSER_INTEGRATION_CONTRACT.md) → "Release 1.0.34 / Contract 1.0.34 Highlights".

### 1.0.33 / Contract 1.0.33 — SV-Slice-33 batch: procedural-statement forms typed (11 rules / 26 annotations)

**What changed:** Closes 7 of statement_item's 19/20 kinds.

```ebnf
disable_statement                    -> 3 kinds (task / block / fork)
jump_statement                       -> 3 kinds (return / break / continue)
wait_statement                       -> 3 kinds (wait / wait_fork / wait_order)
event_trigger_sv_2017                -> 2 kinds (non_blocking / blocking)
event_trigger_sv_2023                -> 2 kinds (parallel; adds `select` field)
procedural_timing_control_statement  -> {control, body}
procedural_timing_control            -> 3 kinds (delay / event / cycle)
subroutine_call                      -> 5 kinds (class_scoped_tf / tf / system_tf / method / randomize)
subroutine_call_statement            -> 2 kinds (call / void_cast)
seq_block                            -> {label, declarations, statements, end_label}
par_block                            -> {label, declarations, statements, join, end_label}
```

**Annotation inventory:** 436 entries (was 410). +26 in this batch.

**Schema version:** stays at `1`.

**Contract section:** [`docs/contracts/PGEN_SYSTEMVERILOG_PARSER_INTEGRATION_CONTRACT.md`](../../contracts/PGEN_SYSTEMVERILOG_PARSER_INTEGRATION_CONTRACT.md) → "Release 1.0.33 / Contract 1.0.33 Highlights".

### 1.0.32 / Contract 1.0.32 — SV-Slice-32 batch: statement_item dispatch typed (3 rules / 43 annotations — crosses 400-annotation milestone)

**What changed:** Closes statement.body field, exposing typed dispatch into all 20 (sv_2017) / 19 (sv_2023) procedural-statement forms.

```ebnf
statement_item_sv_2017  -> 20 kinds (blocking_assignment, nonblocking_assignment, procedural_continuous_assignment, case, conditional, inc_or_dec_expression, subroutine_call, disable, event_trigger, loop, jump, par_block, procedural_timing_control, seq_block, wait, procedural_assertion, clocking_drive, randsequence, randcase, expect_property)
statement_item_sv_2023  -> 19 kinds (sv_2017 minus inc_or_dec_expression — LRM 2023 subsumes into blocking_assignment with ++/--)
block_item_declaration  -> 4 kinds (block_data / local_parameter / parameter / let)
```

**Annotation inventory:** 410 entries (was 367). +43 in this batch — crosses the 400-annotation milestone.

**Schema version:** stays at `1`.

**Contract section:** [`docs/contracts/PGEN_SYSTEMVERILOG_PARSER_INTEGRATION_CONTRACT.md`](../../contracts/PGEN_SYSTEMVERILOG_PARSER_INTEGRATION_CONTRACT.md) → "Release 1.0.32 / Contract 1.0.32 Highlights".

### 1.0.31 / Contract 1.0.31 — SV-Slice-31 batch: action_block + statement framing typed (5 rules / 9 annotations)

**What changed:** Closes action_block walk path (used pervasively by assertions) and statement framing path (used by function/task bodies).

```ebnf
action_block               -> 2 kinds (always {body} / with_else {pass, fail})
statement                  -> {label, attributes, body}
statement_or_null          -> 2 kinds (statement {body} / null {attributes})
function_statement_or_null -> 2 kinds (parallel)
tf_item_declaration        -> 2 kinds (block_item / tf_port)
```

**Annotation inventory:** 367 entries (was 358). +9 in this batch.

**Schema version:** stays at `1`.

**Contract section:** [`docs/contracts/PGEN_SYSTEMVERILOG_PARSER_INTEGRATION_CONTRACT.md`](../../contracts/PGEN_SYSTEMVERILOG_PARSER_INTEGRATION_CONTRACT.md) → "Release 1.0.31 / Contract 1.0.31 Highlights".

### 1.0.30 / Contract 1.0.30 — SV-Slice-30 batch: deferred immediate assertions typed (5 rules / 10 annotations)

**What changed:** Closes assertion_item.kind=="deferred_immediate" walk path.

```ebnf
deferred_immediate_assertion_item      -> {label, body}
deferred_immediate_assertion_statement -> 3 kinds (assert / assume / cover)
deferred_immediate_assert_statement    -> 2 kinds (zero_delay / final) {expression, action}
deferred_immediate_assume_statement    -> 2 kinds (parallel to assert)
deferred_immediate_cover_statement     -> 2 kinds (parallel; uses {expression, statement})
```

**Annotation inventory:** 358 entries (was 348). +10 in this batch.

**Schema version:** stays at `1`.

**Contract section:** [`docs/contracts/PGEN_SYSTEMVERILOG_PARSER_INTEGRATION_CONTRACT.md`](../../contracts/PGEN_SYSTEMVERILOG_PARSER_INTEGRATION_CONTRACT.md) → "Release 1.0.30 / Contract 1.0.30 Highlights".

### 1.0.29 / Contract 1.0.29 — SV-Slice-29 batch: concurrent assertion + constraint family typed (16 rules / 28 annotations)

**What changed:** Closes assertion_item.kind=="concurrent" walk (typed in SV-Slice-24) and class_constraint walk (typed in SV-Slice-27).

```ebnf
concurrent_assertion_statement -> 5 kinds (assert_property / assume_property / cover_property / cover_sequence / restrict_property)
assert_property_statement      -> {spec, action}
assume_property_statement      -> {spec, action}
cover_property_statement       -> {spec, statement}
cover_sequence_statement       -> {clocking, disable_iff, sequence, statement}
restrict_property_statement    -> {spec}
expect_property_statement      -> {spec, action}
constraint_declaration_sv_2017 -> {static_keyword, name, block}
constraint_declaration_sv_2023 -> {static_keyword, dynamic_override, name, block}
constraint_block               -> {items}
constraint_block_item          -> 2 kinds (solve_before {before, after} / expression {body})
constraint_expression          -> 6 kinds (expression / uniqueness / implies / if / foreach / disable_soft)
constraint_prototype_sv_2017   -> {qualifier, static_keyword, name}
constraint_prototype_sv_2023   -> {qualifier, static_keyword, dynamic_override, name}
constraint_prototype_qualifier -> 2 kinds bare (extern / pure)
constraint_set                 -> 2 kinds (single {body} / block {exprs})
```

**Annotation inventory:** 348 entries (was 320). +28 in this batch.

**Schema version:** stays at `1`.

**Contract section:** [`docs/contracts/PGEN_SYSTEMVERILOG_PARSER_INTEGRATION_CONTRACT.md`](../../contracts/PGEN_SYSTEMVERILOG_PARSER_INTEGRATION_CONTRACT.md) → "Release 1.0.29 / Contract 1.0.29 Highlights".

### 1.0.28 / Contract 1.0.28 — SV-Slice-28 batch: class qualifiers typed (3 rules / 6 annotations)

**What changed:** Completes SV-Slice-27's class body picture.

```ebnf
method_qualifier   -> 2 kinds (virtual {pure} / class_item_qualifier {body})
property_qualifier -> 2 kinds (random {body} / class_item_qualifier {body})
random_qualifier   -> 2 kinds (rand / randc — bare {kind})
```

**Annotation inventory:** 320 entries (was 314). +6 in this batch.

**Schema version:** stays at `1`.

**Contract section:** [`docs/contracts/PGEN_SYSTEMVERILOG_PARSER_INTEGRATION_CONTRACT.md`](../../contracts/PGEN_SYSTEMVERILOG_PARSER_INTEGRATION_CONTRACT.md) → "Release 1.0.28 / Contract 1.0.28 Highlights".

### 1.0.27 / Contract 1.0.27 — SV-Slice-27 batch: class body sub-tree typed (6 rules / 30 annotations)

**What changed:** Closes the class body walk path. Method qualifiers, property kind (decl vs const), method kind (task / function / pure_virtual / extern / constructor / extern_constructor) all now `kind`-discriminated.

```ebnf
class_item_sv_2017     -> 8 kinds (property / method / constraint / class / covergroup / local_parameter / parameter / semi)
class_item_sv_2023     -> 9 kinds (same plus interface_class for LRM 2023 nested interface-class decls)
class_item_qualifier   -> 3 kinds (static / protected / local — bare {kind})
class_constraint       -> 2 kinds (prototype / declaration)
class_property         -> 2 kinds (decl {qualifiers, body} / const {qualifiers, data_type, name, init})
class_method           -> 6 kinds (task / function / pure_virtual / extern / constructor / extern_constructor)
```

**Annotation inventory:** 314 entries (was 284). +30 in this batch.

**Schema version:** stays at `1`.

**Contract section:** [`docs/contracts/PGEN_SYSTEMVERILOG_PARSER_INTEGRATION_CONTRACT.md`](../../contracts/PGEN_SYSTEMVERILOG_PARSER_INTEGRATION_CONTRACT.md) → "Release 1.0.27 / Contract 1.0.27 Highlights" with full annotation source + field semantics + profile differences.

### 1.0.26 / Contract 1.0.26 — SV-Slice-26 batch: net_declaration typed via helper-rule extraction (4 rules / 10 annotations + 2 new helper rules)

**What changed:** Closes the net_declaration walk path. Two new helper rules extracted from inline parens-Or to dodge task #38 (same pattern as SV-Slice-23).

```ebnf
net_declaration_sv_2017     -> 3 kinds (wire / alias / interconnect)
net_declaration_sv_2023     -> 3 kinds (alias branch field is `nettype_id` per LRM 2023)
net_strength (NEW)          -> 2 kinds (drive {body} / charge {body})
net_vector_scalar (NEW)     -> 2 kinds (vectored / scalared, bare {kind})
```

**Annotation inventory:** 284 entries (was 274). +10 in this batch.

**Schema version:** stays at `1`.

**Contract section:** [`docs/contracts/PGEN_SYSTEMVERILOG_PARSER_INTEGRATION_CONTRACT.md`](../../contracts/PGEN_SYSTEMVERILOG_PARSER_INTEGRATION_CONTRACT.md) → "Release 1.0.26 / Contract 1.0.26 Highlights" with full annotation source + helper-rule rationale.

### 1.0.25 / Contract 1.0.25 — SV-Slice-25 batch: data/function/task declarations + bodies typed (8 rules / 14 annotations)

**What changed:** Closes the data / function / task walk paths from package_or_generate_item_declaration.

```ebnf
data_declaration_sv_2017     -> 4 kinds (variable_decl / type / package_import / net_type)
data_declaration_sv_2023     -> 4 kinds (same 3 + nettype, LRM 2023 naming)
function_declaration_sv_2017 -> {lifetime, body}
function_declaration_sv_2023 -> {dynamic_override, lifetime, body}
function_body_declaration    -> {return_type, name, items, statements, end_label}
task_declaration_sv_2017     -> {lifetime, body}
task_declaration_sv_2023     -> {dynamic_override, lifetime, body}
task_body_declaration        -> {name, items, statements, end_label}
```

**DEFERRED:** `net_declaration_sv_2017/2023` typing (parens-Or workaround needed; planned for next slice with helper-rule extraction following SV-Slice-23 pattern).

**Annotation inventory:** 274 entries (was 260). +14 in this batch.

**Schema version:** stays at `1`.

**Contract section:** [`docs/contracts/PGEN_SYSTEMVERILOG_PARSER_INTEGRATION_CONTRACT.md`](../../contracts/PGEN_SYSTEMVERILOG_PARSER_INTEGRATION_CONTRACT.md) → "Release 1.0.25 / Contract 1.0.25 Highlights" with full annotation source + field semantics + profile differences (net_type vs nettype).

### 1.0.24 / Contract 1.0.24 — SV-Slice-24 batch: assertion + genvar dispatch typed (7 rules / 26 annotations)

**What changed:** Closes the assertion-item walk path and the loop_generate_construct init/step typed dispatch.

```ebnf
assertion_item              -> 2 kinds (concurrent / deferred_immediate)
assertion_item_declaration  -> 3 kinds (property / sequence / let)
concurrent_assertion_item   -> 2 kinds (statement {label, body} / checker_instantiation {body})
genvar_initialization       -> {genvar_keyword, name, value}
genvar_iteration            -> 3 kinds (assign / prefix_inc_dec / postfix_inc_dec)
assignment_operator         -> 13 kinds (assign / plus_assign / ... / arithmetic_shift_right_assign) — bare {kind}
inc_or_dec_operator         -> 2 kinds (plus_plus / minus_minus) — bare {kind}
```

**Annotation inventory:** 260 entries (was 234). +26 in this batch.

**Schema version:** stays at `1`.

**Contract section:** [`docs/contracts/PGEN_SYSTEMVERILOG_PARSER_INTEGRATION_CONTRACT.md`](../../contracts/PGEN_SYSTEMVERILOG_PARSER_INTEGRATION_CONTRACT.md) → "Release 1.0.24 / Contract 1.0.24 Highlights" with full annotation source + field semantics for each rule.

### 1.0.23 / Contract 1.0.23 — SV-Slice-23 batch: generate-construct internals typed (6 rules / 9 annotations + 1 new helper rule)

**What changed:** Closes the loop / conditional / case-generate dispatch path.

```ebnf
loop_generate_construct        -> {init, condition, step, block}
conditional_generate_construct -> 2 kinds (if / case)
if_generate_construct          -> {condition, then_block, else_clause}
if_generate_else_clause (NEW)  -> 2 kinds (elseif {body} / else_block {body})
case_generate_construct        -> {expr, items: {first, rest}}
case_generate_item             -> 2 kinds (expr_list {exprs: {first, rest}, block} / default {block})
```

**Notable:** New helper rule `if_generate_else_clause` extracted from inline parens-Or to dodge task #38 (parens-grouped-Or trailing-annotation attribution bug). This is now the recommended workaround for similar `( a | b )?` patterns until task #38 is fixed.

**Annotation inventory:** 234 entries (was 225). +9 in this batch.

**Schema version:** stays at `1`.

**Contract section:** [`docs/contracts/PGEN_SYSTEMVERILOG_PARSER_INTEGRATION_CONTRACT.md`](../../contracts/PGEN_SYSTEMVERILOG_PARSER_INTEGRATION_CONTRACT.md) → "Release 1.0.23 / Contract 1.0.23 Highlights" with full annotation source, helper-rule rationale, and full module-item-to-generate-block walker recipe.

### 1.0.22 / Contract 1.0.22 — SV-Slice-22 batch: generate sub-tree typed (3 rules / 7 annotations)

**What changed:** Closes the generate-construct walk path. Every reachable `non_port_module_item.kind=='generate_region'` exposes typed `{items}`, every `generate_item` discriminates which form it carries, and every `generate_block` (anonymous, labeled, or bare-generate_item passthrough) exposes its name/label/items/end_label.

```ebnf
generate_region -> {items}
generate_item   -> 3 kinds: module_or_generate_item / interface_or_generate_item / checker_or_generate_item
generate_block  -> 3 kinds: anonymous {label, items, end_label} / labeled {name, label, items, end_label} / generate_item {body}
```

**Annotation inventory:** 225 entries (was 218). +7 in this batch.

**Schema version:** stays at `1`.

**Contract section:** [`docs/contracts/PGEN_SYSTEMVERILOG_PARSER_INTEGRATION_CONTRACT.md`](../../contracts/PGEN_SYSTEMVERILOG_PARSER_INTEGRATION_CONTRACT.md) → "Release 1.0.22 / Contract 1.0.22 Highlights" with full annotation source + walker recipes for each generate form.

### 1.0.21 / Contract 1.0.21 — SV-Slice-21 batch: module_common_item + package_or_generate_item_declaration typed (4 rules / 55 annotations — biggest batch yet)

**What changed:** Both halves of the cascading walk path that SV-Slice-19/20 set up are now closed: every reachable `module_common_item` and every reachable `package_or_generate_item_declaration` discriminates which actual sub-construct was matched.

```ebnf
module_common_item_sv_2017                    -> 13 kinds (module_or_generate_item_declaration / interface_instantiation / program_instantiation / assertion_item / bind_directive / continuous_assign / net_alias / initial_construct / final_construct / always_construct / loop_generate_construct / conditional_generate_construct / elaboration_system_task)
module_common_item_sv_2023                    -> 13 kinds (same except elaboration_severity_system_task)
package_or_generate_item_declaration_sv_2017  -> 14 kinds (incl. local_parameter_declaration / parameter_declaration / data_declaration / task_declaration / function_declaration / class_declaration / covergroup_declaration / semi / ...)
package_or_generate_item_declaration_sv_2023  -> 15 kinds (same plus interface_class_declaration)
```

The wrapper rules `module_common_item := _sv_2017 | _sv_2023` and `package_or_generate_item_declaration := _sv_2017 | _sv_2023` stay un-annotated (transparent profile-routers, same pattern as `module_declaration` / `interface_declaration`).

**Annotation inventory:** 218 entries (was 163). +55 in this batch — biggest single-slice contribution.

**Schema version:** stays at `1`.

**Contract section:** [`docs/contracts/PGEN_SYSTEMVERILOG_PARSER_INTEGRATION_CONTRACT.md`](../../contracts/PGEN_SYSTEMVERILOG_PARSER_INTEGRATION_CONTRACT.md) → "Release 1.0.21 / Contract 1.0.21 Highlights" with full annotation source + 6-level walker recipe.

### 1.0.20 / Contract 1.0.20 — SV-Slice-20 batch: interface + program items dispatch tree typed (5 rules / 19 annotations)

**What changed:** Mirror of SV-Slice-19's module-items batch, applied to the interface and program sub-trees. Every `header.items` and `body.items` field on every typed interface/program declaration now surfaces kind-discriminated dispatch.

```ebnf
interface_item              -> 2 kinds: port_declaration / non_port_item
interface_or_generate_item  -> 2 kinds (with attributes): module_common_item / extern_tf_declaration
non_port_interface_item     -> 6 kinds: generate_region / interface_or_generate / program_declaration / modport_declaration / interface_declaration / timeunits_declaration
program_item                -> 2 kinds: port_declaration / non_port_item
non_port_program_item       -> 7 kinds: continuous_assign / module_or_generate_item_declaration / initial_construct / final_construct / concurrent_assertion_item / timeunits_declaration / program_generate_item (first 5 with attributes)
```

**Annotation inventory:** 163 entries (was 144). +19 in this batch.

**Schema version:** stays at `1`.

**Contract section:** [`docs/contracts/PGEN_SYSTEMVERILOG_PARSER_INTEGRATION_CONTRACT.md`](../../contracts/PGEN_SYSTEMVERILOG_PARSER_INTEGRATION_CONTRACT.md) → "Release 1.0.20 / Contract 1.0.20 Highlights" with full annotation source + interface/program walker recipes.

### 1.0.19 / Contract 1.0.19 — SV-Slice-19 batch: module-items dispatch tree typed (5 rules / 22 annotations)

**What changed:** Largest batch yet. Every `header.items` and `body.items` field on every typed module/interface/program declaration now surfaces kind-discriminated dispatch.

```ebnf
module_item                          -> 2 kinds: port_declaration / non_port_item
module_or_generate_item              -> 5 kinds (with attributes:): parameter_override / gate_instantiation / udp_instantiation / module_instantiation / module_common_item
module_or_generate_item_declaration  -> 5 kinds: package_or_generate / genvar / clocking / default_clocking / default_disable_iff
non_port_module_item                 -> 8 kinds: generate_region / module_or_generate / specify_block / specparam_declaration / program_declaration / module_declaration / interface_declaration / timeunits_declaration
continuous_assign                    -> 2 kinds: net / variable
```

**Annotation inventory:** 144 entries (was 122). +22 in this batch — largest single-slice contribution to date.

**Schema version:** stays at `1`.

**Contract section:** [`docs/contracts/PGEN_SYSTEMVERILOG_PARSER_INTEGRATION_CONTRACT.md`](../../contracts/PGEN_SYSTEMVERILOG_PARSER_INTEGRATION_CONTRACT.md) → "Release 1.0.19 / Contract 1.0.19 Highlights" with full annotation source + 5-layer module-items walker recipe.

### 1.0.18 / Contract 1.0.18 — SV-Slice-18 batch: UDP truth-table entries typed

**What changed:** 3 rules / 3 annotations completing the UDP truth-table walk path.

```ebnf
combinational_entry   -> {inputs, output}
sequential_entry      -> {inputs, current_state, next_state}
udp_initial_statement -> {name, init_val}
```

**Annotation inventory:** 122 entries (was 119). +3.

**Schema version:** stays at `1`.

**Contract section:** [`docs/contracts/PGEN_SYSTEMVERILOG_PARSER_INTEGRATION_CONTRACT.md`](../../contracts/PGEN_SYSTEMVERILOG_PARSER_INTEGRATION_CONTRACT.md) → "Release 1.0.18 / Contract 1.0.18 Highlights".

### 1.0.17 / Contract 1.0.17 — SV-Slice-17 batch: UDP body sub-tree typed

**What changed:** 6 rules / 8 annotations completing UDP declaration internals.

```ebnf
udp_body                     -> 2 kinds: combinational/sequential
udp_input_declaration        -> {attributes, identifiers}
udp_output_declaration       -> 2 kinds: wire/reg
combinational_body           -> {entries: {first, rest}}
sequential_body              -> {initial, entries: {first, rest}}
list_of_udp_port_identifiers -> {first, rest}
```

**UDP declaration internals fully typed end-to-end** — combined with prior UDP top-level rules (SV-Slice-12) and port lists (SV-Slice-15), consumers walking a `primitive ... endprimitive` get clean typed access at every level.

**Annotation inventory:** 119 entries (was 111). +8 in this batch.

**Schema version:** stays at `1`.

**Contract section:** [`docs/contracts/PGEN_SYSTEMVERILOG_PARSER_INTEGRATION_CONTRACT.md`](../../contracts/PGEN_SYSTEMVERILOG_PARSER_INTEGRATION_CONTRACT.md) → "Release 1.0.17 / Contract 1.0.17 Highlights" with full UDP walker recipe.

### 1.0.16 / Contract 1.0.16 — SV-Slice-16 batch: port + port_direction + package_import family typed

**What changed:** 4 rules / 9 annotations.

```ebnf
port                       -> 2 kinds: expression / named (dot-form)
port_direction             -> 4 kinds: input / output / inout / ref
package_import_declaration -> {items: {first, rest}}
package_import_item        -> 2 kinds: explicit / wildcard
```

**DEFERRED:** `ansi_port_declaration` per-branch typing — branch 0 starts with a parens-grouped Or `( net_port_header | interface_port_header )?` which triggers task #38's branch-attribution bug. Tracked as follow-up either via task #38 fix or grammar refactor.

**Annotation inventory:** 111 entries (was 102). +9 in this batch.

**Schema version:** stays at `1`.

**Contract section:** [`docs/contracts/PGEN_SYSTEMVERILOG_PARSER_INTEGRATION_CONTRACT.md`](../../contracts/PGEN_SYSTEMVERILOG_PARSER_INTEGRATION_CONTRACT.md) → "Release 1.0.16 / Contract 1.0.16 Highlights".

### 1.0.15 / Contract 1.0.15 — SV-Slice-15 batch: port-list family + small structural rules typed

**What changed:** 6 rules / 7 annotations. Every `header.ports` field on every typed module/interface/program/UDP declaration now surfaces a typed shape.

```ebnf
list_of_ports             -> {first, rest}     (mini-mixed-array)
list_of_port_declarations -> $2 (transparent passthrough of inner optional)
udp_port_list             -> {output, inputs: {first, rest}}
udp_declaration_port_list -> {output, inputs: {first, rest}}
anonymous_program         -> {items}
package_export_declaration -> 2 kinds (wildcard / explicit with {first, rest})
```

**Annotation inventory:** 102 entries (was 95). +7 in this batch. **Crossing 100 annotations** — campaign mid-flight.

**Schema version:** stays at `1`.

**Contract section:** [`docs/contracts/PGEN_SYSTEMVERILOG_PARSER_INTEGRATION_CONTRACT.md`](../../contracts/PGEN_SYSTEMVERILOG_PARSER_INTEGRATION_CONTRACT.md) → "Release 1.0.15 / Contract 1.0.15 Highlights" with full annotation source + per-rule notes.

### 1.0.14 / Contract 1.0.14 — SV-Slice-14 batch: bind sub-tree completion + interface_class_declaration + config_declaration

**What changed:** 5 rules typed in one batch:

```ebnf
bind_target_scope          -> 2 kinds: module/interface ({kind, name})
bind_target_instance       -> {name, bit_select}
bind_target_instance_list  -> {first, rest} (mini-mixed-array workaround)
interface_class_declaration -> {name, parameters, extends, items, end_label}
config_declaration         -> {name, local_params, design, rules, end_label}
```

**Bind sub-tree fully typed** — combined with SV-Slice-13's bind_directive/bind_instantiation typing, consumers walking a bind directive get clean typed access at every level (target_scope, target_instance, instances, instantiation).

**Annotation inventory:** 95 entries (was 89). +6 in this batch.

**Schema version:** stays at `1`.

**Contract section:** [`docs/contracts/PGEN_SYSTEMVERILOG_PARSER_INTEGRATION_CONTRACT.md`](../../contracts/PGEN_SYSTEMVERILOG_PARSER_INTEGRATION_CONTRACT.md) → "Release 1.0.14 / Contract 1.0.14 Highlights" with full annotation source + bind walker recipe.

### 1.0.13 / Contract 1.0.13 — SV-Slice-13 batch: bind_directive + bind_instantiation + package_item per-branch typed

**What changed:** 3 Or rules typed. Consumers gain clean kind dispatch on description's `package_item` and `bind_directive` branches.

```ebnf
bind_directive       -> 2 kinds: scoped/single
bind_instantiation   -> 4 kinds: program/module/interface/checker
package_item         -> 4 kinds: declaration/anonymous_program/export/timeunits
```

**Annotation inventory:** 89 entries (was 79). +10 in this batch.

**Schema version:** stays at `1`.

**Contract section:** [`docs/contracts/PGEN_SYSTEMVERILOG_PARSER_INTEGRATION_CONTRACT.md`](../../contracts/PGEN_SYSTEMVERILOG_PARSER_INTEGRATION_CONTRACT.md) → "Release 1.0.13 / Contract 1.0.13 Highlights" with full annotation source + consumer dispatch recipe.

### 1.0.12 / Contract 1.0.12 — SV-Slice-12 batch: UDP declaration family typed

**What changed:** 4 rules typed across the UDP (User-Defined Primitive) declaration family — sibling pattern to module/interface/program with one twist: `udp_declaration_sv_*` nonansi branch has a `udp_port_declaration udp_port_declaration*` mini-mixed-array, handled with the `{first, rest}` workaround.

```ebnf
udp_ansi_declaration     -> {attributes, name, ports}
udp_nonansi_declaration  -> {attributes, name, ports}
udp_declaration_sv_2017  -> 5 per-branch kinds: nonansi/ansi/extern_nonansi/extern_ansi/wildcard
udp_declaration_sv_2023  -> mirror of sv_2017 with positional shift in wildcard
```

**Mini-mixed-array workaround:** the nonansi branch's `udp_port_declaration udp_port_declaration*` pattern uses `port_decls: {first: $2, rest: $3}` to surface the required-first + repeat shape. Same idiom as `attribute_instance: {first, rest}` from SV-Slice-6. Consumers iterate `port_decls.first` once then walk `port_decls.rest`.

**Annotation inventory:** 79 entries (was 67). +12 in this batch.

**Schema version:** stays at `1`.

**Contract section:** [`docs/contracts/PGEN_SYSTEMVERILOG_PARSER_INTEGRATION_CONTRACT.md`](../../contracts/PGEN_SYSTEMVERILOG_PARSER_INTEGRATION_CONTRACT.md) → "Release 1.0.12 / Contract 1.0.12 Highlights" — full annotation source + consumer dispatch recipe + mini-mixed-array workaround documentation.

### 1.0.11 / Contract 1.0.11 — SV-Slice-11 batch: program-header sub-tree typed (sibling of module/interface headers)

**What changed:** 2 rules typed: `program_ansi_header`, `program_nonansi_header`. Both expose the same 6-field shape: `attributes`, `lifetime`, `name`, `imports`, `parameters`, `ports`. Same field names as module / interface header pairs (program is sans `keyword:` since it only has one keyword).

**Verified on `program p; endprogram\n`:** `header.name = "p"` (clean string from SV-Slice-8), all 6 fields present.

**Annotation inventory:** 67 entries (was 65). +2 in this batch.

**Schema version:** stays at `1`.

**Contract section:** [`docs/contracts/PGEN_SYSTEMVERILOG_PARSER_INTEGRATION_CONTRACT.md`](../../contracts/PGEN_SYSTEMVERILOG_PARSER_INTEGRATION_CONTRACT.md) → "Release 1.0.11 / Contract 1.0.11 Highlights".

### 1.0.10 / Contract 1.0.10 — SV-Slice-10 batch: class + package + program declarations typed

**What changed:** 5 rules typed: `class_declaration_sv_2017` and `class_declaration_sv_2023` (single-sequence shapes; sv_2017 has `lifetime:`, sv_2023 has `final_specifier:` per LRM-2023 semantics), `package_declaration` (single sequence with attribute_instance* prefix), `program_declaration_sv_2017` and `program_declaration_sv_2023` (5 per-branch kinds each, mirroring module/interface).

**Verified empirically on `program p; endprogram\n`:**

```text
source_text[0]: {kind: "description", body: {
    kind: "program_declaration",
    body: {kind: "ansi", header: {...}, timeunits: [], items: [], end_label: []}
}}
```

**Module/interface/program tests still pass** with the same regenerated parser — annotations didn't introduce regressions.

**Open follow-up:** `package p; endpackage\n` parse rejected at position 0 despite `package_declaration` being in `description`'s Or set. Annotation registered correctly per the inventory; runtime parse failure appears pre-existing. Module/interface/program parsing unaffected. Tracking separately.

**Class top-level parse:** `class C; endclass\n` is also rejected — but this is expected, since class_declaration isn't directly in source_text_item's reachable set; class declarations are reached through `package_item` or other subsidiary contexts.

**Annotation inventory:** 65 entries (was 53). +12 in this batch.

**Schema version:** stays at `1`.

**Contract section:** [`docs/contracts/PGEN_SYSTEMVERILOG_PARSER_INTEGRATION_CONTRACT.md`](../../contracts/PGEN_SYSTEMVERILOG_PARSER_INTEGRATION_CONTRACT.md) → "Release 1.0.10 / Contract 1.0.10 Highlights".

### 1.0.9 / Contract 1.0.9 — SV-Slice-9 batch: interface declarations typed (full mirror of module pattern)

**What changed:** 4 rules typed: `interface_ansi_header`, `interface_nonansi_header`, `interface_declaration_sv_2017` (5 per-branch kinds), `interface_declaration_sv_2023` (same 5 kinds with positional shift). Interface declarations now have the same typed surface as module declarations. 4-layer typed dispatch end-to-end + clean identifier strings.

**Empirical for `interface bus; endinterface\n`:**

```text
source_text[0]: {kind: "description", body: {
    kind: "interface_declaration",
    body: {
        kind: "ansi",
        header: {name: "bus", attributes: [], lifetime: [], imports: [], parameters: [], ports: []},
        timeunits: [], items: [], end_label: []
    }
}}
```

**Difference from module pattern:** No `keyword:` field on interface_<form>_header (only one `interface` keyword exists). Otherwise field names mirror `module_<form>_header`.

**Annotation inventory:** 53 entries (was 41). +12 in this batch.

**Schema version:** stays at `1`.

**Contract section:** [`docs/contracts/PGEN_SYSTEMVERILOG_PARSER_INTEGRATION_CONTRACT.md`](../../contracts/PGEN_SYSTEMVERILOG_PARSER_INTEGRATION_CONTRACT.md) → "Release 1.0.9 / Contract 1.0.9 Highlights".

### 1.0.8 / Contract 1.0.8 — SV-Slice-8 batch: identifier-leaf rules typed (clean strings propagate through every identifier field)

**What changed:** 4 identifier-leaf rules typed with `-> $2` transparent passthrough. Highest-leverage slice yet — every rule that resolves to an identifier now surfaces a clean JSON string instead of the raw envelope chain.

```ebnf
simple_identifier          := trivia /[a-zA-Z_][a-zA-Z0-9_$]*/                            -> $2
escaped_identifier         := trivia /\\[!-~]+/                                            -> $2
non_keyword_identifier     := !reserved_non_keyword_identifier identifier                  -> $2
simple_identifier_no_scope := trivia /[a-zA-Z_][a-zA-Z0-9_$]*(?![ \t\r\n]*::)/             -> $2
```

**Empirical pre/post for `module m; endmodule\n`:**

```text
# Pre — header.name was raw envelope chain:
"header": {"keyword": {"kind": "module"}, "name": [[], [[], "m"]], ...}

# Post — clean string:
"header": {"keyword": {"kind": "module"}, "name": "m", ...}
```

**Propagation:** `simple_identifier` / `escaped_identifier` are leaves of `identifier` (transparent Or). `non_keyword_identifier` strips the negative lookahead. `declaration_identifier` / `module_identifier` / `class_identifier` / `package_identifier` / `interface_identifier` etc. are all transparent aliases — they automatically surface clean strings now. Every future-typed rule that exposes an identifier as a named field gets a clean string for free.

**Annotation inventory:** 41 entries (was 37). +4 in this batch.

**Schema version:** stays at `1`.

**Contract section:** [`docs/contracts/PGEN_SYSTEMVERILOG_PARSER_INTEGRATION_CONTRACT.md`](../../contracts/PGEN_SYSTEMVERILOG_PARSER_INTEGRATION_CONTRACT.md) → "Release 1.0.8 / Contract 1.0.8 Highlights".

### 1.0.7 / Contract 1.0.7 — SV-Slice-7 batch: `module_keyword` + `lifetime` + `module_ansi_header` + `module_nonansi_header` typed (4 layers of dispatch end-to-end)

**What changed:** 4-rule batch slice typing the header sub-tree of module declarations. Four layers of typed dispatch are now end-to-end.

```ebnf
module_keyword         := kw_module      -> {kind: "module"}
                        | kw_macromodule -> {kind: "macromodule"}

lifetime               := kw_static      -> {kind: "static"}
                        | kw_automatic   -> {kind: "automatic"}

module_ansi_header     := attribute_instance* module_keyword (lifetime)? module_identifier package_import_declaration* (parameter_port_list)? (list_of_port_declarations)? semi
                       -> {attributes: $1, keyword: $2, lifetime: $3, name: $4, imports: $5, parameters: $6, ports: $7}

module_nonansi_header  := attribute_instance* module_keyword (lifetime)? module_identifier package_import_declaration* (parameter_port_list)? list_of_ports semi
                       -> {attributes: $1, keyword: $2, lifetime: $3, name: $4, imports: $5, parameters: $6, ports: $7}
```

**Empirical for `module m; endmodule\n`:** the previously-raw `header:` field of the ansi-kind module_declaration_sv_2017 now resolves to a typed object with `keyword: {kind: "module"}` and named fields for all 7 components. ANSI and non-ANSI forms expose the same field names — consumer code walking the header is uniform across both.

**Annotation inventory:** 37 entries (was 31). +6 in this batch (2 module_keyword + 2 lifetime + 1 module_ansi_header + 1 module_nonansi_header).

**Schema version:** stays at `1`.

**Contract section:** [`docs/contracts/PGEN_SYSTEMVERILOG_PARSER_INTEGRATION_CONTRACT.md`](../../contracts/PGEN_SYSTEMVERILOG_PARSER_INTEGRATION_CONTRACT.md) → "Release 1.0.7 / Contract 1.0.7 Highlights" — has the per-rule annotation source code + 4-layer consumer dispatch recipe.

### 1.0.6 / Contract 1.0.6 — SV-Slice-6 batch: `attribute_instance` + `module_declaration_sv_2017/2023` typed (3 layers of dispatch end-to-end)

**What changed:** Multi-rule batch slice. Three rules typed in one pass: `attribute_instance` (`{first, rest}` shape), `module_declaration_sv_2017` (5 per-branch kind labels: ansi/nonansi/wildcard/extern_nonansi/extern_ansi), `module_declaration_sv_2023` (same kind labels as sv_2017; wildcard branch's positional indices shift to accommodate `dot star` vs `dot_star`).

**Three layers of typed dispatch end-to-end** — `source_text_item.kind` (SV-Slice-3) → `description.kind` (SV-Slice-4) → `module_declaration_sv_<profile>.kind` (this slice). For `module m; endmodule\n`:

```json
{
  "type": "systemverilog_file",
  "source_text": [
    {
      "kind": "description",
      "body": {
        "kind": "module_declaration",
        "body": {
          "kind": "ansi",
          "header": [<module_ansi_header envelope>],
          "timeunits": [],
          "items": [],
          "end_label": []
        }
      }
    }
  ]
}
```

**Annotation inventory:** 31 entries (was 20). +11 in this batch.

**`comment_only_source_region` typing was attempted in this batch but DEFERRED** — blocked by task #38 (parens-grouped-Or trailing-annotation attribution bug). The rule's two `( a | b )` parens-grouped Or expressions cause the trailing `-> ...` annotation to fail to register on the rule. Annotation reverted; this rule's typing is gated on task #38's resolution OR a grammar refactor that flattens the parens-grouped Ors into named helper rules.

**Schema version:** stays at `1`.

**Contract section:** [`docs/contracts/PGEN_SYSTEMVERILOG_PARSER_INTEGRATION_CONTRACT.md`](../../contracts/PGEN_SYSTEMVERILOG_PARSER_INTEGRATION_CONTRACT.md) → "Release 1.0.6 / Contract 1.0.6 Highlights" — has the per-rule annotation source code + consumer dispatch recipe.

### 1.0.5 / Contract 1.0.5 — SV-Slice-5: `compiler_directive` transparent passthrough (clean directive text)

**What changed:** `compiler_directive := trivia /` `` `[^\r\n]*/`` `` `(line 226 of `grammars/systemverilog.ebnf`) annotated with `-> $2`. Drops the leading `trivia` slot and emits just the matched directive text as a clean JSON string. Consumer code receives a directly-usable string for `source_text_item.body` when `source_text_item.kind == "compiler_directive"`.

**Empirical pre/post for an input with `` `define FOO bar `` + `module m; endmodule\n`:**

```text
# Pre — body was raw envelope of `trivia regex_capture`:
"source_text": [
  {"kind": "compiler_directive", "body": [<trivia envelope>, "`define FOO bar"]}
]

# Post — body is the clean directive string:
"source_text": [
  {"kind": "compiler_directive", "body": "`define FOO bar"}
]
```

**Annotation inventory:** 20 entries (was 19). New: `compiler_directive`.

**Heterogeneous body shape per `source_text_item.kind`** — when kind is `"description"`, body is a typed object; when kind is `"compiler_directive"`, body is a string. Same pattern regex AST uses for typed atoms.

**Schema version:** stays at `1`.

**Contract section:** [`docs/contracts/PGEN_SYSTEMVERILOG_PARSER_INTEGRATION_CONTRACT.md`](../../contracts/PGEN_SYSTEMVERILOG_PARSER_INTEGRATION_CONTRACT.md) → "Release 1.0.5 / Contract 1.0.5 Highlights".

### 1.0.4 / Contract 1.0.4 — SV-Slice-4: `description` per-branch typed (`kind:` discriminator on 8 branches; `attribute_instance*` preserved)

**What changed:** 8 per-branch annotations on `description` (line 957 of `grammars/systemverilog.ebnf`). Every Or branch now emits a typed object with a `kind:` discriminator. The two multi-element branches (`attribute_instance* package_item` / `attribute_instance* bind_directive`) preserve the leading attribute_instance* prefix as a separate `attributes:` field while keeping the inner construct as `body:`.

```ebnf
description := module_declaration                 -> {kind: "module_declaration", body: $1}
             | udp_declaration                    -> {kind: "udp_declaration", body: $1}
             | interface_declaration              -> {kind: "interface_declaration", body: $1}
             | program_declaration                -> {kind: "program_declaration", body: $1}
             | package_declaration                -> {kind: "package_declaration", body: $1}
             | attribute_instance* package_item   -> {kind: "package_item", attributes: $1, body: $2}
             | attribute_instance* bind_directive -> {kind: "bind_directive", attributes: $1, body: $2}
             | config_declaration                 -> {kind: "config_declaration", body: $1}
```

**Two layers of typed dispatch end-to-end** — `source_text_item.kind` (SV-Slice-3) routes to which top-level slot the item came from; `description.kind` (this slice) routes to which specific construct when the outer kind is `"description"`.

**Empirical pre/post for `module m; endmodule\n`:**

```text
# Pre — body field of the description-kind source_text_item was raw envelope:
"source_text": [
  {"kind": "description", "body": [<description Or-of-8 raw envelope>]}
]

# Post — body is itself a typed object with its own kind discriminator:
"source_text": [
  {
    "kind": "description",
    "body": {
      "kind": "module_declaration",
      "body": [<module_declaration envelope>]
    }
  }
]
```

**Consumer dispatch unlocked at the description level:**

```rust
for item in obj["source_text"].as_array().unwrap() {
    if item["kind"] == "description" {
        let desc = &item["body"];
        match desc["kind"].as_str().unwrap() {
            "module_declaration"    => walk_module(&desc["body"]),
            "udp_declaration"       => walk_udp(&desc["body"]),
            "interface_declaration" => walk_interface(&desc["body"]),
            "program_declaration"   => walk_program(&desc["body"]),
            "package_declaration"   => walk_package(&desc["body"]),
            "package_item"          => walk_package_item(&desc["attributes"], &desc["body"]),
            "bind_directive"        => walk_bind_directive(&desc["attributes"], &desc["body"]),
            "config_declaration"    => walk_config(&desc["body"]),
            other => panic!("unknown description kind: {}", other),
        }
    }
}
```

**Annotation inventory:** 19 entries (was 11). 8 new per-branch entries on `description`.

**Inner `module_declaration`, `udp_declaration`, etc. shapes still raw envelope** — per-rule typing of those is a follow-up slice.

**Schema version:** stays at `1` (additive discriminator).

**Contract section:** [`docs/contracts/PGEN_SYSTEMVERILOG_PARSER_INTEGRATION_CONTRACT.md`](../../contracts/PGEN_SYSTEMVERILOG_PARSER_INTEGRATION_CONTRACT.md) → "Release 1.0.4 / Contract 1.0.4 Highlights".

### 1.0.3 / Contract 1.0.3 — SV-Slice-3: `source_text_item` per-branch typed (`kind:` discriminator)

**What changed:** 8 per-branch annotations on `source_text_item` (lines 210-217 of `grammars/systemverilog.ebnf`). Every Or branch now emits a typed object with a `kind:` discriminator: `"description"`, `"local_parameter_declaration"`, `"parameter_declaration"`, `"package_import_declaration"`, `"timeunits_declaration"`, `"compiler_directive"`, `"comment_only_source_region"`, `"semi"`.

**Empirical pre/post for `module m; endmodule\n`:**

```text
# Pre — source_text[0] was the matched-branch shape directly:
"source_text": [
  [<description envelope>]
]

# Post — source_text[0] is a typed object with discriminator:
"source_text": [
  {"kind": "description", "body": [<description envelope>]}
]
```

**Consumer dispatch pattern:**

```rust
for item in obj["source_text"].as_array().unwrap() {
    match item["kind"].as_str().unwrap() {
        "description" => walk_description(&item["body"]),
        "local_parameter_declaration" => walk_local_param(&item["body"]),
        "parameter_declaration" => walk_param(&item["body"]),
        "package_import_declaration" => walk_package_import(&item["body"]),
        "timeunits_declaration" => walk_timeunits(&item["body"]),
        "compiler_directive" => walk_compiler_directive(&item["body"]),
        "comment_only_source_region" => walk_comment_region(&item["body"]),
        "semi" => { /* stray ; — nothing to walk */ }
        other => panic!("unknown source_text_item kind: {}", other),
    }
}
```

**Annotation inventory:** 11 entries (was 3). 8 new per-branch entries on `source_text_item`.

**Trailing `semi` dropped** in the `local_parameter_declaration semi` and `parameter_declaration semi` branches — annotations reference `$1` only.

**`@branch_policy: priority_first` and `@priority` preserved** in the rule definition.

**Schema version:** stays at `1` (additive discriminator).

**Contract section:** [`docs/contracts/PGEN_SYSTEMVERILOG_PARSER_INTEGRATION_CONTRACT.md`](../../contracts/PGEN_SYSTEMVERILOG_PARSER_INTEGRATION_CONTRACT.md) → "Release 1.0.3 / Contract 1.0.3 Highlights".

### 1.0.2 / Contract 1.0.2 — SV-Slice-2: `source_text` flatten-spread

**What changed:** `grammars/systemverilog.ebnf` line 2273's `source_text := source_text_item*` rule annotated `-> [$1**]`. The `source_text` field of `systemverilog_file` is now a flat array of `source_text_item` shapes (was a Quantified envelope).

**Empirical pre/post for `module m; endmodule\n`:**

```text
# Pre — source_text was nested Quantified envelope:
{
  "type": "systemverilog_file",
  "source_text": [<Quantified iteration wrap>]
}

# Post — source_text is a flat array (length 1 for minimal_module):
{
  "type": "systemverilog_file",
  "source_text": [<source_text_item shape>]
}
```

**Annotation inventory:** 3 entries (was 2). New: `source_text`.

**Annotation idiom:** `[$1**]` is the canonical flatten-spread form (same as regex.ebnf's `concatenation = piece+ -> [$1**]`). Verified to work for the SV grammar's first array-shaped rule.

**Schema version:** stays at `1` (additive — flat-array shape is a clean-up of the raw envelope).

**Contract section:** [`docs/contracts/PGEN_SYSTEMVERILOG_PARSER_INTEGRATION_CONTRACT.md`](../../contracts/PGEN_SYSTEMVERILOG_PARSER_INTEGRATION_CONTRACT.md) → "Release 1.0.2 / Contract 1.0.2 Highlights".

### 1.0.1 / Contract 1.0.1 — SV-Slice-1: `systemverilog_file` typed (dangling annotation rescued)

**What changed:** `grammars/systemverilog.ebnf` line 184's `systemverilog_file` rule now carries its return annotation on the same multi-line definition (was dangling between the `sv_multi_entry_root` helper rule and `systemverilog_parseable_file`). The annotation `-> {type: "systemverilog_file", source_text: $2}` now correctly latches onto `systemverilog_file`. Same slice removed the `//` prefix from `systemverilog_parseable_file`'s annotation (PGEN's EBNF dialect uses `#` for comments, not `//`, so the `//` prefix was misleading rather than effective).

**Empirical pre/post for `module m; endmodule\n`:**

```text
# Pre-SV-Slice-1 — recursive envelope:
{"content": {"Sequence": [
    {"content": {"Alternative": ...}, "rule_name": "element_0", ...},
    {"content": {"Alternative": ...}, "rule_name": "element_1", ...},
    ...
]}}

# Post-SV-Slice-1 — typed object at root:
{"content": {"Json": {
    "type": "systemverilog_file",
    "source_text": [...]
}}}
```

**Annotation inventory** (from `ast_pipeline`'s reporting): 2 entries (was 1). New: `systemverilog_file`. Existing: `systemverilog_parseable_file` (was already registered via the misleading `//` prefix; now registered via the documented path).

**Manifest update:** `rust/test_data/ast_shape_contract/systemverilog_v1.json` `current_content_kind` updated from placeholder `"sequence"` to calibrated `"json_object"`. Drift status flipped to `calibrated_2026_05_04`. Layout note about line 195 dangling annotation removed (resolved). Calibration history block added.

**Schema version:** stays at `1` (additive shape change within major version 1).

**Contract section:** [`docs/contracts/PGEN_SYSTEMVERILOG_PARSER_INTEGRATION_CONTRACT.md`](../../contracts/PGEN_SYSTEMVERILOG_PARSER_INTEGRATION_CONTRACT.md) → "Release 1.0.1 / Contract 1.0.1 Highlights".

### 1.0.0 / Contract 1.0.0 — Foundation baseline (mdbook + contract Highlights structure)

**What changed:** Initial systemverilog mdBook scaffolded at `docs/systemverilog_parser_book/`. The integration contract `docs/contracts/PGEN_SYSTEMVERILOG_PARSER_INTEGRATION_CONTRACT.md` was upgraded from a thin "stable surface" pointer to the same release-tracked Highlights structure used by the regex parser contract.

**Mdbook chapters landed:** welcome, quickstart, build-recipe, public-api, ast-envelope, parse-content-variants, json-carrier, walking-the-ast, rules-top-level, examples-minimal-module, schema-versioning, glossary, changelog-index. Per-rule and per-feature example chapters land as the annotation campaign progresses.

**Contract section:** [`docs/contracts/PGEN_SYSTEMVERILOG_PARSER_INTEGRATION_CONTRACT.md`](../../contracts/PGEN_SYSTEMVERILOG_PARSER_INTEGRATION_CONTRACT.md) → "Release 1.0.0 / Contract 1.0.0 Highlights".

**Build status:** Generated SV parser is **NOT in default `cargo test` build** — produced on-demand by `sv_stimuli_quality_gate`. See [Build Recipe](build-recipe.md).

**Annotation campaign:** Not yet started. `grammars/systemverilog.ebnf` is un-annotated except for one commented-out trial annotation at line 200. First slice will land in a follow-up commit.

**Schema baseline:** `1` (corresponds to `version: 1` in `rust/test_data/ast_shape_contract/systemverilog_v1.json`).

**Public API surface:** Unchanged. See [Public API Surface](public-api.md).

**Bug ledger:** No SV-NNNN entries blocking the baseline.

## How to track per-slice changes

Each annotation slice gets:

1. A grammar change in `grammars/systemverilog.ebnf` (the `-> ...` annotation).
2. A manifest update in `rust/test_data/ast_shape_contract/systemverilog_v1.json`.
3. A parser-release / contract-version bump in `docs/contracts/PGEN_SYSTEMVERILOG_PARSER_INTEGRATION_CONTRACT.md`.
4. A row in [Schema Versioning](schema-versioning.md) tagging the new schema version.
5. An entry in this changelog index summarising the slice.
6. A regression-lock test in `rust/src/embedding_api.rs` (or related test module) pinning the typed shape.

Per-slice commits should bundle all six (the live-book policy). See `docs/contracts/PGEN_REGEX_PARSER_INTEGRATION_CONTRACT.md` for an example of a mature contract with 50+ Highlights sections to mirror.

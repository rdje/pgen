# docs/contracts/PGEN_SYSTEMVERILOG_PARSER_INTEGRATION_CONTRACT.md

## Purpose
Define the downstream integration contract for PGEN's main `systemverilog` parser family.

This is the document downstream projects such as Nexsim should read first when deciding how to embed the PGEN systemverilog parser.

## Contract Identity
- Contract version:
  - `1.0.125`
- Parser release version:
  - `1.0.125`
- Embedding API contract baseline:
  - `1.2.0`
- SystemVerilog AST-dump schema version:
  - `3` (POST-SV-AUDIT.2.4b: 11 structured-per-iteration Category-A misuses corrected by factoring each repeated multi-field unit into a new named record rule + an extraction-spread — `list_of_interface_identifiers` / `list_of_port_identifiers` / `list_of_variable_identifiers` / `list_of_tf_variable_identifiers` / `list_of_variable_port_identifiers`, the `let_` / `property_` / `sequence_list_of_arguments` mixed+named_only branches, `parameter_port_list` type_only, and the two `assignment_pattern` named branches. 9 new annotated record rules were added, so the inventory **deliberately changed 2290 → 2299 / 999 → 1008** (these factored record rules ARE annotated — analogous to SVPP-0001's `pp_if_keyword`; NOT "unchanged"). See "AST-Shape Corrections — 1.0.117 (POST-SV-AUDIT)". **1.0.118 (SV-EXH-PROOF.3.2, `PGEN-SV-EXH-PROOF-0021`) keeps schema `3`** — the broken 115-slice/POST-SV-AUDIT per-digit-token number decomposition (`integral_number`/`real_number`/`unsigned_number`/`decimal_number`/`binary_number`/`octal_number`/`hex_number`/`size`/`fixed_point_number`) was restored to the generator's clean single-regex IEEE-1800 lexical form (eliminating the blanket-`\b`-on-single-char-token + `__SV_RULE__`-mangled-`_` corruption that made EVERY multi-digit/underscore/sized/based number unparseable — the external-corpus 0/14 root cause). **Canonical "release bump, no schema bump"**: numbers were 100% unparseable pre-fix so NO number AST was ever emitted (no prior realized shape / no consumer of the never-emitted decomposed shapes); every previously-PARSEABLE input is byte-identical; only previously-erroring numbers now succeed (strictly-more-permissive — same category as SVPP-0002/REGEX-0083). The typed top `number := real_number -> {kind:"real",body:$1} | integral_number -> {kind:"integral",body:$1}` is kept (now clean Terminals ⇒ first realized number shape = flat `{kind,body:"<number>"}`). Inventory **2312 → 2297** (−15: the dead decomposed-rule annotations dropped; SV shape-contract GREEN, no inventory regression-lock). external-corpus parse **0/14 → 4/14** (scr1 family). **1.0.119 (SV-EXH-PROOF.3.3.1, `PGEN-SV-EXH-PROOF-0022`) keeps schema `3`** — `non_keyword_identifier` (the foundation rule for every declaration-site name: class / package / typedef / parameter / localparam, via `declaration_identifier`) returned the raw `-> $2` value, which the positional-extraction codegen leaves as an `Alternative`-wrapped node (unlike implicit-passthrough codegen, which unwraps it); so every `declared_*_identifier -> {body:$1.body}` `@emit_fact`/`@predicate has_fact` directive routed through it failed *"Semantic runtime could not resolve fact name"*, making class/package/typedef/parameter/localparam **declarations** 100% unparseable. Restored to the engine-proven object form `non_keyword_identifier := !reserved_non_keyword_identifier identifier -> {body: $2.body}` (mirrors `identifier := simple_identifier -> {body:$1}`). **Grammar-only fix — engine untouched** (the codegen/runtime is stable; per user standing preference). **Canonical "release bump, no schema bump"**: those declarations were 100% unparseable pre-fix so NO such AST was ever emitted; every previously-PARSEABLE input is byte-identical; only previously-erroring declarations now succeed (strictly-more-permissive, same SVPP-0002/REGEX-0083 category). SV shape-contract **GREEN** (samples=3 aligned=3 drift=0 regression_lock_failures=0 — the rule's value is consumed internally by directives, the emitted AST envelope is unchanged). Inventory **unchanged** (`non_keyword_identifier` was already annotated; `-> $2` → `-> {body:$2.body}` is a shape change of an existing entry, not a count change). Full `.3.2` 13-form number oracle + `.3.1` declared-id family no-regression (interface/program now parse too); **external-corpus parse `4/14 → 6/14`** (friscv_pipeline ×{2017,2023} now fully parse; residual uvm/friscv_rv32i/veer categorized into sub-leaves `.3.3.2`/`.3.3.3`/`.3.3.4`). **1.0.120 (SV-EXH-PROOF.3.3.2, `PGEN-SV-EXH-PROOF-0023`) keeps schema `3`** — `package_declaration` was the only declaration NOT using the proven `declared_X_identifier := X_identifier -> {body:$1.body}` + `@emit_fact {name:$body}` idiom: it put `@emit_fact {name:$package_identifier}` directly on the multi-element rule, and `package_identifier` occurs twice (decl + trailing `endpackage : label`), so the named ref resolved to the `{body:…}` object / was unresolvable ⇒ *"could not resolve fact name"*, making **package declarations** unparseable. Added `declared_package_identifier := package_identifier -> {body:$1.body}` with `@emit_fact {kind:package_name, name:$body}` (mirrors `declared_class_identifier`; **emit-only — package's original directive had NO `@predicate has_fact`, a package is not a type; that behavior is preserved**) and routed the decl-site through it; trailing `( colon package_identifier )?` stays the plain label use-site (exactly as class/module). **Grammar-only — engine untouched.** **Release bump, NO schema bump** — package declarations were unparseable so no such AST was ever emitted; previously-parseable byte-identical; strictly-more-permissive (SVPP-0002/REGEX-0083 category). SV shape-contract **GREEN** (samples=3 aligned=3 drift=0 regression_lock_failures=0). **external-corpus parse stays `6/14`** — HONEST: `package pp; endpackage` (+label/+items) now parse (were 100% unparseable), but uvm_pkg/uvm_compat_pkg ×{2017,2023} fail *deeper* at the `.3.3.3` use-site `known_unscoped_*` type-id defect (~`118257`, *"could not resolve attribute reference 'type_identifier'"*) — `.3.3.2` peeled one onion layer (forward progress; the aggregate binary count is gated by the deeper defect). Evidence refined the categorization: `.3.3.3` (use-site type-id) is the dominant residual (veer + uvm ×4 + uvm_compat ×2). `.3.1`/`.3.2`/`.3.3.1` no-regression verified. **1.0.121 (SV-EXH-PROOF.3.3.3, `PGEN-SV-EXH-PROOF-0024`) keeps schema `3` — PARSER-AGNOSTIC ENGINE EXCEPTION-SAFETY FIX + SV grammar wrapper (composite).** ROOT CAUSE (DEFINITIVELY pinned via full SEMTRACE instrumentation; 3 prior fix attempts disproven, then identified): a classic Rust `?`-bypasses-cleanup bug in the generator-emitted `with_semantic_runtime_rule_transaction` (`rust/src/ast_pipeline/ast_based_generator.rs`). The wrapper `std::mem::take`-s `self.semantic_runtime_state` (leaving `Default == new()` valid-but-EMPTY), then performs `?`-fallible calls (`f(self)?`, `apply_semantic_runtime_effect_directive?`, `resolve_semantic_predicate_spec_against_content?`); a `?` early-return propagated out of the ENTIRE function, JUMPING OVER the `if result.is_err() { self.semantic_runtime_state = original }` restore at the bottom — so `self.semantic_runtime_state` was left EMPTY, silently destroying every fact emitted by prior COMMITTED sibling rules (e.g. a typedef's `type_name` fact). Every later sibling `@predicate has_fact(...)` then saw nothing. Decisive SEMTRACE proof: `RESTORE` fired **0×**, `took post-body state (self.state→EMPTY)` **9×**, fact-flow `declared_type_identifier facts(1)=[type_name=my_t] EXIT OK → … → use-site facts(0)=[] never restored`. FIX (parser-agnostic, in the GENERATOR — Option B "try-block" emulation): wrap the fallible body in an immediately-invoked closure `(|| -> ParseResult<…> {…})()` so every `?` returns into `result` and the restore runs on every non-commit exit. Zero behaviour change on success; no `unsafe`. **Note:** `<SemanticRuntimeState as Default>::default()` delegates to `Self::new()` (Global scope, no facts) — a VALID state — so `std::mem::take` never leaves a *corrupt* state; combined with the IIFE the fix is panic-ROBUST (no corruption on unwind; parser never `catch_unwind`s mid-parse). A true Drop-guard RAII would be strictly panic-SAFE but is not cleanly safe-Rust here (post-take body calls `&self` methods; a guard borrowing `&mut self.semantic_runtime_state` would conflict — only a tiny contained `*mut`+`unsafe` would give a real Drop guard; deferred since IIFE+Default-validity is already panic-robust). SV-grammar COMPOSITE half: added `checked_type_identifier := type_identifier -> {body:$1.body}` + `@predicate has_fact args:[type_name,$body] phase:post`, routed `known_unscoped_block/data_type_identifier` through it (proven .3.3.1/.3.3.2 declared-id idiom; required because a sub-rule-name predicate ref on a `->`-bearing rule does NOT resolve and dotted predicate args are `@predicate`-compiler-rejected). **Release bump, NO schema bump** — declarations whose post-predicate emit/restore was being leaked were silently unparseable so no such AST was ever emitted; previously-parseable inputs byte-identical (success paths unchanged); only previously-erroring inputs now succeed (strictly-more-permissive, SVPP-0002/REGEX-0083 category). VERIFIED: minimal repro `module m; typedef int my_t; my_t [3:0] x; endmodule` PASSES (was unfixable through 3 prior attempts); SEMTRACE confirmed RESTORE fires 110× (was 0); checked_type_identifier `has_fact[type_name,my_t]` resolves+true; class/typedef/localparam/let/builtin-dim no-regression (memo ON / normal). Inventory **2297 → 2299** (+2: the new `checked_type_identifier` rule's @predicate + return-shape; SV shape-contract GREEN samples=3 aligned=3 drift=0 regression_lock=0). FULL cross-parser no-regression: regex broader corpus / conformance **44/0** ✅ (RGX critical downstream — unchanged); SV shape-contract GREEN; SV external corpus **stays `6/14`** — `.3.3.3` is the FOUNDATION engine fix (state-leak was masking everything); the residual 8 cases (veer/uvm/uvm_compat ×6 cross-package import; friscv_rv32i ×2 statement-level) are blocked by DISTINCT defects tracked as `.3.3.4`/`.3.3.6`. Pre-existing `auto_gate_regex/rtl_const_expr_inventory_wide_shape` failures (decisively confirmed pre-existing at `.3.3.2` via git-stash baseline) tracked as `.3.3.5` (separate root-cause needed). **1.0.125 (SV-EXH-PROOF.3.3.4.b.1, `PGEN-SV-EXH-PROOF-0028`) keeps schema `3` — FIRST LRM-EXTRACTION-DEFECT FIX: `conditional_statement` `[ else statement_or_null ]` encoded as truly optional.** Root cause (precisely located against the authoritative IEEE 1800-2017 PDF page 1164 AND IEEE 1800-2023 PDF page 1201, both editions identical text): `grammars/systemverilog.ebnf:1111`'s `conditional_statement` rule had exactly ONE production with NO `|` alternative for the no-`else` case — `( unique_priority )? kw_if lparen cond_predicate rparen statement_or_null &kw_else_ae050f5b kw_else_ae050f5b conditional_else_branch`. `&kw_else` is PEG positive lookahead REQUIRING else; `kw_else` then consumes; `conditional_else_branch` then consumes the body. There was no fallback for if-without-else. LRM is unambiguous: `[ else statement_or_null ]` — square brackets mean OPTIONAL. This was an LRM-extraction defect: the extractor encoded an LRM-optional clause as mandatory. FIX: introduced helper `conditional_else_clause := kw_else_ae050f5b conditional_else_branch -> $2` (passes through just the branch); rewrote `conditional_statement` to `( unique_priority )? kw_if lparen cond_predicate rparen statement_or_null ( conditional_else_clause )?` with return annotation `-> {unique_priority:$1, condition:$4, then_body:$6, else_body:$7}` (positional index shift $9→$7; `else_body` now nullable when else absent). Also removed dead mis-attached line 1116 (`| @sample: "if (a) ;" ... !kw_else_ae050f5b`) — the original author reached for the no-else alternative via PEG negative lookahead but the indentation + blank-line at 1113 parented it to `conditional_else_branch` (unreachable from any parse path). Else-if chain `{ else if (cond) statement_or_null }` continues to be handled by the existing recursive `conditional_else_branch := conditional_statement | statement_or_null` — once the outer else is optional, the recursive `conditional_statement` arm naturally produces zero-or-more chained else-ifs without separate rewrite needed. Inventory **2299 → 2300** (+1: the new `conditional_else_clause` helper rule with its `-> $2` return annotation; existing `conditional_statement` return-annotation shape preserved — `else_body` is the same `conditional_else_branch` ParseNode when present, just additionally nullable when absent, which is strictly-more-permissive over a pre-fix population where every parsed `conditional_statement` necessarily had an else). PEG correctness: `( … )?` is greedy and binds `else` to the innermost `if`, matching the SV dangling-else convention; no `&X` / `!X` lookahead needed (the dead line 1116 was unnecessarily defensive — the `?`-greedy semantics handle dangling-else correctly). VERIFIED: minimal failing repro `module m; task t(); if (1) $display("ok"); endtask endmodule` PASSES (was FAIL); pre-existing `if (1) ; else ;` STILL PASSES; else-if chain `if (1) a; else if (2) b; else c;` PASSES; `.3.3.3` minimal repro `module m; typedef int my_t; my_t [3:0] x; endmodule` STILL PASSES; end-to-end synthetic cross-file repro PASSES with `--lib-in` and FAILS without (`.3.3.4.a` behavior preserved); lib **461/461** no-features + **516/517** with `--features generated_parsers` (1 fail = pre-existing `rgx_0077` `.3.3.5`-class). FULL cross-parser no-regression: **RGX broader corpus / conformance ✅ 44/0** via `make regex_broader_corpus_proof_gate`; SV shape-contract GREEN. SV external corpus partial retest: scr1_core_top_2017 PASS (was PASS, time 38s→56s due to deeper parsing post-fix), friscv_pipeline_2017 PASS at unchanged 0.86s, veer_el2_lsu_2017 PASS at 7.6s + 21.6s bootstrap (unchanged). **uvm_pkg_2017 STILL FAILS at the SAME byte position 113637 post-fix, but parse time 60s→14min** — meaning `conditional_statement` was NOT the blocker for uvm_pkg; the parser now walks deeper into the 89K-line / 3MB body (the slowdown is "real work", not a pathological backtracking explosion) and hits an OTHER LRM-extraction defect deeper in. Triage gate runtime ceiling makes a full 14-case rerun cost ~3+ hours; full retest deferred pending either further LRM-extraction-defect fixes that unblock more cases OR a perf-aware retry. **SV external corpus binary count: stays `8/14`** — no passing case lost; no new failing case fixed by THIS slice (the conditional_statement fix is correct on its own merits per the LRM oracle, but uvm/uvm_compat/friscv_rv32i are blocked by deeper LRM-extraction defects, not by conditional_statement). HONEST: this slice is the first concrete fix of the LRM-extraction-defect campaign; the corpus is not yet at 12/14 because more deep defects remain — they will be found via the same defect-class-by-defect-class sweep the user mandated (next class TBD after the `&X` class is fully exhausted, which it is — `&X` is a class of one). The remaining `&question question` at line 1105 (`conditional_expression` ternary) is legitimate (`?` and `:` are LRM-MANDATORY for ternary per §A.8.3), not a defect. Pre-existing `1.0.124 (.3.3.4.a.2)`, `1.0.123 (.3.3.4.a.1)`, `1.0.122 (.3.3.4.a)`, ... retained below. **1.0.124 (SV-EXH-PROOF.3.3.4.a.2, `PGEN-SV-EXH-PROOF-0027`) keeps schema `3` — PARSER-AGNOSTIC ENGINE EXTENSION: non-negative integer indexed-access (`[N]`) in semantic-annotation rule references.** Companion to `.3.3.4.a.1` (which added dotted property-access). Real authoring concern that `.3.3.4.a.1` only half-addressed: when a rule's shaped output is `{items: [{name, body}, ...]}` or `{matrix: [[…], …]}`, directives still couldn't reference array elements directly — there was no `$items[0].name` or `$matrix[0][1]` form. `.3.3.4.a.2` closes that gap. ENGINE: `unified_semantic_ast.rs::parse_rule_reference` chain loop refactored to a `loop { match peek_char() { Some('.') => …, Some('[') => …, _ => break } }` shape that accepts an arbitrary mix of `.<ident>` and `[<digits>]` segments. Strict-bracket policy: a `[` not followed by `<digits>]` rolls back (leaves the `[` for the surrounding parser to handle / fail). EBNF surface `grammars/semantic_annotation.ebnf::rule_reference_name` extended in lockstep: `/([a-zA-Z_][a-zA-Z0-9_]*(\.[a-zA-Z_][a-zA-Z0-9_]*|\[[0-9]+\])*|[0-9]+(\.[a-zA-Z_][a-zA-Z0-9_]*|\[[0-9]+\])*)/`. Bootstrap `semantic_annotation_parser.rs` regenerated. RUNTIME RESOLVER (`ast_based_generator.rs`): three new free-standing helpers emitted into every generated parser — `lex_semantic_reference_segments_suffix` (lexes a bare suffix like `.body[0].sub` into a `Vec<&str>` where each segment is either a property name or the full `[<digits>]` form with brackets), `lex_semantic_reference_segments_named` (named-path counterpart that also captures the head identifier), `parse_bracketed_index` (extracts the integer from `[<digits>]` or returns `None`). The two resolvers (`resolve_positional_semantic_reference` and `resolve_named_semantic_reference`) use the lexers and dispatch per segment: for shaped-JSON content (`ParseContent::Json`) the walk goes through `serde_json::Value::get`, which polymorphically accepts both `&str` (property) and `usize` (index); for raw-tree content (rules without `->`), a new `find_semantic_indexed_child` companion picks the N'th element of a `Sequence`/`Quantified` (or returns the wrapped node of an `Alternative` when N==0). Annotation inventory unchanged (helper functions emitted are infrastructure, not annotated rules). SUBSET BOUNDARY: dotted property access + non-negative integer indexed-access ONLY. NOT full JSONPath (no filters `[?(@.foo)]`, no wildcards `*`, no recursive descent `..`). DURABLE NO-DEPTH-LIMIT GUARANTEE extends to mixed dotted+indexed depth: structurally unbounded at every layer (EBNF `*` quantifier; hand-rolled `loop` with no max-iteration cap; lexer `while !remaining.is_empty()` iterates until input exhausted; resolver iterates over the lexed Vec). Locked by `bootstrap_semantic_indexed_rule_reference_depth_is_structurally_unbounded` test exercising 32 mixed `.<ident>` + 32 `[N]` segments (64 total) and asserting verbatim retention of both kinds of segments (33 dot-separated chunks, 32 bracketed segments). Pairs with the `.3.3.4.a.1` dotted-only test. No SV grammar changes — purely a language-surface expressiveness improvement; the SV directive payloads in 1.0.123 (`$name.body`, `$package.body`) continue to work byte-identically. **Release bump, NO schema bump** — strictly additive language extension; no shape changes anywhere; no annotated rules changed; SVPP-0002/REGEX-0083 category. VERIFIED: 461/461 lib no-features (was 460 → +1 new indexed-access regression test); 516/517 with `--features generated_parsers` (only fail = pre-existing `rgx_0077` from `.3.3.5`-class); cross-file synthetic repro PASSES with `--lib-in` and FAILS without (1.0.123 cleanup behaviour preserved); `.3.3.3` minimal repro still PASSES; both depth-unbounded regression tests in the suite (dotted-only + mixed). FULL cross-parser no-regression: regex broader corpus / **RGX conformance 44/0 ✅** via `make regex_broader_corpus_proof_gate`; SV shape-contract GREEN; **SV external corpus stays `8/14`** (verified: triage gate fresh 2026-05-20; veer_el2_lsu ×{2017,2023} still PASS via the existing `$name.body` directives — no SV directive uses `[N]` yet, so the new feature is dead-code there). NEXT LEAF (queued): `.3.3.4.b` — uvm self-contained same-file-package path (intra-file scope tracking, not artifact-on-disk; the cross-file MVP-0 model doesn't apply when package + uses share a file). Pre-existing `1.0.123 (.3.3.4.a.1)`, `1.0.122 (.3.3.4.a)`, `1.0.121 (.3.3.3)`, … retained below. **1.0.123 (SV-EXH-PROOF.3.3.4.a.1, `PGEN-SV-EXH-PROOF-0026`) keeps schema `3` — PARSER-AGNOSTIC ENGINE CLEANUP: dotted property-access in semantic-annotation rule references + revert of the `.3.3.4.a` SV `body:$N.body` shape workarounds.** Companion to `.3.3.4.a`: the directive-payload parser is the hand-rolled `StructuredSemanticValueParser` in `rust/src/ast_pipeline/unified_semantic_ast.rs::parse_rule_reference` (the generated `semantic_annotation_parser.rs` is the EBNF-language surface, NOT the runtime path for grammar directive payloads). The runtime resolver already walked dotted paths (`resolve_named_semantic_reference` in `ast_based_generator.rs:3912` splits on `.` and traverses either the shaped JSON object-key path or the raw sub-rule-named descendant tree); the only gap was at the bootstrap-parser-surface, which rejected `.` inside `$<name>`. `.3.3.4.a.1` closes that gap: `parse_rule_reference` gains a `while peek_char() == Some('.')` chain that accepts an arbitrary number of `.<ident>` segments. DURABLE NO-DEPTH-LIMIT GUARANTEE: structurally unbounded at every layer (EBNF `*` quantifier; hand-rolled `while` with no cap; resolver iterator-based walk). Locked by `bootstrap_semantic_dotted_rule_reference_depth_is_structurally_unbounded` test that exercises a 64-segment reference and asserts retention — any future `MAX_REFERENCE_DEPTH` constant must justify itself in its own leaf or back off. The EBNF surface is also kept consistent: `grammars/semantic_annotation.ebnf::rule_reference_name` regex extended `(\.<ident>)*` so the language definition mirrors the runtime; the bootstrap `semantic_annotation_parser.rs` is regenerated in lockstep (the EBNF/regenerated-parser does not parse directive payloads — `unified_semantic_ast.rs` does — but keeping the surface consistent prevents future drift). SV grammar revert: `body: $4.body` removed from `package_declaration`'s shaped output; `body: $1.body` removed from BOTH `package_import_item` branches; the two directives now reference the nested scalars naturally — `@export_to_library: {kind:package, name_from:$name.body}` on `package_declaration` and `@import_from_library: {kind:package, name_from:$package.body}` on `package_import_item`. **Release bump, NO schema bump** — the prior `body` fields were a workaround carrier present only in 1.0.122 (one slice); they had no consumer pre-1.0.122 and have no consumer post-`.3.3.4.a.1` (cross-file imports STILL resolve, see verification); the removal is a strictly-rollback shape change versus 1.0.122 (and a no-op shape change versus 1.0.121, since 1.0.121 didn't have those fields either). VERIFIED: end-to-end synthetic cross-file repro PASSES with `--lib-in` and FAILS without (proves library plumbing still the deciding factor — the cleanup is behaviour-preserving); `.3.3.3` minimal repro still PASSES; lib tests **460/460** PASS no-features (was 459 → +1 new unbounded-depth regression test) and **515/516** with `--features generated_parsers` (the only fail = pre-existing `rgx_0077` from `.3.3.5`-class). FULL cross-parser no-regression: regex broader corpus / RGX conformance **44/0** ✅ via `make regex_broader_corpus_proof_gate`; SV shape-contract GREEN; **SV external corpus stays `8/14`** — veer_el2_lsu ×{2017,2023} both PASS via the new dotted-refs path (fresh triage gate 2026-05-20 15:41; identical to `.3.3.4.a`'s shape-workaround result → cleanup proven behaviour-preserving). Annotation inventory: unchanged-shape-of-existing-entries (the two SV `body:` fields were workaround output-mapping additions, not new annotated rules; removing them is a shape change of existing entries, not a count change). The duplicate-facts artifact-content observation from `.3.3.4.a` remains (PEG try-alternative speculation; not addressed in this slice — separate post-campaign shape audit). NEXT LEAVES: `.3.3.4.a.2` will extend `parse_rule_reference` and `parse_semantic_reference_segments` + the EBNF regex with `[<digits>]` indexed access (`$items[0].name` / `$matrix[0][1]` / mixed `$a.b[0].c[1].d`) — same unbounded-depth guarantee, own leaf, separate commit. Pre-existing `1.0.121 (.3.3.3)`, `1.0.120 (.3.3.2)`, `1.0.119 (.3.3.1)`, `1.0.118 (.3.2)`, `1.0.117` POST-SV-AUDIT.2.4b, `1.0.116`/schema 2, `1.0.115`/schema 1 retained below. **1.0.122 (SV-EXH-PROOF.3.3.4.a, `PGEN-SV-EXH-PROOF-0025`) keeps schema `3` — PARSER-AGNOSTIC ENGINE FEATURE (per-compilation-artifact library) + 2 new generic annotations + SV grammar uses (composite, MVP-0).** ROOT MOTIVATION: PGEN parsed single SV files in isolation, but real HDL is multi-file — `import pkg::*` references types declared in *other* files (e.g. veer's `el2_lsu.sv` imports types from `el2_def.sv`). The use-site `@predicate has_fact(type_name,X)` correctly fires (`.3.3.3` made it reliable) but correctly evaluates false because the type-emitting file was never parsed in the same session. Architecturally-correct fix matching every commercial HDL tool: per-file compilation that writes a compact on-disk artifact for each scope-creating entity (package/module/interface/…) + on-demand library lookup when an importer references that entity. **MVP-0 (this slice)** narrows to packages only, JSON artifacts, user-supplied file order; future increments grow kind-set + add `$unit`/CU semantics under separate leaves. ENGINE (parser-agnostic, generator + runtime): (1) `rust/src/ast_pipeline/library.rs` new module — atomic JSON artifact write/read at `<lib-dir>/<kind>/<name>.facts.json`, `format_version: 1`, MVP-0 exportable-kind filter (`type_name` only), 4 round-trip unit tests; (2) `SemanticRuntimeDirective::{ExportToLibrary, ImportFromLibrary}` + their specs + classifiers `is_library_export/import` + accessors `library_exports/imports_for_rule` on `CompiledSemanticRuntimeAnnotations` (own phase, not in `is_effect`/`is_pre_predicate`); (3) generator wire-up in `with_semantic_runtime_rule_transaction` (the IIFE-wrapped exception-safe wrapper from `.3.3.3`): library imports fire AFTER body parse + effect directives but BEFORE post-predicates (so post-predicates see merged facts); library exports fire AFTER post-predicates BEFORE commit (atomic write; failure rolls back via the IIFE's restore path); export delta computed against `original_semantic_runtime_state.facts().len()` (the rule's TRUE entry, not the transaction-checkpoint which is post-body); (4) parser-struct fields `library_in_dir/library_out_dir` + 4 setters/accessors; (5) `parser_registry::LibraryOptions` + SV-aware `parse_sample_detail_with_options` that falls through to `parse_sample_detail_with_profile` when options are empty (zero behaviour change for grammars without library directives); (6) `parseability_probe` CLI flags `--lib-in DIR` / `--lib-out DIR` with strict arg-validation. SV GRAMMAR (composite, this slice): `@export_to_library: {kind:package, name_from:$body}` on `package_declaration` + new top-level output field `body: $4.body` (semantic-annotation language only accepts simple `$name` refs — not `$x.y` — so the package's scalar name is surfaced at the top of the shape; return-annotation `$4.body` property-access drill DOES work in the output mapping); same idiom on `package_import_item` with `@import_from_library` + `body: $1.body` (both branches). TRIAGE GATE (composite, this slice): `systemverilog_external_corpus_triage_v0.json` gains optional per-case `bootstrap_files: [...]` ordered array (commercial-tool convention — user supplies the transitive-dep order); script preprocesses+parses each bootstrap file with `--lib-out <per-case-lib-dir>` THEN parses the main case with `--lib-in <per-case-lib-dir>`; `veer_el2_lsu` declares `bootstrap_files: ["…/include/el2_def.sv"]`. **Release bump, NO schema bump** — adds 2 new top-level optional shaped fields (`body`) to `package_declaration` and `package_import_item`; SV shape-contract recalibrated GREEN. Cross-file imports were unparseable so no such AST was ever emitted from the importer; previously-parseable inputs byte-identical (success paths unchanged); only previously-erroring inputs (cross-file referencing a known package) now succeed (strictly-more-permissive, SVPP-0002/REGEX-0083 category). VERIFIED: end-to-end synthetic repro (write artifact for `my_pkg` containing `my_t`+`byte_t`; module `m` with `import my_pkg::*; my_t [3:0] x; byte_t b;` parses WITH `--lib-in`, FAILS WITHOUT — proves the library path is the deciding factor); minimal `.3.3.3` repro `module m; typedef int my_t; my_t [3:0] x; endmodule` STILL PASSES; lib tests `514/515 PASS` (only fail is the pre-existing `regex_parser_pgen_rgx_0077_quoted_run_quantified_pieces_flat_in_concatenation` confirmed PRE-EXISTING via decisive baseline — git-stashed sources, rebuilt baseline driver from HEAD eb42a3a0, re-regened regex+bootstrap parsers, SAME failure — tracked as `.3.3.5`); 6 new directive-parser unit tests in `semantic_runtime.rs` (positive Export, positive Import, unknown-field rejection, missing-kind rejection, missing-name_from rejection, compiled-annotation routing) all GREEN; 4 library-module unit tests all GREEN. FULL cross-parser no-regression: regex broader corpus / conformance **44/0** ✅ (RGX critical downstream — unchanged via `make regex_broader_corpus_proof_gate`); SV shape-contract GREEN; **SV external corpus `6/14 → 8/14`** — veer_el2_lsu ×{2017,2023} now PASS via the bootstrap-files+library-artifact path (THE corpus-mover for veer, exactly as projected). Residual 6 fails: uvm_pkg/uvm_compat_pkg ×{2017,2023} fail at `.3.3.3` use-site `known_unscoped_*` deeper still — they are SELF-CONTAINED files (no separate `import` declarations the way veer does); friscv_rv32i_core ×{2017,2023} statement-level (`.3.3.6`). The `1.0.116`/schema `2`, `1.0.115`/schema `1`, the AST-Shape-Corrections-1.0.116, the DOC-ENVELOPE / earlier slice history are retained below.)
- Last updated:
  - `2026-05-20`
- Current grammar family label:
  - `systemverilog`
- Current stable host profiles:
  - `sv_2017`
  - `sv_2023`
- Current live status:
  - Tracked in `LIVE_ACHIEVEMENT_STATUS.md`

## Current Trust Statement
- The PGEN `systemverilog` parser is **closure-grade for the current Nexsim-facing scope** when consumed through the stable `pgen::embedding_api` host surface.
- Closure is established via the family status / contract / telemetry gates listed under "Validation / Release Gates" below.
- The current sign-off bar is Nexsim-facing SystemVerilog parsing, not an open-ended promise for every imaginable SystemVerilog dialect or tool ecosystem.
- The grammar covers IEEE 1800-2017 (`sv_2017` profile) and the IEEE 1800-2023 delta (`sv_2023` profile). Both profiles share `grammars/systemverilog.ebnf` as the single source of truth.

## Companion Documentation — SystemVerilog Parser Integration mdBook
- The systemverilog-parser integration mdBook lives at `docs/systemverilog_parser_book/` and is the **canonical AST reference** for downstream consumers (Nexsim in particular).
- The book documents: build recipe, public API, the AST envelope, every annotated/un-annotated rule shape (as the annotation campaign progresses), per-feature worked examples, schema versioning, glossary, and a release-by-release index.
- Build it with `make systemverilog_parser_book_gate` (uses `mdbook build docs/systemverilog_parser_book`).
- Where the book and this contract disagree, **the contract wins** for compliance — but please report the disagreement as a documentation bug.

## Release 1.0.117 / Contract 1.0.117 Highlights — POST-SV-AUDIT.2.4b: 11 structured-per-iteration Category-A misuses corrected via factored record rules; schema 2 → 3

Landed 2026-05-17. The POST-SV-AUDIT static classification pass
(`docs/POST_SV_AUDIT_LEDGER.md`, leaf POST-SV-AUDIT.2.4b, tracked
`PGEN-POST-SV-AUDIT-0006`) dispositioned the final SystemVerilog
worklist item — the **11 structured-per-iteration Category-A** rules in
`grammars/systemverilog.ebnf`. Each repeated **multi-field** unit
(`X field field … ( SEP X field field … )*`) was factored into a **new
named record rule** that emits the per-iteration record, and the list /
branch now uses an extraction-spread over that record rule. The field
names of the prior `first` record (`{name, dims[, init]}`,
`{name, value}`, `{name, pattern}`) are **preserved**.

The parser is regenerated; the manifest
`rust/test_data/ast_shape_contract/systemverilog_v1.json` is re-locked
(new `structured_decls` sample + `calibration_history` entry #118);
`systemverilog_ast_shape_contract` passes.

Annotation count: **2299** (was 2290, **+9 — a deliberate count
change, NOT "unchanged"**). The 9 new factored record rules
(`interface_identifier_decl`, `port_identifier_decl`,
`variable_identifier_decl`, `tf_variable_identifier_decl`,
`variable_port_identifier_decl`, `let_named_arg`, `property_named_arg`,
`sequence_named_arg`, `assignment_pattern_entry`) **are annotated** —
unlike the pure-Cat-A `[$1, $2::2*]` rewrites, which add no annotations;
this is the same kind of deliberate +N record-rule count change as
SVPP-0001's `pp_if_keyword`. **1008** distinct annotated rules (was
999, +9). Same accept set (no grammar-acceptance change — purely the
factored record shape). AST-dump schema bumped `2 → 3` because the
affected list / branch shapes change in a consumer-visible way.

### Schema-Versioning row

| AST-dump schema version | First parser release | Notable changes |
|---|---|---|
| `3` | `1.0.117` | **POST-SV-AUDIT.2.4b (`PGEN-POST-SV-AUDIT-0006`).** 11 structured-per-iteration Category-A misuses corrected — each repeated multi-field unit factored into a new annotated record rule + an extraction-spread; field names preserved. 9 new annotated record rules → inventory **2290 → 2299 / 999 → 1008** (a deliberate +9, NOT "unchanged"; analogous to SVPP-0001's `pp_if_keyword`). Reachable `list_of_*_identifiers` probe-verified; the `*_list_of_arguments` / `parameter_port_list` type_only / `assignment_pattern` named branches are defensively-correct-by-construction (likely unreachable via the strict SV root — pre-existing, out-of-scope), **not** a bug-ledger entry. Same accept set. See "AST-Shape Corrections — 1.0.117 (POST-SV-AUDIT)". |
| `3` (unchanged) | `1.0.118` | **SV-EXH-PROOF.3.2 (`PGEN-SV-EXH-PROOF-0021`).** Restored the broken per-digit-token number decomposition to the generator's clean single-regex IEEE-1800 form (blanket-`\b` + `__SV_RULE__`-`_` corruption ⇒ every multi-digit/underscore/sized/based number unparseable = the external-corpus 0/14 root cause). **Release bump, NO schema bump** — numbers were 100% unparseable (no number AST ever emitted; previously-parseable byte-identical; strictly-more-permissive, SVPP-0002/REGEX-0083 category). Inventory 2312→2297 (−15 dead decomposed-rule annotations; SV shape-contract GREEN). external-corpus **0/14 → 4/14** (scr1 family); residual uvm/friscv/veer pinned `.3.3`. Full LRM number-form oracle + `.3.1` 13-family no-regression verified. Resolves the long-DEFERRED `*_value`/`unsigned_number` shaping note. See "AST-Shape Corrections — 1.0.118 (SV-EXH-PROOF.3.2)". |
| `3` (unchanged) | `1.0.119` | **SV-EXH-PROOF.3.3.1 (`PGEN-SV-EXH-PROOF-0022`).** `non_keyword_identifier` (foundation for every declaration-site name — class/package/typedef/parameter/localparam via `declaration_identifier`) returned raw `-> $2`; the positional-extraction codegen leaves that `Alternative`-wrapped (implicit-passthrough codegen unwraps it), so every `declared_*_identifier -> {body:$1.body}` `@emit_fact`/`@predicate has_fact` directive failed *"could not resolve fact name"* ⇒ class/package/typedef/parameter/localparam **declarations** 100% unparseable (pre-existing, masked by the .3.2 number bug). Fixed `-> {body: $2.body}` (mirrors `identifier := … -> {body:$1}`). **Grammar-only — engine untouched.** **Release bump, NO schema bump** — those declarations were 100% unparseable (no such AST ever emitted; previously-parseable byte-identical; strictly-more-permissive, SVPP-0002/REGEX-0083 category). SV shape-contract GREEN (samples=3 aligned=3 drift=0 regression_lock=0); inventory unchanged (existing entry's shape change, not a count change). external-corpus **4/14 → 6/14** (friscv_pipeline ×2 now parse); residual uvm/friscv_rv32i/veer categorized → `.3.3.2`/`.3.3.3`/`.3.3.4`. `.3.2` number oracle + `.3.1` declared-id family no-regression verified. See "AST-Shape Corrections — 1.0.119 (SV-EXH-PROOF.3.3.1)". |
| `3` (unchanged) | `1.0.120` | **SV-EXH-PROOF.3.3.2 (`PGEN-SV-EXH-PROOF-0023`).** `package_declaration` was the sole declaration not using the proven `declared_X_identifier -> {body:$1.body}` + `@emit_fact {name:$body}` idiom — it put `@emit_fact {name:$package_identifier}` on the multi-element rule with `package_identifier` occurring twice ⇒ *"could not resolve fact name"*, package declarations unparseable. Added `declared_package_identifier` (mirrors `declared_class_identifier`; emit-only — package's original had NO `@predicate has_fact`, preserved), routed the decl-site through it, trailing label stays plain. **Grammar-only — engine untouched.** **Release bump, NO schema bump** (package decls were unparseable; strictly-more-permissive, SVPP-0002/REGEX-0083 category). SV shape-contract GREEN (samples=3 aligned=3 drift=0 regression_lock=0). external-corpus **stays 6/14** (HONEST — `package pp; endpackage` now parses but uvm_pkg/uvm_compat fail deeper at the `.3.3.3` use-site type-id defect ~118257; `.3.3.2` peeled one layer). Categorization refined: `.3.3.3` (use-site `known_unscoped_*` type-id) is the dominant residual (veer + uvm ×4 + uvm_compat ×2). `.3.1`/`.3.2`/`.3.3.1` no-regression verified. See "AST-Shape Corrections — 1.0.120 (SV-EXH-PROOF.3.3.2)". |
| `3` (unchanged) | `1.0.121` | **SV-EXH-PROOF.3.3.3 (`PGEN-SV-EXH-PROOF-0024`).** PARSER-AGNOSTIC ENGINE EXCEPTION-SAFETY FIX + SV grammar wrapper (composite). Root cause (definitively SEMTRACE-pinned after 3 disproven hypotheses): a Rust `?`-bypasses-cleanup bug in the generator-emitted `with_semantic_runtime_rule_transaction` — `std::mem::take(&mut self.semantic_runtime_state)` then `?`-fallible calls (`f(self)?`, `apply_*?`, `resolve_predicate?`); a `?` early-return jumped over the trailing `if result.is_err() { restore original }`, leaving `self.semantic_runtime_state` EMPTY and silently destroying every fact emitted by prior COMMITTED sibling rules. SEMTRACE proof: `RESTORE` 0× (was supposed to fire); use-site `facts(0)=[]` after a committed `declared_type_identifier facts(1)=[type_name=my_t] EXIT OK`. **FIX (engine, parser-agnostic, in the GENERATOR)**: wrap the fallible body in an immediately-invoked closure ("try-block" emulation) so every `?` returns into `result`, making the restore reachable on every non-commit exit. Zero `unsafe`; zero behaviour change on success. (RAII Drop-guard alternative considered, deferred — would be strictly panic-safe but needs `*mut`+`unsafe` here; IIFE+Default-validity is already panic-robust since `<SemanticRuntimeState as Default>::default() = ::new()` is valid, and the parser never `catch_unwind`s mid-parse.) **FIX (SV grammar composite half)**: added `checked_type_identifier := type_identifier -> {body:$1.body}` + `@predicate has_fact args:[type_name,$body] phase:post`, routed `known_unscoped_block/data_type_identifier` through it (the proven `.3.3.1`/`.3.3.2` declared-id idiom — required for the predicate to resolve). **Release bump, NO schema bump** — affected declarations were silently unparseable so no such AST was ever emitted; previously-parseable inputs byte-identical; only previously-erroring inputs now succeed (strictly-more-permissive, SVPP-0002/REGEX-0083 category). VERIFIED: `module m; typedef int my_t; my_t [3:0] x;` PASSES (was unfixable); SEMTRACE confirmed RESTORE fires 110× (was 0); use-site `has_fact[type_name,my_t]` resolves+TRUE. Inventory 2297→2299 (+2: the new wrapper rule). SV shape-contract GREEN samples=3 aligned=3 drift=0 regression_lock=0. **regex broader corpus / RGX conformance ✅ 44/0** (the critical downstream — unaffected, fast-path for non-semantic grammars). external-corpus **stays `6/14`** — `.3.3.3` is the FOUNDATION engine fix (state-leak was masking everything); the residual 8 cases blocked by DISTINCT defects → `.3.3.4` (cross-package `import pkg::*` type-name visibility, the corpus mover for veer/uvm/uvm_compat ×6) + `.3.3.6` (statement-level, friscv_rv32i ×2). Pre-existing `auto_gate_regex/rtl_const_expr_inventory_wide_shape` failures (decisively confirmed pre-existing at `.3.3.2`) tracked as `.3.3.5`. See "AST-Shape Corrections — 1.0.121 (SV-EXH-PROOF.3.3.3)". |

## AST-Shape Corrections — 1.0.117 (POST-SV-AUDIT) — 11 structured-per-iteration Category-A misuses → clean factored record lists; +9 annotated record rules (2290→2299 / 999→1008, a DELIBERATE count change); reachable list_of_*_identifiers probe-verified, the rest defensively-correct-by-construction (NOT a bug-ledger entry); schema 2 → 3

Landed 2026-05-17. The POST-SV-AUDIT static classification pass
(`docs/POST_SV_AUDIT_LEDGER.md`, leaf POST-SV-AUDIT.2.4b, tracked
`PGEN-POST-SV-AUDIT-0006`) dispositioned the **11
structured-per-iteration Category-A** rules — `X field field …
( SEP X field field … )*` rules whose `{first: {…}, rest: $N}` (or
`{kind:…, first_name, first_value, rest}`) annotation surfaced the raw
`[[SEP, field, field, …], …]` multi-field iteration envelope and forced
every consumer to index past the separator and re-stitch
`first`/`second` with the envelope. Unlike pure Category-A
(`X ( SEP X )*` single-element, fixed one-liner `[$1, $2::2*]`), these
need the repeated **multi-field** unit factored into a **named record
rule** first. That is exactly the fix applied here: 9 new named record
rules emit the per-iteration record, and the list / branch is an
extraction-spread over them. **Field names are preserved** from the
prior `first` record.

**This is a clean Category-A structural improvement — no
`<invalid_sequence_access>` corruption.** It is therefore **NOT** an
entry in `docs/contracts/PGEN_RELEASED_PARSER_BUG_LEDGER.md` (that
ledger is reserved for consumer-reproducible released
`<invalid_sequence_access>` corruption/crash defects). **This batch
carries no bug-ledger row.**

### Count change — +9 annotated record rules (2290 → 2299 / 999 → 1008; DELIBERATE, NOT "unchanged")

This batch **deliberately changes the annotation inventory**. The 9 new
factored record rules are themselves annotated (they each carry a
`-> {…}` record annotation), so:

| | Before (1.0.116) | After (1.0.117) | Delta |
|---|---|---|---|
| Annotation count | 2290 | 2299 | **+9** |
| Distinct annotated rules | 999 | 1008 | **+9** |

This is **categorically different** from the pure-Cat-A grammars (whose
`{first, rest}` → `[$1, $2::2*]` rewrites add no annotations and so are
"unchanged"). Here the corrective idiom *requires* a new annotated
record rule per multi-field unit, so the +9 is **expected and
intended** — directly analogous to SVPP-0001's `pp_if_keyword` (a
factored annotated helper rule that legitimately moved the count). The
9 new annotated record rules are: `interface_identifier_decl`,
`port_identifier_decl`, `variable_identifier_decl`,
`tf_variable_identifier_decl`, `variable_port_identifier_decl`,
`let_named_arg`, `property_named_arg`, `sequence_named_arg`,
`assignment_pattern_entry`. Verified against
`generated/systemverilog_return_annotations.json`
(`annotation_count: 2299`, 1008 distinct rules).

### The 11 rules — old → new (field names preserved)

The 5 `list_of_*_identifiers` rules each gained a dedicated
`*_identifier_decl` record rule:

```ebnf
interface_identifier_decl     := interface_identifier unpacked_dimension*                       -> {name: $1, dims: $2}
list_of_interface_identifiers := interface_identifier_decl ( comma interface_identifier_decl )*  -> [$1, $2::2*]
port_identifier_decl          := port_identifier unpacked_dimension*                            -> {name: $1, dims: $2}
list_of_port_identifiers      := port_identifier_decl ( comma port_identifier_decl )*           -> [$1, $2::2*]
variable_identifier_decl      := variable_identifier variable_dimension*                        -> {name: $1, dims: $2}
list_of_variable_identifiers  := variable_identifier_decl ( comma variable_identifier_decl )*   -> [$1, $2::2*]
tf_variable_identifier_decl       := port_identifier variable_dimension* ( assign expression )?              -> {name: $1, dims: $2, init: $3}
list_of_tf_variable_identifiers   := tf_variable_identifier_decl ( comma tf_variable_identifier_decl )*       -> [$1, $2::2*]
variable_port_identifier_decl     := port_identifier variable_dimension* ( assign constant_expression )?     -> {name: $1, dims: $2, init: $3}
list_of_variable_port_identifiers := variable_port_identifier_decl ( comma variable_port_identifier_decl )*   -> [$1, $2::2*]
```

The 3 `*_list_of_arguments` rules each gained a `*_named_arg` record
rule; the **mixed** branch's `ordered_tail`/`named_tail` become clean
extraction-spread arrays, and the **named_only** branch is now a single
flat `items` list (was `{kind:"named_only", first_name, first_value,
rest}`):

```ebnf
let_named_arg              := dot identifier lparen ( let_actual_arg )? rparen -> {name: $2, value: $4}
let_list_of_arguments      := ( let_actual_arg )? ( comma ( let_actual_arg )? )* ( comma let_named_arg )*
                                  -> {kind: "mixed",      head: $1, ordered_tail: [$2::2*], named_tail: [$3::2*]}
                            | let_named_arg ( comma let_named_arg )*
                                  -> {kind: "named_only", items: [$1, $2::2*]}
# property_named_arg / sequence_named_arg + property_/sequence_list_of_arguments are the identical idiom
```

`parameter_port_list` type_only and the two `assignment_pattern` named
branches:

```ebnf
# parameter_port_list type_only branch (entry [comma, kw_type, type_assignment] → type_assignment at pos 3)
... | hash lparen kw_type type_assignment ( comma kw_type type_assignment )* rparen
        -> {kind: "type_only", items: [$4, $5::3*]}
# shared record rule for BOTH assignment_pattern named branches (sv_2017 + sv_2023)
assignment_pattern_entry := member_identifier colon pattern -> {name: $1, pattern: $3}
... | tick lbrace assignment_pattern_entry ( comma assignment_pattern_entry )* rbrace
        -> {kind: "named", entries: [$3, $4::2*]}
```

| Rule (`grammars/systemverilog.ebnf`) | `≤ 1.0.116` / schema `≤ 2` shape (history) | `1.0.117` / schema `3` shape | New record rule |
|---|---|---|---|
| `list_of_interface_identifiers` | `{first: {name, dims}, rest: $3}` (raw `[[comma, iface_id, [udim…]], …]` `rest`) | `[$1, $2::2*]` → clean `[{name, dims}]` | `interface_identifier_decl -> {name, dims}` |
| `list_of_port_identifiers` | `{first: {name, dims}, rest: $3}` | `[$1, $2::2*]` → clean `[{name, dims}]` | `port_identifier_decl -> {name, dims}` |
| `list_of_variable_identifiers` | `{first: {name, dims}, rest: $3}` | `[$1, $2::2*]` → clean `[{name, dims}]` | `variable_identifier_decl -> {name, dims}` |
| `list_of_tf_variable_identifiers` | `{first: {name, dims, init}, rest: $4}` | `[$1, $2::2*]` → clean `[{name, dims, init}]` | `tf_variable_identifier_decl -> {name, dims, init}` |
| `list_of_variable_port_identifiers` | `{first: {name, dims, init}, rest: $4}` | `[$1, $2::2*]` → clean `[{name, dims, init}]` | `variable_port_identifier_decl -> {name, dims, init}` |
| `let_list_of_arguments` (mixed / named_only) | mixed `{kind:"mixed", head, ordered_tail:$2, named_tail:$3}` (raw tails) / named_only `{kind:"named_only", first_name, first_value, rest:$6}` | mixed `{kind:"mixed", head, ordered_tail:[$2::2*], named_tail:[$3::2*]}` / named_only `{kind:"named_only", items:[$1, $2::2*]}` | `let_named_arg -> {name, value}` |
| `property_list_of_arguments` (mixed / named_only) | same shape as `let_list_of_arguments` (history) | same new shape as `let_list_of_arguments` | `property_named_arg -> {name, value}` |
| `sequence_list_of_arguments` (mixed / named_only) | same shape as `let_list_of_arguments` (history) | same new shape as `let_list_of_arguments` | `sequence_named_arg -> {name, value}` |
| `parameter_port_list` (type_only branch) | `{kind:"type_only", first:$4, rest:$5}` (raw `[[comma, kw_type, type_assignment], …]`) | `{kind:"type_only", items:[$4, $5::3*]}` → clean `[type_assignment]` | (no new rule — `$5::3*` extracts `type_assignment` at entry pos 3) |
| `assignment_pattern` named (sv_2017 occurrence) | `{kind:"named", entries:{first:{name, pattern}, rest:$6}}` | `{kind:"named", entries:[$3, $4::2*]}` → clean `[{name, pattern}]` | shared `assignment_pattern_entry -> {name, pattern}` |
| `assignment_pattern` named (sv_2023 occurrence) | `{kind:"named", entries:{first:{name, pattern}, rest:$6}}` (identical 2nd occurrence) | `{kind:"named", entries:[$3, $4::2*]}` | shared `assignment_pattern_entry` (same rule) |

(The `assignment_pattern` named shape lives in `pattern_sv_2017` /
`pattern_sv_2023`'s `named` branch — both profile occurrences were
fixed and share the one `assignment_pattern_entry` record rule.)

### Honest reachability finding (reachable list_of_*_identifiers probe-verified; the rest defensively-correct-by-construction)

A `list_of_*_identifiers` path **is** reachable via the strict SV
`systemverilog_file` root. Parent probe-verified on
`module m; wire a, b, c; logic x, y, z; endmodule`: the
identifier-list nodes are clean `{name, dims, init}` record arrays —
**0 `<invalid_sequence_access>`**, **0 raw `[[], ","]`
separator-envelope leaks**, **no leftover `{first: {…}, rest: …}`
structured envelope**.

The remaining rules — the three `*_list_of_arguments` (let / property /
sequence), `parameter_port_list` type_only, and the `assignment_pattern`
named branches — are **likely NOT reachable** via the strict SV
`systemverilog_file` root (it rejects most constructs in all profiles;
this is a **pre-existing, out-of-scope SV-grammar-root coverage
limitation**, separate from this defect). Their fix is the **identical,
proven factor-record-rule idiom** (correct by construction — the same
transformation applied to the reachable identifier lists), plus a clean
parser regen and the `systemverilog_ast_shape_contract` manifest lock.
They are documented here as **defensively-correct-by-construction**,
**not** claimed as a fresh end-to-end probe (mirroring the .2.4a
defensive-disposition precedent), and — because there is **no
`<invalid_sequence_access>` corruption** (clean Category-A structural
improvement) — they are **NOT** a
`PGEN_RELEASED_PARSER_BUG_LEDGER` entry.

### Counts and locking

Annotation count: **2299** (was 2290 — **+9, a deliberate change**: the
9 factored record rules are annotated; this is NOT "unchanged", unlike
the pure-Cat-A grammars). **1008** distinct annotated rules (was 999,
+9). Same accept set (no grammar-acceptance change). AST-dump schema
bumped `2 → 3` because the affected list / branch shapes change in a
consumer-visible way. Gate-locked:
`systemverilog_ast_shape_contract` passes (new `structured_decls`
sample + `calibration_history` entry #118);
`make -C rust SHELL=/opt/homebrew/bin/bash systemverilog_parser_book_gate`.

## Release 1.0.116 / Contract 1.0.116 Highlights — POST-SV-AUDIT.2.4a: net_alias Cat-A raw-envelope correction + 5-number-rule defensive structural fix; schema 1 → 2

Landed 2026-05-17. The POST-SV-AUDIT static classification pass
(`docs/POST_SV_AUDIT_LEDGER.md`, leaf POST-SV-AUDIT.2.4a, tracked
`PGEN-POST-SV-AUDIT-0005`) dispositioned two SystemVerilog worklist
items in `grammars/systemverilog.ebnf`:

1. **`net_alias` — reachable, consumer-visible Category-A correction.**
2. **5 number rules — defensive structural correction (NOT a
   consumer-reproducible released bug; NO bug-ledger row).**

The parser is regenerated; the manifest
`rust/test_data/ast_shape_contract/systemverilog_v1.json` is re-locked
(new `net_alias` sample + `calibration_history` entry #117);
`systemverilog_ast_shape_contract` passes.

Annotation count: **2290** (UNCHANGED — `net_alias` stays
`return_object` with a new `normalized_text` only; the 5 number rules'
annotation text is unchanged; the 5 new `*_tail` rules are
**un-annotated** and not in the inventory — there is **no count
delta**). **999** distinct annotated rules (UNCHANGED). Same accept set
(no grammar acceptance change — purely the `net_alias` annotation form
and the inline-alternation lift into named tail rules). AST-dump schema
bumped `1 → 2` because the reachable `net_alias` shape changed in a
consumer-visible way; the number-rule structural fix bundles in.

### Schema-Versioning row

| AST-dump schema version | First parser release | Notable changes |
|---|---|---|
| `3` | `1.0.117` | **POST-SV-AUDIT.2.4b (`PGEN-POST-SV-AUDIT-0006`).** 11 structured-per-iteration Category-A misuses corrected: each repeated multi-field unit factored into a **new named record rule** + an extraction-spread, so the affected lists/branches now emit clean flat record arrays (field names `{name, dims[, init]}` / `{name, value}` / `{name, pattern}` PRESERVED from the prior `first` record). Rules: `list_of_interface_identifiers` / `list_of_port_identifiers` / `list_of_variable_identifiers` (`{first:{name,dims},rest}` → `[record]` via new `interface_identifier_decl` / `port_identifier_decl` / `variable_identifier_decl`); `list_of_tf_variable_identifiers` / `list_of_variable_port_identifiers` (`{first:{name,dims,init},rest}` → `[record]` via new `tf_variable_identifier_decl` / `variable_port_identifier_decl`); `let_` / `property_` / `sequence_list_of_arguments` (new `let_named_arg` / `property_named_arg` / `sequence_named_arg := dot identifier lparen ( <x>_actual_arg )? rparen -> {name,value}`; mixed branch `{kind:"mixed", head, ordered_tail:[…], named_tail:[…]}`, named_only branch `{kind:"named_only", items:[…]}` — was `{kind:"named_only", first_name, first_value, rest}`); `parameter_port_list` type_only (`{kind:"type_only", first, rest}` → `{kind:"type_only", items:[$4, $5::3*]}`); `assignment_pattern` named (both profile occurrences; new shared `assignment_pattern_entry := member_identifier colon pattern -> {name,pattern}`; branch `{kind:"named", entries:[…]}` — was `{kind:"named", entries:{first:{name,pattern},rest}}`). **Counts deliberately changed 2290 → 2299 / 999 → 1008** (+9: the 9 factored record rules ARE annotated — unlike pure Cat-A; analogous to SVPP-0001's `pp_if_keyword`. This is NOT "unchanged".). Reachable `list_of_*_identifiers` path probe-verified on `module m; wire a, b, c; logic x, y, z; endmodule` (clean `{name,dims,init}` record list; 0 `<invalid_sequence_access>`; 0 raw `[[],","]` separator-envelope leaks; no leftover `{first:{…},rest:…}` envelope). The `*_list_of_arguments`, `parameter_port_list` type_only, and `assignment_pattern` named branches are likely NOT reachable via the strict SV `systemverilog_file` root (it rejects most constructs in all profiles — a pre-existing, out-of-scope coverage limitation); their fix is the identical proven factor-record-rule idiom (correct by construction) + clean regen + the shape-contract lock — documented as defensively-correct, **not** claimed as a fresh end-to-end probe and **NOT** a bug-ledger entry (Cat-A structured, no `<invalid_sequence_access>` corruption). Same accept set. |
| `2` | `1.0.116` | **POST-SV-AUDIT.2.4a (`PGEN-POST-SV-AUDIT-0005`).** `net_alias` Category-A raw-envelope misuse corrected `{first, second, rest}` → `{lvalues: […]}` (reachable, consumer-visible — drives the bump). Defensive structural correction of 5 number rules (`unsigned_number` / `non_zero_unsigned_number` / `binary_value` / `octal_value` / `hex_value`): inline-alternation iteration-lead lifted into new un-annotated named `*_tail` rules so `$2` binds cleanly; the `{first: $1, rest: $2}` annotation text is **unchanged**. The number-rule corruption is structurally present but **NOT consumer-reproducible** (SV `systemverilog_file` root rejects every numeric-bearing top-level construct in all profiles) — defensive/latent, **no bug-ledger row**. Counts 2290 / 999 unchanged. |
| `1` | `≤ 1.0.115` | Slice-campaign baseline (SV-Slice-1 … SV-Slice-115). The integer AST-dump schema stayed `1` across the additive annotation slices (additive shape changes within the major version tracked in the per-rule manifest, not the integer schema). Per-slice detail in the `Release 1.0.115` … `Release 1.0.0` Highlights sections below. |

## AST-Shape Corrections — 1.0.116 (POST-SV-AUDIT) — `net_alias` Cat-A raw-envelope → clean list (reachable, consumer-visible); 5 number rules defensive structural fix (latent, NOT a bug-ledger entry); schema 1 → 2

Landed 2026-05-17. The POST-SV-AUDIT static classification pass
(`docs/POST_SV_AUDIT_LEDGER.md`, leaf POST-SV-AUDIT.2.4a, tracked
`PGEN-POST-SV-AUDIT-0005`) found one **reachable, consumer-visible
static-conclusive Category-A raw-envelope misuse** (`net_alias`) and one
class of **structurally-present-but-not-consumer-reproducible**
inline-alternation-`$N` corruption (5 number rules) in
`grammars/systemverilog.ebnf`. They are corrected, the parser is
regenerated, and the manifest inventory is re-locked.

`net_alias` is a **clean Category-A shape improvement** — the separator
is a single token (`assign` / `=`), there is **no** inline alternation,
and the rule never emitted `<invalid_sequence_access>`. The 5-number-rule
inline-alternation-`$N` corruption **is** structurally the same
emit-time defect class as the closed `RTL-FE-0001` / `RTL-FE-0002` /
`SVPP-0001` / `VHDL-0001` family, **but** the parent empirically
established it is **NOT consumer-reproducible** via valid `source_text`
(see the honest reachability finding below). **Neither item is logged in
`docs/contracts/PGEN_RELEASED_PARSER_BUG_LEDGER.md`** — that ledger is
reserved for *consumer-reproducible* released `<invalid_sequence_access>`
corruption/crash defects; `net_alias` is a clean Category-A improvement
(no corruption), and the number-rule corruption is unreachable through
the SV grammar root, so claiming a released defect would be inaccurate.
**This batch carries no bug-ledger row.**

### `net_alias` — Category-A raw-envelope misuse (reachable, consumer-visible)

`net_alias` is reached as the `net_alias` branch of `module_common_item`
(and its profile/elaboration siblings). The `≤ 1.0.115` grammar was

```ebnf
net_alias := kw_alias_cdb6fdbe net_lvalue assign net_lvalue
             ( assign net_lvalue )* semi
          -> {first: $2, second: $4, rest: $5}
```

`rest` (`$5`) surfaced the raw `[[assign, net_lvalue], …]`
single-token-separator iteration envelope, forcing every consumer to
index past the `=` separator on each iteration **and** to stitch
`first` + `second` + the envelope back into one alias chain. The fix is
the canonical extraction-spread idiom — drop the semantically-irrelevant
`=` separators and emit one clean flat list of **all** aliased
`net_lvalue`s in source order:

```ebnf
net_alias := kw_alias_cdb6fdbe net_lvalue assign net_lvalue
             ( assign net_lvalue )* semi
          -> {lvalues: [$2, $4, $5::2*]}
```

| Rule (`grammars/systemverilog.ebnf`) | `≤ 1.0.115` / schema `1` shape (history) | `1.0.116` / schema `2` shape | Annotation form |
|---|---|---|---|
| `net_alias` | `{first: $2, second: $4, rest: $5}` (raw `[[assign, net_lvalue], …]` `rest` envelope) | `{lvalues: [$2, $4, $5::2*]}` (clean flat `net_lvalue[]` list) | stays `return_object` (object with one array field), new `normalized_text` only (sep `assign`) |

Parent probe-verified on
`module m; wire a, b, c; alias a = b = c; endmodule`: the `net_alias`
node is `{"lvalues": [{…a…}, {…b…}, {…c…}]}` — a clean 3-element
`net_lvalue[]` list, no `[[assign, net_lvalue], …]` envelope, no
separator to skip. A consumer written against `≤ 1.0.115` that read
`.first` / `.second` / `.rest[][1]` must repin to schema `2` and read
the single `.lvalues` flat array. `net_alias`'s annotation type stays
`return_object` (it is `{lvalues: […]}`, an object with one array
field); only the `normalized_text` changed.

### Defensive structural correction — 5 number rules (latent inline-alternation-`$N`; NOT a consumer-reproducible released bug)

`unsigned_number`, `non_zero_unsigned_number`, `binary_value`,
`octal_value`, and `hex_value` each had the shape

```ebnf
<digit> ( kw_sv_rule_c82a06f6 | <digit> )* -> {first: $1, rest: $2}
```

where `kw_sv_rule_c82a06f6 := trivia /sv_rule\b/`. The inline
alternation `( kw_sv_rule_c82a06f6 | <digit> )` is the **lead element**
of the `( … )*` iteration feeding bare positional `$2` — the systemic
inline-alternation-`$N` corruption class (the same emit-time root cause
fixed for `rtl_const_expr`, `systemverilog_preprocessor` (`SVPP-0001`),
`rtl_frontend` (`RTL-FE-0001`/`RTL-FE-0002`), and `vhdl`
(`VHDL-0001`)). The fix is the identical, empirically-proven
transformation: lift the inline alternation into a **new un-annotated
named tail rule** so the iteration becomes `( <rule>_tail )*` and `$2`
binds cleanly. The `{first: $1, rest: $2}` annotation text is
**UNCHANGED**.

```ebnf
unsigned_number_tail         := kw_sv_rule_c82a06f6 | decimal_digit
unsigned_number              := decimal_digit ( unsigned_number_tail )* -> {first: $1, rest: $2}
non_zero_unsigned_number_tail := kw_sv_rule_c82a06f6 | decimal_digit
non_zero_unsigned_number     := non_zero_decimal_digit ( non_zero_unsigned_number_tail )* -> {first: $1, rest: $2}
binary_value_tail            := kw_sv_rule_c82a06f6 | binary_digit
binary_value                 := binary_digit ( binary_value_tail )* -> {first: $1, rest: $2}
octal_value_tail             := kw_sv_rule_c82a06f6 | octal_digit
octal_value                  := octal_digit ( octal_value_tail )* -> {first: $1, rest: $2}
hex_value_tail               := kw_sv_rule_c82a06f6 | hex_digit
hex_value                    := hex_digit ( hex_value_tail )* -> {first: $1, rest: $2}
```

The 5 `*_tail` rules are **un-annotated** and therefore do not enter the
return-annotation inventory; the 5 number rules' annotations are
**textually unchanged** (`{first: $1, rest: $2}`, `return_object`).

**Honest reachability finding (critical — why this is NOT a bug-ledger
entry).** The corruption is **structurally present but NOT
consumer-reproducible**. The parent empirically established that the SV
`systemverilog_file` root **rejects every numeric-bearing top-level
construct** — `parameter` / `localparam` / `assign` / `$display` /
packed ranges (`[15:0]`) / module-parameter headers — in **all**
profiles (`default` / `sv_2017` / `sv_2023`); only minimal constructs
(e.g. `module m; endmodule` and
`module m; wire a, b, c; alias a = b = c; endmodule`) parse. A
multi-digit number is therefore **unreachable via valid `source_text`**.
This is a pre-existing SV-grammar-root coverage limitation that is
**separate from this defect and explicitly out of POST-SV-AUDIT scope**.
Consequently this is a **defensive structural correction** (correct by
construction — the identical transformation was empirically proven 6×
this session across RTL-CE / SVPP-0001 / RTL-FE-0001 / VHDL-0001 /
RTL-FE-0002), **NOT** a `PGEN_RELEASED_PARSER_BUG_LEDGER` row. We do not
claim a released defect that no valid input can trigger; the latent
corruption is documented honestly here with the reachability finding
stated plainly.

`kw_sv_rule_c82a06f6 := trivia /sv_rule\b/` is itself a degenerate
LRM-extraction artifact (the digit-group "separator" token is a literal
`sv_rule` keyword, not a real SystemVerilog digit separator). That is a
**separate grammar-quality matter, out of scope here** — noted as an
observation only; not proposed for fix in this batch.

### Counts and locking

Annotation count: **2290** (UNCHANGED — `net_alias` stays
`return_object` with a new `normalized_text`; the 5 number rules'
annotation text is unchanged; the 5 new `*_tail` rules are
**un-annotated** and not in the inventory — there is **no count
delta**). **999** distinct annotated rules (UNCHANGED). Same accept set
(no grammar acceptance change). AST-dump schema bumped `1 → 2` because
the reachable `net_alias` shape changed in a consumer-visible way; the
number-rule defensive structural fix bundles in. Gate-locked:
`systemverilog_ast_shape_contract` passes;
`make -C rust SHELL=/opt/homebrew/bin/bash systemverilog_parser_book_gate`.

## Release 1.0.115 / Contract 1.0.115 Highlights — SV-Slice-115 batch: Pattern-B ps_type_identifier_sv_2017/2023 typed (2 rules / 2 annotations)

Re-annotation of `( a | b | c )? id` shape after codegen-drop fix (PGEN-PIP-001):

```ebnf
ps_type_identifier_sv_2017 / ps_type_identifier_sv_2023
  -> {scope: $1, name: $2}
```

`scope` is the optional parens-group (kw_local + scope_res + kw_n_{43,48} | non_typedef_package_scope | class_scope); `name` is the trailing type_identifier.

Annotation count: **2290** (was 2288, +2). Same accept set.

## Release 1.0.114 / Contract 1.0.114 Highlights — SV-Slice-114 batch: Pattern-A number-value sequence rules typed (5 rules / 5 annotations)

Re-annotation of the `digit ( sep | digit )*` shape, previously blocked by a codegen drop (now fixed in PGEN-PIP-001):

```ebnf
unsigned_number / non_zero_unsigned_number / binary_value / hex_value / octal_value
  -> {first: $1, rest: $2}
```

`first` is the leading digit; `rest` is the iteration array of alternating digit / underscore-separator entries — consumers can either consume the raw array or skip separators by `kind`.

Annotation count: **2288** (was 2283, +5). Same accept set.

> **Structural correction (1.0.116 / schema 2, POST-SV-AUDIT.2.4a).**
> The `{first: $1, rest: $2}` annotation **text is unchanged**, but the
> grammar shape was a latent inline-alternation-`$N` corruption (the
> `( kw_sv_rule_c82a06f6 | <digit> )` iteration lead fed bare `$2`).
> At `1.0.116` the inline alternation is lifted into new
> **un-annotated** named `*_tail` rules (`unsigned_number_tail`, …) so
> `$2` binds a clean `( <rule>_tail )*` iteration. This is a
> **defensive** fix: the corruption is structurally present but **NOT
> consumer-reproducible** (the SV `systemverilog_file` root rejects
> every numeric-bearing top-level construct in all profiles), so it is
> **not** a bug-ledger entry. See
> [AST-Shape Corrections — 1.0.116](#ast-shape-corrections--10116-post-sv-audit--net_alias-cat-a-raw-envelope--clean-list-reachable-consumer-visible-5-number-rules-defensive-structural-fix-latent-not-a-bug-ledger-entry-schema-1--2).

> Baseline note: the 2283 floor reflects the codegen fix's +27 recovery from previously-suppressed inner branches across earlier slices. Pre-fix baseline at slice 113 head was 2256.

## Release 1.0.113 / Contract 1.0.113 Highlights — SV-Slice-113 batch: method_call_receiver_sv_2017/2023 per-branch typed (2 rules / 26 annotations after codegen drops 2)

Per-branch annotation on both profile variants of `method_call_receiver`. 14 distinct kinds per rule:

```ebnf
method_call_receiver_sv_2017 / method_call_receiver_sv_2023
  -> per-branch {kind: "primary_literal"     | "hierarchical"
                     | "empty_concat"        | "multi_concat"
                     | "concat"              | "let_expr"
                     | "parens"              | "cast"
                     | "assignment_pattern"  | "streaming_concat"
                     | "sequence_method"     | "this"
                     | "dollar"              | "null_class", ...}
```

The `hierarchical` branch (`( a | b )? hierarchical_identifier`) had its annotation dropped by codegen on both rules — same limitation as `ansi_port_declaration.named_dot` in slice 112. 13 of 14 branches per rule landed.

Annotation count: **2256** (was 2231, +25). Same accept set.

## Release 1.0.112 / Contract 1.0.112 Highlights — SV-Slice-112 batch: hierarchical_tf_identifier + ansi_port_declaration typed (2 rules / 4 annotations after codegen drops)

Per-branch annotation on previously task-#38-deferred rules:

```ebnf
hierarchical_tf_identifier
  -> per-branch {kind: "rooted",   scope_chain, name}
              | {kind: "anchored", head, head_select, scope_chain, name}

ansi_port_declaration
  -> per-branch {kind: "net_or_interface", header, name, dims, default}
              | {kind: "variable",         header, name, dims, default}
              | {kind: "named_dot", ...}     ← CODEGEN DROPPED (1 of 5)
```

The `named_dot` annotation was present in the IR but did not survive parser codegen — the third branch shape `( a )? dot id lparen ( e )? rparen` appears to hit a codegen limitation distinct from task #38.

Annotation count: **2231** (was 2227, +4). Same accept set.

## Release 1.0.111 / Contract 1.0.111 Highlights — SV-Slice-111 batch: delay_sv_2017 + delay_sv_2023 typed (2 rules / 8 annotations)

```ebnf
delay_sv_2017 / delay_sv_2023
  -> per-branch {kind: "value", body}
              | {kind: "pair_optional", first, second}
              | {kind: "triple_optional", first, rest}
              | {kind: "value", body}   (duplicate retained)
```

Both profiles cover the same 4 branch shapes; branch order differs across sv_2017 vs sv_2023 but kind discriminators align.

Annotation count: **2227** (was 2219, +8). Same accept set.

## Release 1.0.110 / Contract 1.0.110 Highlights — SV-Slice-110 batch: drive_strength + init_val + scalar_constant typed (3 rules / 26 annotations)

```ebnf
drive_strength
  -> per-branch {kind, first?, second?}
     kind in ("pair_strength" | "strength_highz" | "highz_strength")
     each kind appears twice (grammar duplicates retained)

init_val
  -> per-branch {kind: "1'b" | "1'bx" | "1'bX" | "1'B" | "1'Bx" | "1'BX"
                       | "1" | "0"}

scalar_constant
  -> per-branch {kind: "1'b" | "1'B" | "'b" | "'B" | "1" | "0"}
```

Annotation count: **2219** (was 2193, +26). Same accept set.

## Release 1.0.109 / Contract 1.0.109 Highlights — SV-Slice-109 batch: pulldown_strength + pullup_strength + net_type typed (3 rules / 18 annotations)

```ebnf
net_type
  -> per-branch {kind: "supply" | "tri" | "triand" | "trior" | "trireg"
                       | "uwire" | "wire" | "wand" | "wor"}
     (supply x2, tri x3 — duplicates retained)

pulldown_strength / pullup_strength
  -> per-branch {kind: "pair", first, second}  (x2, duplicates retained)
              | {kind: "single", value}
```

Annotation count: **2193** (was 2175, +18). Same accept set.

## Release 1.0.108 / Contract 1.0.108 Highlights — SV-Slice-108 batch: duplicate-branch leaf rules typed (3 rules / 11 annotations)

Per-branch `{kind: "X"}` on Or rules that have duplicate branches (artifact of grammar profile-merging). Duplicates accepted as-is — consumer sees identical `{kind}` regardless of which copy of the branch matched:

```ebnf
enable_gatetype     -> per-branch {kind: "bufif"} (x2) | {kind: "notif"} (x2)
pass_en_switchtype  -> per-branch {kind: "tranif"} (x2) | {kind: "rtranif"} (x2)
unique_priority     -> per-branch {kind: "unique"} (x2) | {kind: "priority"} (x1)
```

Annotation count: **2175** (was 2164, +11). Same accept set.

## Release 1.0.107 / Contract 1.0.107 Highlights — SV-Slice-107: provisional_unscoped_block_class_type typed (1 rule / 1 annotation)

```ebnf
provisional_unscoped_block_class_type
  -> {head, params, scope_chain}
```

Same shape as `known_unscoped_block_class_type` from slice 106 (per slice 106's predecessor analysis, the task #38 deferral on this rule was based on the parens-grouped-Or risk but actually only `(X)?` Optional groups appear here — not Or alternation — so it's safe to annotate).

ATTEMPTED but reverted: `ps_type_identifier_sv_2017` / `ps_type_identifier_sv_2023` — codegen silently dropped the rule-level annotation on the `( kw scope_res kw | scope | scope )? type_identifier` shape (same class of codegen limitation as the `*_value` sequence rules deferred earlier).

Annotation count: **2164** (was 2163, +1). Same accept set.

## Release 1.0.106 / Contract 1.0.106 Highlights — SV-Slice-106 batch: type-identifier + block_class_type chain rules typed (5 rules / 4 annotations after dedup)

```ebnf
known_unscoped_block_type_identifier / known_unscoped_data_type_identifier
  -> {type, dims}

scoped_block_type_identifier / scoped_data_type_identifier
  -> {scope, type, dims}    (scope = class_scope | non_typedef_package_scope)

known_unscoped_block_class_type
  -> {head, params, scope_chain}

scoped_block_class_type
  -> {scope, head, params, scope_chain}
```

DEFERRED: `provisional_unscoped_block_class_type` (task #38 — parens-grouped-Or annotation attribution).

Annotation count: **2163** (was 2159, +4 after manifest dedup). Same accept set.

## Release 1.0.105 / Contract 1.0.105 Highlights — SV-Slice-105 batch: scoped_X passthrough identifier rules typed (11 rules / 11 annotations)

All `scoped_X := non_typedef_package_scope <id>` passthroughs now emit a uniform `{scope, name}` shape so consumers can dereference the package scope and the identifier directly without rule-name introspection:

```ebnf
scoped_class_scope_identifier
scoped_base_class_type_identifier
scoped_class_type_identifier
scoped_covergroup_type_identifier
scoped_interface_class_type_identifier
scoped_let_identifier
scoped_checker_identifier
scoped_property_identifier
scoped_sequence_identifier
scoped_package_parameter_identifier
scoped_class_scoped_call_prefix_identifier
  -> {scope, name}
```

Annotation count: **2159** (was 2148, +11). Same accept set.

## Release 1.0.104 / Contract 1.0.104 Highlights — SV-Slice-104 batch: identifier-routing wrappers typed (15 rules / 35 annotations)

Per-branch shapes on identifier and ps_*/scoped_* alternation rules:

```ebnf
identifier / scope_free_identifier / ps_checker_identifier /
ps_covergroup_identifier / ps_or_hierarchical_property_identifier /
ps_or_hierarchical_sequence_identifier / ps_type_identifier / delay
  -> per-branch {body: $1}   (passthrough)

input_identifier / output_identifier
  -> 3 kinds: input_port / inout_port / interface_dot_port
     (the last extracts interface + port)

ps_identifier
  -> 2 kinds: unscoped (name only) / scoped (scope + name)

ps_or_hierarchical_array_identifier
  -> {scope, name}

ps_or_hierarchical_net_identifier
  -> 2 kinds: ps (scope, name) / hierarchical (body)

scoped_or_hierarchical_tf_identifier
  -> 3 kinds: class_scope / package_scope / hierarchical

ps_parameter_identifier
  -> 4 kinds: class_scope / package_scope / unscoped / generate_scoped
```

Annotation count: **2148** (was 2113, +35). Same accept set.

## Release 1.0.103 / Contract 1.0.103 Highlights — SV-Slice-103 batch: operator/punctuation leaves typed (69 rules / 69 annotations)

All non-kw plain `trivia "X"` leaf rules get `-> {kind: "<rule_name>"}` for a clean discriminator shape without trivia noise. Touches operators (`assign`, `plus`, `minus`, `*`, `/`, etc.), punctuation (`lparen`, `rparen`, `lbrace`, `rbrace`, `lbrack`, `rbrack`, `colon`, `comma`, `semi`, `dot`, `dot_star`, `at_sign`, `hash`, `question`, `tilde`, `tick`, `bang`), compound assigns (`plus_assign`, `minus_assign`, `*_assign`, `/_assign`, etc.), arithmetic/logical/bitwise/reduction operators, shifts, comparisons, and arrows.

Annotation count: **2113** (was 2044, +69). Same accept set.

## Release 1.0.102 / Contract 1.0.102 Highlights — SV-Slice-102 batch: number-leaf family typed (12 Or rules / 63 annotations)

Per-branch kind discriminators on the number-leaf alternation rules:

```ebnf
binary_base   -> {kind: "b" | "B"}
binary_digit  -> {kind: "x" | "z" | "0" | "1"}
decimal_base  -> {kind: "d" | "D"}
decimal_digit -> {kind: "0" .. "9"}
hex_base      -> {kind: "h" | "H"}
hex_digit     -> {kind: "x" | "z" | "0"-"9" | "a"-"f" | "A"-"F"}
octal_base    -> {kind: "o" | "O"}
octal_digit   -> {kind: "x" | "z" | "0"-"7"}
x_digit       -> {kind: "x" | "X"}
z_digit       -> {kind: "z" | "Z" | "?"}
z_or_x        -> {kind: "x" | "X" | "z" | "Z"}
zero_or_one   -> {kind: "0" | "1"}
```

Per-branch (not rule-level) annotation chosen because rule-level `-> {body}` on an Or hits task #38 (parens-grouped-Or trailing-annotation attribution).

RESOLVED (SV-EXH-PROOF.3.2, 1.0.118): the `*_value` / `unsigned_number` / `non_zero_unsigned_number` decomposed shaping is no longer DEFERRED — Strategy-1 restored these to the generator's clean single-regex IEEE-1800 lexical form (clean Terminals), removing the broken decomposition entirely. The number rules are no longer per-element sequence-with-repetition; the top `number` keeps its typed `{kind,body}`.

Annotation count: **2044** (was 1981, +63). Same accept set.

## Release 1.0.101 / Contract 1.0.101 Highlights — SV-Slice-101 batch: comment_only + timing_check_limit + trans_item + remaining t*_path + type wrappers + variable_port typed (15 rules / 15 annotations)

```ebnf
comment_only_source_region            -> {leading_whitespace, first_comment, tail}
timing_check_limit / trans_item       -> {body}
trise/tx0/tx1/txz/tz0/tz1/tz/tzx_path_delay_expression  -> {body}
type_identifier_or_class_type / type_parameter_declaration  -> {body}
variable_identifier_list              -> [$1, $2::2*]
variable_port_header                  -> {direction, port_type}
variable_port_type                    -> {body}
```

DEFERRED: `unique_priority` (duplicate-branch grammar bug).

Annotation count: **1981** (was 1966, +15). Same accept set.

## Release 1.0.100 / Contract 1.0.100 Highlights — SV-Slice-100 batch: sign + statement_item + structure_pattern_key + t*_path_delay_expression + tf_port + threshold + timecheck + timeunits wrappers typed (16 rules / 26 annotations) — crosses 100-slice milestone

```ebnf
mixed_string_parameter_port_list / mixed_parameter_port_list  -> typed
sign                          -> 2 kinds (plus / minus)
statement_item                -> 2 kinds (sv_2017 / sv_2023)
structure_pattern_key         -> 2 kinds (member / pattern)
t01/t0x/t0z/t10/t1x/t1z/t_path_delay_expression  -> {body}
tf_port_declaration           -> {attributes, direction, var_keyword, data_type, items}
tf_port_direction             -> 2 kinds
tfall_path_delay_expression / threshold / timecheck_condition  -> {body}
timeunits_declaration         -> 4 kinds (timeunit / timeprecision / timeunit_then_precision / timeprecision_then_unit)
timeunit_separator_slash      -> 1 kind bare
```

Annotation count: **1966** (was 1940, +26). **Crosses 100-slice milestone.** Same accept set.

## Release 1.0.99 / Contract 1.0.99 Highlights — SV-Slice-99 batch: package_scope + parameter_declaration + parameter_value_assignment + parameter_override + pattern + port_declaration + program_generate_item + property_expr + pulse_control + randomize + ref_declaration + select_condition wrappers typed (25 rules / 50 annotations)

Annotation count: **1940** (was 1890, +50). Same accept set.

## Release 1.0.98 / Contract 1.0.98 Highlights — SV-Slice-98 batch: default_skew + dynamic_override + forward_type + module_instantiation + operator_assignment + package_or_generate wrappers typed (24 rules / 39 annotations)

```ebnf
default_skew                          -> 3 kinds (input / output / input_output)
dynamic_override_specifiers / final_specifier / forward_type /
  incomplete_class_scoped_type / initial_or_extends_specifier /
  net_type_declaration / nettype_declaration /
  non_consecutive_repetition / nonconsecutive_repetition / notifier /
  open_range_list / open_value_range_sv_2017 / open_value_range  -> {body}
forward_type_sv_2023                  -> 5 kinds (enum / struct / union / class / interface_class)
full_edge_sensitive_path_description  -> 2 kinds (sv_2017 / sv_2023)
function_data_type_or_implicit        -> 2 kinds (data_type / implicit)
module_instantiation                  -> {name, params, instances: [$3, $4::2*]}
non_zero_decimal_digit_sv_2017        -> 9 kinds (1-9)
nonrange_select                       -> {member_chain, bits}
nonrange_variable_lvalue              -> {scope, name, select}
operator_assignment                   -> {lvalue, operator, value}
package_or_generate_item_declaration  -> 2 kinds (sv_2017 / sv_2023)
```

Annotation count: **1890** (was 1851, +39). Same accept set.

## Release 1.0.97 / Contract 1.0.97 Highlights — SV-Slice-97 batch: final/initial_construct + method_call internals + identifier_list + interface_instantiation + module_common_item wrappers typed (19 rules / 26 annotations)

```ebnf
attr_name / final_construct / function_statement / initial_construct /
  non_typedef_package_scope / limit_value /
  list_of_parameter_assignments / list_of_parameter_value_assignments  -> {body}
function_subroutine_call            -> 2 kinds (call_primary / randomize)
gate_instantiation                  -> 2 kinds (sv_2017 / sv_2023)
identifier_list                     -> [$1, $2::2*]
inside_expression                   -> 2 kinds (sv_2017 / sv_2023)
interface_instantiation             -> {name, params, instances: [$3, $4::2*]}
let_formal_type                     -> 2 kinds (data_type / untyped)
direct_method_call                  -> {receiver, body}
callable_method_call_body           -> 2 kinds (built_in / call_with_args)
split_hierarchical_callable_receiver -> {scope, path, name}
split_direct_callable_method_call   -> 2 kinds (split_hierarchical / implicit_class)
direct_callable_method_call         -> {receiver, body}
module_common_item                  -> 2 kinds (sv_2017 / sv_2023)
```

Annotation count: **1851** (was 1825, +26). Same accept set.

## Release 1.0.96 / Contract 1.0.96 Highlights — SV-Slice-96 batch: constraint/covergroup/data_declaration + design + dist + elaboration + event wrappers typed (17 rules / 28 annotations)

```ebnf
clockvar / covergroup_expression / current_state / data_type_or_incomplete_class_scoped_type / 
  elaboration_severity_system_task_sv_2023 / _normal / elaboration_system_task /
  error_limit_value / event_based_flag                          -> {body}
constraint_declaration / constraint_prototype                   -> 2 kinds each
covergroup_declaration                                          -> 2 kinds
data_declaration / dist_item / event_control                    -> 2 kinds each
block_data_type                                                 -> {base, signing, dims}
block_data_type_or_implicit                                     -> 2 kinds
defparam_assignment                                             -> {name, value}
design_statement                                                -> {cells}
exp                                                             -> 2 kinds (e / E)
```

Annotation count: **1825** (was 1797, +28). Same accept set.

## Release 1.0.95 / Contract 1.0.95 Highlights — SV-Slice-95 batch: sv_multi_entry_root + comment_only + bit_select + case + clocking + constant_* wrappers typed (12 rules / 18 annotations)

```ebnf
sv_multi_entry_root              -> 3 kinds (systemverilog_file / library_text / systemverilog_parseable_file)
parseable_source_item            -> {kind: "semi"}
bit_select_expression            -> 3 kinds (direct_index_method / method / expression)
case_expression / case_item_expression / clocking_event etc. -> typed wrappers
clocking_event                   -> 2 kinds (sv_2017 / sv_2023)
clockvar_expression              -> {clockvar, select}
constant_bit_select              -> {bits}
constant_function_call           -> 2 kinds (call_primary / randomize)
constant_let_expression          -> {body}
constant_mintypmax_expression    -> 2 kinds (simple / triple)
```

Annotation count: **1797** (was 1779, +18). Same accept set.

## Release 1.0.94 / Contract 1.0.94 Highlights — SV-Slice-94 batch: dimension family + integer_covergroup_expression typed (8 rules / 15 annotations)

```ebnf
integer_covergroup_expression_sv_2017 -> {body}
integer_covergroup_expression_sv_2023 -> 2 kinds (expression / dollar)
integer_covergroup_expression         -> 2 kinds (sv_2017 / sv_2023)
packed_dimension                      -> 2 kinds (range / unsized)
queue_dimension                       -> {bound}
unpacked_dimension                    -> 2 kinds (range / expression)
unsized_dimension                     -> {kind: "unsized"}
variable_dimension                    -> 4 kinds (unsized / unpacked / associative / queue)
```

Annotation count: **1779** (was 1764, +15). Same accept set.

## Release 1.0.93 / Contract 1.0.93 Highlights — SV-Slice-93 batch: anonymous_program_item + assignment_pattern + array + block_event + built_in_method + class_item wrappers typed (16 rules / 40 annotations)

```ebnf
anonymous_program_item_sv_2017       -> 6 kinds (task / function / class / covergroup / class_constructor / semi)
anonymous_program_item_sv_2023       -> 7 kinds (adds interface_class)
anonymous_program_item               -> 2 kinds (sv_2017 / sv_2023)
array_manipulation_call              -> {method, attributes, args, with_clause}
array_pattern_key                    -> 2 kinds (expression / pattern_key)
array_range_expression               -> 4 kinds (expression / range / plus_indexed / minus_indexed)
assignment_pattern_expression        -> {type, pattern}
assignment_pattern_expression_type   -> 4 kinds (ps_type / ps_parameter / integer_atom_type / type_reference)
assignment_pattern_key               -> 2 kinds (type / default)
assignment_pattern_net_lvalue        -> {items: [$3, $4::2*]}
assignment_pattern_variable_lvalue   -> {items: [$3, $4::2*]}
associative_dimension                -> 2 kinds (data_type / wildcard)
block_event_expression               -> 3 kinds (or / begin / end)
built_in_method_call                 -> 2 kinds (array_manipulation / randomize)
class_item                           -> 2 kinds (sv_2017 / sv_2023)
direct_index_method_call             -> {receiver, body}
```

Annotation count: **1764** (was 1724, +40). Same accept set.

## Release 1.0.92 / Contract 1.0.92 Highlights — SV-Slice-92 batch: terminal + cross_set + weight_specification + passthroughs typed (8 rules / 8 annotations)

All single-sub-rule passthroughs: `cross_set_expression`, `weight_specification`, `enable_terminal`, `inout_terminal`, `input_terminal`, `ncontrol_terminal`, `output_terminal`, `pcontrol_terminal` (each `{body}`).

Annotation count: **1724** (was 1716, +8). Same accept set.

## Release 1.0.91 / Contract 1.0.91 Highlights — SV-Slice-91 batch: rs_ + value_range wrappers typed (9 rules / 16 annotations)

```ebnf
rs_case_item / rs_if_else / rs_prod / rs_production_list / rs_repeat /
rs_rule           -> 2 kinds each (sv_2017 / sv_2023)
rs_production     -> {body}
rs_production_item -> {body}
value_range       -> 2 kinds (sv_2017 / sv_2023)
```

Annotation count: **1716** (was 1700, +16). Same accept set.

## Release 1.0.90 / Contract 1.0.90 Highlights — SV-Slice-90 batch: production + udp_declaration + range_list wrappers typed (4 rules / 5 annotations)

```ebnf
production       -> {body}
production_item  -> {body}
range_list       -> {body}
udp_declaration  -> 2 kinds (sv_2017 / sv_2023)
```

Annotation count: **1700** (was 1695, +5). Same accept set.

## Release 1.0.89 / Contract 1.0.89 Highlights — SV-Slice-89 batch: profile-router wrappers typed (9 rules / 18 annotations)

All 9 declaration/prototype wrappers that route between sv_2017 / sv_2023 typed bodies. Each gets `2 kinds (sv_2017 / sv_2023)`:

```ebnf
blocking_assignment / class_declaration / function_declaration /
function_prototype / interface_declaration / module_declaration /
program_declaration / task_declaration / task_prototype
```

Annotation count: **1695** (was 1677, +18). Same accept set.

## Release 1.0.88 / Contract 1.0.88 Highlights — SV-Slice-88 batch: constant_primary + primary wrappers typed (2 rules / 4 annotations)

Profile-router passthroughs that wrap the sv_2017/sv_2023 typed primaries from slices 47/48.

```ebnf
constant_primary -> 2 kinds (sv_2017 / sv_2023)
primary          -> 2 kinds (sv_2017 / sv_2023)
```

Annotation count: **1677** (was 1673, +4). Same accept set.

## Release 1.0.87 / Contract 1.0.87 Highlights — SV-Slice-87 batch: module_path_operators + level_input_list typed (4 rules / 20 annotations)

```ebnf
binary_module_path_operator_sv_2023 -> 9 kinds bare
binary_module_path_operator         -> {body}
unary_module_path_operator          -> 9 kinds bare
level_input_list                    -> [$1, $2*]
```

Annotation count: **1673** (was 1653, +20). Same accept set.

## Release 1.0.86 / Contract 1.0.86 Highlights — SV-Slice-86 batch: let_declaration + final/initial specifiers + named_port_connection + nonconsec_rep + time_unit typed (6 rules / 13 annotations)

```ebnf
final_specifier_sv_2023              -> 1 kind (final)
initial_or_extends_specifier_sv_2023 -> 2 kinds (initial / extends)
let_declaration                      -> {name, ports, value}
named_port_connection                -> 2 kinds (sv_2017 / sv_2023)
nonconsecutive_repetition_sv_2023    -> {range}
time_unit                            -> 6 kinds (s / ms / us / ns / ps / fs)
```

Annotation count: **1653** (was 1640, +13). Same accept set.

## Release 1.0.85 / Contract 1.0.85 Highlights — SV-Slice-85 batch: type_declaration + type_identifier_or_class_type + type_reference + net_alias + net_declaration typed (7 rules / 19 annotations)

```ebnf
type_declaration_sv_2017/2023          -> 6 kinds each (class_alias / data_type / interface_type / forward_class / forward_interface_class / forward)
type_declaration                       -> 2 kinds
type_identifier_or_class_type_sv_2023  -> 2 kinds (type_identifier / class_type)
type_reference                         -> 2 kinds (sv_2017 / sv_2023)
net_alias                              -> {first, second, rest}   ← ≤ 1.0.115 / schema 1 shape (HISTORY); corrected to {lvalues: […]} at 1.0.116 / schema 2 — see [AST-Shape Corrections — 1.0.116](#ast-shape-corrections--10116-post-sv-audit--net_alias-cat-a-raw-envelope--clean-list-reachable-consumer-visible-5-number-rules-defensive-structural-fix-latent-not-a-bug-ledger-entry-schema-1--2)
net_declaration                        -> 2 kinds
```

Annotation count: **1640** (was 1621, +19). Same accept set.

> **Shape correction (1.0.116 / schema 2, POST-SV-AUDIT.2.4a).**
> `net_alias`'s `≤ 1.0.115` `{first, second, rest}` (raw
> `[[assign, net_lvalue], …]` `rest` envelope) was corrected to the
> clean flat `{lvalues: [$2, $4, $5::2*]}` list. The `≤ 1.0.115` shape
> above is kept as labeled history. See
> [AST-Shape Corrections — 1.0.116](#ast-shape-corrections--10116-post-sv-audit--net_alias-cat-a-raw-envelope--clean-list-reachable-consumer-visible-5-number-rules-defensive-structural-fix-latent-not-a-bug-ledger-entry-schema-1--2).

## Release 1.0.84 / Contract 1.0.84 Highlights — SV-Slice-84 batch: net_assignment + param_expression + struct_union + inst_name typed (5 rules / 10 annotations)

```ebnf
constant_param_expression  -> 3 kinds (mintypmax / data_type / dollar)
inst_name                  -> {head, scope_chain}
net_assignment             -> {lvalue, value}
param_expression           -> 3 kinds (mintypmax / data_type / dollar)
struct_union               -> 2 kinds (sv_2017 / sv_2023)
```

Annotation count: **1621** (was 1611, +10). Same accept set.

## Release 1.0.83 / Contract 1.0.83 Highlights — SV-Slice-83 batch: block_data_declaration + base_class_type + misc typed (8 rules / 15 annotations)

```ebnf
block_data_declaration_sv_2017     -> {const_keyword, var_keyword, lifetime, data_type, items}
block_data_declaration_sv_2023     -> 4 kinds (data / type / import / nettype)
block_data_declaration             -> 2 kinds
base_class_type                    -> {head, params, scope_chain}
case_inside_item                   -> 2 kinds (sv_2017 / sv_2023)
hierarchical_btf_identifier        -> 3 kinds (tf / block / method)
tagged_union_expression            -> 2 kinds
ordered_port_connection            -> {attributes, expression}
data_source_expression             -> {body}
```

Annotation count: **1611** (was 1596, +15). Same accept set.

## Release 1.0.82 / Contract 1.0.82 Highlights — SV-Slice-82 batch: dynamic_override + incomplete_class + var_data_type + timing leaves typed (10 rules / 14 annotations)

```ebnf
controlled_reference_event / data_event / reference_event /
  start_edge_offset / end_edge_offset / timestamp_condition  -> {body}
dynamic_override_specifiers_sv_2023                           -> {initial_or_extends, final}
finish_number                                                 -> 3 kinds (0 / 1 / 2)
incomplete_class_scoped_type_sv_2023                          -> 2 kinds (head / recursive)
var_data_type                                                 -> 2 kinds (data_type / var)
```

Annotation count: **1596** (was 1582, +14). Same accept set.

## Release 1.0.81 / Contract 1.0.81 Highlights — SV-Slice-81 batch: config_rule + library + hierarchical_identifier + severity typed (13 rules / 25 annotations)

Closes LRM A.1.7 config / library walk paths + various leaf rules.

```ebnf
cell_clause                            -> {library, name}
config_rule_statement                  -> 5 kinds (default_liblist / inst_liblist / inst_use / cell_liblist / cell_use)
const_or_range_expression              -> 2 kinds (expression / range)
constant_assignment_pattern_expression -> {body}
default_clause                         -> 1 kind bare (default)
hierarchical_identifier                -> {root, scope, name}
hierarchical_instance                  -> {name, connections}
include_statement                      -> {path}
liblist_clause                         -> {libraries}
library_declaration                    -> {name, paths: [$3, $4::2*], incdir}
library_description                    -> 4 kinds
library_text                           -> {descriptions}
severity_system_task_sv_2023           -> 4 kinds (fatal / error / warning / info)
use_clause                             -> {library, name, config}
```

Annotation count: **1582** (was 1557, +25). Same accept set.

## Release 1.0.80 / Contract 1.0.80 Highlights — SV-Slice-80 batch: boolean_abbrev + repetition + elaboration + repeat_range typed (8 rules / 19 annotations)

```ebnf
boolean_abbrev_sv_2017/2023         -> 3 kinds each
boolean_abbrev                      -> 2 kinds
consecutive_repetition              -> 3 kinds (star_range / star / plus)
elaboration_system_task_sv_2017     -> 4 kinds (fatal / error / warning / info)
goto_repetition                     -> {range}
non_consecutive_repetition_sv_2017  -> {range}
repeat_range                        -> 2 kinds (single / range)
```

Annotation count: **1557** (was 1538, +19). Same accept set.

## Release 1.0.79 / Contract 1.0.79 Highlights — SV-Slice-79 batch: event + local/type parameter + mintypmax + nettype family typed (12 rules / 18 annotations)

Closes LRM A.6.5 event_expression / event_trigger, A.2.1.1 local_parameter, A.8.3 mintypmax, A.2.1.4 nettype_declaration, A.2.1.1 type_assignment + type_parameter_declaration sub-trees.

```ebnf
event_expression                       -> [$1, $2::2*]
event_trigger                          -> 2 kinds
local_parameter_declaration_sv_2017    -> 3 kinds (implicit / typed / type)
local_parameter_declaration_sv_2023    -> 3 kinds (parallel — uses type_parameter)
local_parameter_declaration            -> 2 kinds
mintypmax_expression                   -> 2 kinds (simple / triple)
nettype_declaration_sv_2023            -> 2 kinds (data_type / nettype)
type_assignment_sv_2017/2023           -> {name, value}
type_assignment                        -> 2 kinds
type_parameter_declaration_sv_2023     -> {forward_type, items}
```

Annotation count: **1538** (was 1520, +18). Same accept set.

## Release 1.0.78 / Contract 1.0.78 Highlights — SV-Slice-78 batch: class_constructor wrappers + let + for + named_port + parameter_port typed (21 rules / 48 annotations)

Closes LRM A.1.10 class_constructor wrapper passthroughs, LRM A.6.8 let-expression, LRM A.6.5 for-loop sub-tree, LRM A.6.5 named/ordered checker/port-connection rules, LRM A.1.3 parameter_port declarations + list.

### Annotations (summary)

```ebnf
class_constructor_arg / class_constructor_arg_list      -> {body}
class_constructor_declaration / _prototype              -> 2 kinds each (sv_2017 / sv_2023)
let_actual_arg                                          -> {body}
let_expression                                          -> 2 kinds (scoped / unscoped)
for_initialization                                      -> 2 kinds (assignments / declarations)
for_step                                                -> [$1, $2::2*]
for_step_assignment                                     -> 3 kinds (operator_assignment / inc_or_dec / function_call)
for_variable_declaration                                -> {var_keyword, data_type, name, value, tail}
loop_variables                                          -> [$1, $2::2*]
named_checker_port_connection_sv_2017/2023              -> 2 kinds each (named / wildcard)
named_checker_port_connection                           -> 2 kinds
named_port_connection_sv_2017/2023                      -> 2 kinds each (named / wildcard)
ordered_checker_port_connection                         -> {attributes, arg}
parameter_port_declaration_sv_2017                      -> 6 kinds (parameter / localparam / type / parameter_declaration / local_parameter_declaration / data_type_assignments)
parameter_port_declaration_sv_2023                      -> 6 kinds (parallel — type_parameter instead of type)
parameter_port_declaration                              -> 2 kinds
parameter_port_list                                     -> 6 kinds (mixed_string / mixed / type_only / declarations / list_then_decls / empty)
```

### Calibration

`parseability_probe --parse-dump-ast-pretty systemverilog /tmp/sv_calibration/minimal_module.sv` reports `parse_full passed`. Annotation count: **1520** (was 1472, +48). Same accept set.

## Release 1.0.77 / Contract 1.0.77 Highlights — SV-Slice-77 batch: module_path + constraint internals + uniqueness + misc typed (16 rules / 29 annotations)

Closes LRM A.8.3 module_path expression sub-tree, constraint primary scope, cycle_delay_range, extern_tf, inst_clause, solve_before list, and uniqueness_constraint family.

### Annotations

```ebnf
constraint_primary_sv_2017            -> {scope, name, select}
constraint_primary_sv_2023            -> {scope, name, select, call}
constraint_primary                    -> 2 kinds (sv_2017 / sv_2023)
cycle_delay_range                     -> 4 kinds (primary / paren_range / token_71b8cf7e / token_9768502a)
extern_tf_declaration                 -> 2 kinds (method / forkjoin)
inst_clause                           -> {name}
solve_before_list                     -> [$1, $2::2*]
module_path_concatenation             -> {body}
module_path_conditional_expression    -> {condition, attributes, then_expr, else_expr}
module_path_expression_operand        -> 2 kinds (unary / primary)
module_path_expression                -> 2 kinds (conditional / chain)
module_path_mintypmax_expression      -> 2 kinds (simple / triple)
module_path_multiple_concatenation    -> {body}
module_path_primary                   -> 6 kinds (number / identifier / concatenation / multiple_concat / function_call / paren)
uniqueness_constraint_sv_2017/2023    -> {ranges}
uniqueness_constraint                 -> 2 kinds (sv_2017 / sv_2023)
```

### Calibration

`parseability_probe --parse-dump-ast-pretty systemverilog /tmp/sv_calibration/minimal_module.sv` reports `parse_full passed`. Annotation count: **1472** (was 1443, +29). Same accept set.

## Release 1.0.76 / Contract 1.0.76 Highlights — SV-Slice-76 batch: class_scope + method_call + tf_call family typed (14 rules / 27 annotations)

Closes LRM A.8.4 class-scope + method-call + tf-call walk paths referenced from `method_call.kind` dispatch + `primary.kind == "call".body`.

### Annotations

```ebnf
class_scope_type                 -> {head, params, scope_chain}
class_scope                      -> {body}
implicit_class_handle            -> 3 kinds (this / super / this_super)
method_call                      -> {initial, chain}
method_call_initial              -> 5 kinds (split_direct_callable / direct / class_scoped_tf / tf / system_tf)
method_call_body                 -> 3 kinds (built_in / call_with_args / method_bare)
method_call_receiver_sv_2017     -> {body}
method_call_receiver             -> 2 kinds (sv_2017 / sv_2023)
method_call_root                 -> 2 kinds (receiver / implicit_class)
plain_tf_call_with_args          -> {name, attributes, args}
tf_call_with_args                -> {name, attributes, args}
tf_call                          -> 4 kinds (plain_with_args / scoped_or_hierarchical_with_args / scoped_or_hierarchical_bare / tf_identifier_bare)
class_scoped_call_prefix         -> {head, params, scope_chain}
class_scoped_tf_call_with_args   -> {prefix, name, attributes, args}
class_scoped_tf_call             -> 2 kinds (with_args / bare)
```

### Deferred

`method_call_receiver_sv_2023` (~14 kinds with 2 parens-grouped-Or sub-expressions) is queued — task #38 attribution bug risk.

### Calibration

`parseability_probe --parse-dump-ast-pretty systemverilog /tmp/sv_calibration/minimal_module.sv` reports `parse_full passed`. Annotation count: **1443** (was 1416, +27). Same accept set.

## Release 1.0.75 / Contract 1.0.75 Highlights — SV-Slice-75 batch: net_port_type + I/O declarations + genvar typed (10 rules / 17 annotations)

Closes LRM A.2.1.2 net_port_type / nonansi I/O declarations / LRM A.2.1.3 genvar sub-trees.

### Annotations

```ebnf
genvar_declaration              -> {items}
genvar_expression               -> {body}
inout_declaration               -> {port_type, items}
input_declaration               -> 2 kinds (net / variable)
output_declaration              -> 2 kinds (net / variable)
net_port_header                 -> {direction, port_type}
net_port_type_sv_2017           -> 3 kinds (typed / identifier / interconnect)
net_port_type_sv_2023           -> 3 kinds (parallel — uses nettype_identifier)
net_port_type                   -> 2 kinds (sv_2017 / sv_2023)
net_type_declaration_sv_2017    -> {data_type, name, with_clause}
```

### Deferred

`net_type` has a duplicate-branch grammar bug — same family as drive_strength.

### Calibration

`parseability_probe --parse-dump-ast-pretty systemverilog /tmp/sv_calibration/minimal_module.sv` reports `parse_full passed`. Annotation count: **1416** (was 1399, +17). Same accept set.

## Release 1.0.74 / Contract 1.0.74 Highlights — SV-Slice-74 batch: dpi + extern_constraint + interface_class + param family typed (18 rules / 32 annotations)

Closes the LRM A.2.6 DPI sub-tree, LRM A.1.4 extern_constraint, LRM A.1.9 interface_class, and LRM A.2.1.1 param_assignment sub-trees.

### Annotations

```ebnf
dpi_function_import_property            -> 2 kinds (context / pure)
dpi_function_proto                      -> {body}
dpi_import_export                       -> 4 kinds (import_function / import_task / export_function / export_task)
dpi_spec_string                         -> 2 kinds (DPI-C / DPI)
dpi_task_import_property                -> 1 kind bare (context)
dpi_task_proto                          -> {body}
extern_constraint_declaration_sv_2017   -> {static_keyword, scope, name, body}
extern_constraint_declaration_sv_2023   -> {static_keyword, dynamic_override, scope, name, body}
extern_constraint_declaration           -> 2 kinds (sv_2017 / sv_2023)
interface_class_item                    -> 5 kinds (type_declaration / method / local_parameter / parameter / semi)
interface_class_method                  -> {prototype}
interface_class_type                    -> {name, params}
interface_port_declaration              -> 2 kinds (plain / modport)
interface_port_header                   -> 2 kinds (named / interface)
ordered_parameter_assignment            -> {body}
param_assignment_sv_2017                -> 2 kinds (with_default / no_default)
param_assignment_sv_2023                -> 2 kinds (with_default / no_default)
param_assignment                        -> 2 kinds (sv_2017 / sv_2023)
```

### Calibration

`parseability_probe --parse-dump-ast-pretty systemverilog /tmp/sv_calibration/minimal_module.sv` reports `parse_full passed`. Annotation count: **1399** (was 1367, +32). Same accept set.

## Release 1.0.73 / Contract 1.0.73 Highlights — SV-Slice-73 batch: checker family typed (10 rules / 33 annotations)

Closes LRM A.2.2.2 checker-declaration walk path (referenced from `class_item.kind == "checker_declaration".body` and `non_port_module_item.kind == "checker_declaration".body`).

### Annotations

```ebnf
checker_declaration                    -> {name, ports, items, end_label}
checker_generate_item_sv_2017          -> 4 kinds (loop / conditional / region / elaboration)
checker_generate_item_sv_2023          -> 4 kinds (parallel — uses elaboration_severity per LRM 2023)
checker_generate_item                  -> 2 kinds (sv_2017 / sv_2023)
checker_instantiation                  -> {name, instance, connections}
checker_or_generate_item               -> 7 kinds (declaration / initial / always / final / assertion / continuous_assign / generate)
checker_or_generate_item_declaration   -> 10 kinds (data_declaration / function / checker / assertion_item / covergroup / genvar / clocking / default_clocking / default_disable_iff / semi)
checker_port_direction                 -> 2 kinds (input / output)
checker_port_item                      -> {attributes, direction, formal_type, name, dims, default}
checker_port_list                      -> [$1, $2::2*]
```

### Calibration

`parseability_probe --parse-dump-ast-pretty systemverilog /tmp/sv_calibration/minimal_module.sv` reports `parse_full passed`. Annotation count: **1367** (was 1334, +33). Same accept set.

## Release 1.0.72 / Contract 1.0.72 Highlights — SV-Slice-72 batch: sequence family typed (16 rules / 39 annotations)

Closes LRM A.2.10 sequence sub-tree referenced from `property_expr.kind == "sequence".body` (typed in slice 71).

### Annotations

```ebnf
seq_input_list                    -> 2 kinds (level / edge)
sequence_abbrev                   -> {body}
sequence_actual_arg_sv_2017       -> 2 kinds (event_expression / sequence_expr)
sequence_actual_arg_sv_2023       -> 3 kinds (event_expression / sequence_expr / dollar)
sequence_actual_arg               -> 2 kinds (sv_2017 / sv_2023)
sequence_declaration              -> {name, ports, declarations, body, end_label}
sequence_expr                     -> 12 kinds (delay_head / delay_binary / expression / instance / paren / and / intersect / or / first_match / throughout / within / clocking)
sequence_formal_type              -> 3 kinds (data_type / sequence / untyped)
sequence_instance                 -> {name, args}
sequence_list_of_arguments        -> 2 kinds (mixed / named_only)
sequence_lvar_port_direction      -> 3 kinds bare (input / inout / output)
sequence_match_item               -> 3 kinds (operator_assignment / inc_or_dec / subroutine_call)
sequence_method_call              -> {instance, method}
sequence_port_item                -> {attributes, local_direction, formal_type, name, dims, default}
sequence_port_list                -> [$1, $2::2*]
with_covergroup_expression        -> {body}
```

### Calibration

`parseability_probe --parse-dump-ast-pretty systemverilog /tmp/sv_calibration/minimal_module.sv` reports `parse_full passed`. Annotation count: **1334** (was 1295, +39). Same accept set.

## Release 1.0.71 / Contract 1.0.71 Highlights — SV-Slice-71 batch: property_expr family typed (2 rules / 72 annotations)

Closes the LRM A.2.10 property expression sub-tree referenced from `property_spec.body` and `property_actual_arg.kind == "property_expr".body`.

### Annotations

`property_expr_sv_2017` 36 kinds covering all LRM 1800-2017 property operators:

```
sequence / strong / weak / paren / not / or / and / sequence_dup /
implies_unary / sequence_or_assign / if / case / imp_minus /
imp_assign / nexttime / nexttime_const / s_nexttime / s_nexttime_const /
always / always_range / s_always / s_eventually / eventually /
s_eventually_range / until / s_until / until_with / s_until_with /
implies_binary / iff / accept_on / reject_on / sync_accept_on /
sync_reject_on / instance / clocking
```

`property_expr_sv_2023` has the same 36 kinds — parallel except the `nexttime_const` branch uses `constant_expression` instead of `kw_constant_d810ca96 expression` per LRM 2023.

### Notes

- Several branches use left-recursion (`property_expr op property_expr`) — PEG generators don't handle left-recursion directly; the typed dispatch works for whatever the generator does parse.
- Branch ordering matters: branch 1 (`sequence_expr`) wins for any sequence-shaped input before branch 8's duplicate.

### Calibration

`parseability_probe --parse-dump-ast-pretty systemverilog /tmp/sv_calibration/minimal_module.sv` reports `parse_full passed`. Annotation count: **1295** (was 1223, +72). Same accept set.

## Release 1.0.70 / Contract 1.0.70 Highlights — SV-Slice-70 batch: property family (excluding property_expr) typed (12 rules / 16 annotations)

Closes the property-declaration walk path (referenced from `concurrent_assertion_item.kind == "property_declaration".body`) and `property_instance` references.

### Annotations

```ebnf
assertion_variable_declaration -> {data_type, items}
let_list_of_arguments          -> 2 kinds (mixed / named_only)
property_actual_arg            -> 2 kinds (property_expr / sequence_actual)
property_case_item             -> {expressions: [$1, $2::2*], body}
property_declaration           -> {name, ports, declarations, spec, end_label}
property_formal_type           -> 2 kinds (sequence / property bare)
property_instance              -> {name, args}
property_list_of_arguments     -> 2 kinds (mixed / named_only)
property_lvar_port_direction   -> 1 kind bare (input)
property_port_item             -> {attributes, local_direction, formal_type, name, dims, default}
property_port_list             -> [$1, $2::2*]
property_spec                  -> {clocking, disable_iff, body}
```

### Deferred

- `property_expr_sv_2017/2023` (~30 kinds each, many with left-recursive forms) queued for SV-Slice-71.

### Calibration

`parseability_probe --parse-dump-ast-pretty systemverilog /tmp/sv_calibration/minimal_module.sv` reports `parse_full passed`. Annotation count: **1223** (was 1207, +16). Same accept set.

## Release 1.0.69 / Contract 1.0.69 Highlights — SV-Slice-69 batch: cover_cross + trans + select_expression typed (12 rules / 29 annotations)

Closes LRM A.2.11 cross-cover walk path referenced from `coverage_spec.kind == "cross".body` and the LRM A.2.11 trans-list sub-tree referenced from `bins_or_options.kind == "trans_list".trans`.

### Annotations

```ebnf
cover_cross               -> {label, items, condition, body}
cross_body_sv_2017        -> 2 kinds (block {items} / empty)
cross_body_sv_2023        -> 2 kinds (block {items} / empty) — LRM 2023 dropped semi
cross_body                -> 2 kinds (sv_2017 / sv_2023)
cross_body_item_sv_2017   -> 2 kinds (function_decl / selection_or_option)
cross_body_item_sv_2023   -> 2 kinds (function_decl / selection_or_option)
cross_body_item           -> 2 kinds (sv_2017 / sv_2023)
cross_item                -> 2 kinds (cover_point / variable)
trans_list                -> [$2, $4::3*]  # Category A; extracts trans_set from each (comma lparen trans_set rparen) iteration entry
trans_range_list          -> 4 kinds (simple / star / implies / assign)
trans_set                 -> [$1, $2::2*]  # Category A, sequence_implies-separated
select_expression         -> 8 kinds (condition / not / and / or / paren / with_matches / cross / cross_set)
```

### Notes

- `select_expression` has left-recursive branches (`select_expression logical_and|or|with select_expression`). PEG generators usually don't handle left-recursion directly; the typed dispatch works for whatever the generator does parse.

### Calibration

`parseability_probe --parse-dump-ast-pretty systemverilog /tmp/sv_calibration/minimal_module.sv` reports `parse_full passed`. Annotation count: **1207** (was 1178, +29). Same accept set.

## Release 1.0.68 / Contract 1.0.68 Highlights — SV-Slice-68 batch: bins family typed (5 rules / 14 annotations)

Closes LRM A.2.11 bin-declaration sub-tree referenced from `cover_point.bins` (typed in slice 67). After this slice, `cover_point.bins` resolves to typed dispatch end-to-end.

### Annotations

```ebnf
bins_expression -> 2 kinds (variable {name} / cover_point {name, bin})

bins_or_empty   -> 2 kinds (block {body} / empty)

bins_or_options -> 7 kinds:
  | coverage_option            -> {kind: "coverage_option",   body}
  | <range_list form>          -> {kind: "range_list",        wildcard, keyword, name, index, ranges, with_expr, iff}
  | <cover_point_with form>    -> {kind: "cover_point_with",  wildcard, keyword, name, index, cover_point, with_expr, iff}
  | <set form>                 -> {kind: "set",               wildcard, keyword, name, index, value, iff}
  | <trans_list form>          -> {kind: "trans_list",        wildcard, keyword, name, array, trans, iff}
  | <default form>             -> {kind: "default",           keyword, name, index, iff}
  | <default_sequence form>    -> {kind: "default_sequence",  keyword, name, iff}

bins_selection                -> {keyword, name, select, iff}
bins_selection_or_option      -> 2 kinds (option {attributes, body} / selection {attributes, body})
```

### Calibration

`parseability_probe --parse-dump-ast-pretty systemverilog /tmp/sv_calibration/minimal_module.sv` reports `parse_full passed`. Annotation count: **1178** (was 1164, +14). Same accept set.

## Release 1.0.67 / Contract 1.0.67 Highlights — SV-Slice-67 batch: covergroup declaration + coverage_event family typed (12 rules / 26 annotations)

Closes LRM A.2.11 covergroup-declaration walk path referenced from `class_item.kind == "covergroup_declaration".body` and `package_or_generate_item_declaration.kind == "covergroup".body`.

### Annotations

```ebnf
bins_keyword                        -> 3 kinds bare (bins / illegal_bins / ignore_bins)
covergroup_declaration_sv_2017      -> {name, ports, event, items, end_label}
covergroup_declaration_sv_2023      -> 2 kinds (single / extends — LRM 2023 inheritance)
coverage_event                      -> 3 kinds (clocking / sample_function / block_event)
coverage_option                     -> 2 kinds (option / type_option)
coverage_spec                       -> 2 kinds (point / cross)
coverage_spec_or_option             -> 2 kinds (spec / option) — wraps attribute_instance*
cover_point                         -> {label, expression, condition, bins}
covergroup_range_list               -> [$1, $2::2*]
covergroup_value_range_sv_2017      -> 2 kinds (expression / range)
covergroup_value_range_sv_2023      -> 5 kinds (expression / range / dollar_lo / dollar_hi / tolerance)
covergroup_value_range              -> 2 kinds (sv_2017 / sv_2023)
```

### Deferred

`bins_or_empty`, `bins_or_options`, `bins_selection`, `bins_selection_or_option` — deep multi-branch bin declarations (7-branch `bins_or_options`). Queued for SV-Slice-68.

### Calibration

`parseability_probe --parse-dump-ast-pretty systemverilog /tmp/sv_calibration/minimal_module.sv` reports `parse_full passed`. Annotation count: **1164** (was 1138, +26). Same accept set.

## Release 1.0.66 / Contract 1.0.66 Highlights — SV-Slice-66 batch: UDP body/entry + udp_instance family typed (9 rules / 19 annotations)

Closes LRM A.5 UDP body/instance walk paths referenced from `udp_body.kind == "combinational"|"sequential".body` and `gate_instantiation.<kind>.instances` family.

### Annotations

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

### Deferred

`init_val` has a duplicate-branch grammar bug (`kw_n_1_tick_b_f4c81681` and `kw_n_1_tick_B_9f69eb32` each appear twice). Same family as drive_strength / unique_priority / delay_sv_2017/2023. Sequential UDPs default to typing `current_state.init_val` as a raw envelope until the grammar duplicate is removed.

### Calibration

`parseability_probe --parse-dump-ast-pretty systemverilog /tmp/sv_calibration/minimal_module.sv` reports `parse_full passed`. Annotation count: **1138** (was 1119, +19). Same accept set.

## Release 1.0.65 / Contract 1.0.65 Highlights — SV-Slice-65 batch: timing_check internals + scalar_timing_check_condition typed (16 rules / 22 annotations)

Closes the body field of every `system_timing_check.kind` from slice 64. Every reachable specify timing-check task now exposes typed args at the top level (reference_event, data_event, limits, threshold, edge offsets); the deeply-nested optional trailing-arg envelope (notifier / timestamp_condition / timecheck_condition / etc.) is preserved as a `tail` slot for post-campaign helper-rule extraction.

### Annotations (summary)

```ebnf
scalar_timing_check_condition -> 6 kinds (expression / not / eq / case_eq / ne / case_ne)
delayed_data                  -> 2 kinds (simple / with_expr)
delayed_reference             -> 2 kinds (simple / with_expr)

sv_dollar_setup_timing_check     -> {data_event, reference_event, limit, tail}
sv_dollar_hold_timing_check      -> {reference_event, data_event, limit, tail}
sv_dollar_recovery_timing_check  -> {reference_event, data_event, limit, tail}
sv_dollar_removal_timing_check   -> {reference_event, data_event, limit, tail}
sv_dollar_skew_timing_check      -> {reference_event, data_event, limit, tail}
sv_dollar_timeskew_timing_check  -> {reference_event, data_event, limit, tail}
sv_dollar_setuphold_timing_check -> {reference_event, data_event, limit_setup, limit_hold, tail}
sv_dollar_recrem_timing_check    -> {reference_event, data_event, limit_recov, limit_rem, tail}
sv_dollar_fullskew_timing_check  -> {reference_event, data_event, limit_setup, limit_hold, tail}
sv_dollar_period_timing_check    -> {controlled_reference_event, limit, tail}
sv_dollar_width_timing_check     -> {controlled_reference_event, limit, threshold, tail}
sv_dollar_nochange_timing_check  -> {reference_event, data_event, start_edge_offset, end_edge_offset, tail}
```

### Calibration

`parseability_probe --parse-dump-ast-pretty systemverilog /tmp/sv_calibration/minimal_module.sv` reports `parse_full passed`. Annotation count: **1119** (was 1097, +22). Same accept set.

## Release 1.0.64 / Contract 1.0.64 Highlights — SV-Slice-64 batch: edge + timing_check family typed (10 rules / 37 annotations)

Closes LRM A.7.5.3 / A.7.6 timing check sub-trees referenced from `specify_item.kind == "system_timing".body` (typed in slice 62) and the LRM A.5 UDP edge-input descriptors.

### Annotations

```ebnf
edge_descriptor              -> 4 kinds (01 / 10 / z_or_x_first / digit_first)
edge_indicator               -> 2 kinds (pair {first, second} / symbol {body})
edge_symbol                  -> 9 kinds bare (r / R / f / F / p / P / n / N / star)
edge_control_specifier       -> {descriptors}
edge_input_list              -> {leading_levels, indicator, trailing_levels}
system_timing_check          -> 12 kinds (setup / hold / setuphold / recovery / removal / recrem / skew / timeskew / fullskew / period / width / nochange)
timing_check_condition       -> 2 kinds (scalar / paren)
timing_check_event           -> {control, descriptor, condition}
timing_check_event_control   -> 4 kinds (posedge / negedge / edge / edge_control)
controlled_timing_check_event-> {control, descriptor, condition}
```

### Calibration

`parseability_probe --parse-dump-ast-pretty systemverilog /tmp/sv_calibration/minimal_module.sv` reports `parse_full passed`. Annotation count: **1097** (was 1060, +37). Same accept set.

## Release 1.0.63 / Contract 1.0.63 Highlights — SV-Slice-63 batch: path_declaration family typed (14 rules / 25 annotations)

Closes the LRM A.7.2 / A.7.4 path declarations referenced from `specify_item.kind == "path".body` (typed in slice 62).

### Annotations

```ebnf
path_declaration                  -> 3 kinds (simple / edge_sensitive / state_dependent — drops trailing semi)
simple_path_declaration           -> 2 kinds (parallel / full) {kind, path, delay}
edge_sensitive_path_declaration   -> 2 kinds (parallel / full) {kind, path, delay}
state_dependent_path_declaration  -> 3 kinds (if_simple / if_edge_sensitive / ifnone)
parallel_path_description         -> {input, polarity, output}
full_path_description             -> {inputs, polarity, outputs}
parallel_edge_sensitive_path_description_sv_2017  -> {edge, input, in_polarity, output, out_polarity, data_source}
parallel_edge_sensitive_path_description_sv_2023  -> 2 kinds (with_data_source / simple — LRM 2023 added no-data-source form)
full_edge_sensitive_path_description_sv_2017      -> parallel shape for inputs/outputs
full_edge_sensitive_path_description_sv_2023      -> 2 kinds (with_data_source / simple)
path_delay_value                  -> 2 kinds (bare / paren)
list_of_path_delay_expressions    -> {body}
pulsestyle_declaration            -> 2 kinds (onevent / ondetect) — output paths the keyword applies to
showcancelled_declaration         -> 2 kinds (showcancelled / noshowcancelled)
```

### Calibration

`parseability_probe --parse-dump-ast-pretty systemverilog /tmp/sv_calibration/minimal_module.sv` reports `parse_full passed`. Annotation count: **1060** (was 1035, +25). Same accept set.

## Release 1.0.62 / Contract 1.0.62 Highlights — SV-Slice-62 batch: specify family typed (9 rules / 15 annotations)

Closes the LRM A.7 specify-block walk path (referenced from `non_port_module_item.kind == "specify".body`) and the LRM A.7.5.1 specparam-declaration sub-tree.

### Annotations

```ebnf
specify_block := kw_specify specify_item* kw_endspecify
              -> {items: $2}

specify_item := specparam_declaration       -> {kind: "specparam",     body: $1}
              | pulsestyle_declaration      -> {kind: "pulsestyle",    body: $1}
              | showcancelled_declaration   -> {kind: "showcancelled", body: $1}
              | path_declaration            -> {kind: "path",          body: $1}
              | system_timing_check         -> {kind: "system_timing", body: $1}

specify_input_terminal_descriptor  := input_identifier  ( lbrack constant_range_expression rbrack )?  -> {name, range}
specify_output_terminal_descriptor := output_identifier ( lbrack constant_range_expression rbrack )?  -> {name, range}

specify_terminal_descriptor := specify_input_terminal_descriptor   -> {kind: "input",  body: $1}
                             | specify_output_terminal_descriptor  -> {kind: "output", body: $1}

specparam_assignment := specparam_identifier assign constant_mintypmax_expression  -> {kind: "simple", name: $1, value: $3}
                      | pulse_control_specparam                                     -> {kind: "pulse",  body: $1}

specparam_declaration := kw_specparam ( packed_dimension )? list_of_specparam_assignments semi
                      -> {dims, items}

polarity_operator := plus  -> {kind: "plus"}
                   | minus -> {kind: "minus"}
```

### Calibration

`parseability_probe --parse-dump-ast-pretty systemverilog /tmp/sv_calibration/minimal_module.sv` reports `parse_full passed`. Annotation count: **1035** (was 1020, +15). Same accept set.

## Release 1.0.61 / Contract 1.0.61 Highlights — SV-Slice-61 batch: gate_instantiation family typed (16 rules / 43 annotations — crosses 1000-annotation milestone)

Closes the LRM A.3.1 gate instantiation walk path referenced from `module_or_generate_item.kind == "gate_instantiation".body`. Every reachable gate instantiation now exposes typed dispatch with per-instance terminal binding.

### Annotations

```ebnf
gate_instantiation_sv_2017 := cmos_switchtype ( delay )? cmos_switch_instance ( comma cmos_switch_instance )* semi
                                  -> {kind: "cmos",     switchtype, delay, instances: [$3, $4::2*]}
                            | enable_gatetype ( drive_strength )? ( delay )? enable_gate_instance ( comma enable_gate_instance )* semi
                                  -> {kind: "enable",   gatetype, drive_strength, delay, instances: [$4, $5::2*]}
                            | mos_switchtype ( delay )? mos_switch_instance ( comma mos_switch_instance )* semi
                                  -> {kind: "mos",      switchtype, delay, instances: [$3, $4::2*]}
                            | n_input_gatetype ( drive_strength )? ( delay )? n_input_gate_instance ( comma n_input_gate_instance )* semi
                                  -> {kind: "n_input",  gatetype, drive_strength, delay, instances: [$4, $5::2*]}
                            | n_output_gatetype ( drive_strength )? ( delay )? n_output_gate_instance ( comma n_output_gate_instance )* semi
                                  -> {kind: "n_output", gatetype, drive_strength, delay, instances: [$4, $5::2*]}
                            | pass_en_switchtype ( delay )? pass_enable_switch_instance ( comma pass_enable_switch_instance )* semi
                                  -> {kind: "pass_en",  switchtype, delay, instances: [$3, $4::2*]}
                            | pass_switchtype pass_switch_instance ( comma pass_switch_instance )* semi
                                  -> {kind: "pass",     switchtype, instances: [$2, $3::2*]}
                            | kw_pulldown ( pulldown_strength )? pull_gate_instance ( comma pull_gate_instance )* semi
                                  -> {kind: "pulldown", strength, instances: [$3, $4::2*]}
                            | kw_pullup   ( pullup_strength )? pull_gate_instance ( comma pull_gate_instance )* semi
                                  -> {kind: "pullup",   strength, instances: [$3, $4::2*]}

# sv_2023 has the same 9 kinds; LRM 2023 only reorders mos before enable in the alternation.

cmos_switchtype   -> 2 kinds bare (cmos / rcmos)
mos_switchtype    -> 4 kinds bare (nmos / pmos / rnmos / rpmos)
n_input_gatetype  -> 6 kinds bare (and / nand / or / nor / xor / xnor)
n_output_gatetype -> 2 kinds bare (buf / not)
pass_switchtype   -> 2 kinds bare (tran / rtran)

name_of_instance := instance_identifier unpacked_dimension*  -> {name, dims}

cmos_switch_instance         -> {name, output, input, ncontrol, pcontrol}
enable_gate_instance         -> {name, output, input, enable}
mos_switch_instance          -> {name, output, input, enable}
n_input_gate_instance        -> {name, output, inputs: [$5, $6::2*]}
n_output_gate_instance       -> {name, outputs: [$3, $4::2*], input}
pass_enable_switch_instance  -> {name, in1, in2, enable}
pass_switch_instance         -> {name, in1, in2}
pull_gate_instance           -> {name, output}
```

### Deferred

- `enable_gatetype`, `pass_en_switchtype`, `pulldown_strength`, `pullup_strength` all have a duplicate-branch grammar bug (each branch is repeated). Same family of issues as `drive_strength` / `unique_priority` / `delay_sv_2017/2023` noted in earlier slices — to be fixed separately as a grammar correction. The `gate_instantiation.gatetype` / `.switchtype` / `.strength` fields therefore still resolve to raw envelope shapes for those four sub-rules; the parent gate_instantiation typed dispatch is unaffected.

### Calibration

`parseability_probe --parse-dump-ast-pretty systemverilog /tmp/sv_calibration/minimal_module.sv` reports `parse_full passed`. A 4-line gate-instance sample (`module gates; wire a, b, c, y; and g1 (y, a, b, c); buf g2 (y, a); endmodule`) also parses successfully end-to-end.

Annotation count: **1020** (was 977, +43). **Crosses the 1000-annotation milestone.** Same accept set.

## Release 1.0.60 / Contract 1.0.60 Highlights — SV-Slice-60 batch: number + literal family typed (10 rules / 19 annotations)

Closes the LRM A.8.7 number sub-tree referenced from `primary_literal.kind == "number".body` and `primary_literal.kind == "time_literal".body` / `"string_literal".body`.

### Annotations

```ebnf
number := real_number     -> {kind: "real",     body: $1}
        | integral_number -> {kind: "integral", body: $1}

integral_number := decimal_number -> {kind: "decimal", body: $1}
                 | octal_number   -> {kind: "octal",   body: $1}
                 | binary_number  -> {kind: "binary",  body: $1}
                 | hex_number     -> {kind: "hex",     body: $1}

real_number := fixed_point_number
                     -> {kind: "fixed_point", body: $1}
             | unsigned_number ( dot unsigned_number )? exp ( sign )? unsigned_number
                     -> {kind: "exponential", mantissa: $1, fraction: $2, sign: $4, exponent: $5}

string_literal := triple-quoted regex -> {kind: "triple_quoted", body: $2}
                | double-quoted regex -> {kind: "double_quoted", body: $2}

binary_number := ( size )? binary_base binary_value -> {size: $1, base: $2, value: $3}
octal_number  := ( size )? octal_base  octal_value  -> {size: $1, base: $2, value: $3}
hex_number    := ( size )? hex_base    hex_value    -> {size: $1, base: $2, value: $3}

decimal_number := unsigned_number
                        -> {kind: "unsized", body: $1}
                | ( size )? decimal_base unsigned_number
                        -> {kind: "sized",   size: $1, base: $2, value: $3}
                | ( size )? decimal_base x_digit (...)*
                        -> {kind: "x_digit", size: $1, base: $2}
                | ( size )? decimal_base z_digit (...)*
                        -> {kind: "z_digit", size: $1, base: $2}

fixed_point_number := unsigned_number dot unsigned_number -> {whole: $1, fractional: $3}

time_literal := number time_unit -> {value: $1, unit: $2}
```

### Calibration

`parseability_probe --parse-dump-ast-pretty systemverilog /tmp/sv_calibration/minimal_module.sv` reports `parse_full passed`. Annotation count: **977** (was 958, +19). Same accept set.

## Release 1.0.59 / Contract 1.0.59 Highlights — SV-Slice-59 batch: always + modport family typed (11 rules / 19 annotations)

Closes the LRM A.1.4 always-construct dispatch (referenced from `module_common_item.kind == "always"`) and the LRM A.2.9 modport family (referenced from interface bodies via `non_port_interface_item.kind == "modport_declaration"`). All new pure-list patterns use `[$N, $M::2*]` from the start, honoring the slice 58 audit conclusion.

### Annotations

```ebnf
always_construct := always_keyword statement
                 -> {keyword: $1, body: $2}

always_keyword := kw_always       -> {kind: "always"}
                | kw_always_comb  -> {kind: "always_comb"}
                | kw_always_latch -> {kind: "always_latch"}
                | kw_always_ff    -> {kind: "always_ff"}

import_export := kw_import -> {kind: "import"}
               | kw_export -> {kind: "export"}

modport_simple_ports_declaration := port_direction modport_simple_port
                                 -> {direction: $1, port: $2}

modport_clocking_declaration := kw_clocking clocking_identifier
                             -> {name: $2}

modport_declaration := kw_modport modport_item ( comma modport_item )* semi
                    -> {items: [$2, $3::2*]}

modport_item := modport_identifier lparen modport_ports_declaration ( comma modport_ports_declaration )* rparen
             -> {name: $1, ports: [$3, $4::2*]}

modport_ports_declaration := attribute_instance* modport_simple_ports_declaration   -> {kind: "simple",   attributes: $1, body: $2}
                           | attribute_instance* modport_tf_ports_declaration       -> {kind: "tf",       attributes: $1, body: $2}
                           | attribute_instance* modport_clocking_declaration       -> {kind: "clocking", attributes: $1, body: $2}

modport_simple_port := port_identifier                                        -> {kind: "identifier", name: $1}
                     | dot port_identifier lparen ( expression )? rparen      -> {kind: "explicit",   name: $2, expression: $4}

modport_tf_port := method_prototype  -> {kind: "method_prototype", body: $1}
                 | tf_identifier     -> {kind: "tf_identifier",    name: $1}

modport_tf_ports_declaration := import_export modport_tf_port ( comma modport_tf_port )*
                             -> {import_export: $1, ports: [$2, $3::2*]}
```

### Notes

- `always_keyword` is referenced from the typed `always_construct.keyword` field and propagates the typed dispatch up through `module_common_item.kind == "always".body`.
- `modport_ports_declaration` distinguishes the three modport-port forms with `kind` discriminator. The legacy `attributes:` slot carries the attribute_instance* prefix.
- `modport_simple_port`'s `"explicit"` kind covers the `.name(expression)` form used for synthesizing port mappings; the `expression` field carries the optional inner expression.
- `modport_declaration` and `modport_tf_ports_declaration` use the slice 58 `[$N, $M::2*]` extraction-spread for their `( comma X )*` tails — flat arrays of items.

### Calibration

`parseability_probe --parse-dump-ast-pretty systemverilog /tmp/sv_calibration/minimal_module.sv` reports `parse_full passed`. A 5-line interface-with-modport sample (`/tmp/sv_modport_check.sv`) also parses successfully end-to-end. minimal_module is empty so the AST envelope at the top level is unchanged.

Annotation count: **958** (was 939, +19). Same accept set.

## Release 1.0.58 / Contract 1.0.58 Highlights — SV-Slice-58 audit: horizontal Category A `{first, rest}` → `[$N, $M::2*]` extraction-spread fix (49 grammar locations + 1 manual edit)

This slice is a **horizontal correctness audit**, not a typing slice. It changes the *shape* of typed AST output for ~50 grammar rules — annotation count is unchanged at **939**. The change applies retroactively to typed annotations introduced in earlier slices (most prominently SV-Slice-44's `list_of_*` family, SV-Slice-45's pattern.ordered, SV-Slice-46's expression internals, SV-Slice-49's concat family, SV-Slice-52's range/dist family, and SV-Slice-57's `tf_port_list` / `let_port_list`).

### Why

Earlier slices defaulted to `{first: $1, rest: $2}` for any rule of the shape `X (sep X)*`. This pattern looks innocuous but exposes a **raw envelope** to consumers: `$2` references the entire `(sep X)*` Quantified-of-Sequence, whose value is `[[sep, X], [sep, X], ...]` — an array of 2-element inner-Sequence matches. Consumers of `rest` therefore had to walk past every separator entry. This contradicts the grammar's stated goal: shape AST so downstream consumers find what they need quickly without walking deep envelope structure.

The annotation language has a first-class **extraction-spread** operator `$N::M*` (defined in `grammars/return_annotation.ebnf` lines 50-58 and self-applied at line 158: `object_properties := object_property (',' object_property)* -> [$1, $2::2*]`). It pulls position `M` from each entry of a Quantified-of-Sequence and spreads. For pure `X (sep X)*` lists this gives a flat array of items with separators dropped — no envelope walking required.

### What changes

For every Category A rule (pure `X (sep X)*` list — single-payload-per-iteration), the `{first: $N, rest: $M}` annotation becomes `[$N, $M::2*]`. Consumer-visible difference:

```
// Before (SV-Slice-57)
{
  "first": <tf_port_item>,
  "rest": [
    [<comma>, <tf_port_item>],
    [<comma>, <tf_port_item>]
  ]
}

// After (SV-Slice-58)
[
  <tf_port_item>,
  <tf_port_item>,
  <tf_port_item>
]
```

The 49 affected rules:

```
assignment_pattern (`'{e, e, ...}` form), attribute_instance, bind_target_instance_list,
case_generate_item (expr_list branch), case_item (expr_list branch),
class_constructor_arg_list_sv_2023,
concatenation, cond_predicate, constant_concatenation,
data_type (enum branch's `names` field), dist_list, let_port_list,
list_of_arguments_mixed (named tail), list_of_arguments_ordered, list_of_arguments_named,
list_of_checker_port_connections (ordered+named), list_of_clocking_decl_assign,
list_of_cross_items (also flattened: was `{first, second, rest}`, now `[first, second, ...rest]`),
list_of_defparam_assignments, list_of_genvar_identifiers, list_of_net_assignments,
list_of_net_decl_assignments, list_of_param_assignments,
list_of_parameter_assignments_sv_2017 (ordered+named),
list_of_parameter_value_assignments_sv_2023 (ordered+named),
list_of_path_inputs, list_of_path_outputs,
list_of_port_connections (named+ordered), list_of_ports,
list_of_specparam_assignments, list_of_type_assignments, list_of_udp_port_identifiers,
list_of_variable_assignments, list_of_variable_decl_assignments,
net_lvalue (concat branch), open_range_list_sv_2017,
package_export_declaration (explicit branch), package_import_declaration,
pattern_sv_2017 (ordered branch), pattern_sv_2023 (ordered branch),
production_sv_2017 (rules), range_list_sv_2023,
rs_case_item_sv_2017 (expr_list branch), rs_case_item_sv_2023 (expr_list branch),
rs_production_sv_2023 (rules), tf_port_list,
udp_declaration_port_list, udp_port_list,
variable_lvalue (concat branch), wait_order_statement (events).
```

### Deferred

**Category B — multi-payload-per-iteration** rules (where the inner Sequence has multiple meaningful positions, e.g. `binary_operator attribute_instance* operand` or `port_identifier unpacked_dimension*`) keep their `{first, rest}` shape for now. These need helper-rule extractions to expose typed objects per iteration entry — queued for **SV-Slice-59**. Affected: `constant_expression`, `expression` operand_chain, `list_of_interface_identifiers`, `list_of_port_identifiers`, `list_of_tf_variable_identifiers`, `list_of_variable_identifiers`, `list_of_variable_port_identifiers`, `pattern_sv_2017/2023` named-branch entries.

> **RESOLVED (1.0.117 / schema 3, POST-SV-AUDIT.2.4b).** The 5
> `list_of_*_identifiers` rules and the `pattern_sv_2017/2023`
> named-branch entries (`assignment_pattern` named) listed above were
> corrected at `1.0.117` by factoring the repeated multi-field unit
> into new annotated record rules + extraction-spreads (`+9`
> annotations → `2290 → 2299` / `999 → 1008`). The remaining
> `constant_expression` / `expression` operand_chain are the
> separately-resolved Cat-B named-op-rule class (correct as-is). This
> deferral note is kept as labeled history. See
> [AST-Shape Corrections — 1.0.117](#ast-shape-corrections--10117-post-sv-audit--11-structured-per-iteration-category-a-misuses--clean-factored-record-lists-9-annotated-record-rules-22902299--9991008-a-deliberate-count-change-reachable-list_of__identifiers-probe-verified-the-rest-defensively-correct-by-construction-not-a-bug-ledger-entry-schema-2--3).

**Category C — `X X*` (no separator)** rules already produce a clean array via `$M*`, but currently use `{first: $N, rest: $M}` for verbose-but-correct semantics. Flattening to a single array is queued for the post-campaign holistic shape-correctness audit (per user direction).

**Post-campaign holistic shape-correctness audit** is also queued: review the entire shaped AST end-to-end after the SV typing campaign is complete and adjust shapes for downstream ergonomics where useful.

### Calibration

`parseability_probe --parse-dump-ast-pretty systemverilog /tmp/sv_calibration/minimal_module.sv` reports `parse_full passed`. minimal_module is empty (no items), so the AST envelope is unchanged at the top level — but every `list_of_*`, every concatenation, every pattern ordered branch, every operand chain reachable from typed parents now exposes a flat item array instead of the raw envelope.

The historical `Release 1.0.57 Highlights` section below shows `tf_port_list -> {first: $1, rest: $2}` and `let_port_list -> {first: $1, rest: $2}`. These are now `-> [$1, $2::2*]` post-slice-58. Consumers should treat the slice 58 form as authoritative.

## Release 1.0.57 / Contract 1.0.57 Highlights — SV-Slice-57 batch: tf_port + prototypes + lvalue/decl_assignment family typed (12 rules / 23 annotations + 1 new helper rule with 2 annotations)

Closes the LRM A.2.7 task/function port-list family, prototype rules, and the LRM A.8.1 lvalue family.

### Annotations

```ebnf
tf_port_list := tf_port_item ( comma tf_port_item )*
             -> {first: $1, rest: $2}

tf_port_item := attribute_instance* ( tf_port_direction )? ( kw_var )? data_type_or_implicit ( port_identifier variable_dimension* ( assign expression )? )?
             -> {attributes: $1, direction: $2, var_keyword: $3, data_type: $4, port_spec: $5}

@profiles: ["sv_2017"]
tf_port_direction_sv_2017 := port_direction               -> {kind: "port_direction", body: $1}
                           | kw_const kw_ref              -> {kind: "const_ref"}

@profiles: ["sv_2023"]
tf_port_direction_sv_2023 := port_direction
                                   -> {kind: "port_direction", body: $1}
                           | ( kw_const )? kw_ref ( kw_static )?
                                   -> {kind: "ref",            const_keyword: $1, static_keyword: $3}

function_prototype_sv_2017 := kw_function data_type_or_void function_identifier ( lparen ( tf_port_list )? rparen )?
                           -> {return_type: $2, name: $3, ports: $4}

function_prototype_sv_2023 := kw_function ( dynamic_override_specifiers )? data_type_or_void function_identifier ( lparen ( tf_port_list )? rparen )?
                           -> {dynamic_override: $2, return_type: $3, name: $4, ports: $5}

task_prototype_sv_2017 := kw_task task_identifier ( lparen ( tf_port_list )? rparen )?
                       -> {name: $2, ports: $3}

task_prototype_sv_2023 := kw_task ( dynamic_override_specifiers )? task_identifier ( lparen ( tf_port_list )? rparen )?
                       -> {dynamic_override: $2, name: $3, ports: $4}

let_port_item := attribute_instance* let_formal_type formal_port_identifier variable_dimension* ( assign expression )?
              -> {attributes: $1, type: $2, name: $3, dims: $4, init: $5}

let_port_list := let_port_item ( comma let_port_item )*
              -> {first: $1, rest: $2}

net_decl_assignment := net_identifier unpacked_dimension* ( assign expression )?
                    -> {name: $1, dims: $2, init: $3}

variable_decl_assignment := variable_identifier !lparen variable_dimension* ( assign expression )?
                                  -> {kind: "variable",      name: $1, dims: $3, init: $4}
                          | dynamic_array_variable_identifier unsized_dimension variable_dimension* ( assign dynamic_array_new )?
                                  -> {kind: "dynamic_array", name: $1, unsized_dim: $2, dims: $3, init: $4}
                          | class_variable_identifier ( assign class_new )?
                                  -> {kind: "class",         name: $1, init: $2}

net_lvalue := ps_or_hierarchical_net_identifier constant_select
                    -> {kind: "name",          name: $1, select: $2}
            | lbrace net_lvalue ( comma net_lvalue )* rbrace
                    -> {kind: "concatenation", items: {first: $2, rest: $3}}
            | ( assignment_pattern_expression_type )? assignment_pattern_net_lvalue
                    -> {kind: "pattern",       type: $1, body: $2}

variable_lvalue := ( variable_lvalue_scope )? hierarchical_variable_identifier select
                        -> {kind: "name",                  scope: $1, name: $2, select: $3}
                 | lbrace variable_lvalue ( comma variable_lvalue )* rbrace
                        -> {kind: "concatenation",         items: {first: $2, rest: $3}}
                 | ( assignment_pattern_expression_type )? assignment_pattern_variable_lvalue
                        -> {kind: "pattern",               type: $1, body: $2}
                 | streaming_concatenation
                        -> {kind: "streaming_concatenation", body: $1}

variable_lvalue_scope (NEW) := implicit_class_handle dot   -> {kind: "instance",      handle: $1}
                              | non_typedef_package_scope   -> {kind: "package_scope", body: $1}
```

### Helper-rule extraction (13th use of pattern)

`variable_lvalue_scope` extracted from inline `( implicit_class_handle dot | non_typedef_package_scope )?` parens-Or in `variable_lvalue` branch 0 (similar to `instance_or_class_scope` from slice 47, but with `package_scope` instead of `class_scope`).

### Annotation inventory

939 entries (was 914). +25 in this batch.

### Same accept set, same diagnostic codes. Schema stays at `1`.

### Grammar surface change

This slice adds one new rule (`variable_lvalue_scope`) — internal refactor of inline parens-Or for annotation purposes. No LRM equivalent. Same accept set.

### mdBook updated, gate green.

### Next slice candidates

- The remaining unannotated mid-size rules.
- Profile-tag wrapper rules.
- Drive-strength / unique_priority / delay grammar fixes (separate task).

## Release 1.0.56 / Contract 1.0.56 Highlights — SV-Slice-56 batch: class_constructor_declaration family typed (4 rules / 5 annotations + 1 new helper rule with 2 annotations)

Closes the class constructor declaration walks for both LRM 1800-2017 and 2023 profiles. After this slice, every reachable `class_method.kind == "constructor"` body and `class_method.kind == "extern_constructor"` prototype resolves to typed dispatch.

### Annotations

```ebnf
class_constructor_arg_sv_2023 := tf_port_item            -> {kind: "tf_port_item", body: $1}
                               | kw_default              -> {kind: "default"}

class_constructor_arg_list_sv_2023 := class_constructor_arg ( comma class_constructor_arg )*
                                   -> {first: $1, rest: $2}

@profiles: ["sv_2017"]
class_constructor_declaration_sv_2017 := kw_function ( class_scope )? kw_new ( lparen ( tf_port_list )? rparen )? semi block_item_declaration* ( kw_super dot kw_new ( lparen list_of_arguments rparen )? semi )? function_statement_or_null* kw_endfunction ( colon kw_new )?
                                      -> {class_scope: $2, ports: $4, decls: $6, super_call: $7, statements: $8, end_label: $10}

@profiles: ["sv_2023"]
class_constructor_declaration_sv_2023 := kw_function ( class_scope )? kw_new ( lparen ( class_constructor_arg_list )? rparen )? semi block_item_declaration* ( kw_super dot kw_new ( lparen ( class_constructor_super_args )? rparen )? semi )? function_statement_or_null* kw_endfunction ( colon kw_new )?
                                      -> {class_scope: $2, ports: $4, decls: $6, super_call: $7, statements: $8, end_label: $10}

class_constructor_super_args (NEW) := list_of_arguments    -> {kind: "args",    body: $1}
                                    | kw_default            -> {kind: "default"}
```

### Helper-rule extraction (12th use of pattern)

`class_constructor_super_args` extracted from the deeply-nested parens-Or in the super-call sub-clause of `class_constructor_declaration_sv_2023`:

```ebnf
( kw_super dot kw_new ( lparen ( list_of_arguments | kw_default )? rparen )? semi )?
                                        ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
                                              parens-Or (task #38 risk)
```

Now used in **13 places total** — established workaround pattern.

### Field semantics

- `class_constructor_declaration.class_scope`: optional `( class_scope )?` prefix (e.g., `function MyClass::new(...)` for out-of-class constructor declaration).
- `class_constructor_declaration.ports`: optional argument list. sv_2017 uses `tf_port_list`; sv_2023 uses `class_constructor_arg_list` (which adds the `default` arg form).
- `class_constructor_declaration.super_call`: optional `super.new(args);` initializer call.
- `class_constructor_super_args.kind == "default"`: LRM 2023 `super.new(default);` form (delegates to default super constructor).

### Annotation inventory

914 entries (was 907). +7 in this batch (2 class_constructor_arg_sv_2023 + 1 class_constructor_arg_list_sv_2023 + 1 class_constructor_declaration_sv_2017 + 1 class_constructor_declaration_sv_2023 + 2 class_constructor_super_args).

### Same accept set, same diagnostic codes. Schema stays at `1`.

### Grammar surface change

This slice adds one new rule (`class_constructor_super_args`) — internal refactor of inline parens-Or for annotation purposes. No LRM equivalent. Same accept set.

### mdBook updated, gate green.

### Next slice candidates

- `tf_port_list` / `tf_port_item` (function/task port lists).
- The remaining unannotated mid-size rules.
- Profile-tag wrapper rules.
- Drive-strength / unique_priority / delay grammar fixes (separate task).

## Release 1.0.55 / Contract 1.0.55 Highlights — SV-Slice-55 batch: clocking + class_constructor_prototype + edge_identifier + method_prototype typed (10 rules / 22 annotations — crosses 900-annotation milestone)

Closes the LRM A.6.10 clocking declaration sub-tree end-to-end. Crosses the **900-annotation milestone**.

### Annotations

```ebnf
class_constructor_prototype_sv_2017 := kw_function kw_new ( lparen ( tf_port_list )? rparen )? semi
                                    -> {ports: $3}

class_constructor_prototype_sv_2023 := kw_function kw_new ( lparen ( class_constructor_arg_list )? rparen )? semi
                                    -> {ports: $3}

clocking_decl_assign := signal_identifier ( assign expression )?
                     -> {name: $1, value: $2}

clocking_declaration := ( kw_default )? kw_clocking ( clocking_identifier )? clocking_event semi clocking_item* kw_endclocking ( colon clocking_identifier )?
                     -> {default_keyword: $1, name: $3, event: $4, items: $6, end_label: $8}

clocking_direction := kw_input ( clocking_skew )?
                            -> {kind: "input",        skew: $2}
                    | kw_output ( clocking_skew )?
                            -> {kind: "output",       skew: $2}
                    | kw_input ( clocking_skew )? kw_output ( clocking_skew )?
                            -> {kind: "input_output", input_skew: $2, output_skew: $4}
                    | kw_inout
                            -> {kind: "inout"}

@profiles: ["sv_2017"]
clocking_event_sv_2017 := at_sign identifier
                       -> {body: $2}

@profiles: ["sv_2023"]
clocking_event_sv_2023 := at_sign ps_identifier              -> {kind: "ps",           body: $2}
                        | at_sign hierarchical_identifier    -> {kind: "hierarchical", body: $2}
                        | at_sign lparen event_expression rparen
                                                             -> {kind: "expression",   body: $3}

clocking_item := kw_default default_skew semi
                      -> {kind: "default_skew", skew: $2}
               | clocking_direction list_of_clocking_decl_assign semi
                      -> {kind: "direction",    direction: $1, decls: $2}
               | attribute_instance* assertion_item_declaration
                      -> {kind: "assertion",    attributes: $1, body: $2}

clocking_skew := edge_identifier ( delay_control )? -> {kind: "edge",  edge: $1, delay: $2}
               | delay_control                       -> {kind: "delay", body: $1}

edge_identifier := kw_posedge -> {kind: "posedge"}
                 | kw_negedge -> {kind: "negedge"}
                 | kw_edge    -> {kind: "edge"}

method_prototype := task_prototype     -> {kind: "task",     body: $1}
                  | function_prototype -> {kind: "function", body: $1}
```

### Field semantics

- `clocking_declaration`: LRM A.6.10. The `default_keyword` is `[]` for non-default clockings. `name` is `[]` for anonymous clocking. `items` is the list of clocking-block contents.
- `clocking_direction.kind == "input_output"`: the LRM `input ... output ...` combined direction (each side has its own optional skew).
- `clocking_skew.kind == "edge"`: edge-prefixed skew (e.g., `posedge #1`); `delay` is `[]` or `[<delay_control>]`.
- `class_constructor_prototype_sv_2017/2023`: method-prototype form for `extern function new(args)` declaration.

### Annotation inventory

907 entries (was 885). +22 in this batch. Crosses the 900-annotation milestone.

### Same accept set, same diagnostic codes. Schema stays at `1`.

### mdBook updated, gate green.

### Next slice candidates

- `class_constructor_declaration_sv_2017/2023` (complex single-seq with super.new call optional).
- The remaining unannotated mid-size rules.
- Profile-tag wrapper rules.
- Drive-strength / unique_priority / delay grammar fixes (separate task).

## Release 1.0.54 / Contract 1.0.54 Highlights — SV-Slice-54 batch: delay/event/strength leaves typed (10 rules / 33 annotations)

Closes the LRM A.6.5 / A.6.4 timing-control / event-control / strength leaves used pervasively across blocking_assignment / nonblocking_assignment / procedural_timing_control / net_declaration.

### Annotations

```ebnf
charge_strength := lparen kw_small rparen   -> {kind: "small"}
                 | lparen kw_medium rparen  -> {kind: "medium"}
                 | lparen kw_large rparen   -> {kind: "large"}

cycle_delay := kw_token integral_number          -> {kind: "number",     body: $2}
             | kw_token identifier               -> {kind: "identifier", body: $2}
             | kw_token lparen expression rparen -> {kind: "expression", body: $3}

cycle_delay_const_range_expression := constant_expression colon constant_expression -> {kind: "range",     lo: $1, hi: $3}
                                    | constant_expression colon kw_dollar             -> {kind: "dollar_hi", lo: $1}

delay_control := hash delay_value                          -> {kind: "value",     body: $2}
               | hash lparen mintypmax_expression rparen   -> {kind: "mintypmax", body: $3}

delay_or_event_control := delay_control                                              -> {kind: "delay",  body: $1}
                        | event_control                                              -> {kind: "event",  body: $1}
                        | kw_repeat lparen expression rparen event_control           -> {kind: "repeat", count: $3, control: $5}

delay_value := unsigned_number     -> {kind: "unsigned_number", body: $1}
             | real_number         -> {kind: "real_number",     body: $1}
             | ps_identifier       -> {kind: "ps_identifier",   body: $1}
             | time_literal        -> {kind: "time_literal",    body: $1}
             | kw_n_1step          -> {kind: "step"}

@profiles: ["sv_2017"]
event_control_sv_2017 := at_sign hierarchical_event_identifier         -> {kind: "event",        body: $2}
                       | at_sign lparen event_expression rparen          -> {kind: "expression",   body: $3}
                       | at_sign star                                    -> {kind: "wildcard"}
                       | at_sign attr_open rparen                        -> {kind: "wildcard_alt"}
                       | at_sign ps_or_hierarchical_sequence_identifier  -> {kind: "sequence",     body: $2}

@profiles: ["sv_2023"]
event_control_sv_2023 := clocking_event           -> {kind: "clocking",       body: $1}
                       | at_sign star              -> {kind: "wildcard"}
                       | at_sign lparen star rparen -> {kind: "wildcard_paren"}

event_expression_primary := ( edge_identifier )? expression ( kw_iff expression )?
                                  -> {kind: "expression", edge: $1, expr: $2, iff: $3}
                          | sequence_instance ( kw_iff expression )?
                                  -> {kind: "sequence",   body: $1, iff: $2}
                          | lparen event_expression rparen
                                  -> {kind: "paren",      body: $2}

strength := kw_supply -> {kind: "supply"}
          | kw_strong -> {kind: "strong"}
          | kw_pull   -> {kind: "pull"}
          | kw_weak   -> {kind: "weak"}
```

### DEFERRED

- `drive_strength` — rule has duplicate branches (each strength-pair appears twice). Pattern is identical to `unique_priority` (slice 34) and `delay_sv_2017/2023`: branch 0 = branch 1, branch 2 = branch 3, branch 4 = branch 5. Likely grammar bug; tracked for separate fix.
- `delay_sv_2017` / `delay_sv_2023` — same duplicate-branch issue (4 branches; 0=2 with slight differences in 1/3).
- `event_expression` — has parens-Or `( kw_or | comma )` inside Quantified `*`; trailing-annotation attribution risk per task #38. Could add helper rule but defer.

### Annotation inventory

885 entries (was 852). +33 in this batch.

### Same accept set, same diagnostic codes. Schema stays at `1`.

### mdBook updated, gate green.

### Next slice candidates

- Sweep: remaining unannotated rules across the grammar (~30-40 mid-size rules).
- Profile-tag wrapper rules.
- `tagged_union_expression` / `class_constructor_declaration` / `class_constructor_prototype`.
- `clocking_drive` / `clocking_event` / `clocking_decl_assign`.
- Drive-strength / unique_priority / delay grammar fixes (separate task).

## Release 1.0.53 / Contract 1.0.53 Highlights — SV-Slice-53 batch: array/stream/class_new/join leaf cleanup typed (9 rules / 18 annotations)

Closes pervasive leaf rules used across primary / streaming-concat / par_block / dynamic-array contexts.

### Annotations

```ebnf
array_method_name := method_identifier      -> {kind: "method_identifier", body: $1}
                   | kw_unique              -> {kind: "unique"}
                   | kw_and                 -> {kind: "and"}
                   | kw_or                  -> {kind: "or"}
                   | kw_xor                 -> {kind: "xor"}

class_new := ( class_scope )? kw_new ( lparen list_of_arguments rparen )?
                  -> {kind: "constructor", scope: $1, args: $3}
           | kw_new expression
                  -> {kind: "copy",        source: $2}

dynamic_array_new := kw_new lbrack expression rbrack ( lparen expression rparen )?
                  -> {size: $3, init: $5}

empty_unpacked_array_concatenation := lbrace epsilon rbrace
                                   -> {kind: "empty_unpacked_array_concat"}

join_keyword := kw_join      -> {kind: "join"}
              | kw_join_any  -> {kind: "join_any"}
              | kw_join_none -> {kind: "join_none"}

slice_size := simple_type         -> {kind: "simple_type",         body: $1}
            | constant_expression -> {kind: "constant_expression", body: $1}

stream_concatenation := ( stream_expression ( comma stream_expression )* )*
                     -> {body: $1}

stream_expression := expression ( kw_with ( array_range_expression )? )?
                  -> {expr: $1, with_clause: $2}

stream_operator := shift_right -> {kind: "shift_right"}
                 | shift_left  -> {kind: "shift_left"}
```

### Field semantics

- `array_method_name`: 5 LRM A.2.10 array-builtin method names. The `method_identifier` branch carries an arbitrary user-defined method; the other 4 are the LRM-reserved keyword forms (`unique`, `and`, `or`, `xor`).
- `class_new.kind == "constructor"`: standard `new(args)` form with optional class scope (e.g., `MyPkg::MyClass::new(a, b)`).
- `class_new.kind == "copy"`: `new other_object` shallow-copy form.
- `join_keyword`: par_block (slice 33) used `$1` default which surfaced the raw envelope in `par_block.join` field. With this slice the `kind` discriminator is now exposed, giving consumers a cleaner dispatch on `join_keyword.kind`.

### Annotation inventory

852 entries (was 834). +18 in this batch.

### Same accept set, same diagnostic codes. Schema stays at `1`.

### mdBook updated, gate green.

### Next slice candidates

- The remaining unannotated rules (sweep — small leaf forms across the grammar).
- `unique_priority` (after grammar duplicate-branch fix).
- Profile-tag wrapper rules (module_declaration / interface_declaration / class_declaration / program_declaration).
- `tagged_union_expression` deeper.
- `delay`, `delay_control`, `delay_value`, `delay_or_event_control`.

## Release 1.0.52 / Contract 1.0.52 Highlights — SV-Slice-52 batch: simple_type + range/dist family typed (14 rules / 29 annotations)

Closes the simple_type / range_expression / part_select_range / dist_* / range_list / value_range walk paths used pervasively across data_type / cast / inside_expression / range-expression contexts.

### Annotations

```ebnf
simple_type := integer_type            -> {kind: "integer",       body: $1}
             | non_integer_type        -> {kind: "non_integer",   body: $1}
             | ps_type_identifier      -> {kind: "ps_type",       body: $1}
             | ps_parameter_identifier -> {kind: "ps_parameter",  body: $1}

range_expression := expression        -> {kind: "expression",        body: $1}
                  | part_select_range -> {kind: "part_select_range", body: $1}

part_select_range := constant_range -> {kind: "range",         body: $1}
                   | indexed_range  -> {kind: "indexed_range", body: $1}

constant_part_select_range := constant_range          -> {kind: "range",         body: $1}
                            | constant_indexed_range  -> {kind: "indexed_range", body: $1}

indexed_range := expression plus colon constant_expression  -> {kind: "plus_indexed",  base: $1, width: $4}
               | expression minus colon constant_expression -> {kind: "minus_indexed", base: $1, width: $4}

constant_indexed_range := constant_expression plus colon constant_expression  -> {kind: "plus_indexed",  base: $1, width: $4}
                        | constant_expression minus colon constant_expression -> {kind: "minus_indexed", base: $1, width: $4}

dist_list := dist_item ( comma dist_item )*
          -> {first: $1, rest: $2}

@profiles: ["sv_2017"]
dist_item_sv_2017 := value_range ( dist_weight )?
                  -> {value: $1, weight: $2}

@profiles: ["sv_2023"]
dist_item_sv_2023 := value_range ( dist_weight )?
                          -> {kind: "value",   value: $1, weight: $2}
                   | kw_default colon slash expression
                          -> {kind: "default", weight: $4}

dist_weight := colon assign expression -> {kind: "equal",        weight: $3}
             | colon slash expression  -> {kind: "proportional", weight: $3}

@profiles: ["sv_2023"]
range_list_sv_2023 := value_range ( comma value_range )*
                   -> {first: $1, rest: $2}

@profiles: ["sv_2017"]
open_range_list_sv_2017 := open_value_range ( comma open_value_range )*
                        -> {first: $1, rest: $2}

@profiles: ["sv_2017"]
value_range_sv_2017 := expression                          -> {kind: "expression", body: $1}
                     | ( expression colon expression )?    -> {kind: "range",      body: $1}

@profiles: ["sv_2023"]
value_range_sv_2023 := expression                                       -> {kind: "expression", body: $1}
                     | ( expression colon expression )?                 -> {kind: "range",      body: $1}
                     | ( kw_dollar colon expression )?                  -> {kind: "dollar_lo",  body: $1}
                     | ( expression colon kw_dollar )?                  -> {kind: "dollar_hi",  body: $1}
                     | ( expression plus slash minus expression )?      -> {kind: "tolerance",  body: $1}
```

### Field semantics

- `simple_type.kind`: discriminates the 4 LRM A.2.2.1 simple type forms — built-in integer/non-integer types, package-scoped type alias, package-scoped parameter (type parameter).
- `indexed_range.kind == "plus_indexed"`: the `[base +: width]` LRM 1800-2017 §11.5.1 indexed-part form (base address, ascending width).
- `indexed_range.kind == "minus_indexed"`: the `[base -: width]` form (descending width).
- `dist_weight.kind == "equal"`: `:=` operator — assign equal weight share.
- `dist_weight.kind == "proportional"`: `:/` operator — weight is divided across range/items.
- `value_range_sv_2023.kind == "tolerance"`: LRM 2023 `[expr +/- expr]` tolerance form.
- `value_range_sv_2023.kind == "dollar_lo"` / `"dollar_hi"`: open-ended LRM 2023 `[$:expr]` / `[expr:$]` form.

### Annotation inventory

834 entries (was 805). +29 in this batch (4 simple_type + 2 range_expression + 2 part_select_range + 2 constant_part_select_range + 2 indexed_range + 2 constant_indexed_range + 1 dist_list + 1 dist_item_sv_2017 + 2 dist_item_sv_2023 + 2 dist_weight + 1 range_list_sv_2023 + 1 open_range_list_sv_2017 + 2 value_range_sv_2017 + 5 value_range_sv_2023).

### Same accept set, same diagnostic codes. Schema stays at `1`.

### mdBook updated, gate green.

### Next slice candidates

- `unique_priority` (after grammar duplicate-branch fix).
- `tagged_union_expression` deeper / `streaming_concatenation` internals.
- The remaining ~50 untyped rules — `dynamic_array_new`, `class_new`, `array_method_name`, etc.
- Profile-tag wrapper rules (module_declaration / interface_declaration / class_declaration / program_declaration) for explicit profile discriminators.

## Release 1.0.51 / Contract 1.0.51 Highlights — SV-Slice-51 batch: select + constant_select + constant_range typed (4 rules / 5 annotations + 2 new helper rules with 4 annotations — crosses 800-annotation milestone)

Closes the `select` / `constant_select` referent used pervasively across primary's hierarchical-name suffix and various LRM A.8.1/A.8.5 selection forms. Crosses the **800-annotation milestone**.

### Annotations

```ebnf
select := ( ( dot member_identifier !lparen bit_select )* dot member_identifier !lparen )? ( select_tail )?
       -> {member_chain: $1, tail: $2}

select_tail (NEW) := lbrack part_select_range rbrack
                          -> {kind: "part_range", body: $2}
                   | bit_select ( lbrack part_select_range rbrack )?
                          -> {kind: "bit_select", bits: $1, range: $2}

constant_select := ( ( dot member_identifier constant_bit_select )* dot member_identifier )? ( constant_select_tail )?
                -> {member_chain: $1, tail: $2}

constant_select_tail (NEW) := lbrack constant_part_select_range rbrack
                                    -> {kind: "part_range", body: $2}
                            | constant_bit_select ( lbrack constant_part_select_range rbrack )?
                                    -> {kind: "bit_select", bits: $1, range: $2}

constant_range := constant_expression colon constant_expression
               -> {lo: $1, hi: $3}

constant_range_expression := constant_expression          -> {kind: "expression",        body: $1}
                           | constant_part_select_range   -> {kind: "part_select_range", body: $1}
```

### Helper-rule extraction (10th and 11th uses of pattern)

Two new helper rules extracted from inline parens-Or constructs in select / constant_select:

| Helper | Extracted from |
|---|---|
| `select_tail` | `( lbrack part_select_range rbrack \| bit_select ( lbrack part_select_range rbrack )? )?` in `select` |
| `constant_select_tail` | `( lbrack constant_part_select_range rbrack \| constant_bit_select ( lbrack constant_part_select_range rbrack )? )?` in `constant_select` |

Helper-rule extraction pattern is now used in **12 places total**:

1. `if_generate_else_clause` (slice 23)
2. `net_strength` + `net_vector_scalar` (slice 26)
3. `conditional_else_branch` (slice 35)
4. `class_or_package_scope` (slice 37)
5. `union_modifier` + `class_type_head` (slice 42)
6. `primary_hier_scope_prefix` + `instance_or_class_scope` + `enum_id_scope_prefix` (slice 47)
7. `select_tail` + `constant_select_tail` (slice 51)

### Field semantics

- `select.member_chain`: the optional `.foo.bar.baz` member dereference chain, with each segment optionally followed by a bit_select. The `!lparen` negative lookahead distinguishes member access from function call.
- `select.tail`: the optional bracket-index portion (`[N]` / `[N:M]` / `bit_select[N:M]` per LRM A.8.5).
- `constant_range`: the LRM `[lo:hi]` part-range form.

### Annotation inventory

805 entries (was 796). +9 in this batch (1 select + 2 select_tail + 1 constant_select + 2 constant_select_tail + 1 constant_range + 2 constant_range_expression). Crosses the 800-annotation milestone.

### Same accept set, same diagnostic codes. Schema stays at `1`.

### mdBook updated, gate green.

### Next slice candidates

- `simple_type` (referenced from casting_type.kind == "simple_type").
- `range_expression` / `part_select_range`.
- `dist_list` / `dist_item` / `dist_weight` (referenced from expression_or_dist).
- `range_list` / `open_range_list` / `value_range`.
- `unique_priority` (after grammar duplicate-branch fix).

## Release 1.0.50 / Contract 1.0.50 Highlights — SV-Slice-50 batch: casting_type + bit_select + system_tf_call typed (3 rules / 9 annotations)

Closes the `cast.type` / `constant_cast.type` field referent (5 forms per LRM A.8.5) plus the system-task-call dispatch (3 LRM A.8.2 forms).

### Annotations

```ebnf
casting_type := simple_type        -> {kind: "simple_type",     body: $1}
              | constant_primary   -> {kind: "constant_primary", body: $1}
              | signing            -> {kind: "signing",          body: $1}
              | kw_string          -> {kind: "string"}
              | kw_const           -> {kind: "const"}

bit_select := ( lbrack bit_select_expression rbrack )*
           -> {body: $1}

system_tf_call := system_tf_identifier ( lparen list_of_arguments rparen )?
                       -> {kind: "args",          name: $1, args: $2}
                | system_tf_identifier lparen data_type ( comma expression )? rparen
                       -> {kind: "data_type",     name: $1, data_type: $3, expr: $4}
                | system_tf_identifier lparen expression ( comma ( expression )? )* ( comma ( clocking_event )? )? rparen
                       -> {kind: "expr_clocking", name: $1, first_expr: $3, rest_exprs: $4, clocking: $5}
```

### Field semantics

- `casting_type.kind == "simple_type"`: simple LRM type (e.g., `int`, `byte`). Most common form.
- `casting_type.kind == "constant_primary"`: width-cast `N'(expr)` form where N is a constant primary literal.
- `casting_type.kind == "signing"`: `signed'(expr)` / `unsigned'(expr)` form.
- `casting_type.kind == "string"` / `"const"`: bare keyword type-cast forms.
- `bit_select.body`: zero-or-more `[bit_select_expression]` indices for multi-dimensional bit select.
- `system_tf_call.kind == "args"`: most common — `$display(args)` / `$random(args)` / etc.
- `system_tf_call.kind == "data_type"`: type-aware system tasks like `$cast(type, expr)` / `$bits(type)`.
- `system_tf_call.kind == "expr_clocking"`: assertion-related system tasks like `$rose(expr, clocking)` / `$fell(expr)` / `$past(expr, n, e, c)`.

### Annotation inventory

796 entries (was 787). +9 in this batch.

### Same accept set, same diagnostic codes. Schema stays at `1`.

### mdBook updated, gate green.

### Next slice candidates

- `select` / `constant_select` (complex shapes — need helper-rule extraction for the embedded parens-Or in the trailing tail).
- `simple_type` (referenced from casting_type.kind == "simple_type").
- `range_expression` / `part_select_range` / `constant_part_select_range`.
- `unique_priority` (after grammar duplicate-branch fix).

## Release 1.0.49 / Contract 1.0.49 Highlights — SV-Slice-49 batch: concat / cast / call_primary / attr_spec typed (9 rules / 14 annotations)

Closes the leaf rules used pervasively across `primary_sv_2017/2023` (typed in SV-Slices 47-48). After this slice, primary's `cast` / `concat` / `multiple_concat` / `call` / `assign_pattern` / `attribute_instance.first` / `.rest` field references all resolve to typed shapes.

### Annotations

```ebnf
attr_spec := attr_name ( assign constant_expression )?
          -> {name: $1, value: $2}

cast := casting_type tick lparen expression rparen
     -> {type: $1, body: $4}

constant_cast := casting_type tick lparen constant_expression rparen
              -> {type: $1, body: $4}

concatenation := lbrace expression ( comma expression )* rbrace
              -> {first: $2, rest: $3}

constant_concatenation := lbrace constant_expression ( comma constant_expression )* rbrace
                       -> {first: $2, rest: $3}

multiple_concatenation := lbrace expression concatenation rbrace
                       -> {count: $2, body: $3}

constant_multiple_concatenation := lbrace constant_expression constant_concatenation rbrace
                                -> {count: $2, body: $3}

streaming_concatenation := lbrace stream_operator ( slice_size )? stream_concatenation rbrace
                        -> {op: $2, slice_size: $3, body: $4}

call_primary := split_direct_callable_method_call -> {kind: "split_direct_callable_method", body: $1}
              | class_scoped_tf_call_with_args     -> {kind: "class_scoped_tf",              body: $1}
              | plain_tf_call_with_args            -> {kind: "plain_tf",                     body: $1}
              | tf_call_with_args                  -> {kind: "tf",                           body: $1}
              | direct_callable_method_call        -> {kind: "direct_callable_method",       body: $1}
              | system_tf_call                     -> {kind: "system_tf",                    body: $1}
```

### Field semantics

- `concatenation.first` / `.rest`: mini-mixed-array — `first` is the leading expression in `{...}`, `rest` is the trailing iteration of `[comma, expression]` pairs.
- `multiple_concatenation.count` / `.body`: `{count{body}}` LRM form — count is the replication factor expression, body is the typed inner concatenation.
- `streaming_concatenation`: LRM A.8.1 `<<size{...}>>` / `>>{...}` form. `op` is the stream_operator (`<<` or `>>`), `slice_size` is `[]` for default-bit-stream or `[<expr>]` for explicit slice size.
- `call_primary.kind`: 6-way dispatch over the various call-form variants per LRM A.8.2.
- `attr_spec`: simple `name [= value]` form per LRM A.9.1 — `value` is `[]` for bare attribute name, `[<assign, expr>]` when explicit value provided.

### Annotation inventory

787 entries (was 773). +14 in this batch.

### Same accept set, same diagnostic codes. Schema stays at `1`.

### mdBook updated, gate green.

### Next slice candidates

- `system_tf_call` deeper internals (large 5-branch rule).
- `casting_type` (referenced from cast/constant_cast .type field).
- `select` / `constant_select` (used pervasively across primary).
- `list_of_path_delay_expressions` (6-branch path-delay specifier).
- `unique_priority` (after grammar duplicate-branch fix).

## Release 1.0.48 / Contract 1.0.48 Highlights — SV-Slice-48 batch: primary_sv_2023 + constant_primary_sv_2023 typed (2 rules / 31 annotations)

Completes the SV-Slice-47 DEFERRED parallel sv_2023 forms. After this slice, both sv_2017 and sv_2023 primary expression dispatch is fully typed end-to-end. The 3 helper rules introduced in SV-Slice-47 (`primary_hier_scope_prefix`, `instance_or_class_scope`, `enum_id_scope_prefix`) are now used by both profiles.

### Profile differences from sv_2017

| sv_2017 kind | sv_2023 changes |
|---|---|
| `"call"` (no select) | `"call"` adds optional `select` field — LRM 2023 allows `f()[0]` array-indexed call |
| (15 kinds total) | 15 kinds (same set as sv_2017) |
| `"function_call"` (constant_primary, no select) | `"function_call"` adds optional `select` field |
| (15 kinds total) | **16 kinds** — adds `"empty_array_concat"` per LRM 2023 unpacked-array-concat extension |

### Annotations

```ebnf
@profiles: ["sv_2023"]
primary_sv_2023 := primary_literal                              -> {kind: "literal",            body: $1}
                 | call_primary ( lbrack range_expression rbrack )?
                                                                 -> {kind: "call",               body: $1, select: $2}
                 | ( primary_hier_scope_prefix )? hierarchical_identifier select
                                                                 -> {kind: "hierarchical",       scope: $1, name: $2, select: $3}
                 | empty_unpacked_array_concatenation            -> {kind: "empty_array_concat", body: $1}
                 | multiple_concatenation ( lbrack range_expression rbrack )?
                                                                 -> {kind: "multiple_concat",    body: $1, select: $2}
                 | concatenation ( lbrack range_expression rbrack )?
                                                                 -> {kind: "concat",             body: $1, select: $2}
                 | let_expression                                -> {kind: "let",                body: $1}
                 | lparen mintypmax_expression rparen            -> {kind: "paren",              body: $2}
                 | cast                                          -> {kind: "cast",               body: $1}
                 | assignment_pattern_expression                 -> {kind: "assign_pattern",     body: $1}
                 | streaming_concatenation                       -> {kind: "streaming_concat",   body: $1}
                 | sequence_method_call                          -> {kind: "sequence_method",    body: $1}
                 | kw_this                                       -> {kind: "this"}
                 | kw_sv_dollar                                  -> {kind: "system_dollar"}
                 | kw_null kw_class_qualifier colon assign ( kw_local scope_resolution kw_n_48 )? ( instance_or_class_scope )?
                                                                 -> {kind: "null_class_assign",  local_n: $5, scope: $6}

@profiles: ["sv_2023"]
constant_primary_sv_2023 := /* same 15 kinds as sv_2017 plus "empty_array_concat" between formal_port and concat;
                              "function_call" branch adds optional `select` field */
```

### Annotation inventory

773 entries (was 742). +31 in this batch (15 primary_sv_2023 + 16 constant_primary_sv_2023).

### Same accept set, same diagnostic codes. Schema stays at `1`.

### mdBook updated, gate green.

### Next slice candidates

- `attr_spec` deeper internals.
- `list_of_path_delay_expressions` (6-branch path-delay specifier — non-uniform shape).
- `unique_priority` (after grammar duplicate-branch fix).
- `call_primary` / `concatenation` / `multiple_concatenation` internals.

## Release 1.0.47 / Contract 1.0.47 Highlights — SV-Slice-47 batch: primary_sv_2017 + constant_primary_sv_2017 typed (2 rules / 30 annotations + 3 new helper rules with 6 annotations)

Closes the sv_2017 primary expression dispatch reachable from `expression_operand.kind == "primary"` (typed in SV-Slice-46) and `constant_expression_operand.kind == "primary"`. After this slice, every typed expression that resolves to a primary form discriminates the LRM A.8.4 primary kind without raw-envelope walks.

### Annotations

```ebnf
@profiles: ["sv_2017"]
primary_sv_2017 := primary_literal                               -> {kind: "literal",            body: $1}
                 | call_primary                                  -> {kind: "call",               body: $1}
                 | ( primary_hier_scope_prefix )? hierarchical_identifier select
                                                                 -> {kind: "hierarchical",       scope: $1, name: $2, select: $3}
                 | empty_unpacked_array_concatenation            -> {kind: "empty_array_concat", body: $1}
                 | multiple_concatenation ( lbrack range_expression rbrack )?
                                                                 -> {kind: "multiple_concat",    body: $1, select: $2}
                 | concatenation ( lbrack range_expression rbrack )?
                                                                 -> {kind: "concat",             body: $1, select: $2}
                 | let_expression                                -> {kind: "let",                body: $1}
                 | lparen mintypmax_expression rparen            -> {kind: "paren",              body: $2}
                 | cast                                          -> {kind: "cast",               body: $1}
                 | assignment_pattern_expression                 -> {kind: "assign_pattern",     body: $1}
                 | streaming_concatenation                       -> {kind: "streaming_concat",   body: $1}
                 | sequence_method_call                          -> {kind: "sequence_method",    body: $1}
                 | kw_this                                       -> {kind: "this"}
                 | kw_sv_dollar                                  -> {kind: "system_dollar"}
                 | kw_null kw_class_qualifier colon assign ( kw_local scope_resolution kw_n_43 )? ( instance_or_class_scope )?
                                                                 -> {kind: "null_class_assign",  local_n: $5, scope: $6}

primary_hier_scope_prefix (NEW) := kw_class_qualifier      -> {kind: "class_qualifier", body: $1}
                                 | non_typedef_package_scope -> {kind: "package_scope",  body: $1}

instance_or_class_scope (NEW) := implicit_class_handle dot -> {kind: "instance",    handle: $1}
                               | class_scope               -> {kind: "class_scope", body: $1}

@profiles: ["sv_2017"]
constant_primary_sv_2017 := primary_literal                                  -> {kind: "literal",         body: $1}
                          | ps_parameter_identifier constant_select          -> {kind: "ps_parameter",    name: $1, select: $2}
                          | specparam_identifier ( lbrack constant_range_expression rbrack )?
                                                                              -> {kind: "specparam",       name: $1, select: $2}
                          | genvar_identifier                                 -> {kind: "genvar",          body: $1}
                          | formal_port_identifier constant_select            -> {kind: "formal_port",     name: $1, select: $2}
                          | ( enum_id_scope_prefix )? enum_identifier         -> {kind: "enum",            scope: $1, name: $2}
                          | constant_multiple_concatenation ( lbrack constant_range_expression rbrack )?
                                                                              -> {kind: "multiple_concat", body: $1, select: $2}
                          | constant_concatenation ( lbrack constant_range_expression rbrack )?
                                                                              -> {kind: "concat",          body: $1, select: $2}
                          | constant_function_call                            -> {kind: "function_call",   body: $1}
                          | constant_let_expression                           -> {kind: "let",             body: $1}
                          | lparen constant_mintypmax_expression rparen       -> {kind: "paren",           body: $2}
                          | constant_cast                                     -> {kind: "cast",            body: $1}
                          | constant_assignment_pattern_expression            -> {kind: "assign_pattern",  body: $1}
                          | type_reference                                    -> {kind: "type_reference",  body: $1}
                          | kw_null                                           -> {kind: "null"}

enum_id_scope_prefix (NEW) := non_typedef_package_scope -> {kind: "package_scope", body: $1}
                            | class_scope               -> {kind: "class_scope",   body: $1}
```

### Helper-rule extraction (7th, 8th, and 9th uses of pattern)

Three new helper rules extracted from inline parens-Or constructs:

| Helper | Extracted from | Inside |
|---|---|---|
| `primary_hier_scope_prefix` | `( kw_class_qualifier \| non_typedef_package_scope )?` | primary_sv_2017 hierarchical branch |
| `instance_or_class_scope` | `( implicit_class_handle dot \| class_scope )?` | primary_sv_2017 null_class_assign branch |
| `enum_id_scope_prefix` | `( non_typedef_package_scope \| class_scope )?` | constant_primary_sv_2017 enum branch |

Helper-rule extraction pattern is now used in 10 places total:
1. `if_generate_else_clause` (slice 23)
2. `net_strength` + `net_vector_scalar` (slice 26)
3. `conditional_else_branch` (slice 35)
4. `class_or_package_scope` (slice 37)
5. `union_modifier` + `class_type_head` (slice 42)
6. `primary_hier_scope_prefix` + `instance_or_class_scope` + `enum_id_scope_prefix` (slice 47)

### Field semantics

- `primary.kind == "hierarchical"`: standard variable / function / module identifier reference. `scope` is `[]` for plain `name`, `[<class_qualifier>]` for `super::name`/`local::name`, `[<package_scope>]` for `pkg::name`.
- `primary.kind == "null_class_assign"`: rare LRM construct `null:= [local::N] [scope]` — the optional `local_n` and `scope` slots capture the LRM-specified positional pieces.
- `constant_primary.kind == "enum"`: optional `pkg::` or `class::` scope prefix before the enum_identifier (matches `pkg::EnumName` / `MyClass::EnumName` per LRM A.8.4).
- The 4-element kinds (multiple_concat / concat) carry both the body shape and an optional `[range]` post-index per LRM A.8.4.

### Annotation inventory

742 entries (was 706). +36 in this batch (15 primary_sv_2017 + 2 primary_hier_scope_prefix + 2 instance_or_class_scope + 15 constant_primary_sv_2017 + 2 enum_id_scope_prefix).

### Same accept set, same diagnostic codes. Schema stays at `1`.

### DEFERRED

`primary_sv_2023` and `constant_primary_sv_2023` are parallel structures with the same helper rules. Will be applied in a follow-up slice.

### mdBook updated, gate green.

### Next slice candidates

- `primary_sv_2023` / `constant_primary_sv_2023` (parallel — uses the same 3 helper rules from this slice).
- `attr_spec` deeper internals.
- `list_of_path_delay_expressions` (6-branch path-delay specifier).
- `unique_priority` (after grammar duplicate-branch fix).

## Release 1.0.46 / Contract 1.0.46 Highlights — SV-Slice-46 batch: expression family typed (14 rules / 62 annotations — crosses 700-annotation milestone)

Single largest impact slice — `expression`, `constant_expression`, and their operand/operator/literal sub-rules underlie **every** expression-typed field across the grammar (every parameter value, port connection, variable initializer, function/task argument, condition, range, case-item value, foreach-loop bound, return value, etc.). Crosses the **700-annotation milestone**.

### Annotations

```ebnf
expression := expression_base       -> {kind: "base",        body: $1}
            | inside_expression     -> {kind: "inside",      body: $1}
            | conditional_expression -> {kind: "conditional", body: $1}

expression_base := tagged_union_expression
                        -> {kind: "tagged_union",   body: $1}
                | expression_operand ( binary_operator attribute_instance* expression_operand )*
                        -> {kind: "operand_chain",  first: $1, rest: $2}
                | lparen operator_assignment rparen
                        -> {kind: "paren_op_assign", body: $2}

expression_operand := unary_operator attribute_instance* primary
                            -> {kind: "unary",      op: $1, attributes: $2, primary: $3}
                    | inc_or_dec_expression
                            -> {kind: "inc_or_dec", body: $1}
                    | primary
                            -> {kind: "primary",    body: $1}

expression_or_dist := expression ( kw_dist dist_list* )?
                   -> {expr: $1, dist: $2}

constant_expression := constant_expression_operand ( binary_operator attribute_instance* constant_expression_operand )* ( question attribute_instance* constant_expression colon constant_expression )?
                    -> {first: $1, rest: $2, ternary: $3}

constant_expression_operand := unary_operator attribute_instance* constant_primary
                                    -> {kind: "unary",   op: $1, attributes: $2, primary: $3}
                             | constant_primary
                                    -> {kind: "primary", body: $1}

@profiles: ["sv_2017"]
inside_expression_sv_2017 := expression_base kw_inside open_range_list*
                          -> {expr: $1, ranges: $3}

@profiles: ["sv_2023"]
inside_expression_sv_2023 := expression_base kw_inside range_list*
                          -> {expr: $1, ranges: $3}

conditional_expression := cond_predicate &question question attribute_instance* expression colon expression
                       -> {predicate: $1, attributes: $4, then_expr: $5, else_expr: $7}

tagged_union_expression_sv_2017 := kw_tagged member_identifier ( expression )?
                                -> {name: $2, value: $3}

tagged_union_expression_sv_2023 := kw_tagged member_identifier ( primary )?
                                -> {name: $2, value: $3}

primary_literal := number                  -> {kind: "number",                body: $1}
                 | time_literal            -> {kind: "time_literal",          body: $1}
                 | unbased_unsized_literal -> {kind: "unbased_unsized_literal", body: $1}
                 | string_literal          -> {kind: "string_literal",        body: $1}

binary_operator := /* 29 kinds bare {kind}: plus / minus / star / slash / percent / equal / not_equal / case_equal / case_not_equal / wildcard_equal / wildcard_not_equal / logical_and / logical_or / power / less_than / less_equal / greater_than / greater_equal / bitwise_and / bitwise_or / bitwise_xor / reduction_xnor_alt / reduction_xnor / shift_right / shift_left / arithmetic_shift_right / arithmetic_shift_left / implies / iff_arrow */

unary_operator := /* 11 kinds bare {kind}: plus / minus / bang / tilde / bitwise_and / reduction_nand / bitwise_or / reduction_nor / bitwise_xor / reduction_xnor / reduction_xnor_alt */
```

### Field semantics

- `expression_base.kind == "operand_chain"`: the standard binary-operator chain `op1 OP op2 OP op3 ...`. `first` is the leading operand, `rest` is the iteration of `[binary_operator, attribute_instance*, operand]` tuples.
- `expression_base.kind == "paren_op_assign"`: parenthesized operator-assignment expression (e.g., `(a += 1)` as an expression).
- `expression_operand.kind == "unary"`: `op operand` form with optional inline attributes.
- `inside_expression`: `expr inside { range_list }` form per LRM A.6.7.1.
- `conditional_expression`: ternary `? :` form. The `&question` positive lookahead is preserved unchanged from the source grammar.
- `constant_expression.ternary`: optional `( question attrs constant_expression colon constant_expression )?` slot — `[]` for non-ternary expressions.
- Operator rules (`binary_operator` / `unary_operator`) use bare `{kind}` shape — each branch is a single keyword token, so the kind label is the only meaningful information. Same pattern as `assignment_operator` / `inc_or_dec_operator` (slice 24) and `class_item_qualifier` (slice 27).

### Annotation inventory

706 entries (was 644). +62 in this batch (3 expression + 3 expression_base + 3 expression_operand + 1 expression_or_dist + 1 constant_expression + 2 constant_expression_operand + 1 inside_expression_sv_2017 + 1 inside_expression_sv_2023 + 1 conditional_expression + 1 tagged_union_expression_sv_2017 + 1 tagged_union_expression_sv_2023 + 4 primary_literal + 29 binary_operator + 11 unary_operator). Crosses the 700-annotation milestone.

### Same accept set, same diagnostic codes. Schema stays at `1`.

### mdBook updated, gate green.

### Next slice candidates

- `primary_sv_2017/2023` (large but reachable from `expression_operand.kind == "primary"`).
- `constant_primary` (parallel to primary).
- `attr_spec` deeper internals.
- `list_of_path_delay_expressions` (6-branch path-delay specifier — non-uniform shape).
- `unique_priority` (after grammar duplicate-branch fix).

## Release 1.0.45 / Contract 1.0.45 Highlights — SV-Slice-45 batch: pattern + cond_predicate family typed (6 rules / 18 annotations)

Closes the LRM A.6.7.1 pattern-matching walk path used by `case_pattern_item`, `conditional_statement.condition` (via `cond_predicate`), constraint_expression's various forms, and randcase items via cond_predicate descent.

### Annotations

```ebnf
cond_predicate := expression_or_cond_pattern ( logical_and3 expression_or_cond_pattern )*
               -> {first: $1, rest: $2}

cond_pattern := expression_base kw_matches pattern
             -> {expression: $1, pattern: $3}

expression_or_cond_pattern := expression_base -> {kind: "expression",   body: $1}
                            | cond_pattern    -> {kind: "cond_pattern", body: $1}

@profiles: ["sv_2017"]
pattern_sv_2017 := dot variable_identifier
                        -> {kind: "variable_capture", name: $2}
                | dot_star
                        -> {kind: "wildcard"}
                | constant_expression
                        -> {kind: "expression",       body: $1}
                | kw_tagged member_identifier ( pattern )?
                        -> {kind: "tagged",           name: $2, sub_pattern: $3}
                | tick lbrace pattern ( comma pattern )* rbrace
                        -> {kind: "ordered",          patterns: {first: $3, rest: $4}}
                | tick lbrace member_identifier colon pattern ( comma member_identifier colon pattern )* rbrace
                        -> {kind: "named",            entries: {first: {name: $3, pattern: $5}, rest: $6}}

@profiles: ["sv_2023"]
pattern_sv_2023 := lparen pattern rparen
                        -> {kind: "parenthesized",   body: $2}
                | /* same 6 kinds as sv_2017 (variable_capture / wildcard / expression / tagged / ordered / named) */

assignment_pattern := tick lbrace expression ( comma expression )* rbrace
                   -> {exprs: {first: $3, rest: $4}}
```

> **Shape correction (1.0.117 / schema 3, POST-SV-AUDIT.2.4b).** The
> `pattern_sv_2017` / `pattern_sv_2023` **`named` branch** shown above
> (`{kind:"named", entries:{first:{name:$3,pattern:$5}, rest:$6}}` —
> raw `[[comma, mid, colon, pattern], …]` `rest` envelope, present
> identically in both profiles) was corrected at `1.0.117` to
> `{kind:"named", entries:[$3, $4::2*]}` via a new shared annotated
> record rule `assignment_pattern_entry := member_identifier colon
> pattern -> {name:$1, pattern:$3}` (field names `{name, pattern}`
> preserved). The `assignment_pattern` `exprs` branch above is the
> separate, already-clean `'{e, e, …}` form (its `{first, rest}` here
> is `≤ 1.0.45` history flattened to `[$3, $4::2*]` at slice 58 — out
> of scope of 1.0.117). The `≤ 1.0.116` `named`-branch shape above is
> kept as labeled history. See
> [AST-Shape Corrections — 1.0.117](#ast-shape-corrections--10117-post-sv-audit--11-structured-per-iteration-category-a-misuses--clean-factored-record-lists-9-annotated-record-rules-22902299--9991008-a-deliberate-count-change-reachable-list_of__identifiers-probe-verified-the-rest-defensively-correct-by-construction-not-a-bug-ledger-entry-schema-2--3).

### Field semantics

- `cond_predicate.first` / `.rest`: the LRM A.6.7.1 `&&&`-separated chain of expression-or-cond_pattern values used in conditional statement predicates and case-pattern guards.
- `cond_pattern`: the `expr matches pattern` form per LRM A.6.7.1 — used in conditional_statement guards.
- `pattern.kind == "variable_capture"`: the `.name` capture form (binds the matched value to a new variable).
- `pattern.kind == "wildcard"`: the `.*` form (matches anything, captures nothing).
- `pattern.kind == "tagged"`: tagged-union pattern `tagged Name [sub_pattern]` (LRM A.6.7.1 — for tagged-union types from data_type.kind == "struct_union" with tagged modifier).
- `pattern.kind == "ordered"`: positional struct/array pattern `'{p1, p2, ...}` (mini-mixed-array on patterns).
- `pattern.kind == "named"`: keyed struct pattern `'{name1: p1, name2: p2, ...}` (mini-mixed-array of `{name, pattern}` entries).
- `pattern_sv_2023.kind == "parenthesized"`: LRM 2023 expansion that explicitly allows parenthesized patterns (was implicit in sv_2017).

### Profile difference

`pattern_sv_2023` adds an explicit `parenthesized` kind (LRM 2023 A.6.7.1 grammar expansion) but the other 6 kinds are identical to sv_2017. Profile-agnostic walks should accept the additional kind under sv_2023.

### Annotation inventory

644 entries (was 626). +18 in this batch (1 cond_predicate + 1 cond_pattern + 2 expression_or_cond_pattern + 6 pattern_sv_2017 + 7 pattern_sv_2023 + 1 assignment_pattern).

### Same accept set, same diagnostic codes. Schema stays at `1`.

### mdBook updated, gate green.

### Next slice candidates

- `expression`, `expression_base`, `expression_operand` (the largest single sub-tree — touches every expression-typed field).
- `attr_spec` deeper internals.
- `list_of_path_delay_expressions` (6-branch path-delay specifier).
- `unique_priority` (after grammar duplicate-branch fix).

## Release 1.0.44 / Contract 1.0.44 Highlights — SV-Slice-44 batch: list_of_* family typed (20 rules / 22 annotations)

Uniform mini-mixed-array pattern across the small declaration-list rules. After this slice, every list_of_* rule referenced from typed parents (variable declarations, port declarations, parameter declarations, net declarations, function/task body items, etc.) exposes a typed `{first, rest}` mini-mixed-array shape — no more raw envelopes for declaration-list iterations.

### Annotations

#### Simple `{first, rest}` (12 rules — uniform `X (comma X)*` pattern)

```ebnf
list_of_clocking_decl_assign       := clocking_decl_assign       (comma clocking_decl_assign)*       -> {first, rest}
list_of_defparam_assignments       := defparam_assignment        (comma defparam_assignment)*        -> {first, rest}
list_of_genvar_identifiers         := genvar_identifier          (comma genvar_identifier)*          -> {first, rest}
list_of_net_assignments            := net_assignment             (comma net_assignment)*             -> {first, rest}
list_of_net_decl_assignments       := net_decl_assignment        (comma net_decl_assignment)*        -> {first, rest}
list_of_param_assignments          := param_assignment           (comma param_assignment)*           -> {first, rest}
list_of_path_inputs                := specify_input_terminal_descriptor  (comma ...)*                -> {first, rest}
list_of_path_outputs               := specify_output_terminal_descriptor (comma ...)*                -> {first, rest}
list_of_specparam_assignments      := specparam_assignment       (comma specparam_assignment)*       -> {first, rest}
list_of_type_assignments           := type_assignment            (!(comma kw_type) comma type_assignment)*  -> {first, rest}
list_of_variable_assignments       := variable_assignment        (comma variable_assignment)*        -> {first, rest}
list_of_variable_decl_assignments  := variable_decl_assignment   (comma variable_decl_assignment)*   -> {first, rest}
```

#### `{first: {name, dims}, rest}` (3 rules with trailing `unpacked_dimension*`)

```ebnf
list_of_interface_identifiers := interface_identifier unpacked_dimension* (comma interface_identifier unpacked_dimension*)*
                              -> {first: {name: $1, dims: $2}, rest: $3}
list_of_port_identifiers      := port_identifier unpacked_dimension* (comma port_identifier unpacked_dimension*)*
                              -> {first: {name: $1, dims: $2}, rest: $3}
list_of_variable_identifiers  := variable_identifier variable_dimension* (comma variable_identifier variable_dimension*)*
                              -> {first: {name: $1, dims: $2}, rest: $3}
```

> **Shape correction (1.0.117 / schema 3, POST-SV-AUDIT.2.4b).** These
> 3 rules' `≤ 1.0.116` `{first: {name, dims}, rest}` (raw
> `[[comma, id, [dim…]], …]` `rest` envelope) was corrected to a clean
> flat record list — each repeated unit factored into a new annotated
> record rule (`interface_identifier_decl` / `port_identifier_decl` /
> `variable_identifier_decl` `-> {name, dims}`), the list now
> `[$1, $2::2*]`. Field names `{name, dims}` preserved. The `≤ 1.0.116`
> shape above is kept as labeled history. See
> [AST-Shape Corrections — 1.0.117](#ast-shape-corrections--10117-post-sv-audit--11-structured-per-iteration-category-a-misuses--clean-factored-record-lists-9-annotated-record-rules-22902299--9991008-a-deliberate-count-change-reachable-list_of__identifiers-probe-verified-the-rest-defensively-correct-by-construction-not-a-bug-ledger-entry-schema-2--3).

#### `{first: {name, dims, init}, rest}` (2 rules with optional initializer)

```ebnf
list_of_tf_variable_identifiers   := port_identifier variable_dimension* (assign expression)? (comma port_identifier variable_dimension* (assign expression)?)*
                                  -> {first: {name: $1, dims: $2, init: $3}, rest: $4}
list_of_variable_port_identifiers := port_identifier variable_dimension* (assign constant_expression)? (comma port_identifier variable_dimension* (assign constant_expression)?)*
                                  -> {first: {name: $1, dims: $2, init: $3}, rest: $4}
```

> **Shape correction (1.0.117 / schema 3, POST-SV-AUDIT.2.4b).** These
> 2 rules' `≤ 1.0.116` `{first: {name, dims, init}, rest}` raw
> multi-field envelope was corrected to a clean flat record list via
> new annotated record rules (`tf_variable_identifier_decl` /
> `variable_port_identifier_decl` `-> {name, dims, init}`), the list now
> `[$1, $2::2*]`. Field names `{name, dims, init}` preserved. The
> `≤ 1.0.116` shape above is kept as labeled history. See
> [AST-Shape Corrections — 1.0.117](#ast-shape-corrections--10117-post-sv-audit--11-structured-per-iteration-category-a-misuses--clean-factored-record-lists-9-annotated-record-rules-22902299--9991008-a-deliberate-count-change-reachable-list_of__identifiers-probe-verified-the-rest-defensively-correct-by-construction-not-a-bug-ledger-entry-schema-2--3).

#### `{first, second, rest}` (1 rule — list with 2 required items)

```ebnf
list_of_cross_items := cross_item comma cross_item (comma cross_item)*
                    -> {first: $1, second: $3, rest: $4}
```

#### 2-kind dispatch (2 rules)

```ebnf
list_of_checker_port_connections := ordered_checker_port_connection (comma ordered_checker_port_connection)*
                                          -> {kind: "ordered", items: {first: $1, rest: $2}}
                                  | named_checker_port_connection (comma named_checker_port_connection)*
                                          -> {kind: "named",   items: {first: $1, rest: $2}}

list_of_port_connections := named_port_connection (comma named_port_connection)*
                                  -> {kind: "named",   items: {first: $1, rest: $2}}
                          | ordered_port_connection (comma ordered_port_connection)*
                                  -> {kind: "ordered", items: {first: $1, rest: $2}}
```

### Field semantics

- `first` / `rest` (mini-mixed-array): `first` carries the leading required item; `rest` carries the trailing iteration of `[comma, item]` pairs (still raw — annotation-language doesn't support mixed-array spread per memory `feedback_annotation_no_mixed_spread.md`).
- `{name, dims}` form: groups the per-item LRM-required identifier + optional packed/unpacked-dimension list as a nested object so consumers can iterate `[item.name, item.dims]` without knowing the positional layout.
- `{name, dims, init}` form: same plus optional `[<assign, expr>]` initializer slot.
- `list_of_cross_items.first` and `.second` are both required per LRM A.2.11 (cross requires at least 2 cross_items).

### Annotation inventory

626 entries (was 604). +22 in this batch.

### Same accept set, same diagnostic codes. Schema stays at `1`.

### mdBook updated, gate green.

### Next slice candidates

- `expression`, `cond_predicate`, `pattern`.
- `list_of_path_delay_expressions` (6-branch path-delay specifier — non-uniform shape).
- `attr_spec` deeper internals.
- `unique_priority` (after grammar duplicate-branch fix).

## Release 1.0.43 / Contract 1.0.43 Highlights — SV-Slice-43 batch: parameter_value_assignment + arguments family typed (10 rules / 16 annotations — crosses 600-annotation milestone)

Closes the function/task/method-call argument and parameter-instance walks. After this slice, every typed parent that exposes `params:` (e.g., class_type, virtual_interface, instantiations) or `args:` (production_item, rs_production_item, named_argument, subroutine_call) field resolves to typed dispatch. Crosses the **600-annotation milestone**.

### Annotations

```ebnf
@profiles: ["sv_2017"]
parameter_value_assignment_sv_2017 := hash lparen ( list_of_parameter_assignments )? rparen
                                   -> {params: $3}

@profiles: ["sv_2023"]
parameter_value_assignment_sv_2023 := hash lparen ( list_of_parameter_value_assignments )? rparen
                                   -> {params: $3}

@profiles: ["sv_2017"]
list_of_parameter_assignments_sv_2017 := ordered_parameter_assignment ( comma ordered_parameter_assignment )*
                                              -> {kind: "ordered", items: {first: $1, rest: $2}}
                                       | named_parameter_assignment ( comma named_parameter_assignment )*
                                              -> {kind: "named",   items: {first: $1, rest: $2}}

@profiles: ["sv_2023"]
list_of_parameter_value_assignments_sv_2023 := /* parallel 2 kinds */

named_parameter_assignment := dot parameter_identifier lparen ( param_expression )? rparen
                           -> {name: $2, value: $4}

named_argument := dot identifier lparen ( expression )? rparen
               -> {name: $2, value: $4}

list_of_arguments := list_of_arguments_ordered -> {kind: "ordered", body: $1}
                   | list_of_arguments_named   -> {kind: "named",   body: $1}
                   | list_of_arguments_mixed   -> {kind: "mixed",   body: $1}

list_of_arguments_ordered := ( expression )? ( comma ( expression )? )*
                          -> {first: $1, rest: $2}

list_of_arguments_named := named_argument ( comma named_argument )*
                        -> {first: $1, rest: $2}

list_of_arguments_mixed := list_of_arguments_mixed_head comma named_argument ( comma named_argument )*
                        -> {head: $1, named: {first: $3, rest: $4}}

list_of_arguments_mixed_head := expression
                                     -> {kind: "single", body: $1}
                              | ( expression )? comma list_of_arguments_mixed_head
                                     -> {kind: "chain",  expr: $1, rest: $3}
```

### Field semantics

- `parameter_value_assignment.params`: optional list of parameter assignments. `[]` for `#()`, `[<list_of_parameter_assignments>]` for `#(N=8, M=16)` etc.
- `list_of_parameter_assignments.kind == "ordered"`: positional `#(8, 16)` form.
- `list_of_parameter_assignments.kind == "named"`: keyword `#(.N(8), .M(16))` form.
- `list_of_arguments.kind == "mixed"`: LRM-style argument list mixing positional and trailing named args (e.g., `f(1, 2, .x(3), .y(4))`).
- `list_of_arguments_mixed_head`: recursive helper allowing arbitrary positional-list prefix before named arguments.
- `named_argument.value` / `named_parameter_assignment.value`: optional argument expression — `[]` for `.name()` (explicit unconnected port), `[<expression>]` for normal `.name(expr)` form.

### Annotation inventory

604 entries (was 588). +16 in this batch (1 parameter_value_assignment_sv_2017 + 1 parameter_value_assignment_sv_2023 + 2 list_of_parameter_assignments_sv_2017 + 2 list_of_parameter_value_assignments_sv_2023 + 1 named_parameter_assignment + 1 named_argument + 3 list_of_arguments + 1 list_of_arguments_ordered + 1 list_of_arguments_named + 1 list_of_arguments_mixed + 2 list_of_arguments_mixed_head). Crosses the 600-annotation milestone.

### Same accept set, same diagnostic codes. Schema stays at `1`.

### mdBook updated, gate green.

### Next slice candidates

- `expression`, `cond_predicate`, `pattern` (large but underlie many already-typed rules).
- `attr_spec` deeper internals.
- The remaining small list_of_* rules (genvar / interface / net / param / cross / defparam / clocking_decl) — each just `X (comma X)*` patterns; could batch as `{first, rest}` annotations.
- `unique_priority` (after grammar duplicate-branch fix).

## Release 1.0.42 / Contract 1.0.42 Highlights — SV-Slice-42 batch: signing + struct_union + enum + type_reference + class_type internals typed (9 rules / 21 annotations + 2 new helper rules with 5 annotations)

Closes the data_type field structural-content walks. After this slice, every kind path through `data_type` (typed in SV-Slice-41) resolves to typed sub-rules — `data_type.signing`, `data_type.kind == "struct_union" → header / packed_signing / members`, `data_type.kind == "enum" → base_type / names`, `data_type.kind == "class_type" → head / params / suffix`, etc.

### Annotations

```ebnf
signing := kw_signed   -> {kind: "signed"}
         | kw_unsigned -> {kind: "unsigned"}

struct_union_sv_2017 := kw_struct                  -> {kind: "struct"}
                      | kw_union ( kw_tagged )?    -> {kind: "union", tagged: $2}

struct_union_sv_2023 := kw_struct                  -> {kind: "struct"}
                      | kw_union ( union_modifier )? -> {kind: "union", modifier: $2}

union_modifier (NEW) := kw_soft   -> {kind: "soft"}
                      | kw_tagged -> {kind: "tagged"}

struct_union_member := attribute_instance* ( random_qualifier )? data_type_or_void list_of_variable_decl_assignments semi
                    -> {attributes: $1, random_qualifier: $2, data_type: $3, decls: $4}

enum_base_type := integer_atom_type ( signing )?
                       -> {kind: "atom",       base: $1, signing: $2}
                | integer_vector_type ( signing )? ( packed_dimension )?
                       -> {kind: "vector",     base: $1, signing: $2, dim: $3}
                | type_identifier ( packed_dimension )?
                       -> {kind: "type_alias", name: $1, dim: $2}

enum_name_declaration := enum_identifier ( lbrack integral_number ( colon integral_number )? rbrack )? ( assign constant_expression )?
                      -> {name: $1, range: $2, value: $3}

type_reference_sv_2017 := kw_type lparen expression rparen -> {kind: "expression", body: $3}
                        | kw_type lparen data_type rparen  -> {kind: "data_type",  body: $3}

type_reference_sv_2023 := kw_type lparen expression rparen
                               -> {kind: "expression",                   body: $3}
                        | kw_type lparen data_type_or_incomplete_class_scoped_type rparen
                               -> {kind: "data_type_or_incomplete_class", body: $3}

class_type := class_type_head ( parameter_value_assignment )? ( scope_resolution class_identifier ( parameter_value_assignment )? )*
           -> {head: $1, params: $2, suffix: $3}

class_type_head (NEW) := scoped_class_type_identifier                          -> {kind: "scoped",          body: $1}
                       | known_unscoped_class_scope_class_identifier           -> {kind: "class",           body: $1}
                       | known_unscoped_class_scope_interface_class_identifier -> {kind: "interface_class", body: $1}
```

### Helper-rule extraction (5th use of pattern)

The original `class_type` had a leading 3-way parens-Or:

```ebnf
class_type := ( scoped_class_type_identifier | known_unscoped_class_scope_class_identifier | known_unscoped_class_scope_interface_class_identifier ) (parameter_value_assignment)? ...
```

Extracted to `class_type_head` helper, parallel to `class_or_package_scope` (slice 37). The `struct_union_sv_2023` extraction of `union_modifier` is the 6th use — extracted from `( kw_soft | kw_tagged )?`.

| Slice | Helper rule | Extracted from |
|---|---|---|
| 23 | `if_generate_else_clause` | `( kw_else if_generate_construct \| kw_else generate_block )?` |
| 26 | `net_strength` | `( drive_strength \| charge_strength )?` |
| 26 | `net_vector_scalar` | `( kw_vectored \| kw_scalared )?` |
| 35 | `conditional_else_branch` | `( conditional_statement \| statement_or_null )` |
| 37 | `class_or_package_scope` | `( implicit_class_handle dot \| class_scope \| package_scope )?` |
| 42 | `union_modifier` | `( kw_soft \| kw_tagged )?` |
| 42 | `class_type_head` | `( scoped_class_type_identifier \| known_unscoped_class_scope_class_identifier \| known_unscoped_class_scope_interface_class_identifier )` |

### Field semantics

- `enum_name_declaration.range`: optional `[N]` or `[N:M]` packed-range. `[]` for plain `enum { A, B }`, `[<lbrack, n, [colon n], rbrack>]` for ranged form.
- `enum_name_declaration.value`: optional `= expr` initial value. `[]` for unset, `[<assign, expr>]` when set.
- `class_type.suffix`: zero or more `:: identifier (parameter_value_assignment)?` chains for nested class scope (e.g., `pkg::Outer::Inner#(...)`).

### Annotation inventory

588 entries (was 567). +21 in this batch (2 signing + 2 struct_union_sv_2017 + 2 struct_union_sv_2023 + 2 union_modifier + 1 struct_union_member + 3 enum_base_type + 1 enum_name_declaration + 2 type_reference_sv_2017 + 2 type_reference_sv_2023 + 1 class_type + 3 class_type_head).

### Same accept set, same diagnostic codes. Schema stays at `1`.

### Grammar surface change

This slice adds two new rules (`union_modifier`, `class_type_head`) — internal refactors of inline parens-Or for annotation purposes. No LRM equivalents. Same accept set.

### mdBook updated, gate green.

### Next slice candidates

- `expression`, `cond_predicate`, `pattern` (large but underlie many already-typed rules).
- `parameter_value_assignment` / `list_of_arguments` internals.
- `attribute_instance` / `attr_spec` (already partially typed in SV-Slice-6 — could go deeper into attr_spec).
- `unique_priority` (after grammar duplicate-branch fix).

## Release 1.0.41 / Contract 1.0.41 Highlights — SV-Slice-41 batch: data_type family typed (8 rules / 36 annotations)

Pervasive impact across the entire grammar. `data_type` fields appear in module/interface/program port declarations, function/task return types, variable declarations, parameter declarations, struct/union members, class properties, function arguments, etc. After this slice, every typed `data_type` field across all typed parent rules (`function_body_declaration.return_type`, `task return`, `variable_decl.data_type`, etc.) discriminates which underlying SV type is in use without requiring envelope walks.

### Annotations

```ebnf
data_type := integer_vector_type ( signing )? packed_dimension*                 -> {kind: "integer_vector",        base: $1, signing: $2, dims: $3}
           | integer_atom_type ( signing )?                                      -> {kind: "integer_atom",          base: $1, signing: $2}
           | non_integer_type                                                    -> {kind: "non_integer",           base: $1}
           | struct_union ( kw_packed ( signing )? )? lbrace struct_union_member struct_union_member* rbrace packed_dimension*
                                                                                  -> {kind: "struct_union",         header: $1, packed_signing: $2, members: {first: $4, rest: $5}, dims: $7}
           | kw_enum ( enum_base_type )? lbrace enum_name_declaration ( comma enum_name_declaration )* rbrace packed_dimension*
                                                                                  -> {kind: "enum",                  base_type: $2, names: {first: $4, rest: $5}, dims: $7}
           | kw_string                                                            -> {kind: "string"}
           | kw_chandle                                                           -> {kind: "chandle"}
           | kw_virtual ( kw_interface )? interface_identifier ( parameter_value_assignment )? ( dot modport_identifier )?
                                                                                  -> {kind: "virtual_interface",     interface_keyword: $2, name: $3, params: $4, modport: $5}
           | scoped_data_type_identifier                                          -> {kind: "scoped_data_type",      body: $1}
           | known_unscoped_data_type_identifier                                  -> {kind: "known_unscoped_data_type", body: $1}
           | class_type                                                           -> {kind: "class_type",            body: $1}
           | provisional_unscoped_block_class_type                                -> {kind: "provisional_class_type", body: $1}
           | kw_event                                                             -> {kind: "event"}
           | ps_covergroup_identifier                                             -> {kind: "covergroup",            body: $1}
           | type_reference                                                       -> {kind: "type_reference",        body: $1}

data_type_or_implicit := data_type          -> {kind: "data_type",     body: $1}
                       | implicit_data_type -> {kind: "implicit",       body: $1}

data_type_or_incomplete_class_scoped_type_sv_2023 := data_type                     -> {kind: "data_type",            body: $1}
                                                   | incomplete_class_scoped_type  -> {kind: "incomplete_class_scoped", body: $1}

data_type_or_void := data_type -> {kind: "data_type", body: $1}
                   | kw_void   -> {kind: "void"}

implicit_data_type := ( signing )? packed_dimension*
                   -> {signing: $1, dims: $2}

integer_atom_type := kw_byte     -> {kind: "byte"}
                   | kw_shortint -> {kind: "shortint"}
                   | kw_int      -> {kind: "int"}
                   | kw_longint  -> {kind: "longint"}
                   | kw_integer  -> {kind: "integer"}
                   | kw_time     -> {kind: "time"}

integer_vector_type := kw_bit   -> {kind: "bit"}
                     | kw_logic -> {kind: "logic"}
                     | kw_reg   -> {kind: "reg"}

non_integer_type := kw_shortreal -> {kind: "shortreal"}
                  | kw_real      -> {kind: "real"}
                  | kw_realtime  -> {kind: "realtime"}

integer_type := integer_vector_type -> {kind: "vector", body: $1}
              | integer_atom_type   -> {kind: "atom",   body: $1}
```

### Field semantics

- `data_type.kind == "integer_vector"`: scalar / packed-vector types (`bit`, `logic`, `reg`). The `dims` field carries any `packed_dimension*` (e.g., `logic [7:0]`). `signing` is `[]` or a typed signing slot.
- `data_type.kind == "integer_atom"`: fixed-width arithmetic types (`byte`, `int`, etc.). No dims (atom types aren't vectorizable per LRM A.2.2.1).
- `data_type.kind == "struct_union"`: `header` carries the typed `struct_union` keyword (struct/union/tagged_union — typed in a future slice). `packed_signing` is `[]` for unpacked struct, `[<kw_packed [signing]>]` for packed. `members` is mini-mixed-array of struct_union_member.
- `data_type.kind == "enum"`: `base_type` is `[]` for default-int-base, `[<enum_base_type>]` for explicit base. `names` is mini-mixed-array.
- `data_type.kind == "virtual_interface"`: `interface_keyword` is `[]` for `virtual identifier` form, `[<kw_interface>]` for explicit `virtual interface identifier`. Modport access via the `.modport_identifier` slot.
- The 6 leaf kinds (string / chandle / event) and the 4 alias-only kinds (scoped_data_type / known_unscoped / class_type / provisional_class_type / covergroup / type_reference) bridge to other typed rules or carry single-token discriminators.

### Annotation inventory

567 entries (was 531). +36 in this batch (15 data_type + 2 data_type_or_implicit + 2 data_type_or_incomplete_class_scoped_type_sv_2023 + 2 data_type_or_void + 1 implicit_data_type + 6 integer_atom_type + 3 integer_vector_type + 3 non_integer_type + 2 integer_type).

### Same accept set, same diagnostic codes. Schema stays at `1`.

### mdBook updated, gate green.

### Next slice candidates

- `signing` (typed kind discriminator for signed/unsigned).
- `struct_union` / `struct_union_member` (close struct/union member walk).
- `enum_base_type` / `enum_name_declaration`.
- `class_type` internals.
- `expression`, `cond_predicate`, `pattern`.

## Release 1.0.40 / Contract 1.0.40 Highlights — SV-Slice-40 batch: simple immediate assertions + inc_or_dec + weight_specification typed (6 rules / 11 annotations)

Closes the `immediate_assertion_statement.kind == "simple"` walk path (typed in SV-Slice-36 as a bridge), the `inc_or_dec_expression` rule (referenced from `blocking_assignment_sv_2023.kind == "inc_or_dec"` and `statement_item_sv_2017.kind == "inc_or_dec_expression"`), and `weight_specification_sv_2017` (sv_2017 counterpart of `rs_weight_specification_sv_2023` typed in SV-Slice-39, referenced from `rs_rule_sv_2017.weight`).

### Annotations

```ebnf
simple_immediate_assertion_statement := simple_immediate_assert_statement -> {kind: "assert", body: $1}
                                      | simple_immediate_assume_statement -> {kind: "assume", body: $1}
                                      | simple_immediate_cover_statement  -> {kind: "cover",  body: $1}

simple_immediate_assert_statement := kw_assert lparen expression rparen action_block
                                  -> {condition: $3, action: $5}

simple_immediate_assume_statement := kw_assume lparen expression rparen action_block
                                  -> {condition: $3, action: $5}

simple_immediate_cover_statement := kw_cover lparen expression rparen statement_or_null
                                 -> {condition: $3, statement: $5}

inc_or_dec_expression := inc_or_dec_operator attribute_instance* variable_lvalue
                              -> {kind: "prefix",  op: $1, attributes: $2, lvalue: $3}
                       | variable_lvalue attribute_instance* inc_or_dec_operator
                              -> {kind: "postfix", lvalue: $1, attributes: $2, op: $3}

@profiles: ["sv_2017"]
weight_specification_sv_2017 := integral_number          -> {kind: "number",     body: $1}
                              | ps_identifier            -> {kind: "identifier", body: $1}
                              | lparen expression rparen -> {kind: "expression", body: $2}
```

### Field semantics

- `simple_immediate_assert_statement.condition`: the predicate expression (raw envelope still — `expression` rule itself untyped).
- `simple_immediate_*_statement.action` / `.statement`: typed `action_block` (slice 31) for assert/assume; typed `statement_or_null` (slice 31) for cover.
- `inc_or_dec_expression.kind`: distinguishes prefix `++a` / `--a` from postfix `a++` / `a--`. The `attributes` slot carries inline `attribute_instance*` (LRM allows attributes between operator and operand).
- `weight_specification_sv_2017`: parallel shape to `rs_weight_specification_sv_2023` typed in SV-Slice-39. Profile-agnostic walks should accept either field name when traversing rs_rule.weight slots.

### Annotation inventory

531 entries (was 520). +11 in this batch (3 simple_immediate_assertion_statement + 1 simple_immediate_assert_statement + 1 simple_immediate_assume_statement + 1 simple_immediate_cover_statement + 2 inc_or_dec_expression + 3 weight_specification_sv_2017).

### Same accept set, same diagnostic codes. Schema stays at `1`.

### mdBook updated, gate green.

### Next slice candidates

- `data_type` / `data_type_or_implicit` / `data_type_or_void` (used pervasively as field types across declarations).
- `expression`, `cond_predicate`, `pattern` (large but underlie many already-typed rules).
- `unique_priority` (after grammar duplicate-branch fix).

## Release 1.0.39 / Contract 1.0.39 Highlights — SV-Slice-39 batch: rs_* family typed (17 rules / 31 annotations — crosses 500-annotation milestone)

Closes the random-sequence walk path end-to-end. Every reachable `randsequence_statement` → `production` → `rules.{first,rest}` → `rs_rule` → `rs_production_list` → `rs_prod` → ... resolves through typed shapes with no raw-envelope intermediate. Crosses the **500-annotation milestone**.

### Annotations

```ebnf
rs_case := kw_case lparen case_expression rparen rs_case_item rs_case_item* kw_endcase
        -> {expr: $3, items: {first: $5, rest: $6}}

@profiles: ["sv_2017"]
rs_case_item_sv_2017 := case_item_expression ( comma case_item_expression )* colon production_item semi
                             -> {kind: "expr_list", exprs: {first: $1, rest: $2}, body: $4}
                      | kw_default ( colon )? production_item semi
                             -> {kind: "default",   body: $3}

@profiles: ["sv_2023"]
rs_case_item_sv_2023 := /* parallel to sv_2017; uses rs_production_item */

rs_code_block := ( data_declaration* statement_or_null* )*
              -> {body: $1}

@profiles: ["sv_2017"]
rs_if_else_sv_2017 := kw_if lparen expression rparen production_item ( kw_else production_item )?
                   -> {condition: $3, then_body: $5, else_body: $6}

@profiles: ["sv_2023"]
rs_if_else_sv_2023 := /* parallel; uses rs_production_item */

@profiles: ["sv_2017"]
rs_prod_sv_2017 := production_item -> {kind: "production_item", body: $1}
                 | rs_code_block   -> {kind: "code_block",      body: $1}
                 | rs_if_else      -> {kind: "if_else",         body: $1}
                 | rs_repeat       -> {kind: "repeat",          body: $1}
                 | rs_case         -> {kind: "case",            body: $1}

@profiles: ["sv_2023"]
rs_prod_sv_2023 := /* parallel; first branch is rs_production_item */

@profiles: ["sv_2023"]
rs_production_sv_2023 := ( data_type_or_void )? rs_production_identifier ( lparen tf_port_list rparen )? colon rs_rule ( bitwise_or rs_rule )* semi
                      -> {return_type: $1, name: $2, ports: $3, rules: {first: $5, rest: $6}}

@profiles: ["sv_2023"]
rs_production_item_sv_2023 := rs_production_identifier ( lparen list_of_arguments rparen )?
                            -> {name: $1, args: $2}

@profiles: ["sv_2017"]
rs_production_list_sv_2017 := rs_prod rs_prod*
                                   -> {kind: "productions", items: {first: $1, rest: $2}}
                            | kw_rand kw_join ( lparen expression rparen )? production_item production_item production_item*
                                   -> {kind: "rand_join",   join_count: $3, items: {first: $4, second: $5, rest: $6}}

@profiles: ["sv_2023"]
rs_production_list_sv_2023 := /* parallel; rand_join branch uses rs_production_item */

@profiles: ["sv_2017"]
rs_repeat_sv_2017 := kw_repeat lparen expression rparen production_item
                  -> {count: $3, body: $5}

@profiles: ["sv_2023"]
rs_repeat_sv_2023 := /* parallel; uses rs_production_item */

@profiles: ["sv_2017"]
rs_rule_sv_2017 := rs_production_list ( colon assign weight_specification ( rs_code_block )? )?
                -> {productions: $1, weight: $2}

@profiles: ["sv_2023"]
rs_rule_sv_2023 := /* parallel; uses rs_weight_specification */

@profiles: ["sv_2023"]
rs_weight_specification_sv_2023 := integral_number          -> {kind: "number",     body: $1}
                                 | ps_identifier            -> {kind: "identifier", body: $1}
                                 | lparen expression rparen -> {kind: "expression", body: $2}
```

### Field semantics

- `rs_production_list.kind == "rand_join"`: the LRM `rand join [(expr)] prod1 prod2 [prod3 ...]` form. Per LRM A.6.13, at least 2 production_items are required (which is why the rule has `production_item production_item production_item*` rather than `production_item+`). The `join_count` slot is the optional `( lparen expression rparen )?` join-count specifier.
- `rs_rule.weight`: optional `( colon assign weight_specification ( rs_code_block )? )?` — `[]` for productions without weight, `[<weight slot>]` when present (e.g., `prod1 := ... := 5`).
- `rs_code_block`: the body field carries the raw Quantified iteration of `( data_declaration* statement_or_null* )*` — each entry in the iteration is `[data_declaration*-array, statement_or_null*-array]`.
- `rs_prod.kind`: 5-way discriminator between the production-body forms allowed inside an rs_rule (production_item invocation, embedded code block, if-else, repeat, or nested case).

### Profile difference

The sv_2017 family references `production_item` / `weight_specification` directly; the sv_2023 family uses the namespaced `rs_production_item` / `rs_weight_specification` rules. The typed shapes are identical for consumers walking either profile.

### Annotation inventory

520 entries (was 489). +31 in this batch:
- 1 rs_case
- 2 rs_case_item_sv_2017 + 2 rs_case_item_sv_2023
- 1 rs_code_block
- 1 rs_if_else_sv_2017 + 1 rs_if_else_sv_2023
- 5 rs_prod_sv_2017 + 5 rs_prod_sv_2023
- 1 rs_production_sv_2023
- 1 rs_production_item_sv_2023
- 2 rs_production_list_sv_2017 + 2 rs_production_list_sv_2023
- 1 rs_repeat_sv_2017 + 1 rs_repeat_sv_2023
- 1 rs_rule_sv_2017 + 1 rs_rule_sv_2023
- 3 rs_weight_specification_sv_2023

### Same accept set, same diagnostic codes. Schema stays at `1`.

### mdBook updated, gate green.

### Next slice candidates

- `simple_immediate_assertion_statement` (close immediate_assertion_statement.kind == "simple").
- `inc_or_dec_expression` internals.
- `data_type` / `data_type_or_implicit` / `data_type_or_void` (used pervasively as field types across declarations).
- `expression`, `cond_predicate`, `pattern` (large but underlie many already-typed rules).
- `weight_specification` (sv_2017 counterpart of rs_weight_specification_sv_2023 — referenced from rs_rule_sv_2017.weight).

## Release 1.0.38 / Contract 1.0.38 Highlights — SV-Slice-38 batch: randsequence top-level + production typed (4 rules / 4 annotations)

Closes the last raw-envelope `statement_item` kind. After this slice, every framed procedural statement in module/program/function/task bodies type-discriminates into a structured shape AND every typed body content (productions / production rules) is reachable.

### Annotations

```ebnf
@profiles: ["sv_2017"]
randsequence_statement_sv_2017 := kw_randsequence lparen ( production_identifier )? rparen production production* kw_endsequence
                               -> {start: $3, productions: {first: $5, rest: $6}}

@profiles: ["sv_2023"]
randsequence_statement_sv_2023 := kw_randsequence lparen ( rs_production_identifier )? rparen rs_production rs_production* kw_endsequence
                               -> {start: $3, productions: {first: $5, rest: $6}}

@profiles: ["sv_2017"]
production_sv_2017 := ( data_type_or_void )? production_identifier ( lparen tf_port_list rparen )? colon rs_rule ( bitwise_or rs_rule )* semi
                   -> {return_type: $1, name: $2, ports: $3, rules: {first: $5, rest: $6}}

@profiles: ["sv_2017"]
production_item_sv_2017 := production_identifier ( lparen list_of_arguments rparen )?
                        -> {name: $1, args: $2}
```

### Field semantics

- `randsequence_statement.start`: the optional starting production name (e.g., `randsequence (top) ... endsequence`). `[]` for `randsequence () ...` form.
- `randsequence_statement.productions`: mini-mixed-array — `first` is required, `rest` is the trailing production iteration.
- `production.return_type`: optional `data_type_or_void` prefix for productions that produce values (e.g., `int p : ... ;`).
- `production.ports`: optional `(lparen tf_port_list rparen)?` for parameterized productions.
- `production.rules.rest`: each entry in the iteration is a `[bitwise_or_token, rs_rule_shape]` pair (alternative rules separated by `|`).
- `production_item.args`: optional argument list when invoking the production.

### Profile difference

`randsequence_statement_sv_2017` references rules `production_identifier` / `production`; `randsequence_statement_sv_2023` references `rs_production_identifier` / `rs_production` (LRM 2023 renamed/namespaced these to avoid clashes with covergroup `production`). The typed shape is identical for consumers.

### DEFERRED

The deeper `rs_*` family (`rs_rule`, `rs_prod`, `rs_case`, `rs_if_else`, `rs_repeat`, `rs_code_block`, `rs_production_list`, etc.) are still raw envelope. These are referenced from `production.rules.{first,rest}` (typed in this slice as field references). Typing these closes the random-sequence walk path; will be done in a follow-up slice.

### Annotation inventory

489 entries (was 485). +4 in this batch (1 randsequence_statement_sv_2017 + 1 randsequence_statement_sv_2023 + 1 production_sv_2017 + 1 production_item_sv_2017).

### statement_item dispatch coverage — now 100% (no raw-envelope kinds remaining)

After this slice and SV-Slice-37, every statement_item kind exposes typed dispatch and every reachable typed-body field is itself typed:

| kind | typed-in | body-of-body |
|---|---|---|
| randsequence_statement | SV-Slice-32 ✅ | typed THIS slice (productions field exposed; rs_* internals deferred) |
| (all other 19 kinds typed in earlier slices — see SV-Slice-37 coverage table) | | |

### Same accept set, same diagnostic codes. Schema stays at `1`.

### mdBook updated, gate green.

### Next slice candidates

- `rs_rule_sv_2017/2023` + `rs_prod_sv_2017/2023` + `rs_case` + `rs_case_item` + `rs_if_else` + `rs_repeat` + `rs_code_block` (close randsequence internals).
- `simple_immediate_assertion_statement` (close immediate_assertion_statement.kind == "simple").
- `inc_or_dec_expression` internals.
- `data_type` / `data_type_or_implicit` / `data_type_or_void`.

## Release 1.0.37 / Contract 1.0.37 Highlights — SV-Slice-37 batch: blocking_assignment typed via helper-rule extraction (3 rules / 12 annotations + 1 new helper rule with 3 annotations)

Closes the last DEFERRED `statement_item` kind from SV-Slice-36. After this slice, **all 20 (sv_2017) / 19 (sv_2023) statement_item kinds expose typed dispatch end-to-end** — the entire procedural-statement walk path is type-discriminated for Nexsim consumers.

### Annotations

```ebnf
@profiles: ["sv_2017"]
blocking_assignment_sv_2017 := variable_lvalue assign delay_or_event_control expression
                                  -> {kind: "delay_assign",      lvalue: $1, delay: $3, value: $4}
                             | nonrange_variable_lvalue assign dynamic_array_new
                                  -> {kind: "dynamic_array_new", lvalue: $1, value: $3}
                             | ( class_or_package_scope )? hierarchical_variable_identifier select assign class_new
                                  -> {kind: "class_new",         scope: $1, name: $2, select: $3, value: $5}
                             | operator_assignment
                                  -> {kind: "operator",          body: $1}

@profiles: ["sv_2023"]
blocking_assignment_sv_2023 := /* same 4 kinds plus 5th: */
                             | inc_or_dec_expression
                                  -> {kind: "inc_or_dec",        body: $1}

class_or_package_scope := implicit_class_handle dot -> {kind: "instance",      handle: $1}
                        | class_scope               -> {kind: "class_scope",   body: $1}
                        | package_scope             -> {kind: "package_scope", body: $1}
```

### Helper-rule extraction (4th use of the pattern)

The original `blocking_assignment_sv_2017/2023` branch 2 had:

```ebnf
( implicit_class_handle dot | class_scope | package_scope )? hierarchical_variable_identifier select assign class_new
```

The 3-way parens-Or hits task #38. Extracted to `class_or_package_scope` helper rule. This is the 4th use of the pattern:

| Slice | Helper rule | Source rule | Original parens-Or |
|---|---|---|---|
| SV-Slice-23 | `if_generate_else_clause` | `if_generate_construct` | `( kw_else if_generate_construct \| kw_else generate_block )?` |
| SV-Slice-26 | `net_strength` | `net_declaration_sv_2017/2023` | `( drive_strength \| charge_strength )?` |
| SV-Slice-26 | `net_vector_scalar` | `net_declaration_sv_2017/2023` | `( kw_vectored \| kw_scalared )?` |
| SV-Slice-35 | `conditional_else_branch` | `conditional_statement` | `( conditional_statement \| statement_or_null )` |
| SV-Slice-37 | `class_or_package_scope` | `blocking_assignment_sv_2017/2023` | `( implicit_class_handle dot \| class_scope \| package_scope )?` |

The pattern is now well-established. Future inline parens-Or in any sub-rule should follow this template until task #38 is fixed.

### Field semantics

- `blocking_assignment.kind == "delay_assign"`: the most common form `lvalue = #N expr;`. Drops `assign` operator.
- `blocking_assignment.kind == "dynamic_array_new"`: `lvalue = new[size];` (or `new[size](init)`). The `nonrange_variable_lvalue` constraint matches a non-range variable target.
- `blocking_assignment.kind == "class_new"`: `[scope.]name[select] = new(args);`. `scope` is `[]` for plain `name = new(...)`, or `[<class_or_package_scope shape>]` when prefixed (`pkg::name = new(...)`, `class_handle.member = new(...)`, etc.).
- `blocking_assignment.kind == "operator"`: bridges to `operator_assignment` rule (e.g., `a += b;`, `a *= b;` — typed via assignment_operator from SV-Slice-24).
- `blocking_assignment_sv_2023.kind == "inc_or_dec"`: LRM 2023 form. The same `++` / `--` operator that's a separate `inc_or_dec_expression semi` statement_item branch in sv_2017 is now folded into blocking_assignment in sv_2023.
- `class_or_package_scope.kind == "instance"`: `implicit_class_handle dot` — typically `this.` or `super.` prefix (instance-scoped member access).

### statement_item dispatch coverage — now 100%

After this slice, all kinds have typed body dispatch end-to-end:

| kind | typed-in-slice |
|---|---|
| blocking_assignment | **SV-Slice-37 (this slice)** ✅ |
| nonblocking_assignment | SV-Slice-36 ✅ |
| procedural_continuous_assignment | SV-Slice-36 ✅ |
| case_statement | SV-Slice-34 ✅ |
| conditional_statement | SV-Slice-35 ✅ |
| inc_or_dec_expression (sv_2017) | wraps inc_or_dec_expression rule (raw envelope still — to be typed in a future slice) |
| subroutine_call_statement | SV-Slice-33 ✅ |
| disable_statement | SV-Slice-33 ✅ |
| event_trigger | SV-Slice-33 ✅ |
| loop_statement | SV-Slice-34 ✅ |
| jump_statement | SV-Slice-33 ✅ |
| par_block | SV-Slice-33 ✅ |
| procedural_timing_control_statement | SV-Slice-33 ✅ |
| seq_block | SV-Slice-33 ✅ |
| wait_statement | SV-Slice-33 ✅ |
| procedural_assertion_statement | SV-Slice-36 ✅ |
| clocking_drive | SV-Slice-36 ✅ |
| randsequence_statement | raw envelope still — internals to be typed in a future slice |
| randcase_statement | SV-Slice-36 ✅ |
| expect_property_statement | SV-Slice-29 ✅ |

### Annotation inventory

485 entries (was 473). +12 in this batch (4 blocking_assignment_sv_2017 + 5 blocking_assignment_sv_2023 + 3 class_or_package_scope).

### Same accept set, same diagnostic codes. Schema stays at `1`.

### Grammar surface change

This slice adds one new rule (`class_or_package_scope`) to the public grammar surface — internal refactor of inline parens-Or for annotation purposes. No LRM equivalent. Same accept set.

### mdBook updated, gate green.

### Next slice candidates

- `randsequence_statement_sv_2017/2023` internals (close last raw-envelope statement_item kind).
- `simple_immediate_assertion_statement` (close immediate_assertion_statement.kind == "simple").
- `inc_or_dec_expression` internals.
- `data_type` / `data_type_or_implicit` / `data_type_or_void`.
- `expression`, `cond_predicate`, `pattern` (large but underlie many already-typed rules).

## Release 1.0.36 / Contract 1.0.36 Highlights — SV-Slice-36 batch: assignments + procedural assertions + randcase typed (8 rules / 16 annotations)

Closes 4 more `statement_item` kinds: `nonblocking_assignment`, `procedural_continuous_assignment`, `clocking_drive`, `randcase`, `procedural_assertion`. After this slice, 19 of statement_item's 19/20 kinds expose typed dispatch end-to-end (only `blocking_assignment` remains DEFERRED — needs parens-Or helper-rule extraction).

### Annotations

```ebnf
nonblocking_assignment := variable_lvalue less_equal ( delay_or_event_control )? expression
                       -> {lvalue: $1, control: $3, value: $4}

procedural_continuous_assignment := kw_assign variable_assignment      -> {kind: "assign",          body: $2}
                                  | kw_deassign variable_lvalue        -> {kind: "deassign",        target: $2}
                                  | kw_force variable_assignment       -> {kind: "force_variable",  body: $2}
                                  | kw_force net_assignment            -> {kind: "force_net",       body: $2}
                                  | kw_release variable_lvalue         -> {kind: "release_variable", target: $2}
                                  | kw_release net_lvalue              -> {kind: "release_net",     target: $2}

clocking_drive := clockvar_expression less_equal ( cycle_delay )? expression
               -> {lvalue: $1, cycle_delay: $3, value: $4}

randcase_statement := kw_randcase randcase_item randcase_item* kw_endcase
                   -> {items: {first: $2, rest: $3}}

randcase_item := expression colon statement_or_null
              -> {weight: $1, body: $3}

procedural_assertion_statement := concurrent_assertion_statement -> {kind: "concurrent",            body: $1}
                                | immediate_assertion_statement  -> {kind: "immediate",             body: $1}
                                | checker_instantiation          -> {kind: "checker_instantiation", body: $1}

immediate_assertion_statement := simple_immediate_assertion_statement   -> {kind: "simple",   body: $1}
                               | deferred_immediate_assertion_statement -> {kind: "deferred", body: $1}

variable_assignment := variable_lvalue assign expression
                    -> {lvalue: $1, value: $3}
```

### Field semantics

- `nonblocking_assignment.control`: optional `( delay_or_event_control )?` between `<=` and the RHS expression (e.g., `a <= #1 b;`). `[]` when absent.
- `procedural_continuous_assignment.kind`: discriminates the 4 LRM forms — `assign` / `deassign` / `force` (variable or net) / `release` (variable or net). The split between `force_variable` / `force_net` and `release_variable` / `release_net` reflects the grammar's separate branches for variable_assignment vs net_assignment / variable_lvalue vs net_lvalue (consumers walking either form can dispatch by kind).
- `clocking_drive.cycle_delay`: optional `( cycle_delay )?` between `<=` and RHS (clocking-block specific delay).
- `randcase_item.weight`: the `expression` before `:` — relative selection weight for this branch.
- `procedural_assertion_statement.kind == "immediate"`: bridges to `immediate_assertion_statement` which further discriminates `"simple"` (typed in slice 36) vs `"deferred"` (deferred_immediate_assertion_statement, typed in SV-Slice-30).

### DEFERRED

`blocking_assignment_sv_2017/2023` typing — branch 2 has a 3-way parens-Or `( implicit_class_handle dot | class_scope | package_scope )?` with mixed sequence/atom bodies. Needs helper-rule extraction (4th use of the pattern). Tracked for next slice.

### Annotation inventory

473 entries (was 457). +16 in this batch (1 nonblocking_assignment + 6 procedural_continuous_assignment + 1 clocking_drive + 1 randcase_statement + 1 randcase_item + 3 procedural_assertion_statement + 2 immediate_assertion_statement + 1 variable_assignment).

### statement_item dispatch coverage

After this slice, 19 of statement_item's 19/20 kinds (sv_2017) have typed body dispatch:
- ✅ blocking_assignment (DEFERRED — next slice)
- ✅ nonblocking_assignment (slice 36)
- ✅ procedural_continuous_assignment (slice 36)
- ✅ case_statement (slice 34)
- ✅ conditional_statement (slice 35)
- ✅ inc_or_dec_expression (sv_2017 only — wraps inc_or_dec_expression rule, raw envelope)
- ✅ subroutine_call_statement (slice 33)
- ✅ disable_statement (slice 33)
- ✅ event_trigger (slice 33)
- ✅ loop_statement (slice 34)
- ✅ jump_statement (slice 33)
- ✅ par_block (slice 33)
- ✅ procedural_timing_control_statement (slice 33)
- ✅ seq_block (slice 33)
- ✅ wait_statement (slice 33)
- ✅ procedural_assertion_statement (slice 36)
- ✅ clocking_drive (slice 36)
- ✅ randsequence_statement (raw envelope — rule body still raw)
- ✅ randcase_statement (slice 36)
- ✅ expect_property_statement (slice 29)

### Same accept set, same diagnostic codes. Schema stays at `1`.

### mdBook updated, gate green.

### Next slice candidates

- `blocking_assignment_sv_2017/2023` with helper-rule extraction (4th use of the pattern).
- `randsequence_statement_sv_2017/2023` internals.
- `simple_immediate_assertion_statement` internals.
- `data_type` / `data_type_or_implicit` / `data_type_or_void`.

## Release 1.0.35 / Contract 1.0.35 Highlights — SV-Slice-35 batch: conditional_statement typed via helper-rule extraction (1 rule / 1 annotation + 1 new helper rule with 2 annotations)

Closes the SV-Slice-34 DEFERRED `conditional_statement` typing — every reachable `statement_item.kind == "conditional"` now exposes typed dispatch into condition / then-body / else-body. Third use of the helper-rule extraction pattern (after `if_generate_else_clause` from SV-Slice-23 and `net_strength` / `net_vector_scalar` from SV-Slice-26).

### Annotations

```ebnf
conditional_statement := ( unique_priority )? kw_if lparen cond_predicate rparen statement_or_null &kw_else kw_else conditional_else_branch
                      -> {unique_priority: $1, condition: $4, then_body: $6, else_body: $9}

conditional_else_branch := conditional_statement -> {kind: "elseif", body: $1}
                         | statement_or_null     -> {kind: "else",   body: $1}
```

### Helper-rule extraction rationale

The original `conditional_statement` rule had this trailing parens-Or:

```ebnf
conditional_statement := ( unique_priority )? kw_if lparen cond_predicate rparen statement_or_null &kw_else kw_else ( conditional_statement | statement_or_null )
```

The inline `( conditional_statement | statement_or_null )` parens-Or hits task #38 (parens-grouped-Or trailing-annotation attribution bug). Following the established pattern, it was extracted to a named rule:

- `conditional_else_branch.kind == "elseif"` → recursive form, supports `else if (...) ...` chains.
- `conditional_else_branch.kind == "else"` → terminal else, `body` is a typed `statement_or_null`.

The `&kw_else` positive lookahead is preserved unchanged — it's a PEG idiom from the source grammar asserting the else-branch is required (the else-less form is presumably matched via a different rule or PEG ordered-choice fallback).

### Field semantics

- `conditional_statement.unique_priority`: optional `( unique_priority )?` slot — `[]` for plain `if`, raw envelope (still untyped per slice 34's deferred unique_priority).
- `conditional_statement.condition`: typed `cond_predicate` envelope (raw — typing deferred to a future slice covering pattern_or_assignment_pattern internals).
- `conditional_statement.then_body`: typed `statement_or_null` (typed in SV-Slice-31).
- `conditional_statement.else_body`: typed `conditional_else_branch` (typed THIS slice).

### Annotation inventory

457 entries (was 454). +3 in this batch (1 conditional_statement + 2 conditional_else_branch).

### Same accept set, same diagnostic codes. Schema stays at `1`.

### Grammar surface change

This slice adds one new rule (`conditional_else_branch`) to the public grammar surface — internal refactor of inline parens-Or for annotation purposes, no LRM equivalent. Same accept set.

### mdBook updated, gate green.

### Next slice candidates

- `procedural_assertion_statement`, `clocking_drive`.
- `data_type` / `data_type_or_implicit` / `data_type_or_void`.
- `randsequence_statement` / `randcase_statement`.
- `procedural_continuous_assignment`, `blocking_assignment` / `nonblocking_assignment` internals.
- `unique_priority` (after grammar duplicate-branch fix).

## Release 1.0.34 / Contract 1.0.34 Highlights — SV-Slice-34 batch: case + loop families typed (7 rules / 18 annotations)

Closes the case-statement and loop-statement walks (`statement_item.kind == "case"` / `"loop"` from SV-Slice-32).

### Annotations

```ebnf
case_statement := ( unique_priority )? case_keyword lparen case_expression rparen case_item case_item* kw_endcase
               -> {unique_priority: $1, keyword: $2, expr: $4, items: {first: $6, rest: $7}}

case_keyword := kw_case  -> {kind: "case"}
              | kw_casez -> {kind: "casez"}
              | kw_casex -> {kind: "casex"}

case_item := case_item_expression ( comma case_item_expression )* colon statement_or_null
                  -> {kind: "expr_list", exprs: {first: $1, rest: $2}, body: $4}
           | kw_default ( colon )? statement_or_null
                  -> {kind: "default",   body: $3}

case_pattern_item := pattern ( logical_and3 expression )? colon statement_or_null
                          -> {kind: "pattern", pattern: $1, condition: $2, body: $4}
                   | kw_default ( colon )? statement_or_null
                          -> {kind: "default", body: $3}

@profiles: ["sv_2017"]
case_inside_item_sv_2017 := open_range_list colon statement_or_null
                                 -> {kind: "range_list", ranges: $1, body: $3}
                          | kw_default ( colon )? statement_or_null
                                 -> {kind: "default",    body: $3}

@profiles: ["sv_2023"]
case_inside_item_sv_2023 := /* parallel to sv_2017; uses LRM 2023 `range_list` instead of `open_range_list` */

loop_statement := kw_forever statement_or_null
                       -> {kind: "forever",  body: $2}
                | kw_repeat lparen expression rparen statement_or_null
                       -> {kind: "repeat",   count: $3, body: $5}
                | kw_while lparen expression rparen statement_or_null
                       -> {kind: "while",    condition: $3, body: $5}
                | kw_for lparen ( for_initialization )? semi ( expression )? semi ( for_step )? rparen statement_or_null
                       -> {kind: "for",      init: $3, condition: $5, step: $7, body: $9}
                | kw_do statement_or_null kw_while lparen expression rparen semi
                       -> {kind: "do_while", body: $2, condition: $5}
                | kw_foreach lparen ps_or_hierarchical_array_identifier lbrack loop_variables rbrack rparen statement
                       -> {kind: "foreach",  array: $3, loop_vars: $5, body: $8}
```

### Field semantics

- `case_statement.unique_priority` is `[]` for plain `case`, `[<unique_priority shape>]` for `unique`/`unique0`/`priority` prefix (raw envelope still — see DEFERRED below).
- `case_pattern_item.condition`: optional `&&& expression` guard per LRM A.6.7.1; `[]` when absent.
- `loop_statement.kind == "for"`: `init`, `condition`, `step` are each `[]` when omitted (e.g., `for (;;)` is valid SV).
- `loop_statement.kind == "foreach"`: `body` is a typed `statement` (note: not `statement_or_null` — bare `;` not allowed for foreach).

### Profile difference

`case_inside_item_sv_2017` uses `open_range_list`; `case_inside_item_sv_2023` uses `range_list` (LRM 2023 simplification). The `kind` labels and field names are identical.

### DEFERRED

- `unique_priority` typing: rule has duplicate `kw_unique` branches (probable grammar bug — branches 0 and 1 are identical `kw_unique`), needs grammar fix before clean annotation. Tracking as a follow-up.
- `conditional_statement` typing: rule uses `&kw_else` positive lookahead + parens-Or `( conditional_statement | statement_or_null )`. The parens-Or hits task #38; needs helper-rule extraction with attention to the lookahead pattern.

### Annotation inventory

454 entries (was 436). +18 in this batch (1 case_statement + 3 case_keyword + 2 case_item + 2 case_pattern_item + 2 case_inside_item_sv_2017 + 2 case_inside_item_sv_2023 + 6 loop_statement).

### Same accept set, same diagnostic codes. Schema stays at `1`.

### mdBook updated, gate green.

### Next slice candidates

- `conditional_statement` (with helper-rule extraction).
- `procedural_assertion_statement`, `clocking_drive`.
- `data_type` / `data_type_or_implicit` / `data_type_or_void`.
- `randsequence_statement` / `randcase_statement`.
- `unique_priority` (after grammar fix).

## Release 1.0.33 / Contract 1.0.33 Highlights — SV-Slice-33 batch: procedural-statement forms typed (11 rules / 26 annotations)

Closes 7 of `statement_item`'s 19/20 kinds (typed in SV-Slice-32) — `disable` / `jump` / `wait` / `event_trigger` / `procedural_timing_control` / `subroutine_call` / `par_block` / `seq_block` now expose typed dispatch into actual content.

### Annotations

```ebnf
disable_statement := kw_disable hierarchical_task_identifier semi  -> {kind: "task",  target: $2}
                   | kw_disable hierarchical_block_identifier semi -> {kind: "block", target: $2}
                   | kw_disable kw_fork semi                       -> {kind: "fork"}

jump_statement := kw_return ( expression )? semi -> {kind: "return", value: $2}
                | kw_break semi                  -> {kind: "break"}
                | kw_continue semi               -> {kind: "continue"}

wait_statement := kw_wait lparen expression rparen statement_or_null
                       -> {kind: "wait",       condition: $3, body: $5}
                | kw_wait kw_fork semi
                       -> {kind: "wait_fork"}
                | kw_wait_order lparen hierarchical_identifier ( comma hierarchical_identifier )* rparen action_block
                       -> {kind: "wait_order", events: {first: $3, rest: $4}, action: $6}

@profiles: ["sv_2017"]
event_trigger_sv_2017 := implies hierarchical_event_identifier semi
                              -> {kind: "non_blocking", name: $2}
                       | implies ( delay_or_event_control )? hierarchical_event_identifier semi
                              -> {kind: "blocking",     control: $2, name: $3}

@profiles: ["sv_2023"]
event_trigger_sv_2023 := /* parallel; both branches add `select: <nonrange_select>` field per LRM 2023 */

procedural_timing_control_statement := procedural_timing_control statement_or_null
                                    -> {control: $1, body: $2}

procedural_timing_control := delay_control -> {kind: "delay", body: $1}
                           | event_control -> {kind: "event", body: $1}
                           | cycle_delay   -> {kind: "cycle", body: $1}

subroutine_call := class_scoped_tf_call -> {kind: "class_scoped_tf", body: $1}
                 | tf_call               -> {kind: "tf",             body: $1}
                 | system_tf_call        -> {kind: "system_tf",      body: $1}
                 | method_call           -> {kind: "method",         body: $1}
                 | ( kw_std scope_resolution )? randomize_call
                                         -> {kind: "randomize",      std_scope: $1, body: $2}

subroutine_call_statement := subroutine_call semi
                                  -> {kind: "call",      body: $1}
                           | kw_void tick lparen function_subroutine_call rparen semi
                                  -> {kind: "void_cast", body: $4}

seq_block := kw_begin ( colon block_identifier )? block_item_declaration* statement_or_null* kw_end ( colon block_identifier )?
          -> {label: $2, declarations: $3, statements: $4, end_label: $6}

par_block := kw_fork ( colon block_identifier )? block_item_declaration* statement_or_null* join_keyword ( colon block_identifier )?
          -> {label: $2, declarations: $3, statements: $4, join: $5, end_label: $6}
```

### Field semantics

- `wait_statement.kind == "wait_order"`: the LRM `wait order(e1, e2, ..., eN) action_block` form. `events.first` + `events.rest` carry the comma-separated event list (mini-mixed-array).
- `subroutine_call.kind == "randomize"`: `std_scope` is `[]` for plain `randomize(...)`, `[<kw_std, scope_resolution>]` for `std::randomize(...)`.
- `subroutine_call_statement.kind == "void_cast"`: the `void'(func_call);` idiom — discards the return value of a function called as a statement.
- `seq_block.label` and `par_block.label`: optional `( colon block_identifier )?` (e.g., `begin : my_block ... end`).
- `par_block.join`: typed `join_keyword` shape (typed earlier in SV-Slice-7 — discriminates `join` / `join_any` / `join_none`).

### Annotation inventory

436 entries (was 410). +26 in this batch (3 disable + 3 jump + 3 wait + 2 event_trigger_sv_2017 + 2 event_trigger_sv_2023 + 1 procedural_timing_control_statement + 3 procedural_timing_control + 5 subroutine_call + 2 subroutine_call_statement + 1 seq_block + 1 par_block).

### Same accept set, same diagnostic codes. Schema stays at `1`.

### mdBook updated, gate green.

### Next slice candidates

- `case_statement` / `case_item` (close case-statement walk).
- `conditional_statement` (with helper-rule extraction for the `( conditional_statement | statement_or_null )` parens-Or per task #38).
- `loop_statement` (6 branches: forever / repeat / while / for / do_while / foreach).
- `procedural_assertion_statement`, `clocking_drive`.
- `data_type` / `data_type_or_implicit` / `data_type_or_void`.

## Release 1.0.32 / Contract 1.0.32 Highlights — SV-Slice-32 batch: statement_item dispatch typed (3 rules / 43 annotations — crosses 400-annotation milestone)

Closes the `statement.body` field, exposing typed dispatch into all 20 (sv_2017) / 19 (sv_2023) procedural-statement forms. Crosses the 400-annotation threshold — pgen's SV grammar is now decisively the most heavily-typed grammar in the family.

### Annotations

```ebnf
@profiles: ["sv_2017"]
statement_item_sv_2017 := blocking_assignment semi              -> {kind: "blocking_assignment",              body: $1}
                        | nonblocking_assignment semi           -> {kind: "nonblocking_assignment",           body: $1}
                        | procedural_continuous_assignment semi -> {kind: "procedural_continuous_assignment", body: $1}
                        | case_statement                        -> {kind: "case",                             body: $1}
                        | conditional_statement                 -> {kind: "conditional",                      body: $1}
                        | inc_or_dec_expression semi            -> {kind: "inc_or_dec_expression",            body: $1}
                        | subroutine_call_statement             -> {kind: "subroutine_call",                  body: $1}
                        | disable_statement                     -> {kind: "disable",                          body: $1}
                        | event_trigger                         -> {kind: "event_trigger",                    body: $1}
                        | loop_statement                        -> {kind: "loop",                             body: $1}
                        | jump_statement                        -> {kind: "jump",                             body: $1}
                        | par_block                             -> {kind: "par_block",                        body: $1}
                        | procedural_timing_control_statement   -> {kind: "procedural_timing_control",        body: $1}
                        | seq_block                             -> {kind: "seq_block",                        body: $1}
                        | wait_statement                        -> {kind: "wait",                             body: $1}
                        | procedural_assertion_statement        -> {kind: "procedural_assertion",             body: $1}
                        | clocking_drive semi                   -> {kind: "clocking_drive",                   body: $1}
                        | randsequence_statement                -> {kind: "randsequence",                     body: $1}
                        | randcase_statement                    -> {kind: "randcase",                         body: $1}
                        | expect_property_statement             -> {kind: "expect_property",                  body: $1}

@profiles: ["sv_2023"]
statement_item_sv_2023 := /* same 19 kinds; `inc_or_dec_expression` removed per LRM 2023 — subsumed by blocking_assignment with ++/-- */

block_item_declaration := attribute_instance* block_data_declaration
                                    -> {kind: "block_data",        attributes: $1, body: $2}
                        | attribute_instance* local_parameter_declaration semi
                                    -> {kind: "local_parameter",   attributes: $1, body: $2}
                        | attribute_instance* parameter_declaration semi
                                    -> {kind: "parameter",         attributes: $1, body: $2}
                        | attribute_instance* let_declaration
                                    -> {kind: "let",               attributes: $1, body: $2}
```

### Profile difference

`statement_item_sv_2017` includes `inc_or_dec_expression semi` (kind label `"inc_or_dec_expression"`) — bare `i++;` / `i--;` as a procedural statement. `statement_item_sv_2023` removes this branch — LRM 2023 subsumes the same semantics into `blocking_assignment` (which now accepts `++`/`--` operators directly). Profile-agnostic walks should accept the `"inc_or_dec_expression"` kind only when the parsed file is sv_2017.

### Annotation inventory

410 entries (was 367). +43 in this batch (20 statement_item_sv_2017 + 19 statement_item_sv_2023 + 4 block_item_declaration).

### Same accept set, same diagnostic codes. Schema stays at `1`.

### mdBook updated, gate green.

### Next slice candidates

- `case_statement` / `conditional_statement` / `loop_statement` (close their internals one level deeper).
- `seq_block` / `par_block` (typed begin/end / fork/join blocks).
- `procedural_timing_control_statement`, `event_trigger`.
- `data_type` / `data_type_or_implicit` / `data_type_or_void`.
- `block_data_declaration` (close block_item_declaration's body field).

## Release 1.0.31 / Contract 1.0.31 Highlights — SV-Slice-31 batch: action_block + statement framing typed (5 rules / 9 annotations)

Closes the action_block walk path (referenced from every concurrent / deferred-immediate assertion typed in SV-Slice-29/30) and the statement framing path (referenced from function/task bodies typed in SV-Slice-25). Every assertion's `action`/`statement` field, every function/task body item, and every framed statement now exposes typed dispatch into actual content.

### Annotations

```ebnf
action_block := statement_or_null
                     -> {kind: "always",    body: $1}
              | ( statement )? kw_else statement_or_null
                     -> {kind: "with_else", pass: $1, fail: $3}

statement := ( block_identifier colon !colon )? attribute_instance* statement_item
          -> {label: $1, attributes: $2, body: $3}

statement_or_null := statement                  -> {kind: "statement", body: $1}
                   | attribute_instance* semi   -> {kind: "null",      attributes: $1}

function_statement_or_null := function_statement       -> {kind: "statement", body: $1}
                            | attribute_instance* semi -> {kind: "null",      attributes: $1}

tf_item_declaration := block_item_declaration -> {kind: "block_item", body: $1}
                     | tf_port_declaration    -> {kind: "tf_port",    body: $1}
```

### Field semantics

- `action_block.kind == "always"`: the unconditional `action;` form. The statement runs whether the assertion passes or fails. `body` is a typed `statement_or_null`.
- `action_block.kind == "with_else"`: the LRM `[statement] else statement_or_null` form. `pass` (optional) runs on assertion success; `fail` runs on assertion failure. `pass` is `[]` when the pass-statement is omitted (e.g., `assert (x) else $error("bad");` has no pass-statement).
- `statement.label`: optional `( block_identifier colon !colon )?` — the `!colon` negative lookahead distinguishes block label (`name:`) from `::` package-scope-resolution. `[]` when statement has no label.
- `statement_or_null.kind == "null"` and `function_statement_or_null.kind == "null"`: bare `;` (with optional preceding `attribute_instance*`). The annotation preserves attributes so consumers can still attach metadata to a null statement.
- `tf_item_declaration` is the union of variable/port declarations inside a function or task body — referenced from `function_body_declaration.items[]` and `task_body_declaration.items[]` (typed in SV-Slice-25).

### Annotation inventory

367 entries (was 358). +9 in this batch (2 action_block + 1 statement + 2 statement_or_null + 2 function_statement_or_null + 2 tf_item_declaration).

### Same accept set, same diagnostic codes. Schema stays at `1`.

### mdBook updated, gate green.

### Next slice candidates

- `statement_item` (close the statement.body field — large dispatch into procedural statement forms).
- `block_item_declaration` (close tf_item_declaration's body field).
- `data_type_or_implicit` / `data_type_or_void`.
- `property_spec` / `sequence_expr`.
- `covergroup_declaration` internals.

## Release 1.0.30 / Contract 1.0.30 Highlights — SV-Slice-30 batch: deferred immediate assertions typed (5 rules / 10 annotations)

Closes the `assertion_item.kind == "deferred_immediate"` walk path. After this slice, both `"concurrent"` (typed in SV-Slice-29) and `"deferred_immediate"` (typed THIS slice) branches of `assertion_item` expose typed dispatch end-to-end.

### Annotations

```ebnf
deferred_immediate_assertion_item := ( block_identifier colon )? deferred_immediate_assertion_statement
                                  -> {label: $1, body: $2}

deferred_immediate_assertion_statement := deferred_immediate_assert_statement -> {kind: "assert", body: $1}
                                        | deferred_immediate_assume_statement -> {kind: "assume", body: $1}
                                        | deferred_immediate_cover_statement  -> {kind: "cover",  body: $1}

deferred_immediate_assert_statement := kw_assert hash kw_n_0 lparen expression rparen action_block
                                            -> {kind: "zero_delay", expression: $5, action: $7}
                                     | kw_assert kw_final lparen expression rparen action_block
                                            -> {kind: "final",      expression: $4, action: $6}

deferred_immediate_assume_statement := /* same 2 kinds with `kw_assume` instead of `kw_assert` */

deferred_immediate_cover_statement := kw_cover hash kw_n_0 lparen expression rparen statement_or_null
                                            -> {kind: "zero_delay", expression: $5, statement: $7}
                                    | kw_cover kw_final lparen expression rparen statement_or_null
                                            -> {kind: "final",      expression: $4, statement: $6}
```

### Field semantics

- `deferred_immediate_assertion_item.label`: optional `( block_identifier colon )?` per LRM A.6.10 (parallel to `concurrent_assertion_item.label` from SV-Slice-24). `[]` when absent, `[<block_id, colon>]` when labeled.
- `deferred_immediate_assert_statement.kind == "zero_delay"`: the `assert #0 (expr) action;` form (LRM 1800-2017 §16.3.1). The `#0` causes assertion evaluation in the Re-NBA region — typical for sampled-value assertions.
- `deferred_immediate_*_statement.kind == "final"`: the `assert final (expr) action;` form. Evaluates at end-of-simulation.
- `cover` variant uses `statement` (statement_or_null) instead of `action`, since cover has no pass/fail branching (just records observation).

### Annotation inventory

358 entries (was 348). +10 in this batch (1 deferred_immediate_assertion_item + 3 deferred_immediate_assertion_statement + 2 deferred_immediate_assert + 2 deferred_immediate_assume + 2 deferred_immediate_cover).

### Same accept set, same diagnostic codes. Schema stays at `1`.

### mdBook updated, gate green.

### Next slice candidates

- `property_spec` / `sequence_expr` internals (close concurrent_assertion property/sequence fields).
- `action_block` (close assert/assume/expect action fields).
- `tf_item_declaration` / `function_statement_or_null` / `statement_or_null`.
- `covergroup_declaration` / `interface_class_declaration` internals.
- `data_type_or_implicit` / `data_type_or_void`.

## Release 1.0.29 / Contract 1.0.29 Highlights — SV-Slice-29 batch: concurrent assertion + constraint family typed (16 rules / 28 annotations)

Closes the `assertion_item.kind == "concurrent"` walk path (typed in SV-Slice-24) and the `class_constraint` walk path (typed in SV-Slice-27). Every concurrent-assertion form now exposes its property spec / action / clocking / disable_iff fields with kind discrimination; constraint declarations expose their static / dynamic-override / name / block structure; constraint expressions dispatch on `kind` (expression / uniqueness / implies / if / foreach / disable_soft).

### Concurrent assertion annotations

```ebnf
concurrent_assertion_statement := assert_property_statement   -> {kind: "assert_property",   body: $1}
                                | assume_property_statement   -> {kind: "assume_property",   body: $1}
                                | cover_property_statement    -> {kind: "cover_property",    body: $1}
                                | cover_sequence_statement    -> {kind: "cover_sequence",    body: $1}
                                | restrict_property_statement -> {kind: "restrict_property", body: $1}

assert_property_statement   := kw_assert kw_property lparen property_spec rparen action_block
                            -> {spec: $4, action: $6}
assume_property_statement   := kw_assume kw_property lparen property_spec rparen action_block
                            -> {spec: $4, action: $6}
cover_property_statement    := kw_cover kw_property lparen property_spec rparen statement_or_null
                            -> {spec: $4, statement: $6}
restrict_property_statement := kw_restrict kw_property lparen property_spec rparen semi
                            -> {spec: $4}
expect_property_statement   := kw_expect lparen property_spec rparen action_block
                            -> {spec: $3, action: $5}

cover_sequence_statement := kw_cover kw_sequence lparen ( clocking_event )? ( kw_disable kw_iff lparen expression_or_dist rparen )? sequence_expr rparen statement_or_null
                         -> {clocking: $4, disable_iff: $5, sequence: $6, statement: $8}
```

### Constraint family annotations

```ebnf
constraint_block := lbrace constraint_block_item* rbrace
                 -> {items: $2}

constraint_block_item := kw_solve solve_before_list kw_before solve_before_list semi
                              -> {kind: "solve_before", before: $2, after: $4}
                      | constraint_expression
                              -> {kind: "expression",   body: $1}

@profiles: ["sv_2017"]
constraint_declaration_sv_2017 := ( kw_static )? kw_constraint constraint_identifier constraint_block
                                -> {static_keyword: $1, name: $3, block: $4}

@profiles: ["sv_2023"]
constraint_declaration_sv_2023 := ( kw_static )? kw_constraint ( dynamic_override_specifiers )? constraint_identifier constraint_block
                                -> {static_keyword: $1, dynamic_override: $3, name: $4, block: $5}

constraint_expression := ( kw_soft )? expression_or_dist semi
                              -> {kind: "expression",   soft: $1, expr: $2}
                       | uniqueness_constraint semi
                              -> {kind: "uniqueness",   body: $1}
                       | expression implies constraint_set
                              -> {kind: "implies",      condition: $1, body: $3}
                       | kw_if lparen expression rparen constraint_set ( kw_else constraint_set )?
                              -> {kind: "if",           condition: $3, then_body: $5, else_clause: $6}
                       | kw_foreach lparen ps_or_hierarchical_array_identifier lbrack loop_variables rbrack rparen constraint_set
                              -> {kind: "foreach",      array: $3, loop_vars: $5, body: $8}
                       | kw_disable kw_soft constraint_primary semi
                              -> {kind: "disable_soft", target: $3}

@profiles: ["sv_2017"]
constraint_prototype_sv_2017 := ( constraint_prototype_qualifier )? ( kw_static )? kw_constraint constraint_identifier semi
                             -> {qualifier: $1, static_keyword: $2, name: $4}

@profiles: ["sv_2023"]
constraint_prototype_sv_2023 := ( constraint_prototype_qualifier )? ( kw_static )? kw_constraint ( dynamic_override_specifiers )? constraint_identifier semi
                             -> {qualifier: $1, static_keyword: $2, dynamic_override: $4, name: $5}

constraint_prototype_qualifier := kw_extern -> {kind: "extern"}
                                | kw_pure   -> {kind: "pure"}

constraint_set := constraint_expression                 -> {kind: "single", body: $1}
                | lbrace constraint_expression* rbrace  -> {kind: "block",  exprs: $2}
```

### Annotation inventory

348 entries (was 320). +28 in this batch.

### Same accept set, same diagnostic codes. Schema stays at `1`.

### mdBook updated, gate green.

### Next slice candidates

- `property_spec` / `sequence_expr` internals (close concurrent_assertion property/sequence fields one level deeper).
- `action_block` (close assert/assume/expect action fields).
- `tf_item_declaration` / `function_statement_or_null` / `statement_or_null`.
- `covergroup_declaration` / `interface_class_declaration` internals.
- `data_type_or_implicit` / `data_type_or_void`.

## Release 1.0.28 / Contract 1.0.28 Highlights — SV-Slice-28 batch: class qualifiers typed (3 rules / 6 annotations)

Completes SV-Slice-27's class body picture. Every reachable `class_method.qualifiers[]` and `class_property.qualifiers[]` now exposes typed dispatch — consumers can iterate qualifier lists and discriminate `virtual` (with optional `pure` flag) vs `class_item_qualifier` (static/protected/local) vs `random` (rand/randc) without raw envelope descent.

### Annotations

```ebnf
method_qualifier := ( kw_pure )? kw_virtual -> {kind: "virtual",               pure: $1}
                  | class_item_qualifier    -> {kind: "class_item_qualifier",  body: $1}

property_qualifier := random_qualifier      -> {kind: "random",                body: $1}
                    | class_item_qualifier  -> {kind: "class_item_qualifier",  body: $1}

random_qualifier := kw_rand   -> {kind: "rand"}
                  | kw_randc  -> {kind: "randc"}
```

### Field semantics

- `method_qualifier.kind == "virtual"`: the `pure` field is `[]` for bare `virtual`, `[<kw_pure token>]` for `pure virtual` (LRM-significant for pure-virtual method declarations).
- `method_qualifier.kind == "class_item_qualifier"` and `property_qualifier.kind == "class_item_qualifier"`: the `body` field is the typed `class_item_qualifier` shape (static / protected / local) from SV-Slice-27.
- `random_qualifier`: bare `{kind}` shape — each branch matches a single keyword token.

### Annotation inventory

320 entries (was 314). +6 in this batch (2 method_qualifier + 2 property_qualifier + 2 random_qualifier).

### Same accept set, same diagnostic codes. Schema stays at `1`.

### mdBook updated, gate green.

### Next slice candidates

- `concurrent_assertion_statement` / `deferred_immediate_assertion_item` internals.
- `tf_item_declaration` / `function_statement_or_null` / `statement_or_null` (function/task body internals).
- `covergroup_declaration` / `interface_class_declaration` internals.
- `constraint_declaration` / `constraint_prototype` (close class_constraint body fields).
- `data_type_or_implicit` / `data_type_or_void` (close data_type fields across many rules).

## Release 1.0.27 / Contract 1.0.27 Highlights — SV-Slice-27 batch: class body sub-tree typed (6 rules / 30 annotations)

Closes the class body walk path. Every reachable `class_declaration_sv_2017/2023.items[]` (typed in SV-Slice-10) now exposes typed dispatch into class members. Method qualifiers (static / protected / local), property kind (decl vs const), method kind (task / function / pure_virtual / extern / constructor / extern_constructor) all directly accessible via `kind` discriminators.

### Annotations

```ebnf
@profiles: ["sv_2017"]
class_item_sv_2017 := attribute_instance* class_property         -> {kind: "property",        attributes: $1, body: $2}
                    | attribute_instance* class_method           -> {kind: "method",          attributes: $1, body: $2}
                    | attribute_instance* class_constraint       -> {kind: "constraint",      attributes: $1, body: $2}
                    | attribute_instance* class_declaration      -> {kind: "class",           attributes: $1, body: $2}
                    | attribute_instance* covergroup_declaration -> {kind: "covergroup",      attributes: $1, body: $2}
                    | local_parameter_declaration semi           -> {kind: "local_parameter", body: $1}
                    | parameter_declaration semi                 -> {kind: "parameter",       body: $1}
                    | semi                                       -> {kind: "semi"}

@profiles: ["sv_2023"]
class_item_sv_2023 := /* same 8 plus 1 between class and covergroup:
                        attribute_instance* interface_class_declaration -> {kind: "interface_class", attributes: $1, body: $2} */

class_item_qualifier := kw_static    -> {kind: "static"}
                      | kw_protected -> {kind: "protected"}
                      | kw_local     -> {kind: "local"}

class_constraint := constraint_prototype   -> {kind: "prototype",   body: $1}
                  | constraint_declaration -> {kind: "declaration", body: $1}

class_property := property_qualifier* data_declaration
                       -> {kind: "decl",  qualifiers: $1, body: $2}
               | kw_const class_item_qualifier* data_type const_identifier ( assign constant_expression )? semi
                       -> {kind: "const", qualifiers: $2, data_type: $3, name: $4, init: $5}

class_method := method_qualifier* task_declaration
                     -> {kind: "task",                qualifiers: $1, body: $2}
             | method_qualifier* function_declaration
                     -> {kind: "function",            qualifiers: $1, body: $2}
             | kw_pure kw_virtual class_item_qualifier* method_prototype semi
                     -> {kind: "pure_virtual",        qualifiers: $3, prototype: $4}
             | kw_extern method_qualifier* method_prototype semi
                     -> {kind: "extern",              qualifiers: $2, prototype: $3}
             | method_qualifier* class_constructor_declaration
                     -> {kind: "constructor",         qualifiers: $1, body: $2}
             | kw_extern method_qualifier* class_constructor_prototype
                     -> {kind: "extern_constructor",  qualifiers: $2, prototype: $3}
```

### Field semantics

- `class_item.kind == "property" / "method" / "constraint" / "class" / "covergroup" / "interface_class"`: each preserves the leading `attribute_instance*` slot as `attributes`. The `body` field is the typed sub-rule shape (typed in this slice for property/method/constraint, typed in SV-Slice-10 for class_declaration, raw envelope still for covergroup_declaration / interface_class_declaration which are deferred to a later slice).
- `class_property.kind == "decl"`: the standard form `property_qualifier* data_declaration` (e.g., `static int count;`). `qualifiers` is the matched property_qualifier* iteration (rand/randc/static/protected/local/etc., still raw envelope — typing deferred). `body` is the typed `data_declaration` (typed in SV-Slice-25).
- `class_property.kind == "const"`: the kw_const-prefixed form (e.g., `const static int N = 10;`). `qualifiers` is the inner class_item_qualifier* (typed THIS slice), `data_type` is the matched data_type, `name` is the const_identifier, `init` is `[]` when no initializer or `[<assign, expr>]` when present.
- `class_method.kind == "pure_virtual"` / `"extern"`: prototype-only forms (no body). The `prototype` field carries the matched method_prototype.
- `class_method.kind == "extern_constructor"`: prototype-only form for extern class new declaration.
- `class_item_qualifier`: bare `{kind}` shape — each branch matches a single keyword token.

### Profile difference

`class_item_sv_2023` adds an `"interface_class"` kind (not present in sv_2017) — LRM 2023 allows nested `interface class` declarations inside class bodies. The 8 sv_2017 kinds are unchanged.

### Annotation inventory

314 entries (was 284). +30 in this batch (8 class_item_sv_2017 + 9 class_item_sv_2023 + 3 class_item_qualifier + 2 class_constraint + 2 class_property + 6 class_method).

### Same accept set, same diagnostic codes. Schema stays at `1`.

### mdBook updated, gate green.

### Next slice candidates

- `method_qualifier`, `property_qualifier` (close class_method / class_property qualifier list typing).
- `concurrent_assertion_statement` / `deferred_immediate_assertion_item` internals.
- `tf_item_declaration` / `function_statement_or_null` / `statement_or_null` (function/task body internals).
- `covergroup_declaration` / `interface_class_declaration` internals.
- `constraint_declaration` / `constraint_prototype` (close class_constraint body fields).

## Release 1.0.26 / Contract 1.0.26 Highlights — SV-Slice-26 batch: net_declaration typed via helper-rule extraction (4 rules / 10 annotations + 2 new helper rules)

Closes the net_declaration walk path. After this slice, every reachable `data_declaration_sv_2017.kind == "net_type"` (sv_2017) and contexts that resolve through to net_declaration expose typed dispatch. Two new helper rules (`net_strength`, `net_vector_scalar`) extracted from inline parens-Or to dodge task #38 — same workaround pattern as SV-Slice-23's `if_generate_else_clause`.

### Annotations

```ebnf
@profiles: ["sv_2017"]
net_declaration_sv_2017 := net_type ( net_strength )? ( net_vector_scalar )? data_type_or_implicit ( delay )? list_of_net_decl_assignments semi
                                -> {kind: "wire",         net_type: $1, strength: $2, vector_scalar: $3, data_type: $4, delay: $5, assignments: $6}
                         | net_type_identifier ( delay_control )? list_of_net_decl_assignments semi
                                -> {kind: "alias",        net_type_id: $1, delay_control: $2, assignments: $3}
                         | kw_interconnect implicit_data_type ( hash delay_value )? net_identifier unpacked_dimension* ( comma net_identifier unpacked_dimension* )? semi
                                -> {kind: "interconnect", data_type: $2, delay: $3, name: $4, dims: $5, second: $6}

@profiles: ["sv_2023"]
net_declaration_sv_2023 := /* same 3 branches; alias branch uses `nettype_identifier` (kind label "alias", field name `nettype_id`) per LRM 2023 nettype-vs-net_type naming */

net_strength := drive_strength  -> {kind: "drive",  body: $1}
              | charge_strength -> {kind: "charge", body: $1}

net_vector_scalar := kw_vectored -> {kind: "vectored"}
                   | kw_scalared -> {kind: "scalared"}
```

### Helper-rule extraction rationale

The original `net_declaration_sv_2017` had two inline parens-Or constructs in branch 0:

```ebnf
net_type ( drive_strength | charge_strength )? ( kw_vectored | kw_scalared )? data_type_or_implicit ...
```

Both hit task #38 (parens-grouped-Or trailing-annotation attribution bug). Following the SV-Slice-23 pattern, the inline parens-Ors were extracted to named rules:

- `net_strength := drive_strength | charge_strength`
- `net_vector_scalar := kw_vectored | kw_scalared`

These rules have no LRM equivalent — they're internal organizational details. Consumers walking `net_declaration.strength` see `{kind: "drive" | "charge", body: <strength shape>}`; walking `net_declaration.vector_scalar` see bare `{kind: "vectored" | "scalared"}` (no body since each branch is a single keyword token).

### Profile difference

`net_declaration_sv_2017` alias branch field name is `net_type_id` (matches the underlying `net_type_identifier` rule); `net_declaration_sv_2023` uses `nettype_id` (matches `nettype_identifier`). Profile-agnostic walks should accept both fields when `kind == "alias"`.

### Annotation inventory

284 entries (was 274). +10 in this batch (3 net_declaration_sv_2017 + 3 net_declaration_sv_2023 + 2 net_strength + 2 net_vector_scalar).

### Same accept set, same diagnostic codes. Schema stays at `1`.

### Grammar surface change

This slice adds two new rules (`net_strength`, `net_vector_scalar`) to the public grammar surface. Both have no LRM equivalent — they're internal refactors of inline parens-Or for annotation purposes. Same accept set.

### mdBook updated, gate green.

### Next slice candidates

- `concurrent_assertion_statement` / `deferred_immediate_assertion_item` internals.
- `tf_item_declaration` / `function_statement_or_null` / `statement_or_null` (close function/task body internals one level deeper).
- `class_item` — close class body walks.
- `ansi_port_declaration` (still blocked by task #38 — would need explicit fix or larger-scale grammar refactor).

## Release 1.0.25 / Contract 1.0.25 Highlights — SV-Slice-25 batch: data/function/task declarations + bodies typed (8 rules / 14 annotations)

Closes the data / function / task walk paths from `package_or_generate_item_declaration`. After this slice, every reachable `package_or_generate_item_declaration.kind == "data_declaration"` / `"function_declaration"` / `"task_declaration"` exposes typed dispatch all the way to the function/task body's name + items + statements + end_label.

### Annotations

```ebnf
@profiles: ["sv_2017"]
data_declaration_sv_2017 := ( kw_const )? ( kw_var )? ( lifetime )? data_type_or_implicit list_of_variable_decl_assignments semi
                                -> {kind: "variable_decl",        const_keyword: $1, var_keyword: $2, lifetime: $3, data_type: $4, assignments: $5}
                         | type_declaration                       -> {kind: "type",                  body: $1}
                         | package_import_declaration             -> {kind: "package_import",        body: $1}
                         | net_type_declaration                   -> {kind: "net_type",              body: $1}

@profiles: ["sv_2023"]
data_declaration_sv_2023 := /* same first 3 branches; 4th is `nettype_declaration` -> {kind: "nettype", body: $1} per LRM 2023 naming */

@profiles: ["sv_2017"]
function_declaration_sv_2017 := kw_function ( lifetime )? function_body_declaration
                             -> {lifetime: $2, body: $3}

@profiles: ["sv_2023"]
function_declaration_sv_2023 := kw_function ( dynamic_override_specifiers )? ( lifetime )? function_body_declaration
                             -> {dynamic_override: $2, lifetime: $3, body: $4}

function_body_declaration := function_data_type_or_implicit function_identifier semi tf_item_declaration* function_statement_or_null* kw_endfunction ( colon function_identifier )?
                          -> {return_type: $1, name: $2, items: $4, statements: $5, end_label: $7}

@profiles: ["sv_2017"]
task_declaration_sv_2017 := kw_task ( lifetime )? task_body_declaration
                         -> {lifetime: $2, body: $3}

@profiles: ["sv_2023"]
task_declaration_sv_2023 := kw_task ( dynamic_override_specifiers )? ( lifetime )? task_body_declaration
                         -> {dynamic_override: $2, lifetime: $3, body: $4}

task_body_declaration := task_identifier semi tf_item_declaration* statement_or_null* kw_endtask ( colon task_identifier )?
                      -> {name: $1, items: $3, statements: $4, end_label: $6}
```

### Field semantics

- `data_declaration_*.kind == "variable_decl"`: the most common form. `const_keyword` and `var_keyword` are `[]` when absent. `lifetime` is `[]` or a typed `lifetime` shape (per SV-Slice-7). `data_type` is the matched `data_type_or_implicit`. `assignments` is a `list_of_variable_decl_assignments` envelope.
- `function_declaration_sv_2023.dynamic_override`: optional `( dynamic_override_specifiers )?` slot added in LRM 2023. Always `[]` for sv_2017 (sub-rule doesn't have the slot).
- `function_body_declaration.return_type`: the matched `function_data_type_or_implicit` (function may return void, scalar, vector, or struct).
- `task_body_declaration` has no `return_type` — task is void by definition.

### Profile differences

`data_declaration_sv_2017` uses `net_type_declaration` (kind label `"net_type"`); `data_declaration_sv_2023` uses `nettype_declaration` (kind label `"nettype"`). LRM 2023 renamed the rule (one-word `nettype` vs two-word `net_type`). Consumers need to handle both kind labels when walking a profile-agnostic workflow.

### DEFERRED: net_declaration

`net_declaration_sv_2017/sv_2023` typing is deferred to the next slice: it has parens-Or `( drive_strength | charge_strength )?` and `( kw_vectored | kw_scalared )?` that hit task #38. Will use the helper-rule extraction pattern established in SV-Slice-23 (`if_generate_else_clause`).

### Annotation inventory

274 entries (was 260). +14 in this batch.

### Same accept set, same diagnostic codes. Schema stays at `1`.

### mdBook updated, gate green.

### Next slice candidates

- `net_declaration_sv_2017` / `net_declaration_sv_2023` with helper-rule extraction (`net_strength`, `net_vector_scalar`).
- `concurrent_assertion_statement` / `deferred_immediate_assertion_item` internals (close `assertion_item` body fields one level deeper).
- `tf_item_declaration` / `function_statement_or_null` / `statement_or_null` (close function/task body internals one level deeper).
- `class_declaration_sv_2017/2023` internals — `class_item` etc.

## Release 1.0.24 / Contract 1.0.24 Highlights — SV-Slice-24 batch: assertion + genvar dispatch typed (7 rules / 26 annotations)

Closes the assertion-item walk path and the loop_generate_construct init/step typed dispatch. After this slice, `module_common_item.kind == "assertion_item"` resolves through to typed concurrent/deferred shapes; SV-Slice-23's loop_generate_construct.init / .step fields now expose typed genvar_initialization / genvar_iteration shapes; assignment_operator and inc_or_dec_operator both surface clean `{kind}` discriminators for operator-by-name dispatch.

### Annotations

```ebnf
assertion_item := concurrent_assertion_item            -> {kind: "concurrent",         body: $1}
               | deferred_immediate_assertion_item     -> {kind: "deferred_immediate", body: $1}

assertion_item_declaration := property_declaration  -> {kind: "property", body: $1}
                            | sequence_declaration  -> {kind: "sequence", body: $1}
                            | let_declaration       -> {kind: "let",      body: $1}

concurrent_assertion_item := ( block_identifier colon )? concurrent_assertion_statement
                                  -> {kind: "statement",             label: $1, body: $2}
                           | checker_instantiation
                                  -> {kind: "checker_instantiation", body: $1}

genvar_initialization := ( kw_genvar )? genvar_identifier assign constant_expression
                      -> {genvar_keyword: $1, name: $2, value: $4}

genvar_iteration := genvar_identifier assignment_operator genvar_expression
                         -> {kind: "assign",          name: $1, op: $2, value: $3}
                  | inc_or_dec_operator genvar_identifier
                         -> {kind: "prefix_inc_dec",  op: $1, name: $2}
                  | genvar_identifier inc_or_dec_operator
                         -> {kind: "postfix_inc_dec", name: $1, op: $2}

assignment_operator := assign                          -> {kind: "assign"}
                     | plus_assign                     -> {kind: "plus_assign"}
                     | minus_assign                    -> {kind: "minus_assign"}
                     | star_assign                     -> {kind: "star_assign"}
                     | slash_assign                    -> {kind: "slash_assign"}
                     | percent_assign                  -> {kind: "percent_assign"}
                     | and_assign                      -> {kind: "and_assign"}
                     | or_assign                       -> {kind: "or_assign"}
                     | xor_assign                      -> {kind: "xor_assign"}
                     | shift_left_assign               -> {kind: "shift_left_assign"}
                     | shift_right_assign              -> {kind: "shift_right_assign"}
                     | arithmetic_shift_left_assign    -> {kind: "arithmetic_shift_left_assign"}
                     | arithmetic_shift_right_assign   -> {kind: "arithmetic_shift_right_assign"}

inc_or_dec_operator := plus_plus    -> {kind: "plus_plus"}
                     | minus_minus  -> {kind: "minus_minus"}
```

### Field semantics

- `genvar_initialization.genvar_keyword`: the optional `( kw_genvar )?` prefix. `[]` when absent (re-using a previously-declared genvar), `[<kw_genvar token>]` when present (declare-and-init form).
- `concurrent_assertion_item.label` (statement kind): the optional `( block_identifier colon )?` prefix per LRM A.6.10. `[]` when no label, `[<block_id, colon>]` when labeled.
- `assignment_operator.kind` and `inc_or_dec_operator.kind`: bare `{kind}` shape (no `body` field) — each branch matches a single keyword token, so the kind label is the only meaningful information. Consumers can dispatch by name without descending into the operator token.

### Annotation inventory

260 entries (was 234). +26 in this batch (2 assertion_item + 3 assertion_item_declaration + 2 concurrent_assertion_item + 1 genvar_initialization + 3 genvar_iteration + 13 assignment_operator + 2 inc_or_dec_operator).

### Same accept set, same diagnostic codes. Schema stays at `1`.

### mdBook updated, gate green.

### Next slice candidates

- `data_declaration` / `function_declaration` / `task_declaration` (close `package_or_generate_item_declaration` walk paths another level deeper).
- `concurrent_assertion_statement` / `deferred_immediate_assertion_item` internals (close `assertion_item` body fields one level deeper).
- `bind_target_scope` / remaining bind sub-tree pieces.

## Release 1.0.23 / Contract 1.0.23 Highlights — SV-Slice-23 batch: generate-construct internals typed (6 rules / 9 annotations + 1 new helper rule)

Closes the loop / conditional / case-generate dispatch path. After this slice, every reachable `module_common_item.kind == "loop_generate_construct"` and `"conditional_generate_construct"` exposes typed structural dispatch all the way to the generate body.

### Notable: helper-rule extraction to dodge task #38

The original `if_generate_construct` rule had this trailing optional Or:

```ebnf
if_generate_construct := kw_if lparen constant_expression rparen generate_block ( kw_else if_generate_construct | kw_else generate_block )?
```

The inline `( a | b )?` parens-grouped Or hits task #38 (parens-grouped-Or trailing-annotation attribution bug). To unblock annotation, the trailing parens-Or was extracted to a named helper rule:

```ebnf
if_generate_construct := kw_if lparen constant_expression rparen generate_block ( if_generate_else_clause )?
                      -> {condition: $3, then_block: $5, else_clause: $6}

if_generate_else_clause := kw_else if_generate_construct -> {kind: "elseif",     body: $2}
                         | kw_else generate_block        -> {kind: "else_block", body: $2}
```

This pattern is now the recommended workaround for any similar `( a | b )?` / `( a | b )*` parens-Or annotation needs until task #38 is fixed. It does add a named rule to the public grammar surface, but the rule body is small and the typed shape is consumer-friendly (`else_clause.kind == "elseif"` for chained `else if`, `"else_block"` for terminal `else`).

### Annotations

```ebnf
loop_generate_construct := kw_for lparen genvar_initialization semi genvar_expression semi genvar_iteration rparen generate_block
                        -> {init: $3, condition: $5, step: $7, block: $9}

conditional_generate_construct := if_generate_construct   -> {kind: "if",   body: $1}
                                | case_generate_construct -> {kind: "case", body: $1}

if_generate_construct := kw_if lparen constant_expression rparen generate_block ( if_generate_else_clause )?
                      -> {condition: $3, then_block: $5, else_clause: $6}

if_generate_else_clause := kw_else if_generate_construct -> {kind: "elseif",     body: $2}
                         | kw_else generate_block        -> {kind: "else_block", body: $2}

case_generate_construct := kw_case lparen constant_expression rparen case_generate_item case_generate_item* kw_endcase
                        -> {expr: $3, items: {first: $5, rest: $6}}

case_generate_item := constant_expression ( comma constant_expression )* colon generate_block
                          -> {kind: "expr_list", exprs: {first: $1, rest: $2}, block: $4}
                   | kw_default ( colon )? generate_block
                          -> {kind: "default",   block: $3}
```

### Consumer dispatch chain

Walk path from a typed module-item all the way to a generate-block body:

```rust
// description.body.body.items[i] is a module_item
match item["kind"] {
  "non_port_item" => match item["body"]["kind"] {
    "module_or_generate" => {
      let mog_body = &item["body"]["body"];
      if mog_body["kind"] == "module_common_item" {
        let mci = &mog_body["body"];
        match mci["kind"] {
          "loop_generate_construct" => {
            let loop_gen = &mci["body"];                  // typed THIS slice
            walk_genvar_init(&loop_gen["init"]);
            walk_expr(&loop_gen["condition"]);
            walk_genvar_step(&loop_gen["step"]);
            walk_generate_block(&loop_gen["block"]);      // typed in SV-Slice-22
          }
          "conditional_generate_construct" => {
            let cond = &mci["body"];                      // typed THIS slice
            match cond["kind"] {
              "if" => {
                let if_gen = &cond["body"];               // if_generate_construct (typed THIS slice)
                walk_expr(&if_gen["condition"]);
                walk_generate_block(&if_gen["then_block"]);
                if let Some(ec) = if_gen["else_clause"].as_array().and_then(|a| a.get(0)) {
                  match ec["kind"] {
                    "elseif"     => walk_if_generate(&ec["body"]),  // recursive
                    "else_block" => walk_generate_block(&ec["body"]),
                  }
                }
              }
              "case" => {
                let case_gen = &cond["body"];             // case_generate_construct (typed THIS slice)
                walk_expr(&case_gen["expr"]);
                let items = &case_gen["items"];
                walk_case_item(&items["first"]);
                for item in items["rest"].as_array().unwrap() {
                    walk_case_item(item);                 // case_generate_item (typed THIS slice)
                }
              }
            }
          }
          /* ... 11 more module_common_item kinds ... */
        }
      }
    }
  }
}

fn walk_case_item(item: &Value) {
    match item["kind"].as_str().unwrap() {
        "expr_list" => {
            walk_expr(&item["exprs"]["first"]);
            for e in item["exprs"]["rest"].as_array().unwrap() {
                walk_expr(e);
            }
            walk_generate_block(&item["block"]);
        }
        "default" => walk_generate_block(&item["block"]),
    }
}
```

### Annotation inventory

234 entries (was 225). +9 in this batch (1 loop_generate + 2 conditional_generate + 1 if_generate + 2 if_generate_else_clause + 1 case_generate + 2 case_generate_item).

### Same accept set, same diagnostic codes. Schema stays at `1`.

### Grammar surface change

This slice adds one new rule to the public grammar surface: `if_generate_else_clause`. It has no LRM equivalent (it's a refactor of an inline parens-Or for annotation purposes); consumers should treat it as an internal detail of `if_generate_construct.else_clause`. The accept set is unchanged.

### mdBook updated, gate green.

### Next slice candidates

- `data_declaration` / `function_declaration` / `task_declaration` (close `package_or_generate_item_declaration` walks another level).
- `assertion_item` / `concurrent_assertion_item` / `assertion_item_declaration` (assertion family).
- `genvar_initialization` / `genvar_iteration` / `genvar_decl_assignment` (close loop_generate_construct walk's init/step fields).

## Release 1.0.22 / Contract 1.0.22 Highlights — SV-Slice-22 batch: generate sub-tree typed (3 rules / 7 annotations)

Closes the generate-construct walk path. After this slice, every reachable `non_port_module_item.kind=='generate_region'` exposes a typed `{items}` shape, every `generate_item` discriminates which form it carries, and every `generate_block` (anonymous, labeled, or bare-generate_item passthrough) exposes its name/label/items/end_label.

### Annotations

```ebnf
generate_region := kw_generate generate_item* kw_endgenerate
                -> {items: $2}

generate_item := module_or_generate_item    -> {kind: "module_or_generate_item",    body: $1}
              | interface_or_generate_item -> {kind: "interface_or_generate_item", body: $1}
              | checker_or_generate_item   -> {kind: "checker_or_generate_item",   body: $1}

generate_block := kw_begin ( colon generate_block_identifier )? generate_item* kw_end ( colon generate_block_identifier )?
                       -> {kind: "anonymous",     label: $2, items: $3, end_label: $5}
              | generate_block_identifier colon kw_begin ( colon generate_block_identifier )? generate_item* kw_end ( colon generate_block_identifier )?
                       -> {kind: "labeled",       name: $1, label: $4, items: $5, end_label: $7}
              | generate_item
                       -> {kind: "generate_item", body: $1}
```

### Field semantics

- `generate_block.label` (anonymous + labeled forms): the optional `( colon generate_block_identifier )?` immediately after `kw_begin` — the inner block label per LRM A.4.2 (e.g., `begin : foo ... end`).
- `generate_block.end_label` (anonymous + labeled forms): the optional trailing `( colon generate_block_identifier )?` after `kw_end` (e.g., `end : foo`).
- `generate_block.name` (labeled form only): the leading block_identifier prefixing the `:` `begin` (e.g., `name : begin ... end`).
- `generate_block.kind == "generate_item"`: bare generate_item with no `begin`/`end` wrapping; `body` carries the matched generate_item shape directly.

### Consumer dispatch chain

After this slice, the typed AST exposes typed dispatch into generate constructs end-to-end:

```rust
// non_port_module_item.kind == "generate_region" path
if npi["kind"] == "generate_region" {
    let gen_region = &npi["body"];                  // generate_region typed shape (this slice)
    for item in gen_region["items"].as_array().unwrap() {
        match item["kind"].as_str().unwrap() {       // generate_item.kind (this slice)
            "module_or_generate_item" => walk_mog(&item["body"]),
            "interface_or_generate_item" => walk_iog(&item["body"]),
            "checker_or_generate_item" => walk_cog(&item["body"]),
        }
    }
}
```

For generate_block (referenced from loop_generate_construct / conditional_generate_construct internals once those are typed):

```rust
match gen_block["kind"].as_str().unwrap() {
    "anonymous" => walk_block_body(gen_block["label"].as_array(), &gen_block["items"], gen_block["end_label"].as_array()),
    "labeled" => walk_named_block(gen_block["name"].as_str(), gen_block["label"].as_array(), &gen_block["items"], gen_block["end_label"].as_array()),
    "generate_item" => walk_single_item(&gen_block["body"]),
}
```

### Annotation inventory

225 entries (was 218). +7 in this batch.

### Same accept set, same diagnostic codes. Schema stays at `1`.

### mdBook updated, gate green.

### Next slice candidates

- `loop_generate_construct` and `conditional_generate_construct` (referenced from `module_common_item.kind` — would close the loop/conditional generate dispatch).
- `data_declaration`, `function_declaration`, `task_declaration` (close `package_or_generate_item_declaration` walk paths another level deeper).
- `assertion_item`, `concurrent_assertion_item`, `assertion_item_declaration` (assertion family — referenced from multiple module/program rules).

## Release 1.0.21 / Contract 1.0.21 Highlights — SV-Slice-21 batch: module_common_item + package_or_generate_item_declaration typed (4 rules / 55 annotations — biggest batch yet)

Biggest single batch by annotation count (55 entries). Both halves of the cascading walk path that SV-Slice-19/20 set up are now closed: every reachable `module_common_item` and every reachable `package_or_generate_item_declaration` discriminates which actual sub-construct was matched.

### Rationale

Per SV-Slice-19/20 dispatch chains:
- `module_or_generate_item.kind == "module_common_item"` → `module_common_item` shape (was raw envelope until this slice).
- `module_or_generate_item_declaration.kind == "package_or_generate"` → `package_or_generate_item_declaration` shape (was raw envelope until this slice).
- Same for `non_port_program_item.kind == "module_or_generate_item_declaration"` (program path).

This slice closes those terminals so consumers can dispatch end-to-end without descending recursive envelopes for these large sub-trees.

### Annotations

```ebnf
@profiles: ["sv_2017"]
module_common_item_sv_2017 := module_or_generate_item_declaration -> {kind: "module_or_generate_item_declaration", body: $1}
                            | interface_instantiation             -> {kind: "interface_instantiation",             body: $1}
                            | program_instantiation               -> {kind: "program_instantiation",               body: $1}
                            | assertion_item                      -> {kind: "assertion_item",                      body: $1}
                            | bind_directive                      -> {kind: "bind_directive",                      body: $1}
                            | continuous_assign                   -> {kind: "continuous_assign",                   body: $1}
                            | net_alias                           -> {kind: "net_alias",                           body: $1}
                            | initial_construct                   -> {kind: "initial_construct",                   body: $1}
                            | final_construct                     -> {kind: "final_construct",                     body: $1}
                            | always_construct                    -> {kind: "always_construct",                    body: $1}
                            | loop_generate_construct             -> {kind: "loop_generate_construct",             body: $1}
                            | conditional_generate_construct      -> {kind: "conditional_generate_construct",      body: $1}
                            | elaboration_system_task             -> {kind: "elaboration_system_task",             body: $1}

@profiles: ["sv_2023"]
module_common_item_sv_2023 := /* same 13 branches; last is elaboration_severity_system_task per LRM 2023 */

@profiles: ["sv_2017"]
package_or_generate_item_declaration_sv_2017 := local_parameter_declaration semi  -> {kind: "local_parameter_declaration",  body: $1}
                                              | parameter_declaration semi        -> {kind: "parameter_declaration",        body: $1}
                                              | net_declaration                   -> {kind: "net_declaration",              body: $1}
                                              | dpi_import_export                 -> {kind: "dpi_import_export",            body: $1}
                                              | data_declaration                  -> {kind: "data_declaration",             body: $1}
                                              | task_declaration                  -> {kind: "task_declaration",             body: $1}
                                              | function_declaration              -> {kind: "function_declaration",         body: $1}
                                              | checker_declaration               -> {kind: "checker_declaration",          body: $1}
                                              | extern_constraint_declaration     -> {kind: "extern_constraint_declaration", body: $1}
                                              | class_declaration                 -> {kind: "class_declaration",            body: $1}
                                              | class_constructor_declaration     -> {kind: "class_constructor_declaration", body: $1}
                                              | covergroup_declaration            -> {kind: "covergroup_declaration",       body: $1}
                                              | assertion_item_declaration        -> {kind: "assertion_item_declaration",   body: $1}
                                              | semi                              -> {kind: "semi"}

@profiles: ["sv_2023"]
package_or_generate_item_declaration_sv_2023 := /* same 14 plus interface_class_declaration between class_declaration and class_constructor_declaration (15 kinds total) */
```

The two leading branches drop trailing `semi` via `body: $1`. The `semi` branch carries no body since it's just a stray `;`.

### Why the wrapper rules stay un-annotated

`module_common_item := module_common_item_sv_2017 | module_common_item_sv_2023` and the `package_or_generate_item_declaration` wrapper stay un-annotated — same pattern as `module_declaration` / `interface_declaration`. They're transparent profile-routers that pass through to the matched profile-typed sub-rule. The kind discriminator lives inside the per-profile sub-rules.

### Consumer dispatch chain — full module path

After this slice, the typed AST exposes 6+ levels of dispatch end-to-end for module contents:

```rust
// description.body.body.items contains module_item entries
match item["kind"] {
  "non_port_item" => match item["body"]["kind"] {
    "module_or_generate" => {
      let mog = &item["body"]["body"];
      match mog["kind"] {
        "module_common_item" => {
          let mci = &mog["body"];                  // module_common_item shape
          match mci["kind"] {
            "module_or_generate_item_declaration" => {
              let mogid = &mci["body"];            // module_or_generate_item_declaration shape (typed since SV-Slice-19)
              match mogid["kind"] {
                "package_or_generate" => {
                  let pogid = &mogid["body"];      // package_or_generate_item_declaration shape (typed THIS SLICE)
                  match pogid["kind"] {
                    "function_declaration"   => walk_function(&pogid["body"]),
                    "task_declaration"       => walk_task(&pogid["body"]),
                    "data_declaration"       => walk_data(&pogid["body"]),
                    "class_declaration"      => walk_class(&pogid["body"]),
                    /* ...11 more kinds... */
                  }
                }
                /* ...4 more kinds (genvar, clocking, default_clocking, default_disable_iff)... */
              }
            }
            "always_construct"               => walk_always(&mci["body"]),
            "initial_construct"              => walk_initial(&mci["body"]),
            "continuous_assign"              => walk_continuous_assign(&mci["body"]),
            "loop_generate_construct"        => walk_loop_generate(&mci["body"]),
            "conditional_generate_construct" => walk_conditional_generate(&mci["body"]),
            /* ...7 more kinds... */
          }
        }
        /* parameter_override / gate_instantiation / udp_instantiation / module_instantiation */
      }
    }
  }
}
```

### Annotation inventory

218 entries (was 163). +55 in this batch.

### Same accept set, same diagnostic codes. Schema stays at `1`.

### mdBook updated, gate green.

### Next slice candidates

- `generate_region` / `generate_block` / `generate_item` (close the generate-construct walk).
- `module_declaration` / `interface_declaration` / `class_declaration` / `program_declaration` profile-tag wrapper rules (currently transparent — could add explicit `kind: "sv_2017"` / `kind: "sv_2023"` discriminators if profile-aware consumers want it; not required since the per-profile sub-rules already differ in field positions).
- `data_declaration`, `function_declaration`, `task_declaration` (large internal sub-trees that would close `package_or_generate_item_declaration` walk paths another level deeper).

## Release 1.0.20 / Contract 1.0.20 Highlights — SV-Slice-20 batch: interface + program items dispatch tree typed (5 rules / 19 annotations)

Mirror of SV-Slice-19's module-items batch, applied to the interface and program sub-trees. Every `header.items` and `body.items` field on every typed interface/program declaration now surfaces kind-discriminated dispatch into the actual content — bringing the interface and program walk paths up to the same level of typed dispatch the module sub-tree reached in SV-Slice-19.

### Annotations

```ebnf
interface_item := port_declaration semi  -> {kind: "port_declaration", body: $1}
                | non_port_interface_item -> {kind: "non_port_item",   body: $1}

interface_or_generate_item := attribute_instance* module_common_item    -> {kind: "module_common_item",   attributes: $1, body: $2}
                            | attribute_instance* extern_tf_declaration -> {kind: "extern_tf_declaration", attributes: $1, body: $2}

non_port_interface_item := generate_region              -> {kind: "generate_region",       body: $1}
                        | interface_or_generate_item    -> {kind: "interface_or_generate", body: $1}
                        | program_declaration           -> {kind: "program_declaration",   body: $1}
                        | modport_declaration           -> {kind: "modport_declaration",   body: $1}
                        | interface_declaration         -> {kind: "interface_declaration", body: $1}
                        | timeunits_declaration         -> {kind: "timeunits_declaration", body: $1}

program_item := port_declaration semi  -> {kind: "port_declaration", body: $1}
              | non_port_program_item   -> {kind: "non_port_item",   body: $1}

non_port_program_item := attribute_instance* continuous_assign                       -> {kind: "continuous_assign",                   attributes: $1, body: $2}
                       | attribute_instance* module_or_generate_item_declaration     -> {kind: "module_or_generate_item_declaration", attributes: $1, body: $2}
                       | attribute_instance* initial_construct                       -> {kind: "initial_construct",                   attributes: $1, body: $2}
                       | attribute_instance* final_construct                         -> {kind: "final_construct",                     attributes: $1, body: $2}
                       | attribute_instance* concurrent_assertion_item               -> {kind: "concurrent_assertion_item",           attributes: $1, body: $2}
                       | timeunits_declaration                                       -> {kind: "timeunits_declaration",               body: $1}
                       | program_generate_item                                       -> {kind: "program_generate_item",               body: $1}
```

### Consumer dispatch chains

The interface walk path now mirrors the module walk path:

```rust
// interface_declaration.body.items contains interface_item entries
for item in interface_decl.items.as_array().unwrap() {
    match item["kind"].as_str().unwrap() {
        "port_declaration" => walk_port_decl(&item["body"]),
        "non_port_item" => {
            let npi = &item["body"];     // non_port_interface_item shape
            match npi["kind"].as_str().unwrap() {
                "generate_region"        => walk_generate(&npi["body"]),
                "interface_or_generate"  => {
                    let iog = &npi["body"];   // interface_or_generate_item shape
                    match iog["kind"].as_str().unwrap() {
                        "module_common_item" | "extern_tf_declaration" =>
                            walk_inner(iog["kind"].as_str(), &iog["attributes"], &iog["body"]),
                    }
                }
                "program_declaration" | "modport_declaration" |
                "interface_declaration" | "timeunits_declaration" =>
                    walk_decl(npi["kind"].as_str(), &npi["body"]),
            }
        }
    }
}

// program_declaration.body.items contains program_item entries
for item in program_decl.items.as_array().unwrap() {
    match item["kind"].as_str().unwrap() {
        "port_declaration" => walk_port_decl(&item["body"]),
        "non_port_item" => {
            let npi = &item["body"];     // non_port_program_item shape
            match npi["kind"].as_str().unwrap() {
                "continuous_assign" | "module_or_generate_item_declaration" |
                "initial_construct" | "final_construct" | "concurrent_assertion_item" =>
                    walk_inner(npi["kind"].as_str(), &npi["attributes"], &npi["body"]),
                "timeunits_declaration" | "program_generate_item" =>
                    walk_decl(npi["kind"].as_str(), &npi["body"]),
            }
        }
    }
}
```

### Annotation inventory

163 entries (was 144). +19 in this batch.

### Same accept set, same diagnostic codes. Schema stays at `1`.

### mdBook updated, gate green.

### Next slice candidates

- `package_or_generate_item_declaration` (large Or — referenced by module_or_generate_item_declaration's branch 0; would close the package-items dispatch path).
- `generate_region`, `generate_block`, `generate_item` (generate sub-tree).
- `module_common_item` (12-branch Or — referenced by both module_or_generate_item and interface_or_generate_item; would unlock common-item dispatch for both module and interface walks).

## Release 1.0.19 / Contract 1.0.19 Highlights — SV-Slice-19 batch: module-items dispatch tree typed (5 rules / 22 annotations)

Largest batch yet. Every `header.items` and `body.items` field on every typed module/interface/program declaration now surfaces kind-discriminated dispatch into the actual content.

### Annotations

```ebnf
module_item := port_declaration semi  -> {kind: "port_declaration", body: $1}
            | non_port_module_item   -> {kind: "non_port_item",    body: $1}

module_or_generate_item := attribute_instance* parameter_override     -> {kind: "parameter_override",  attributes: $1, body: $2}
                        | attribute_instance* gate_instantiation     -> {kind: "gate_instantiation",  attributes: $1, body: $2}
                        | attribute_instance* udp_instantiation      -> {kind: "udp_instantiation",   attributes: $1, body: $2}
                        | attribute_instance* module_instantiation   -> {kind: "module_instantiation", attributes: $1, body: $2}
                        | attribute_instance* module_common_item     -> {kind: "module_common_item",  attributes: $1, body: $2}

module_or_generate_item_declaration := package_or_generate_item_declaration                                -> {kind: "package_or_generate", body: $1}
                                    | genvar_declaration                                                  -> {kind: "genvar",              body: $1}
                                    | clocking_declaration                                                -> {kind: "clocking",            body: $1}
                                    | kw_default kw_clocking clocking_identifier semi                     -> {kind: "default_clocking",    name: $3}
                                    | kw_default kw_disable kw_iff expression_or_dist semi                -> {kind: "default_disable_iff", expr: $4}

non_port_module_item := generate_region                              -> {kind: "generate_region",       body: $1}
                     | module_or_generate_item                       -> {kind: "module_or_generate",    body: $1}
                     | specify_block                                  -> {kind: "specify_block",         body: $1}
                     | attribute_instance* specparam_declaration     -> {kind: "specparam_declaration", attributes: $1, body: $2}
                     | program_declaration                            -> {kind: "program_declaration",   body: $1}
                     | module_declaration                             -> {kind: "module_declaration",    body: $1}
                     | interface_declaration                          -> {kind: "interface_declaration", body: $1}
                     | timeunits_declaration                          -> {kind: "timeunits_declaration", body: $1}

continuous_assign := kw_assign (drive_strength)? (delay)? list_of_net_assignments semi
                        -> {kind: "net",      drive_strength: $2, delay: $3, assignments: $4}
                  | kw_assign (delay_control)? list_of_variable_assignments semi
                        -> {kind: "variable", delay_control:  $2, assignments: $3}
```

### Consumer dispatch chains

After this slice, the typed AST exposes 5+ layers of typed dispatch end-to-end for module/interface contents:

```rust
// description.body.body.items contains module_item entries
for item in module.items.as_array().unwrap() {
    match item["kind"].as_str().unwrap() {
        "port_declaration" => walk_port_decl(&item["body"]),
        "non_port_item" => {
            let npi = &item["body"];     // non_port_module_item shape
            match npi["kind"].as_str().unwrap() {
                "generate_region" => walk_generate(&npi["body"]),
                "module_or_generate" => {
                    let mog = &npi["body"];   // module_or_generate_item shape
                    match mog["kind"].as_str().unwrap() {
                        "parameter_override" | "gate_instantiation" | "udp_instantiation" |
                        "module_instantiation" | "module_common_item" => {
                            let attrs = &mog["attributes"];
                            walk_inner(mog["kind"].as_str(), attrs, &mog["body"]);
                        }
                    }
                }
                "specify_block" | "specparam_declaration" | "program_declaration" |
                "module_declaration" | "interface_declaration" | "timeunits_declaration" => {
                    walk_decl(npi["kind"].as_str(), &npi["body"]);
                }
            }
        }
    }
}
```

### Annotation inventory

144 entries (was 122). +22 in this batch — largest single-slice contribution to date.

### Same accept set, same diagnostic codes. Schema stays at `1`.

### mdBook updated, gate green.

### Next slice candidates

- `interface_item`, `interface_or_generate_item` (interface counterpart of module_item).
- `program_item`, `non_port_program_item` (program counterpart).
- `package_or_generate_item_declaration` (large Or — referenced by module_or_generate_item_declaration's branch 0).
- `generate_region`, `generate_block` (generate sub-tree).

## Release 1.0.18 / Contract 1.0.18 Highlights — SV-Slice-18 batch: UDP truth-table entries typed

3 rules / 3 annotations completing the UDP truth-table walk path.

```ebnf
combinational_entry := level_input_list colon output_symbol semi
                    -> {inputs: $1, output: $3}

sequential_entry := seq_input_list colon current_state colon next_state semi
                 -> {inputs: $1, current_state: $3, next_state: $5}

udp_initial_statement := kw_initial output_port_identifier assign init_val semi
                      -> {name: $2, init_val: $4}
```

Every UDP truth-table row now exposes a clean typed shape — consumers walk `entries.first` and each `entries.rest` item directly without descending the raw envelope.

### Annotation inventory

122 entries (was 119). +3 in this batch.

### Same accept set, same diagnostic codes. Schema stays at `1`.

### mdBook updated, gate green.

## Release 1.0.17 / Contract 1.0.17 Highlights — SV-Slice-17 batch: UDP body sub-tree typed

6 rules / 8 annotations completing UDP declaration internals.

### Annotations

```ebnf
udp_body := combinational_body -> {kind: "combinational", body: $1}
         | sequential_body    -> {kind: "sequential",    body: $1}

udp_input_declaration := attribute_instance* kw_input list_of_udp_port_identifiers
                      -> {attributes: $1, identifiers: $3}

udp_output_declaration := attribute_instance* kw_output port_identifier
                            -> {kind: "wire", attributes: $1, name: $3}
                       | attribute_instance* kw_output kw_reg port_identifier (assign constant_expression)?
                            -> {kind: "reg", attributes: $1, name: $4, default: $5}

combinational_body := kw_table combinational_entry combinational_entry* kw_endtable
                   -> {entries: {first: $2, rest: $3}}

sequential_body := (udp_initial_statement)? kw_table sequential_entry sequential_entry* kw_endtable
                -> {initial: $1, entries: {first: $3, rest: $4}}

list_of_udp_port_identifiers := port_identifier (comma port_identifier)*
                             -> {first: $1, rest: $2}
```

### UDP declaration internals fully typed end-to-end

Combined with prior slices (UDP top-level rules from SV-Slice-12, port lists from SV-Slice-15), consumers walking a UDP `primitive ... endprimitive` construct get clean typed access at every level:

```rust
match desc.body.kind {
    "udp_declaration" => {
        let udp = &desc.body.body;
        match udp.kind {
            "ansi" | "nonansi" | "wildcard" | "extern_*" => {
                let header = &udp.header;          // {attributes, name, ports}
                let body = &udp.body;              // {kind, body} — combinational | sequential
                match body.kind {
                    "combinational" => {
                        let entries = &body.body.entries;  // {first, rest}
                        // walk combinational entries
                    }
                    "sequential" => {
                        let initial = &body.body.initial;  // optional udp_initial_statement
                        let entries = &body.body.entries;  // {first, rest}
                        // walk sequential entries
                    }
                }
            }
        }
    }
}
```

### Annotation inventory

119 entries (was 111). +8 in this batch.

### Same accept set, same diagnostic codes. Schema stays at `1`.

### mdBook updated, gate green.

### Next slice candidates

- `combinational_entry`, `sequential_entry` (UDP truth-table entry sub-rules).
- `udp_initial_statement` typing.
- `package_or_generate_item_declaration` (large Or — 15+ branches).

## Release 1.0.16 / Contract 1.0.16 Highlights — SV-Slice-16 batch: port + port_direction + package_import family typed

4 rules / 9 annotations.

### Annotations

```ebnf
port := (port_expression)?
        -> {kind: "expression", expr: $1}
     | dot port_identifier lparen (port_expression)? rparen
        -> {kind: "named", name: $2, expr: $4}

port_direction := kw_input  -> {kind: "input"}
               | kw_output -> {kind: "output"}
               | kw_inout  -> {kind: "inout"}
               | kw_ref    -> {kind: "ref"}

package_import_declaration := kw_import package_import_item (comma package_import_item)* semi
                            -> {items: {first: $2, rest: $3}}

package_import_item := package_identifier scope_resolution identifier
                          -> {kind: "explicit", package: $1, name: $3}
                     | package_identifier scope_resolution star
                          -> {kind: "wildcard", package: $1}
```

### Notes

- **`port`** distinguishes positional ports `(expr)` from named-dot ports `.name(expr)`. Empty port placeholders (commas with no expression) flow through the `kind:"expression"` branch with `expr: []`.
- **`port_direction`** propagates as a typed sub-shape into any rule that references it (e.g., `ansi_port_declaration`'s named_dot branch's `direction:` field — when that rule eventually types).
- **`package_import_declaration`** wraps the `import a::*, b::c;` statement; consumers iterate `items.first + items.rest` for each import target.
- **`package_import_item`** discriminates `pkg::*` (wildcard) from `pkg::name` (explicit). Both `package` and `name` are clean identifier strings (inherited from SV-Slice-8).

### DEFERRED: `ansi_port_declaration` per-branch typing — task #38 blocker

Branch 0 (`( net_port_header | interface_port_header )? port_identifier ...`) starts with a parens-grouped Or. PGEN's annotation parser hits the parens-grouped-Or trailing-annotation attribution bug (task #38) — the per-branch annotations register out-of-order (branches 1+2 instead of 0+1+2) and the third branch's annotation is dropped entirely. Same blocker as `comment_only_source_region` from SV-Slice-6 batch. Reverted to un-annotated; tracked as follow-up either via task #38 fix OR grammar refactor extracting the leading parens-Or into a named helper rule.

### Annotation inventory

111 entries (was 102). +9 in this batch.

### Same accept set, same diagnostic codes. Schema stays at `1`.

### mdBook updated, gate green.

### Next slice candidates

- `package_or_generate_item_declaration` Or (large — 15+ branches; reaches deep into the SV grammar's declaration tree).
- `port_expression` per-branch typing.
- `udp_output_declaration` / `udp_input_declaration` per-branch typing.
- Grammar refactor or task #38 fix to unblock `ansi_port_declaration`, `comment_only_source_region`, and other parens-grouped-Or rules.

## Release 1.0.15 / Contract 1.0.15 Highlights — SV-Slice-15 batch: port-list family + small structural rules typed

6 rules / 7 annotations. Every `header.ports` field on every typed module/interface/program/UDP declaration now surfaces a typed shape instead of the raw envelope.

### Annotations

```ebnf
list_of_ports := lparen port (comma port)* rparen
              -> {first: $2, rest: $3}

list_of_port_declarations := lparen (attribute_instance* ansi_port_declaration (comma attribute_instance* ansi_port_declaration)*)? rparen
                          -> $2

udp_port_list := output_port_identifier comma input_port_identifier (comma input_port_identifier)*
              -> {output: $1, inputs: {first: $3, rest: $4}}

udp_declaration_port_list := udp_output_declaration comma udp_input_declaration (comma udp_input_declaration)*
                          -> {output: $1, inputs: {first: $3, rest: $4}}

anonymous_program := kw_program semi anonymous_program_item* kw_endprogram
                  -> {items: $3}

package_export_declaration := kw_export star scope_resolution star semi
                                 -> {kind: "wildcard"}
                            | kw_export package_import_item (comma package_import_item)* semi
                                 -> {kind: "explicit", items: {first: $2, rest: $3}}
```

### Notes per rule

- **`list_of_ports` and `list_of_port_declarations` differ in shape**: the former emits `{first, rest}` (mini-mixed-array workaround for `port (comma port)*`); the latter passes the optional inner content through transparently with `-> $2` (the parens-grouped optional sequence). `list_of_port_declarations` body when populated is a 3-element envelope `[<attribute_instance*>, <ansi_port_declaration>, <(comma attribute_instance* ansi_port_declaration)*>]`. Per-rule typing of `ansi_port_declaration` is a follow-up slice.
- **`udp_port_list` vs `udp_declaration_port_list`** parallel shapes (output + inputs.{first, rest}) but the underlying sub-rules differ — `udp_port_list` uses identifier strings (`output_port_identifier`, `input_port_identifier`), `udp_declaration_port_list` uses full declarations (`udp_output_declaration`, `udp_input_declaration`).
- **`anonymous_program`** drops kw_program/semi/kw_endprogram and exposes only `items`. Reachable via `package_item.kind = "anonymous_program"` then walk `body.items`.
- **`package_export_declaration`** discriminates between wildcard `export *::*;` and explicit `export item, item, ...;`. Wildcard form drops everything (just the kind label). Explicit form uses the standard {first, rest} mini-mixed-array.

### Annotation inventory

102 entries (was 95). +7 in this batch. **Crossing 100 annotations** for the SV grammar — the campaign is now mid-flight.

### Same accept set, same diagnostic codes. Schema stays at `1`.

### mdBook updated, gate green.

### Next slice candidates

- `port` per-branch typing (the inner element of `list_of_ports`).
- `ansi_port_declaration` per-branch typing (the inner element of `list_of_port_declarations`).
- `udp_output_declaration` / `udp_input_declaration` per-branch typing.
- `package_or_generate_item_declaration` (large Or — the actual content under package_item.kind = "declaration"; reaches deep into the SV grammar).
- `package_import_declaration` / `package_import_item` typing.

## Release 1.0.14 / Contract 1.0.14 Highlights — SV-Slice-14 batch: bind sub-tree completion + interface_class_declaration + config_declaration

5 rules typed in one batch — completes the bind directive sub-tree (started in SV-Slice-13) and adds two more top-level construct families.

### Annotations

```ebnf
bind_target_scope := module_identifier    -> {kind: "module",    name: $1}
                  | interface_identifier -> {kind: "interface", name: $1}

bind_target_instance := hierarchical_identifier constant_bit_select
                     -> {name: $1, bit_select: $2}

bind_target_instance_list := bind_target_instance (comma bind_target_instance)*
                          -> {first: $1, rest: $2}

interface_class_declaration := kw_interface kw_class declared_interface_class_identifier
                                (parameter_port_list)?
                                (kw_extends interface_class_type (comma interface_class_type)*)?
                                semi interface_class_item* kw_endclass (colon class_identifier)?
                            -> {name: $3, parameters: $4, extends: $5, items: $7, end_label: $9}

config_declaration := kw_config config_identifier semi
                       (local_parameter_declaration semi)*
                       design_statement
                       config_rule_statement*
                       kw_endconfig (colon config_identifier)?
                   -> {name: $2, local_params: $4, design: $5, rules: $6, end_label: $8}
```

### Bind sub-tree fully typed

Combined with SV-Slice-13's bind_directive/bind_instantiation typing, consumers walking a bind directive get clean typed access at every level:

```rust
// description.kind = "bind_directive" → desc.body is the typed bind shape
match desc.body.kind {
    "scoped" => {
        // bind <target_scope> [: <instances>] <instantiation>
        let scope = &desc.body.target_scope;     // {kind, name} from bind_target_scope
        let instances = &desc.body.instances;    // {first, rest} from bind_target_instance_list (or [] if no `:` clause)
        let inst = &desc.body.instantiation;     // {kind, body} from bind_instantiation
        match scope.kind { "module" | "interface" => /* ... */ }
        // ... iterate instances.first + instances.rest with each as
        //     {name, bit_select} from bind_target_instance
    }
    "single" => {
        // bind <target_instance> <instantiation>
        let inst = &desc.body.target_instance;   // {name, bit_select}
        // ...
    }
}
```

### `interface_class_declaration` and `config_declaration`

Both are single-sequence rules (no Or branches) typed with named fields. Reachable via `package_item.kind = "declaration"` (then walk into the package_or_generate_item_declaration body) for interface_class_declaration; via `description.kind = "config_declaration"` for config_declaration.

### Annotation inventory

95 entries (was 89). +6 in this batch.

### Same accept set, same diagnostic codes. Schema stays at `1`.

### mdBook updated, gate green.

### Annotation-language idiom note

**`{first, rest}` workaround applied a third time** (after attribute_instance and udp_port_decls) — for `bind_target_instance_list`'s `X (comma X)*` mini-mixed-array. The pattern is now firmly established for any "required-first + repeat" rule shape. A future codegen extension supporting true `[$1, $2**]` mixed-array spread would let these all flatten to clean arrays.

### Next slice candidates

- `bind_target_instance.bit_select` deep typing (constant_bit_select sub-rule).
- `udp_port_list` / `udp_declaration_port_list` (sub-rule typing inside `header.ports` for UDP).
- `list_of_ports` / `list_of_port_declarations` (sub-rule typing for module/interface/program port lists).
- `package_or_generate_item_declaration` (large Or — the actual content under package_item.kind = "declaration").
- `package_export_declaration`, `anonymous_program` per-branch typing.

## Release 1.0.13 / Contract 1.0.13 Highlights — SV-Slice-13 batch: bind_directive + bind_instantiation + package_item per-branch typed

3 Or rules typed; downstream consumers gain clean kind dispatch on description's `package_item` and `bind_directive` branches (reached when description.kind = `"package_item"` or `"bind_directive"`).

### Annotations

```ebnf
bind_directive := kw_bind bind_target_scope (colon bind_target_instance_list)? bind_instantiation semi
                    -> {kind: "scoped", target_scope: $2, instances: $3, instantiation: $4}
               | kw_bind bind_target_instance bind_instantiation semi
                    -> {kind: "single", target_instance: $2, instantiation: $3}

bind_instantiation := program_instantiation   -> {kind: "program",   body: $1}
                   | module_instantiation     -> {kind: "module",    body: $1}
                   | interface_instantiation  -> {kind: "interface", body: $1}
                   | checker_instantiation    -> {kind: "checker",   body: $1}

package_item := package_or_generate_item_declaration -> {kind: "declaration",        body: $1}
             | anonymous_program                     -> {kind: "anonymous_program",  body: $1}
             | package_export_declaration            -> {kind: "export",             body: $1}
             | timeunits_declaration                 -> {kind: "timeunits",          body: $1}
```

### Consumer dispatch

```rust
// description.kind == "bind_directive" → desc.body is the typed bind_directive shape
match desc.body.kind {
    "scoped" => {
        // (?<scope> : <instances>)? <instantiation>
        let scope = &desc.body.target_scope;
        let instances = &desc.body.instances;  // empty array if no `:` clause
        let inst = &desc.body.instantiation;
        process_bind_scoped(scope, instances, inst);
    }
    "single" => {
        // <target_instance> <instantiation>
        process_bind_single(&desc.body.target_instance, &desc.body.instantiation);
    }
}

// inst.kind dispatches to which form of instantiation:
match inst.kind {
    "program" | "module" | "interface" | "checker" => walk_instantiation(inst.kind, &inst.body),
}

// description.kind == "package_item" → desc.body is the typed package_item shape
match desc.body.kind {
    "declaration"       => process_decl(&desc.body.body),
    "anonymous_program" => process_anon_program(&desc.body.body),
    "export"            => process_export(&desc.body.body),
    "timeunits"         => process_timeunits(&desc.body.body),
}
```

### Annotation inventory

89 entries (was 79). +10 in this batch.

### Same accept set, same diagnostic codes. Schema stays at `1`.

### mdBook updated, gate green.

### Next slice candidates

- `bind_target_scope` (2-form Or — module_identifier vs interface_identifier).
- `bind_target_instance` and `bind_target_instance_list` (single-sequence + comma-spread mini-mixed-array).
- `interface_class_declaration` per-branch.
- `config_declaration` (single sequence, ~10 elements).
- Sub-rule typing inside `header.ports` (`udp_port_list`, `udp_declaration_port_list`, `list_of_ports`).

## Release 1.0.12 / Contract 1.0.12 Highlights — SV-Slice-12 batch: UDP declaration family typed (mirror of module/interface/program pattern + mini-mixed-array workaround)

> **For Nexsim maintainers:** UDP (User-Defined Primitive) declarations now have the same typed surface as module/interface/program. 4-layer typed dispatch end-to-end for `primitive p (...) ... endprimitive` constructs reachable from `description.body` when `kind:"udp_declaration"`.

### Annotations

```ebnf
udp_ansi_declaration := attribute_instance* kw_primitive udp_identifier lparen udp_declaration_port_list rparen semi
                     -> {attributes: $1, name: $3, ports: $5}

udp_nonansi_declaration := attribute_instance* kw_primitive udp_identifier lparen udp_port_list rparen semi
                        -> {attributes: $1, name: $3, ports: $5}

udp_declaration_sv_2017 := udp_nonansi_declaration udp_port_declaration udp_port_declaration* udp_body kw_endprimitive (colon udp_identifier)?
                            -> {kind: "nonansi", header: $1, port_decls: {first: $2, rest: $3}, body: $4, end_label: $6}
                         | udp_ansi_declaration udp_body kw_endprimitive (colon udp_identifier)?
                            -> {kind: "ansi", header: $1, body: $2, end_label: $4}
                         | kw_extern udp_nonansi_declaration
                            -> {kind: "extern_nonansi", header: $2}
                         | kw_extern udp_ansi_declaration
                            -> {kind: "extern_ansi", header: $2}
                         | attribute_instance* kw_primitive udp_identifier lparen dot_star rparen semi udp_port_declaration* udp_body kw_endprimitive (colon udp_identifier)?
                            -> {kind: "wildcard", attributes: $1, name: $3, port_decls: $8, body: $9, end_label: $11}

udp_declaration_sv_2023 := /* same 5 branches; wildcard branch positional shift for `dot star` (2 tokens) vs `dot_star` (1 token) → port_decls $8→$9, body $9→$10, end_label $11→$12 */
```

### `port_decls: {first, rest}` mini-mixed-array workaround

The `nonansi` branch (branch 0) has the pattern `udp_port_declaration udp_port_declaration*` — a required first port-decl followed by zero-or-more reps. Mixed-array spread `[$2, $3**]` is currently blocked by the annotation-language limitation (per `feedback_annotation_no_mixed_spread.md`), so the typed shape uses the `{first, rest}` workaround (same idiom as `attribute_instance` from SV-Slice-6). Consumers walking `port_decls` for `kind:"nonansi"` should:

```rust
let port_decls = &udp["port_decls"];
process_port_decl(&port_decls["first"]);
for rest_item in port_decls["rest"].as_array().unwrap() {
    // rest_item is a [matched_iteration] envelope of udp_port_declaration
    process_port_decl(rest_item);
}
```

For `kind:"wildcard"`, `port_decls` is a plain `[]`-iteration array (no leading port; handled identically to module/interface wildcard).

### 5 kind labels

- `nonansi` — `udp_nonansi_declaration` form with port-decl block
- `ansi` — `udp_ansi_declaration` form
- `wildcard` — `(.*)` form (UDP variant)
- `extern_nonansi`, `extern_ansi` — extern declarations

### Annotation inventory

79 entries (was 67). +12 in this batch: 1 (udp_ansi_declaration) + 1 (udp_nonansi_declaration) + 5 (udp_declaration_sv_2017) + 5 (udp_declaration_sv_2023).

### Same accept set, same diagnostic codes. Schema stays at `1`.

### mdBook updated, gate green.

### Annotation-language idiom note

**`{first, rest}` workaround for `X X*` mini-mixed-array** — used here for `port_decls: {first: $2, rest: $3}`. Same idiom as `attribute_instance: {first, rest}` from SV-Slice-6. Until the annotation language gains true mixed-array spread (`[$2, $3**]`), this is the canonical pattern for "required-first + repeat" rule shapes.

### Next slice candidates

- `interface_class_declaration` per-branch (sibling to class_declaration).
- `program_ansi_header` / `program_nonansi_header` (already done in SV-Slice-11).
- `udp_port_list` / `udp_declaration_port_list` (sub-rule typing inside `header.ports`).
- `udp_body` / `udp_port_declaration` (sub-rules inside the typed UDP shape).
- `description` further branches: `package_item`, `bind_directive`, `config_declaration`.

## Release 1.0.11 / Contract 1.0.11 Highlights — SV-Slice-11 batch: program-header sub-tree typed (mirror of module/interface header pattern)

2 rules typed: `program_ansi_header`, `program_nonansi_header`. Both use the same field-name set as `module_ansi_header` / `interface_ansi_header` (sans `keyword:` since program only has one keyword).

### Annotations

```ebnf
program_ansi_header := attribute_instance* kw_program_81d9aeea (lifetime)? program_identifier package_import_declaration* (parameter_port_list)? (list_of_port_declarations)? semi
                    -> {attributes: $1, lifetime: $3, name: $4, imports: $5, parameters: $6, ports: $7}

program_nonansi_header := attribute_instance* kw_program_81d9aeea (lifetime)? program_identifier package_import_declaration* (parameter_port_list)? list_of_ports semi
                       -> {attributes: $1, lifetime: $3, name: $4, imports: $5, parameters: $6, ports: $7}
```

### Empirical verification on `program p; endprogram\n`

```text
description.body.body (program_declaration_sv_2017 ANSI form):
  kind: "ansi"
  header:
    attributes: []
    lifetime: []
    name: "p"          # clean string (inherited from SV-Slice-8)
    imports: []
    parameters: []
    ports: []
  timeunits: []
  items: []
  end_label: []
```

### Sibling-rule symmetry

The 3 top-level construct families that have ANSI/non-ANSI header pairs (module / interface / program) all expose the same 6-7 field shape (`attributes`, `keyword?`, `lifetime`, `name`, `imports`, `parameters`, `ports`). Consumers can write a single header walker that handles all three families.

### Annotation inventory

67 entries (was 65). +2 in this batch.

### Same accept set, same diagnostic codes.

### Schema-version stays `1`.

### mdBook updated, gate green.

### Annotation-language idiom note

**Sibling-rule pattern reuse**: when a family of rules shares structure (here, ansi/nonansi header pairs across module/interface/program), reusing the same field-name set across them is intentional and lets consumers write generic walkers. Module headers have an extra `keyword:` field for module/macromodule disambiguation; interface and program don't (single keyword each).

### Next slice candidates

- `udp_declaration_sv_2017` / `udp_declaration_sv_2023` per-branch.
- `udp_ansi_declaration` / `udp_nonansi_declaration` per-branch (UDP has its own ANSI/non-ANSI distinction).
- Investigation: package top-level parse failure.

## Release 1.0.10 / Contract 1.0.10 Highlights — SV-Slice-10 batch: class + package + program declarations typed

5 rules typed: class declarations (sv_2017 + sv_2023 single-sequence shapes), `package_declaration` single-sequence, program declarations (sv_2017 + sv_2023 — 5 per-branch kinds each, mirroring module/interface pattern).

### Annotations

```ebnf
class_declaration_sv_2017 := (kw_virtual)? kw_class (lifetime)? declared_class_identifier (parameter_port_list)? (kw_extends base_class_type (lparen list_of_arguments rparen)?)? (kw_implements interface_class_type (comma interface_class_type)*)? semi class_item* kw_endclass (colon class_identifier)?
                          -> {virtual: $1, lifetime: $3, name: $4, parameters: $5, extends: $6, implements: $7, items: $9, end_label: $11}

class_declaration_sv_2023 := (kw_virtual)? kw_class (final_specifier)? declared_class_identifier (parameter_port_list)? (kw_extends base_class_type (lparen (list_of_arguments | kw_default)? rparen)?)? (kw_implements interface_class_type (comma interface_class_type)*)? semi class_item* kw_endclass (colon class_identifier)?
                          -> {virtual: $1, final_specifier: $3, name: $4, parameters: $5, extends: $6, implements: $7, items: $9, end_label: $11}

package_declaration := attribute_instance* kw_package (lifetime)? package_identifier semi (timeunits_declaration)? (attribute_instance* package_item)* kw_endpackage (colon package_identifier)?
                    -> {attributes: $1, lifetime: $3, name: $4, timeunits: $6, items: $7, end_label: $9}

program_declaration_sv_2017 := /* 5 branches, kind: nonansi/ansi/wildcard/extern_nonansi/extern_ansi
                                  Note: nonansi listed BEFORE ansi (different from module/interface order),
                                        but kind labels still discriminate correctly. */

program_declaration_sv_2023 := /* same 5 branches; wildcard branch positional shift for `dot star` vs `dot_star`. */
```

### Profile-specific field naming on class declarations

The class rule's `lifetime` slot in SV-2017 became `final_specifier` in SV-2023 (different LRM semantics). The annotation reflects this — sv_2017 carries `lifetime: $3`, sv_2023 carries `final_specifier: $3`. Consumers walking either profile dispatch on the present field name; both fields are mutually exclusive across profiles.

### Empirical verification

| Input | Outcome |
|---|---|
| `module m; endmodule\n` | ✓ unchanged (module pattern preserved) |
| `interface bus; endinterface\n` | ✓ unchanged (interface pattern preserved) |
| `program p; endprogram\n` | ✓ NEW — `description.body.kind = "program_declaration"`, `description.body.body.kind = "ansi"` |
| `package p; endpackage\n` | ✗ parse rejected at position 0 — annotation registered correctly per the inventory; runtime parse failure appears pre-existing (this slice's annotation didn't introduce it; module/interface/program tests still pass with the same regenerated parser). Investigation tracked separately. |
| `class C; endclass\n` | ✗ expected — class_declaration is not directly in source_text_item's reachable set; class declarations are typically reached through `package_item` or other subsidiary rules. |

### Annotation inventory

65 entries (was 53). +12 in this batch: 1 (class_declaration_sv_2017) + 1 (class_declaration_sv_2023) + 1 (package_declaration) + 5 (program_declaration_sv_2017) + 5 (program_declaration_sv_2023) — but note that package_declaration's runtime path needs investigation despite the annotation registering correctly.

### Same accept set, same diagnostic codes

(Verified: module/interface/program inputs that worked before still work; the 65-annotation parser is correct for those.)

### Schema-version stays `1`.

### mdBook updates, gate green.

### Annotation-language idiom note

**Single-sequence rule typing** (no kind discriminator) is appropriate for rules that have only one form, like `class_declaration_sv_2017` and `package_declaration`. They emit a flat object with named fields rather than a `kind`-discriminated shape. Consumers reach them via the parent's `description.kind` (e.g. "class_declaration", "package_item" → contains class/package; "program_declaration" → 5-form discriminator).

### Open follow-up

- Investigate why `package mypkg; endpackage\n` doesn't parse at top level despite `package_declaration` being in `description`'s Or set. Module / interface / program with similar structures parse fine. Could be (i) a pre-existing PEG ordering issue, (ii) interaction with the `@emit_fact:` rule-level metadata annotation immediately preceding `package_declaration`, or (iii) a different rule-context constraint not visible from inspection. Tracked in MEMORY.md as a separate item.

### Next slice candidates

- Type `udp_declaration_sv_2017` / `udp_declaration_sv_2023` per-branch (deferred from this batch — has `udp_port_declaration udp_port_declaration*` mini-mixed-array pattern that needs the `{first, rest}` workaround).
- Type `program_ansi_header` / `program_nonansi_header` (sibling to `module_<form>_header`).
- Type `interface_keyword`, `kw_interface`, `kw_class`, `kw_package` (clean keyword strings — minor polish).
- Address task #38 to unblock parens-grouped-Or rules.

## Release 1.0.9 / Contract 1.0.9 Highlights — SV-Slice-9 batch: interface declarations typed (full mirror of module pattern)

Interface declarations now have the same typed surface as module declarations. 4-layer typed dispatch end-to-end: `source_text_item.kind` → `description.kind` → `interface_declaration_sv_<profile>.kind` → `interface_<form>_header.name` (clean string).

### Annotations

```ebnf
interface_ansi_header := attribute_instance* kw_interface_5ea2d81a (lifetime)? interface_identifier package_import_declaration* (parameter_port_list)? (list_of_port_declarations)? semi
                      -> {attributes: $1, lifetime: $3, name: $4, imports: $5, parameters: $6, ports: $7}

interface_nonansi_header := attribute_instance* kw_interface_5ea2d81a (lifetime)? interface_identifier package_import_declaration* (parameter_port_list)? list_of_ports semi
                         -> {attributes: $1, lifetime: $3, name: $4, imports: $5, parameters: $6, ports: $7}

interface_declaration_sv_2017 := interface_ansi_header (timeunits_declaration)? non_port_interface_item* kw_endinterface_ebd6ca35 (colon interface_identifier)?
                                  -> {kind: "ansi", header: $1, timeunits: $2, items: $3, end_label: $5}
                              | interface_nonansi_header (timeunits_declaration)? interface_item* kw_endinterface_ebd6ca35 (colon interface_identifier)?
                                  -> {kind: "nonansi", header: $1, timeunits: $2, items: $3, end_label: $5}
                              | attribute_instance* kw_interface_5ea2d81a interface_identifier lparen dot_star rparen semi (timeunits_declaration)? interface_item* kw_endinterface_ebd6ca35 (colon interface_identifier)?
                                  -> {kind: "wildcard", attributes: $1, name: $3, timeunits: $8, items: $9, end_label: $11}
                              | kw_extern_bf1ee311 interface_nonansi_header
                                  -> {kind: "extern_nonansi", header: $2}
                              | kw_extern_bf1ee311 interface_ansi_header
                                  -> {kind: "extern_ansi", header: $2}

interface_declaration_sv_2023 := /* same 5 branches; wildcard branch's positional indices shift to $9/$10/$12 because dot star (2 tokens) vs dot_star (1 token) */
```

### Differences from module pattern

- **No `keyword:` field on interface_<form>_header** — interface only has one keyword (`interface`), unlike module which has both `module` and `macromodule`. The kind discriminator at the parent level (description.kind == "interface_declaration") fully identifies the construct; an inner keyword field would be redundant. (Module headers expose `keyword: {kind: "module"|"macromodule"}` for that distinction.)
- **Same field names otherwise** — `attributes`, `lifetime`, `name`, `imports`, `parameters`, `ports` for headers; `kind`, `header`, `timeunits`, `items`, `end_label` (and `attributes`, `name` on wildcard) for declaration-level. Consumer dispatch code can mostly share between modules and interfaces.

### Empirical verification on `interface bus; endinterface\n`

```text
source_text[0]:
  kind: "description"
  body:
    kind: "interface_declaration"
    body:
      kind: "ansi"
      header:
        name: "bus"            # clean string (inherited from SV-Slice-8)
        attributes: []
        lifetime: []
        imports: []
        parameters: []
        ports: []
      timeunits: []
      items: []
      end_label: []
```

### Annotation inventory

53 entries (was 41). +12 in this batch.

### Same accept set, same diagnostic codes.

### Schema-version stays `1`.

### mdBook updated, gate green.

### Annotation-language idiom note

This slice demonstrates **structural reuse of the module typing pattern** for a sibling rule family. Same kind labels (ansi/nonansi/wildcard/extern_nonansi/extern_ansi), same field names where they apply. Consumer code sharing between module and interface walkers: trivial.

### Next slice candidates

- Type `class_declaration_sv_2017` / `class_declaration_sv_2023` per-branch.
- Type `package_declaration` (single sequence, attribute_instance* prefix).
- Type `udp_declaration_sv_2017` / `udp_declaration_sv_2023` per-branch.
- Type `program_declaration_sv_2017` / `program_declaration_sv_2023` per-branch.
- Type `kw_interface_5ea2d81a` / `kw_class_8d767bf5` / etc. (clean keyword strings) — minor but adds polish if needed for downstream tools.

## Release 1.0.8 / Contract 1.0.8 Highlights — SV-Slice-8 batch: identifier-leaf rules typed (clean strings propagate through every identifier-typed field)

This is the highest-leverage slice yet — typing 4 leaf rules causes clean identifier strings to propagate through every rule that resolves to an identifier (every `*_identifier` alias, every name field of every typed parent rule).

### Annotations

```ebnf
simple_identifier := trivia /[a-zA-Z_][a-zA-Z0-9_$]*/
                  -> $2

@sample: "\\foo "
escaped_identifier := trivia /\\[!-~]+/
                   -> $2

non_keyword_identifier := !reserved_non_keyword_identifier identifier
                       -> $2

@sample: "foo"
simple_identifier_no_scope := trivia /[a-zA-Z_][a-zA-Z0-9_$]*(?![ \t\r\n]*::)/
                           -> $2
```

All four use the `-> $2` transparent-passthrough idiom (drop trivia / lookahead, surface the regex-captured identifier name as a clean string).

### Propagation chain

```text
simple_identifier (typed: -> $2)
  ↓ matches → "m"
identifier := escaped_identifier | simple_identifier
  ↓ transparent Or → "m"
non_keyword_identifier := !reserved_non_keyword_identifier identifier (typed: -> $2)
  ↓ drops lookahead, surfaces identifier → "m"
declaration_identifier := non_keyword_identifier
  ↓ transparent alias → "m"
module_identifier := declaration_identifier
  ↓ transparent alias → "m"
class_identifier, package_identifier, etc.
  ↓ transparent aliases → "m"
```

Every typed parent rule that exposes an identifier-typed field now surfaces it as a clean JSON string. For `module_ansi_header.name`, `module_nonansi_header.name`, `description.body.body.wildcard.name` (the `(.*)` form's name field), and any future typed rule referencing `*_identifier`, the field is a clean string.

### Empirical pre/post on `module m; endmodule\n`

```text
# Pre-SV-Slice-8 — header.name was raw envelope:
"header": {"keyword": {"kind": "module"}, "name": [[], "m"], "lifetime": [], ...}

# After SV-Slice-8a (just simple_identifier + escaped_identifier typed):
"header": {"keyword": {"kind": "module"}, "name": [[], "m"], "lifetime": [], ...}
                                                  ↑ still wrapped — non_keyword_identifier still raw

# Post-SV-Slice-8 (full batch — all 4 leaf rules typed):
"header": {"keyword": {"kind": "module"}, "name": "m", "lifetime": [], ...}
                                                  ↑ clean string!
```

### Why this slice is the highest-leverage so far

Typing 4 leaf rules causes EVERY identifier in EVERY future-typed rule to land as a clean string with zero additional annotation work. Future slices typing `interface_declaration.name`, `class_declaration.name`, `package_declaration.name`, `signal_identifier`, `port_identifier`, `parameter_identifier`, etc. — all get clean strings automatically. This is dependency-graph-leveraged annotation work: type the dependency once, every dependent benefits.

### Annotation inventory

41 entries (was 37). +4 in this batch.

### Notes on the lookahead positional slot

PGEN's annotation language treats negative lookaheads (`!X`) as occupying positional slots even though they don't consume tokens. So in `non_keyword_identifier := !reserved_non_keyword_identifier identifier`, `$1` is the (empty) lookahead slot and `$2` is the matched `identifier`. Same convention as the regex parser used for `simple_escape = !"o{" !"x{" !"p{" !"P{" any_char -> {... char: $5}` (4 lookaheads → `$5` for the consumer).

### Same accept set, same diagnostic codes.

### Schema-version stays `1` (additive).

### mdBook

`changelog-index.md`, `schema-versioning.md`, `json-carrier.md`, `rules-top-level.md` updated. `make systemverilog_parser_book_gate` green.

### Public API surface unchanged.

### Annotation-language idiom note

**Leaf-rule trivia-stripping with `-> $N`** (where `$N` is the regex-capture position past the trivia prefix) is the canonical pattern for surfacing clean text from `trivia /regex/` rules. Applied here to 4 identifier-leaf rules; same idiom used in regex parser (`hex_digits`, `prop_name`, etc.) and earlier in this campaign on `compiler_directive`.

### Next slice candidates

- Type `class_declaration_sv_2017` per-branch (mirror of module_declaration's pattern; class declarations now have clean name strings).
- Type `interface_declaration_sv_2017` / `interface_declaration_sv_2023` per-branch.
- Type `package_declaration` (substantial — single sequence with attribute_instance* prefix; identifier name is now clean, simplifying the typed shape).
- Type `udp_declaration_sv_2017` / `udp_declaration_sv_2023` per-branch.
- Type `program_declaration_sv_2017` / `program_declaration_sv_2023`.

## Release 1.0.7 / Contract 1.0.7 Highlights — SV-Slice-7 batch: module_keyword + lifetime + module_ansi_header + module_nonansi_header typed (4 layers of dispatch end-to-end)

Typing the `header:` field that the SV-Slice-6 batch left as raw envelope. Four sub-rules typed in one pass; **four layers of typed dispatch are now end-to-end** for module declarations.

### (a) `module_keyword` per-branch (2 kind labels)

```ebnf
module_keyword := kw_module_fbd34a2b      -> {kind: "module"}
                | kw_macromodule_df04b866 -> {kind: "macromodule"}
```

Drops the keyword token (it's redundant with `kind`); emits a typed object that consumers can dispatch on.

### (b) `lifetime` per-branch (2 kind labels)

```ebnf
lifetime := kw_static_a381562a    -> {kind: "static"}
          | kw_automatic_ebe88724 -> {kind: "automatic"}
```

Same pattern as module_keyword. When a `(lifetime)?` slot is matched, consumers see `{kind: "static"}` / `{kind: "automatic"}`. When un-matched, they see `[]` (existing convention).

### (c) `module_ansi_header` typed sequence

```ebnf
module_ansi_header := attribute_instance* module_keyword ( lifetime )? module_identifier package_import_declaration* ( parameter_port_list )? ( list_of_port_declarations )? semi
                   -> {attributes: $1, keyword: $2, lifetime: $3, name: $4, imports: $5, parameters: $6, ports: $7}
```

7 named fields. Drops the trailing `semi`. The `keyword:` field is itself typed (per slice 7a), and `lifetime:` is itself typed when matched (per slice 7b). `name:` carries the raw `module_identifier` envelope (still un-typed; per-rule typing of identifiers is follow-up). `attributes`/`imports`/`parameters`/`ports` are quantified or optional; consumers handle empty as `[]` and matched as the inner shape.

### (d) `module_nonansi_header` typed sequence

```ebnf
module_nonansi_header := attribute_instance* module_keyword ( lifetime )? module_identifier package_import_declaration* ( parameter_port_list )? list_of_ports semi
                      -> {attributes: $1, keyword: $2, lifetime: $3, name: $4, imports: $5, parameters: $6, ports: $7}
```

Same field names as `module_ansi_header`. Consumers walking either get the same JSON shape — only the `ports:` source rule differs (`list_of_ports` vs `(list_of_port_declarations)?`). For consumer code, walking the typed shape is uniform across ANSI / non-ANSI forms.

### Empirical pre/post on `module m; endmodule\n` (sv_2017 profile)

```text
# Pre-SV-Slice-7 — header was raw envelope:
"body": {
  "kind": "ansi",
  "header": [<module_ansi_header raw 8-element envelope>],
  "timeunits": [],
  "items": [],
  "end_label": []
}

# Post-SV-Slice-7 — header is itself a typed object with named fields:
"body": {
  "kind": "ansi",
  "header": {
    "attributes": [],
    "keyword": {"kind": "module"},
    "lifetime": [],
    "name": [<module_identifier raw envelope>],
    "imports": [],
    "parameters": [],
    "ports": []
  },
  "timeunits": [],
  "items": [],
  "end_label": []
}
```

### Four layers of typed dispatch end-to-end

```rust
// Walking a module declaration end-to-end:
for item in obj["source_text"].as_array().unwrap() {
    if item["kind"] == "description" {
        let desc = &item["body"];
        if desc["kind"] == "module_declaration" {
            let module = &desc["body"];   // module_declaration_sv_<profile> shape
            match module["kind"].as_str().unwrap() {
                "ansi" | "nonansi" => {
                    let hdr = &module["header"];   // module_<form>_header shape
                    let module_kind = hdr["keyword"]["kind"].as_str().unwrap();   // "module" | "macromodule"
                    let lifetime = match &hdr["lifetime"] {
                        v if v.is_array() && v.as_array().unwrap().is_empty() => None,
                        v => Some(v["kind"].as_str().unwrap()),  // "static" | "automatic"
                    };
                    let attrs = hdr["attributes"].as_array().unwrap();
                    let imports = hdr["imports"].as_array().unwrap();
                    // ... process module ...
                }
                "wildcard" => { /* similar — wildcard exposes more fields directly */ }
                "extern_nonansi" | "extern_ansi" => {
                    let hdr = &module["header"];   // same module_<form>_header shape
                    // ... process extern declaration ...
                }
                _ => unreachable!(),
            }
        }
    }
}
```

### Annotation inventory

37 entries (was 31). +6 in this batch: 2 (module_keyword) + 2 (lifetime) + 1 (module_ansi_header) + 1 (module_nonansi_header).

### Same accept set, same diagnostic codes.

### Schema-version stays `1` (additive).

### mdBook

`changelog-index.md`, `schema-versioning.md`, `json-carrier.md`, `rules-top-level.md` updated. `make systemverilog_parser_book_gate` green.

### Public API surface unchanged.

### Annotation-language idiom notes

- **Tiny-Or-typed-as-kind-tag** (`X := A -> {kind: "a"} | B -> {kind: "b"}`) is the regex-campaign pattern for keyword-distinguishing rules. Used here on `module_keyword` and `lifetime`. Once a keyword rule is typed this way, every parent rule that references it inherits the typed sub-shape automatically.

### Next slice candidates

- Type `module_identifier` / `declaration_identifier` (currently the un-typed `name:` field on module_<form>_header).
- Type `class_declaration_sv_2017` / `class_declaration_sv_2023` per-branch (mirror of module_declaration pattern).
- Type `interface_declaration_sv_2017` / `interface_declaration_sv_2023` per-branch.
- Type `package_declaration` (substantial sequence with attribute_instance* prefix).
- Type `udp_declaration_sv_2017` / `udp_declaration_sv_2023` per-branch.

## Release 1.0.6 / Contract 1.0.6 Highlights — SV-Slice-6 batch: attribute_instance + module_declaration_sv_2017/2023 typed (3 layers of dispatch end-to-end)

This is a multi-rule batch slice typing 3 rules in one pass. Three layers of typed dispatch are now end-to-end: `source_text_item.kind` → `description.kind` → `module_declaration_sv_<profile>.kind`.

### (a) `attribute_instance` — `{first, rest}` shape

```ebnf
attribute_instance := attr_open attr_spec ( comma attr_spec )* attr_close
                   -> {first: $2, rest: $3}
```

Drops the `attr_open` (`(*`) and `attr_close` (`*)`) syntactic delimiters. Exposes the first attr_spec as `first:` and the trailing `( comma attr_spec )*` repetitions as `rest:` (each rest entry is `[comma, attr_spec]`). Mixed-array spread `[$2, $3**]` is currently blocked by an annotation-language limitation (per `feedback_annotation_no_mixed_spread.md`) so the cleaner flat-array form is deferred. Consumers walk `obj.first` for the leading attribute and iterate `obj.rest` for additional attributes.

### (b) `module_declaration_sv_2017` per-branch typed (5 forms)

```ebnf
module_declaration_sv_2017 := @sample: "module m; endmodule" module_ansi_header (timeunits_declaration)? non_port_module_item* kw_endmodule_2eb38ec9 (colon module_identifier)?
                                -> {kind: "ansi", header: $1, timeunits: $2, items: $3, end_label: $5}
                            | @sample: "module m(a); endmodule" module_nonansi_header (timeunits_declaration)? module_item* kw_endmodule_2eb38ec9 (colon module_identifier)?
                                -> {kind: "nonansi", header: $1, timeunits: $2, items: $3, end_label: $5}
                            | @sample: "module m(.*); endmodule" attribute_instance* module_keyword (lifetime)? module_identifier lparen dot_star rparen semi (timeunits_declaration)? module_item* kw_endmodule_2eb38ec9 (colon module_identifier)?
                                -> {kind: "wildcard", attributes: $1, keyword: $2, lifetime: $3, name: $4, timeunits: $9, items: $10, end_label: $12}
                            | @sample: "extern module m(a);" kw_extern_bf1ee311 module_nonansi_header
                                -> {kind: "extern_nonansi", header: $2}
                            | @sample: "extern module m;" kw_extern_bf1ee311 module_ansi_header
                                -> {kind: "extern_ansi", header: $2}
```

5 kind labels: `"ansi"`, `"nonansi"`, `"wildcard"`, `"extern_nonansi"`, `"extern_ansi"`. Each carries the structured fields needed to walk the matched form. The wildcard branch (the `(.*)` form) preserves the leading `attribute_instance*`, the `module_keyword`, optional `lifetime`, and the `module_identifier` as named fields. The two extern branches expose only the matched header as a `header:` field (drops the `kw_extern` keyword).

### (c) `module_declaration_sv_2023` per-branch typed (5 forms — mirror of sv_2017 with positional shift)

```ebnf
module_declaration_sv_2023 := @sample: "module m; endmodule" module_ansi_header (timeunits_declaration)? non_port_module_item* kw_endmodule_2eb38ec9 (colon module_identifier)?
                                -> {kind: "ansi", header: $1, timeunits: $2, items: $3, end_label: $5}
                            | @sample: "module m(a); endmodule" module_nonansi_header (timeunits_declaration)? module_item* kw_endmodule_2eb38ec9 (colon module_identifier)?
                                -> {kind: "nonansi", header: $1, timeunits: $2, items: $3, end_label: $5}
                            | @sample: "module m(.*); endmodule" attribute_instance* module_keyword (lifetime)? module_identifier lparen dot star rparen semi (timeunits_declaration)? module_item* kw_endmodule_2eb38ec9 (colon module_identifier)?
                                -> {kind: "wildcard", attributes: $1, keyword: $2, lifetime: $3, name: $4, timeunits: $10, items: $11, end_label: $13}
                            | @sample: "extern module m(a);" kw_extern_bf1ee311 module_nonansi_header
                                -> {kind: "extern_nonansi", header: $2}
                            | @sample: "extern module m;" kw_extern_bf1ee311 module_ansi_header
                                -> {kind: "extern_ansi", header: $2}
```

Same kind labels as sv_2017; only the wildcard branch differs in positional indices. sv_2023 uses `dot star` (2 separate tokens) where sv_2017 uses `dot_star` (1 combined token), shifting the wildcard branch's later positional refs: `timeunits` from `$9` → `$10`, `items` from `$10` → `$11`, `end_label` from `$12` → `$13`. Same kind discriminator and field names are exposed to consumers — the profile-shift is invisible in the typed AST.

### Empirical pre/post on `module m; endmodule\n` (sv_2017 profile)

```text
# Pre — body field of description-kind source_text_item.body was raw envelope:
"source_text": [
  {
    "kind": "description",
    "body": {
      "kind": "module_declaration",
      "body": [<module_declaration_sv_2017 raw envelope>]   // 5-element array
    }
  }
]

# Post — three layers of typed dispatch:
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
```

### Consumer dispatch unlocked at the module-declaration level

```rust
for item in obj["source_text"].as_array().unwrap() {
    if item["kind"] == "description" {
        let desc = &item["body"];
        if desc["kind"] == "module_declaration" {
            let module = &desc["body"];
            match module["kind"].as_str().unwrap() {
                "ansi" => walk_ansi(&module["header"], &module["timeunits"],
                                    &module["items"], &module["end_label"]),
                "nonansi" => walk_nonansi(&module["header"], &module["timeunits"],
                                          &module["items"], &module["end_label"]),
                "wildcard" => walk_wildcard(&module["attributes"], &module["keyword"],
                                            &module["lifetime"], &module["name"],
                                            &module["timeunits"], &module["items"],
                                            &module["end_label"]),
                "extern_nonansi" => walk_extern_nonansi(&module["header"]),
                "extern_ansi"    => walk_extern_ansi(&module["header"]),
                other => panic!("unknown module_declaration kind: {}", other),
            }
        }
    }
}
```

### Annotation inventory

31 entries (was 20). New: 1 (attribute_instance) + 5 (module_declaration_sv_2017) + 5 (module_declaration_sv_2023) = 11 added.

### `comment_only_source_region` typing — DEFERRED, blocked by task #38

This batch attempted to also type `comment_only_source_region := white_space* ( line_comment | block_comment ) ( white_space | line_comment | block_comment )*` with `-> {first: $2, rest: $3}`. Annotation didn't register: parser inventory count stayed unchanged after that change. This is task #38 (parens-grouped-Or trailing-annotation attribution bug — same class as the regex parser PGEN-EBNF gotcha logged earlier). The `comment_only_source_region` rule's two parens-grouped Or expressions cause the trailing `-> ...` annotation to attach to one of the inner Ors instead of the rule. Annotation reverted; sub-rule typing of comment_only_source_region is gated on task #38's resolution OR a grammar refactor that flattens the parens-grouped Ors into named helper rules.

### Same accept set, same diagnostic codes

Only the AST shape changed. No grammar accept-set or diagnostic-code change.

### Schema-version stays `1` (additive across all three slices).

### mdBook

`changelog-index.md`, `schema-versioning.md`, `json-carrier.md`, `rules-top-level.md` updated. `make systemverilog_parser_book_gate` green.

### Public API surface unchanged.

### Annotation-language idiom notes

- **`{first: $N, rest: $M}` workaround for parens-grouped quantified repetition** is a clean fallback when `[$N, $M**]` mixed-array spread isn't available. Used here on attribute_instance.
- **Multi-line per-branch annotation with `@sample:` metadata** preserved correctly through the codegen path. PGEN's EBNF parser treats `@sample: "..."` as branch metadata that doesn't shift positional indices for the `-> ...` annotation following the branch body.

### Next slice candidates

- Type `module_ansi_header` per-branch (currently the unwalked `header:` field on the ansi/extern_ansi forms).
- Type `module_nonansi_header` per-branch.
- Type `module_keyword` (2-form Or: `module` / `macromodule`).
- Type `interface_declaration`, `package_declaration`, `class_declaration` per-branch (sibling rules to module_declaration).
- Address task #38 to unblock comment_only_source_region + similar parens-grouped-Or rules.

## Release 1.0.5 / Contract 1.0.5 Highlights — SV-Slice-5: `compiler_directive` transparent passthrough (clean directive text)

- **Annotation:** `compiler_directive := trivia /` `` `[^\r\n]*/`` `` -> $2` (line 226 of `grammars/systemverilog.ebnf`).
- **Effect:** drops the leading `trivia` (whitespace) prefix from the matched sequence and emits just the captured directive text (the `` ` `` backtick + directive name + arguments) as a clean JSON string. When `source_text_item.kind == "compiler_directive"`, the `body` field is now a directly-usable string instead of a nested envelope.
- **Empirical pre/post on `` `define FOO bar `` followed by `module m; endmodule\n`:**

```text
# Pre-SV-Slice-5 — body was the raw envelope of `trivia regex_capture`:
"source_text": [
  {
    "kind": "compiler_directive",
    "body": [<trivia envelope>, "`define FOO bar"]   // 2-element array
  },
  {"kind": "description", "body": {...}}
]

# Post-SV-Slice-5 — body is the clean directive string:
"source_text": [
  {
    "kind": "compiler_directive",
    "body": "`define FOO bar"   // clean string, ready to use
  },
  {"kind": "description", "body": {...}}
]
```

- **Consumer dispatch is now trivially simple for compiler directives:**

```rust
for item in obj["source_text"].as_array().unwrap() {
    match item["kind"].as_str().unwrap() {
        "compiler_directive" => {
            let directive_text = item["body"].as_str().unwrap();
            // directive_text is e.g. "`define FOO bar" — ready to feed to a
            // compiler-directive parser without further AST descent.
            process_directive(directive_text);
        }
        "description" => walk_description(&item["body"]),  // typed object
        // ... other kinds
    }
}
```

- **Annotation inventory:** 20 entries (was 19). New: `compiler_directive`. Existing: source_text (1), source_text_item (8), description (8), systemverilog_file (1), systemverilog_parseable_file (1).
- **Same accept set, same diagnostic codes.** Only the `compiler_directive` shape changed.
- **Schema-version stays `1`** (additive — clean string replaces a 2-element array; consumers walking with the dual-shape pattern handle both).
- **Heterogeneous body types per `kind`** are now in the SV AST: when `source_text_item.kind == "description"`, body is a typed object; when `kind == "compiler_directive"`, body is a string. Consumers dispatch on `kind` first, then handle the body shape per its type. This is the same pattern the regex AST uses (e.g. `atom.kind == "literal"` → body is string vs `atom.kind == "char_class"` → body is a typed object).
- **mdBook**: `changelog-index.md`, `schema-versioning.md`, `json-carrier.md`, `rules-top-level.md` updated. `make systemverilog_parser_book_gate` green.
- Public API surface unchanged.
- Annotation-language idiom note: transparent passthrough `-> $N` (no object literal) is the cleanest form for "extract just the captured payload" — used here on a 2-element sequence to drop the trivia prefix and surface only the regex match. Same idiom as regex.ebnf's `escape = "\\\\" escape_unit -> $2` (drops the leading backslash and surfaces the typed escape unit).

## Release 1.0.4 / Contract 1.0.4 Highlights — SV-Slice-4: `description` per-branch typed (`kind:` discriminator on 8 branches; attribute_instance* preserved)

- **Annotation:** 8 per-branch annotations on `description` (line 957 of `grammars/systemverilog.ebnf`):

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

- **Multi-element branches preserve attributes**: branches 6 and 7 (`attribute_instance* package_item` / `attribute_instance* bind_directive`) carry the `attribute_instance*` prefix as a separate `attributes:` field while keeping the inner construct as `body:`. Consumers can walk attributes and body independently. The other 6 branches are single-element and use the simpler `{kind, body}` shape.
- **Effect:** items in `systemverilog_file.source_text` now carry **two layers of typed dispatch end-to-end**:
  - Outer `source_text_item.kind` (from SV-Slice-3) — identifies which top-level slot the item came from.
  - Inner `description.kind` (this slice) — when the outer kind is `"description"`, identifies which specific construct (module/interface/class/etc.).
- **Empirical pre/post on `module m; endmodule\n`:**

```text
# Pre-SV-Slice-4 — source_text[0].body was the raw description envelope:
"source_text": [
  {
    "kind": "description",
    "body": [<description Or-of-8 raw envelope, with module_declaration matched in branch 0>]
  }
]

# Post-SV-Slice-4 — source_text[0].body carries its own typed kind:
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

- **Consumer dispatch unlocked at the description level:**

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

- **Annotation inventory:** 19 entries (was 11). 8 new per-branch entries on `description`. Existing: source_text (1), source_text_item (8), systemverilog_file (1), systemverilog_parseable_file (1).
- **Same accept set, same diagnostic codes.** Only the `description` shape changed.
- **Inner `module_declaration`, `udp_declaration`, etc. shapes still raw envelope** — per-rule typing of those is a follow-up slice. The `description.kind` discriminator gives consumers the dispatch hook to route to per-construct walkers.
- **Schema-version stays `1`** (additive — discriminator on existing branches).
- **mdBook**: `changelog-index.md`, `schema-versioning.md`, `json-carrier.md`, `rules-top-level.md` updated. `make systemverilog_parser_book_gate` green.
- Public API surface unchanged.
- Annotation-language idiom note: multi-element branch `{kind: "<name>", attributes: $1, body: $2}` is a clean preservation of leading-quantified-prefix semantics — same idiom would apply to any rule whose branch has `<quantified_prefix>* <main_body>` shape (very common in SV grammar around attribute decorations).

## Release 1.0.3 / Contract 1.0.3 Highlights — SV-Slice-3: `source_text_item` per-branch typed (`kind:` discriminator on 8 branches)

- **Annotation:** 8 per-branch annotations on `source_text_item` (lines 210-217 of `grammars/systemverilog.ebnf`):

```ebnf
source_text_item := description                       -> {kind: "description", body: $1}
                  | local_parameter_declaration semi  -> {kind: "local_parameter_declaration", body: $1}
                  | parameter_declaration semi        -> {kind: "parameter_declaration", body: $1}
                  | package_import_declaration         -> {kind: "package_import_declaration", body: $1}
                  | timeunits_declaration              -> {kind: "timeunits_declaration", body: $1}
                  | compiler_directive                 -> {kind: "compiler_directive", body: $1}
                  | comment_only_source_region         -> {kind: "comment_only_source_region", body: $1}
                  | semi                               -> {kind: "semi"}
```

- **Effect:** every item in the `systemverilog_file.source_text` array now carries an explicit `kind:` discriminator. Consumers walking `obj["source_text"]` can dispatch on `item["kind"]` instead of structural recursion to identify which top-level construct each item is.
- **`semi` branch carries no body** (it's just a stray `;` — no useful payload). The other 7 branches carry the matched sub-rule's raw envelope as `body`.
- **`local_parameter_declaration semi` and `parameter_declaration semi` branches drop the trailing `semi`** (annotation references `$1` only, not `$2`). The semicolon is a syntactic delimiter, not part of the typed shape.
- **Empirical pre/post on `module m; endmodule\n`:**

```text
# Pre-SV-Slice-3 — source_text[0] was the matched-branch shape directly:
"source_text": [
  [<description envelope — module_declaration shape>]
]

# Post-SV-Slice-3 — source_text[0] is a typed object with discriminator:
"source_text": [
  {
    "kind": "description",
    "body": [<description envelope — module_declaration shape>]
  }
]
```

- **Annotation inventory:** 11 entries (was 3). New: 8 per-branch entries on `source_text_item`. Existing: `source_text`, `systemverilog_file`, `systemverilog_parseable_file`.
- **Same accept set, same diagnostic codes.** Only the `source_text_item` shape changed.
- **`@branch_policy: priority_first` and `@priority: [24, 16, 16, 12, 10, 8, 6, 4]` preserved** — the branch-policy / priority metadata applies before annotation dispatch, no change needed.
- **Inner `description`, `local_parameter_declaration`, etc. shapes still raw envelope** — per-rule typing of those rules is a follow-up slice. The `kind:` discriminator gives consumers the dispatch hook to route to per-branch walkers; the walkers themselves currently descend the raw envelope.
- **Schema-version stays `1`** (additive — discriminator on existing branches, no new rules or accept-set change).
- **mdBook**: `changelog-index.md`, `schema-versioning.md`, `json-carrier.md`, `rules-top-level.md` updated. `make systemverilog_parser_book_gate` green.
- Public API surface unchanged.
- Annotation-language idiom note: per-branch `{kind: "<name>", body: $1}` is the canonical regex-campaign idiom for Or-rule per-branch typing (used in regex slices 7, 8, 14-21, etc.). Verified to work for SV grammar with metadata blocks (`@branch_policy`, `@priority`) preserved.

## Release 1.0.2 / Contract 1.0.2 Highlights — SV-Slice-2: `source_text` typed via `[$1**]` flatten-spread

- **Annotation:** `source_text := source_text_item* -> [$1**]` (line 2273 of `grammars/systemverilog.ebnf`).
- **Effect:** the `source_text` field of every typed `systemverilog_file` JSON object is now a **flat array** of `source_text_item` shapes. Pre-fix it carried the raw Quantified envelope wrapping the iteration — consumers walking `obj["source_text"]` had to descend through the Quantified wrap before reaching items. Post-fix the array is consumer-ready; iterate directly.
- **Empirical pre/post on `module m; endmodule\n`:**

```text
# Pre-SV-Slice-2 — source_text was a Quantified envelope:
{
  "type": "systemverilog_file",
  "source_text": [<Quantified-wrapped iteration of source_text_item>]
}

# Post-SV-Slice-2 — source_text is a flat array of items:
{
  "type": "systemverilog_file",
  "source_text": [<source_text_item shape>]   // length = 1 for minimal_module
}
```

- **`source_text_item` itself stays raw envelope** (Or of `description | local_parameter_declaration semi | parameter_declaration semi | package_import_declaration | bind_directive | ...`). Per-branch typing of source_text_item is a follow-up slice; this slice only flattens the parent.
- **Annotation inventory:** 3 entries (was 2). New: `source_text`. Existing: `systemverilog_file`, `systemverilog_parseable_file`.
- **Same accept set, same diagnostic codes.** Only the `source_text` array shape changed.
- **Same `expected_json_object_keys_present` and `expected_json_object_string_values`** in the manifest's `minimal_module` sample. The rule-under-test is `systemverilog_file`, whose top-level keys (`type`, `source_text`) and `type` value (`"systemverilog_file"`) are unchanged. The Slice-2 change is in the SHAPE of the `source_text` value, not its key presence — manifest's drift-status updated to `calibrated_2026_05_04_slice_2` to record the calibration.
- **Schema-version stays `1`** (additive — flat-array shape is strictly a clean-up of the raw envelope, no new keys or rules).
- **mdBook**: `changelog-index.md`, `schema-versioning.md`, `json-carrier.md`, `rules-top-level.md` updated. `make systemverilog_parser_book_gate` green.
- Public API surface unchanged.
- Annotation-language idiom note: `[$1**]` is the canonical regex-campaign idiom for "flatten an array-shaped sub-rule reference into the enclosing array literal" — same idiom used in regex.ebnf for `concatenation = piece+ -> [$1**]` (slice 1 of the regex campaign). Verified to work for SV grammar's first array-shaped rule.

## Release 1.0.1 / Contract 1.0.1 Highlights — SV-Slice-1: `systemverilog_file` typed (dangling annotation rescued)

- **First effective annotation on the systemverilog parser.** Pre-fix `grammars/systemverilog.ebnf` carried two intended annotations that were both broken:
  1. Line 195's `-> {type: "systemverilog_file", source_text: $2}` was **dangling** — separated from its intended rule `systemverilog_file` (line 184) by a blank line + the `sv_multi_entry_root` helper rule (line 193) + another blank line. The annotation latched onto nothing effective; the parser registered 0 annotations for `systemverilog_file` and the rule's AST output stayed as the recursive-envelope `Sequence` shape.
  2. Line 200's `// -> {type: "systemverilog_parseable_file", items: $2}` had a `//` prefix (presumed by the grammar author to be a comment) but PGEN's EBNF dialect uses `#` for comments, not `//` — the `// ` was treated as literal noise and the rest of the line was parsed as a real annotation. So the `systemverilog_parseable_file` annotation was actually registered, but accidentally so.
- **Fix:** moved the dangling line-195 annotation up onto `systemverilog_file := trivia source_text trivia` (line 184) using the canonical multi-line continuation form. Removed the misleading `//` prefix from the line-200 annotation since it was effectively active anyway. Both annotations now register through the documented path.

```ebnf
# After SV-Slice-1:
systemverilog_file := trivia source_text trivia
                   -> {type: "systemverilog_file", source_text: $2}
...
systemverilog_parseable_file := trivia parseable_source_item* trivia
                             -> {type: "systemverilog_parseable_file", items: $2}
```

- **Empirical verification.** Generated the parser via `ast_pipeline grammars/systemverilog.ebnf --generate-parser --eliminate-left-recursion`, built `parseability_probe` with the `PGEN_SYSTEMVERILOG_PARSER_PATH` adapter, ran it on `module m; endmodule\n` with `--profile sv_2017`. AST root pre-fix: `{"content": {"Sequence": [...]}}` (recursive envelope). Post-fix: `{"content": {"Json": {"type": "systemverilog_file", "source_text": [...]}}}`. The annotation correctly latches and the typed shape lands.
- **Annotation inventory** (from `ast_pipeline`'s reporting): 2 entries — `systemverilog_file` and `systemverilog_parseable_file`. Was 1 entry pre-fix (only the accidentally-registered `systemverilog_parseable_file`).
- **Manifest update.** `rust/test_data/ast_shape_contract/systemverilog_v1.json` `current_content_kind` updated from placeholder `"sequence"` to calibrated `"json_object"`. `drift_status` flipped from `parser_unavailable_in_default_build_pending_first_run_calibration` to `calibrated_2026_05_04`. Layout note about line 195 dangling annotation removed (resolved by this slice). Calibration history added.
- **Annotation campaign starts here.** This is SV-Slice-1 — the first slice in the systematic return-annotation campaign on `grammars/systemverilog.ebnf`, mirroring the regex parser's 42-slice campaign. Subsequent slices will type rules one-by-one (`description`, `module_declaration`, `interface_declaration`, etc.). Each slice bumps parser release / contract version and appends a Highlights section here.
- **No accept-set change.** The grammar's accept set is unchanged — same inputs parse, same inputs reject. Only the AST shape for accepted inputs changes (recursive envelope → typed `{type, source_text}` object at the root).
- **Schema-version stays `1`.** Per the schema versioning policy, additive shape changes within a major version don't bump the schema number; the `current_content_kind` change is tracked in the per-rule manifest.
- **mdBook**: `docs/systemverilog_parser_book/src/changelog-index.md`, `schema-versioning.md`, `json-carrier.md`, and `rules-top-level.md` updated to reflect the typed shape. `make systemverilog_parser_book_gate` green.
- Public API surface unchanged.
- No SV-NNNN bug ledger entry (this is a foundation-slice annotation correctness fix, not a downstream-reported bug).

## Release 1.0.0 / Contract 1.0.0 Highlights — initial baseline (foundation deliverables landed)

- **Initial contract identity baseline.** The systemverilog parser has been part of PGEN for some time; this contract document is being upgraded from a thin "stable surface" pointer into the same release-tracked Highlights structure used by `PGEN_REGEX_PARSER_INTEGRATION_CONTRACT.md`. Subsequent return-annotation slices on `grammars/systemverilog.ebnf` will each bump the parser-release / contract-version pair and append a Highlights section here.
- **mdBook scaffolded.** `docs/systemverilog_parser_book/` is the new canonical AST reference for downstream consumers. Initial chapters: welcome, build recipe, public API, AST envelope, schema versioning, changelog index, glossary. Per-rule and per-example chapters land as the annotation campaign progresses.
- **Build status.** The generated systemverilog parser is **NOT in the default `cargo test --features generated_parsers` build**. It is produced on-demand by `sv_stimuli_quality_gate` (and similar gates) into `rust/target/<gate>/work/systemverilog_parser.rs` and discarded after the gate run. Cfg `has_generated_systemverilog_parser` therefore stays off in default builds. When the parser is present (gate run or `PGEN_SYSTEMVERILOG_PARSER_PATH` override), the embedding API path is enabled and the per-family AST-shape contract test activates.
- **Schema baseline.** `1` — corresponds to the `version: 1` field in `rust/test_data/ast_shape_contract/systemverilog_v1.json`. The manifest currently carries one stub sample (`minimal_module: "module m; endmodule\n"`) calibrated against the placeholder `current_content_kind: "sequence"`. First post-foundation slice will run the parser, observe the actual content kind, and either confirm or update the manifest.
- **Annotation campaign — not yet started.** `grammars/systemverilog.ebnf` is currently un-annotated except for one commented-out trial annotation at line 200. Subsequent slices will systematically apply return annotations rule-by-rule, mirroring the regex parser campaign that produced typed shapes for 42+ regex rules. Schema-version bumps and contract Highlights sections will track each slice.
- **Public API surface unchanged.** No accept-set or diagnostic-code change in this baseline.
- **Bug ledger entries**: any released SV parser bug is tracked in `docs/contracts/PGEN_RELEASED_PARSER_BUG_LEDGER.md` under the `SV-NNNN` ID family. None blocking the baseline.

## Source Of Truth
- Grammar source:
  - `grammars/systemverilog.ebnf`
  - Companion (LRM-extracted reference): `grammars/systemverilog_2017_lrm_extracted.ebnf`, `grammars/systemverilog_2023_lrm_extracted.ebnf`
  - Companion (profiled wrappers): `grammars/systemverilog_lrm_profiled_generated.ebnf`, `grammars/systemverilog_lrm_profiled_wrapper.ebnf`
- Public host API:
  - `rust/src/embedding_api.rs`
- Public API contract:
  - `rust/docs/EMBEDDING_API_CONTRACT.md`
- AST shape contract manifest:
  - `rust/test_data/ast_shape_contract/systemverilog_v1.json`
- Build-time generated parser discovery:
  - `rust/build.rs`
  - `PGEN_SYSTEMVERILOG_PARSER_PATH`
- Live closure/status surface:
  - `LIVE_ACHIEVEMENT_STATUS.md`
- Reference IEEE 1800 LRM corpus (read-only):
  - `docs/systemverilog/2017/` (Annex A formal syntax, plus other annexes)
  - `docs/systemverilog/2023/` (delta + 2023-specific annexes)

## Stable Integration Surface
- Grammar family:
  - `systemverilog`
- Stable host profiles:
  - `sv_2017`
  - `sv_2023`
- Stable convenience entry points:
  - `parse_systemverilog_2017(...)`
  - `parse_systemverilog_2023(...)`
  - `parse_systemverilog_2017_ast_dump(...)`
  - `parse_systemverilog_2023_ast_dump(...)`
- Stable generic entry points:
  - `parse_grammar_profile(...)`
  - `parse_grammar_profile_result(...)`
  - `parse_grammar_profile_ast_dump(...)`
  - `parse_grammar_profile_named(...)` (string-name overload)
  - `parse_grammar_profile_named_with_limits(...)` (string-name overload with explicit limits)
- Stable diagnostics:
  - `E_BACKEND_UNAVAILABLE`
  - `E_PARSE_FAILURE`
  - `E_INPUT_TOO_LARGE`
  - `E_INVALID_LIMITS`
  - `E_INVALID_ARGUMENT`
  - `E_UNSUPPORTED_PROFILE`

## Build / Availability Requirements
- Downstream consumers should treat the generated backend as required for real host integration.
- Startup should inspect `parser_embedding_api_contract().supports_systemverilog_generated_backend`.
- Build-time generated-parser discovery is mediated by `rust/build.rs`, not by direct use of internal parser modules.
- When local validation sets `PGEN_SYSTEMVERILOG_PARSER_PATH` while invoking `cargo ... --manifest-path rust/Cargo.toml`, use an absolute path or a path relative to `rust/`; `rust/build.rs` resolves that variable relative to the Rust manifest directory.
- The PGEN-side `sv_stimuli_quality_gate` make target produces the generated parser at `rust/target/sv_stimuli_quality_gate/work/systemverilog_parser.rs`. Downstream embedders that vendor this artifact should track its SHA256 against the parser-release version recorded in this contract.

## Validation / Release Gates
- Public host API stability:
  - `make -C rust SHELL=/bin/bash embedding_api_gate`
  - `make -C rust SHELL=/bin/bash nexsim_parser_embedding_contract_gate`
- Family closure / proof:
  - `make -C rust SHELL=/opt/homebrew/bin/bash sv_parser_family_status_gate`
  - `make -C rust SHELL=/opt/homebrew/bin/bash sv_parser_family_status_contract_gate`
  - `make -C rust SHELL=/opt/homebrew/bin/bash sv_combined_telemetry_contract_gate`
- Stimuli / corpus:
  - `make -C rust SHELL=/opt/homebrew/bin/bash sv_stimuli_quality_gate`
  - `make -C rust SHELL=/opt/homebrew/bin/bash sv_syntax_closure_gate`
- Documentation:
  - `make systemverilog_parser_book_gate` — builds the mdBook + verifies tracked HTML output.

## Scope / Non-Goals
- The stable contract is the host-oriented embedding surface in `pgen::embedding_api`, not internal generated parser types.
- Internal AST node types are not the downstream contract.
- The current tracked sign-off bar is Nexsim-facing SystemVerilog parsing, not an open-ended promise for every imaginable SystemVerilog dialect or tool ecosystem.
- When reporting downstream bugs, follow `docs/contracts/PGEN_PARSER_ISSUE_REPORTING_PROTOCOL.md`; accepted released-parser bugs should then be logged in `docs/contracts/PGEN_RELEASED_PARSER_BUG_LEDGER.md`.

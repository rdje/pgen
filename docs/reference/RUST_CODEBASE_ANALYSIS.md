# docs/reference/RUST_CODEBASE_ANALYSIS.md

Last updated: 2026-04-18

## Purpose
Live architecture and state assessment for the Rust codebase.

This document exists to preserve the high-level understanding needed to steer implementation, review future changes, and ramp up a new session without having to rediscover the whole Rust stack from scratch.

This is a live document, not an archival write-up. It should be amended whenever the Rust architecture, major risks, public integration surfaces, or codebase shape materially change.

## Session-Start Maintenance Rule
- Review this file at the start of any new session that materially touches the Rust codebase.
- Refresh it when the current codebase no longer matches this snapshot in a meaningful way.
- Prefer amending this file over scattering duplicate architectural assessments into ad hoc chat history.
- Keep historical detail in `CHANGES.md` / `DEVELOPMENT_NOTES.md`; keep this file focused on current structure, current risks, and the current steering picture.

## Scope Of This Assessment
- This is a source-structure and architecture assessment of the maintained Rust-first platform.
- It covers the main Rust crate, the generated-parser integration layer, the major Rust-owned binaries, and the Rust-owned gate/build ecosystem around them.
- It is not a claim that every parser family is closed.
- It is not a replacement for the live closure tracker in `LIVE_ACHIEVEMENT_STATUS.md`.
- It should be read alongside the roadmap priority rule:
  - active parser-family closure work is now centered on the remaining main-`systemverilog` debt and still outranks deferred maintainability refactors.
  - `vhdl`, `systemverilog_preprocessor`, and `regex` have reached their current published closure bars and should be treated as no-regression proof baselines unless their contracts are intentionally widened.
  - downstream regex hardening should now be treated as maintained PCRE2-compatibility contract precision work, not as a reason to reopen the `regex` family row by default.
  - RGX's downstream review now treats regex handoff `1.1.29` / integration contract `1.1.31` as the current public handoff. That line carries the accepted-tree transport fixes, named recursion-condition support, returned-capture subroutine forms, strict PCRE2 VERSION conditionals, quoted literal handling both outside and inside character classes, quoted class range endpoints, short Unicode property escapes, bounded variable-length lookbehind, Unicode capture names, orphan class `\E` as a zero-width scanner marker, dedicated `\C` single-code-unit escape transport, callout-prefixed conditional assertions, PCRE2 POSIX word-boundary aliases, DEFINE-in-lookbehind zero-width handling, UTF width start-option aliases, scan-substring forward-reference validation, plain class `\N` rejection, nonliteral class range endpoint rejection, decoded escaped class-range endpoint ordering including bare octal `\NNN`, POSIX-class fallback/adapter clarifications, MARK/PRUNE/SKIP/THEN payload generalization, braced padded `\g{...}` / `\k{...}` references, generated-host depth resilience for legal deep PCRE2 inputs, and the 2026-04-14 through 2026-04-18 source-derived PCRE2 audit slices.
  - `ci_workflow_local_gate` now also audits against that current regex handoff (`1.1.29` / `1.1.31`) and the current user-guide regex parseability total (`5911`), so filtered local workflow replays no longer fail on stale regex-public-surface literals.
  - `ci_workflow_local_gate` is now also self-pruning on success: it still exports a tracked-only `run.*` tree under `rust/target/ci_workflow_local_gate`, but successful runs delete that scratch directory by default while failed runs retain it for triage; `PGEN_CI_WORKFLOW_LOCAL_KEEP_RUNS=1` preserves successful runs when a bundle needs to be inspected or archived.
  - PCRE2 compatibility work has a clear source-of-truth workflow:
    - read `pcre2syntax(3)` and `pcre2pattern(3)` for documented intent
    - cross-reference `src/pcre2_compile.c` for exact edge cases
    - validate against upstream `testdata/testinput*` plus expected outputs as the executable oracle
  - remaining regex caveats are now deliberate scope-widening questions around full PCRE2 parity, stronger JS/Lua shielding, host-language wrapper parsing, and any future AST-semantic stability promises beyond the current JSON schema contract.
  - current Phase S `rtl_frontend` generated-contract work is proof tightening rather than closure promotion:
    - `expected_rule_texts` in `rtl_frontend_generated_contract_probe` remains the exact full-vector retained-text assertion
    - `required_rule_texts` remains for subset retained-text assertions over recursive rules such as `conditional_expr`, `additive_expr`, `shift_expr`, and `signal_reference`, where the salient span should be proven without freezing every incidental scalar expression subtree
    - the generated contract now tightens `always_ff_well_formed`, `always_star_scalar_if_else_block`, `always_latch_scalar_nonblocking_block`, and `always_latch_unknown_body_identifier_parse_surface` so scalar procedural lanes prove retained port-shell context and selected `signal_reference` evidence around already locked event-control, event-edge, procedural-block, assignment-operator, assignment-target, and procedural keyword spans
    - the generated contract now tightens `procedural_and_dataflow_concat_member_paths`, `procedural_and_dataflow_ternary_binary_exprs`, `rich_assignment_targets_ternary_exprs`, `procedural_concatenated_assignment_target_ternary_exprs`, and `continuous_ranged_member_assignment_target_ternary_exprs` so mixed procedural/dataflow lanes prove retained parameter, port, inline struct, struct-field, net-declaration, packed-range, and unpacked-dimension context where applicable around already locked `always_comb`, continuous-assignment, assignment-target, concatenation, ranged-reference, and ternary/binary expression spans
    - the generated contract now tightens `always_star_rich_assignment_targets` and `always_latch_rich_assignment_targets` so rich plain `always @(*)` / `always_latch` lanes prove retained parameter, port, inline struct, struct-field, net-declaration, packed-range, and unpacked-dimension context around already locked procedural-block, assignment-operator, assignment-target, continuous-assign, and concatenation-expression spans
    - the generated contract now tightens `always_comb_struct_member_concatenation_target` so the isolated `always_comb` struct-member concatenated-target lane proves retained parameter, port, inline struct, struct-field, net-declaration, packed-range, and unpacked-dimension context around already locked `always_comb`, procedural-block, assignment-operator, and assignment-target spans
    - the generated contract now tightens `always_ff_rich_nonblocking_assignment_targets`, `always_ff_struct_member_bitselect_nonblocking_target`, `always_ff_struct_member_concatenation_value`, and `always_ff_unknown_event_identifier_parse_surface` so `always_ff` lanes prove retained parameter, port, inline struct, struct-field, net-declaration, packed-range, unpacked-dimension, and syntax-only unknown event identifier context around already locked event-control, procedural-block, nonblocking assignment, assignment-target, concatenation, ranged-reference, and expression spans
    - the generated contract now tightens `scalar_named_parameter_override_and_named_ports` and `parameterized_instance_array_with_named_ports` so hierarchy lanes prove retained child/top parameter declaration context, parameter group context, and scalar packed ranges around already locked named overrides, named ports, symbolic instance-array ranges, expression evidence, and signal references
    - the generated contract now tightens `unpacked_array_struct_member_actual` and `unpacked_array_element_actual` so unpacked-array actual lanes prove retained parameter context around already locked array/member actual text
    - the generated contract now tightens `module_local_parameter_and_localparam_items` so module-local parameter/localparam syntax proves retained parameter-sequence vectors, `parameter` / `localparam` keyword text, port-list context, and selected recursive expression evidence
    - the generated contract now tightens `labeled_always_comb_block` and `labeled_always_comb_parameter_exprs_and_packed_multi_nets` so labeled procedural samples prove retained `begin` / `if` / `else` keyword text, declaration/range/port context, and selected recursive expression evidence around the existing procedural-block locks
    - the generated contract now tightens `generate_for_symbolic_limit_nonunit_stride` so symbolic non-unit generate-for syntax proves retained `generate` / `for` / `genvar` keyword text plus the `parameter LIMIT = 5` parameter sequence/group around the existing loop-expression evidence
    - the generated contract now tightens `generate_if_with_dataflow_and_named_instantiation`, `generate_if_else_with_dataflow`, and `generate_if_else_with_local_net_declarations` so generate/dataflow lanes prove local net declarations, local packed range, parameter-sequence, port-list, and ternary-expression retained context around already locked generate structures
    - the generated contract now tightens continuous struct-member assignment lanes again so their `struct_union_field` vectors exact-lock `logic [7:0] data;` and `logic valid;` across bit-select, unknown-member, concatenated-target, and concatenation-value samples
    - the generated contract now tightens `continuous_struct_member_bitselect_assignment_target`, `continuous_unknown_struct_member_target_parse_surface`, `continuous_unknown_struct_member_value_parse_surface`, `continuous_unknown_struct_member_concatenated_target_parse_surface`, `continuous_struct_member_concatenation_assignment_target`, and `continuous_struct_member_concatenation_value` so continuous struct-member assignment lanes prove inline struct declaration, input-port context, and `BIT` parameter context around assignment targets and values
    - the generated contract now tightens `named_port_bitselect_and_concat_actuals`, `named_port_member_bitselect_and_repeat_actuals`, and `named_port_actual_ternary_member_paths` so named-port actual expression lanes prove declaration and parameter context around bit-select, concatenation, repetition, and ternary member-path actuals
    - the generated contract now tightens `ordered_parameter_and_port_actual_repetition`, `ordered_actuals_repeat_concat_member_ranges`, and `named_port_actuals_repeat_member_ranges` so repeat-concat actual lanes prove retained parameter, port, net, packed-range, unpacked-dimension, and struct-field context around already locked ordered/named actual expressions
    - the generated contract now tightens `ordered_parameter_override_ternary_binary_expr`, `named_parameter_override_repeat_expr`, and `named_parameter_override_ternary_binary_expr` so parameter-override expression lanes prove retained child/top parameter declarations, port shells, and packed ranges around already locked override expressions
    - the generated contract now tightens `unpacked_array_struct_member_actual`, `unpacked_array_element_actual`, and `unpacked_array_struct_member_bitselect_actual` so indexed unpacked-array actual lanes prove richer member-path, inline struct body, array declaration, packed/unpacked dimension, and module-instantiation retained text while leaving array-indexing and parameter semantics in elaboration
    - the generated contract now tightens `unindexed_unpacked_array_struct_member_actual_parse_surface` and `unknown_inline_struct_member_actual_parse_surface` so inline struct-member actual lanes subset-lock the inline struct body, with `cfgs.data` plus `[0:1]` proven on the unindexed unpacked-array lane and `struct packed { ... } cfg;` proven on the unknown inline-member lane
    - the generated contract now tightens `named_port_union_member_actual` and `named_port_unknown_union_member_actual_parse_surface` so inline union-member actual lanes exact-lock the union body and `payload` net declaration, with the known-member lane also exact-locking the successful `payload.data` signal-reference path
    - the generated contract now tightens `packed_union_width_mismatch_parse_surface` and `builtin_integral_packed_union_width_mismatch_parse_surface` so inline and builtin-integral packed-union mismatch lanes exact-lock full module declarations, simple output-port shells, union bodies, datatype/range or builtin keyword spans, and final net declarations while leaving semantic width evaluation in elaboration
    - the generated contract now tightens `typedef_backed_struct_member_actual`, `file_scope_typedef_backed_struct_member_actual`, `package_wildcard_import_typedef_backed_struct_member_actual`, `package_named_import_typedef_backed_struct_member_actual`, `header_named_import_typedef_backed_struct_member_actual`, `unknown_typedef_backed_struct_member_actual_parse_surface`, and `typedef_backed_packed_union_width_mismatch_parse_surface` so typedef-backed struct-member actual lanes exact-lock typedef declarations and struct bodies, while the typedef-backed packed-union mismatch lane exact-locks typedef declaration, union body, and packed ranges
    - the generated contract now tightens `inline_struct_typed_net_declaration`, `inline_union_typed_net_declaration`, `typedef_union_named_net_declaration`, and `typedef_enum_named_net_declaration` so inline aggregate typed-net and typedef-backed aggregate named-net lanes exact-lock full module declarations, port shells, builtin datatype vectors, packed ranges, aggregate type bodies, typedef declarations, named data-type uses, and final net declarations where applicable
    - the generated contract now tightens `typedef_struct_named_net_declaration`, `file_scope_typedef_struct_named_net`, `file_scope_typedef_struct_port_and_net_multimodule`, and `package_typedef_struct_port_and_wildcard_net_multimodule` so local/file-scope/multi-module/package struct typedef lanes exact-lock typedef declarations, struct bodies, builtin datatype vectors, packed ranges, ANSI port shells, full module declarations, and `cfg_t cfg;` net declarations where applicable
    - the generated contract now tightens `package_qualified_typedef_struct_port`, `package_wildcard_import_typedef_struct_named_net`, and `package_named_import_typedef_struct_named_net` so package/import struct typedef lanes exact-lock compact typedef declarations, struct bodies, builtin datatype vectors, packed ranges, package-qualified/named data-type uses, port shells, and imported `cfg_t cfg;` net declarations where applicable
    - the generated contract now tightens the header-imported typedef port family so `header_imported_enum_typedef_port`, `header_imported_union_typedef_port`, and `header_imported_struct_typedef_port` exact-lock imported named-type uses, ANSI port-list/group shells, compact typedef/type bodies, relevant builtin datatype vectors, and packed ranges
    - the generated contract now tightens `multiple_empty_modules_without_port_lists` so the no-port multi-module lane exact-locks both `module` keyword spans, module identifiers `first` / `second`, and both `endmodule` keyword spans alongside the existing full declaration locks and forbidden-rule checks
    - the generated contract now exact-locks statement-level `parameter_declaration_statement` retained text for module-local parameter/localparam items and package-backed constant-flow samples, while header parameter-port declarations remain covered by `parameter_declaration_head` / `parameter_declaration_tail`
    - the generated contract now exact-locks aggregate typed `net_declaration` retained text for inline enum/union declarations and typedef-backed struct/enum/union named net uses
    - the generated contract now retains the integrated handwritten-baseline arithmetic/procedural/generate sample `arithmetic_integrated_generate_and_procedural_flow`, combining dependent parameters, ANSI port ranges, module-body parameter/localparam statements, packed nets, continuous ternary dataflow, labeled `always_comb`, generate `if/else`, and generate `for` with exact retained text for compact structural rules and subset retained text for recursive expression spans
    - the generated contract now retains the integrated child/generate hierarchy sample `integrated_child_parameter_generate_instances`, combining direct, generate-if, and generate-for child instantiations with named parameter overrides and named port connections while explicitly keeping elaboration semantics out of the generated-parser proof claim
    - the generated contract now retains `generate_for_symbolic_limit_nonunit_stride`, proving symbolic generate-for bounds plus non-unit step syntax (`i < LIMIT`, `i + 2`) without claiming generated semantic unrolling
    - the generated contract now tightens `parameterized_instance_array_with_named_ports` so symbolic instance-array range text (`LANES-1`) and both `LANES` signal-reference uses are proven alongside parameter/port declaration context and exact hierarchy spans
    - the generated contract now tightens `unpacked_array_ports_and_nets` so packed/unpacked dimensional declaration text, the full port-list/port-group surface, and repeated `DEPTH` references are proven for the foundational unpacked-array declaration lane
    - the generated contract now tightens `builtin_integral_atom_typed_net_declarations` so builtin datatype text, builtin keyword text, and the simple output-port shell are proven for the `byte` / `shortint` / `longint` declaration lane
    - the generated contract now tightens `inline_enum_logic_typed_net_declaration` so duplicate `logic` datatype spans, enum base/range text, full enum body text, and the output-port shell are proven for the inline enum declaration lane
    - the generated contract now tightens `inline_enum_byte_base_typed_net_declaration` so `logic` / `byte` datatype spans, `byte` keyword text, full enum body text, and the output-port shell are proven for the byte-base inline enum declaration lane
    - the generated contract now exact-locks generate `if/else` structural retained text for the existing dataflow and local-net branch samples by requiring `generate_region`, `generate_if`, and branch-level `generate_body` spans
    - the generated contract now exact-locks compact hierarchy retained text for package-backed constant-flow `module_instantiation`, unpacked-array struct-member `instance_item`, and generate-contained `module_instantiation` / `instance_item` spans
    - the generated contract now exact-locks the single-branch generate-if named-instantiation sample by requiring `module_instantiation` and exact `generate_region`, `generate_if`, `generate_body`, full `module_instantiation`, and `instance_item` retained text
    - the generated contract now exact-locks generate `for` structural retained text for the existing local-net and named-instantiation/dataflow loop samples by requiring `generate_region` and branch-level `generate_body` spans, with exact hierarchy locks for the looped instantiation sample
    - the generated contract now also locks the plain unpacked-array element named-port actual `child u_child (.a(banks[IDX]), .y(y));` plus a nearby malformed `banks[]` rejection, matching the handwritten baseline's simpler unpacked-array element actual lane without changing the live status label
    - the generated-contract gate now also runs handwritten `rtl_frontend::parse_design` and ratcheted optional `expected_elaboration` replays over the same `120`-sample manifest; the manifest still has zero active `expected_handwritten_parse_ok` annotations after the handwritten parser was aligned to reject the two mixed named/ordered instance-list cases, reject scalar `always_ff` blocking assignments at parse time, and accept selector/concat-rich runtime expression text at the parse boundary, and it now retains at least `40` semantic replay samples (`30` accepts / `10` rejects), `7` child-path samples, `15` top-parameter checks, `11` child-parameter checks, and `49` child-port-binding checks for procedural blocks, hierarchy/instance arrays, package constants, aggregate member actuals, union-width checks, unknown event/member diagnostics, unindexed unpacked-array members, unknown parent actual identifiers, scalar wildcard expansion, and richer signal/member, bit/part-select, concat, repeat, expression, and expression-text actual shapes
    - the live `rtl_frontend` row remains `In Progress` until broader generated grammar exhaustiveness, semantic elaboration parity, and parser-stack closure are achieved
  - downstream regex hardening under `regex_corpus_bundle/` now also has two distinct external-corpus roles:
    - `regex_pcre2_textsafe_corpus_gate` for accepted-syntax widening
    - `regex_pcre2_compile_oracle_gate` for compile-truth comparison against pinned PCRE2 source truth
  - the 2026-04-14/2026-04-15 regex audits deliberately split grammar syntax from compile-contract enforcement:
    - `grammars/regex.ebnf` now models more PCRE2 source-derived syntax such as short Unicode property escapes, quoted class literals, quoted class range endpoints, escaped quoted-literal body characters, bounded variable-length lookbehind surface, Unicode capture-name characters, orphan class `\E` as `stray_class_end_quote`, `\C` as `single_byte_escape`, callout-prefixed conditional assertions, `\K`, one-digit and whitespace-braced hex/octal escapes, string callouts, `(*atomic:...)`, non-atomic symbolic and alpha lookarounds, scan-substring groups, script-run groups, strict inline modifier letters, strict no-whitespace `VERSION>=...` / `VERSION=...` conditionals, and comma-only `{,}` as literal text rather than a counted quantifier
    - `rust/src/regex_compile_validation.rs` now mirrors PCRE2 compile-time checks that are not clean grammar productions: unbounded lookbehind rejection, malformed named references, PCRE2 capture-name byte/first-character limits, malformed short Unicode properties, empty quoted or orphan-quote class regions, plain class `\N` rejection, shorthand/property class range endpoint rejection, numeric callout range, start-option and verb-name tables, non-ACCEPT verb quantification, `\K` in lookaround, forbidden character-class escapes, POSIX class-name validation, scan-substring capture-list existence, unsupported default escapes, and `(?R1)` rejection
    - the maintained `regex_pcre2_compile_oracle_gate` baseline is ratcheted to the measured `pcre2-10.47` result of `1843` matches, `352` mismatches, `307` false accepts, and `45` false rejects over `2195` normalized compile-oracle cases
  - the current HDL closure tactic has now shifted away from broad hint sweeps:
    - when a family is down to a small stubborn replay/rejection set, prefer the new branch-level triage tooling over more blanket grammar/sample nudges
    - the retained `systemverilog_preprocessor` orphan-closer seam is now solved on the focused and aggregate lanes by a stimuli-only generator fix: repeated `pp_item` expansions are forced onto separate lines when the previous generated item did not already end with newline
    - current retained preprocessor truth is now:
      - focused quality `33/33/0/0` with `final_targets=0`
      - aggregate contract `parseability_rejected_total=0`, `parseability_parser_rejections_total=0`, `counterexamples_captured_total=0`
      - zero-plausible-gap proof is now explicit and green through `sv_preprocessor_zero_plausible_gap_proof_gate`
      - formal exhaustive closure is now green through `sv_preprocessor_formal_exhaustive_closure_gate`
    - main-SV replay-shadow triage now also has one important reporting guard:
      - primary-entry parser-rejection counts remain the tracked debt surface
      - alternate-entry probe failures are still useful telemetry, but they should not be mixed into the main parser-debt counterexample set
      - the retained Rust path now carries entry-context metadata through target-drive validation so the replay-shadow counterexamples can stay primary-entry-only
    - treat `systemverilog_preprocessor` as a closed no-regression baseline unless its published contract is intentionally widened; active HDL closure work should now bias toward the remaining `systemverilog` main-parser debt and the broader Phase `S` build-out

## Rust-Adjacent Cargo Surface
- Main product crate
  - `rust/Cargo.toml`
  - This is the primary crate for parser generation, runtime parser exposure, embeddings, operational binaries, and the main proof-facing Rust surfaces.
- Active companion crates
  - `rtl_const_expr/Cargo.toml`
    - standalone constant-expression crate used by the RTL/frontend track
  - `rtl_frontend/Cargo.toml`
    - standalone frontend crate layered on top of `rtl_const_expr`
  - These are not part of the main `rust/` crate, but they are still part of the project’s live Rust implementation story.
- Auxiliary/peripheral Rust crates
  - `tools/generators/Cargo.toml`
    - auxiliary generator-tool surface, not the main runtime/proof spine
  - `test_parsers/json_test/Cargo.toml`
    - narrow test/example parser surface, not a central product architecture owner
  - root `Cargo.toml`
    - repository-local Rust surface, but not the canonical source for the main parser-generation architecture described here

Operational rule:
- When a task says “the Rust codebase,” default to `rust/` first, then pull in `rtl_const_expr/` and `rtl_frontend/` when the task touches Phase S or frontend/constant-expression ownership.
- Do not let peripheral Cargo manifests distract from the main architecture unless the task is explicitly about those support crates.

## Rust-Facing Repo Doc Crosswalk
- `README.md`
  - Use for:
    - repo-level orientation
    - current doc map
    - first-hop navigation into the maintained project surfaces
    - high-level entrypoints into aggregate annotation proof surfaces
- `QUICKSTART_AI_ONBOARDING.md`
  - Use for:
    - session-start ramp-up expectations
    - which current docs a new session should read first
- `PGEN_USER_GUIDE.md`
  - Use for:
    - user-facing workflow framing
    - understanding which operational surfaces are meant to be consumed externally
    - operator-facing map of aggregate annotation / semantic / return local gates
- `docs/contracts/PGEN_PARSER_INTEGRATION_CONTRACTS.md` and `docs/contracts/PGEN_*_PARSER_INTEGRATION_CONTRACT.md`
  - Use for:
    - downstream parser handoff
    - family-specific integration promises and caveats
    - build/availability checks a host project should perform before relying on a parser family
- `docs/contracts/PGEN_PARSER_ISSUE_REPORTING_PROTOCOL.md`
  - Use for:
    - downstream parser bug-report bundles
    - exact repro artifacts PGEN expects back from host projects
    - trace / AST-dump / structured-outcome capture procedure for integration bugs
- `docs/contracts/PGEN_RELEASED_PARSER_BUG_LEDGER.md`
  - Use for:
    - release-support continuity
    - tracking accepted downstream parser bugs through fix and release
    - linking real integration bugs back to proof and regression artifacts
- `LIVE_ACHIEVEMENT_STATUS.md`
  - Use for:
    - current closure/status truth
    - distinguishing architecture work from “family actually closed” claims
- `docs/reference/PGEN_SOTA_IMPLEMENTATION_ROADMAP.md`
  - Use for:
    - current project doctrine
    - phase/closure expectations
    - deferred or still-open engineering directions
- `docs/reference/PGEN_ANNOTATION_NORMATIVE_SPEC.md`
  - Use for:
    - return/semantic annotation meaning
    - typed-annotation intent that should constrain Rust-side parser/generator/runtime changes
    - annotation proof obligations and gate targets behind aggregate annotation claims
- `docs/reference/PGEN_SEMANTIC_STEERING_CONTROL_MATRIX.md`
  - Use for:
    - semantic-steering behavior expectations
    - understanding whether a semantic-runtime or stimuli/generator change still matches repo policy
- `COMMIT.md`
  - Use for:
    - workflow and continuity expectations
    - knowing when `docs/reference/RUST_CODEBASE_ANALYSIS.md` itself should be refreshed as part of a task
- `CHANGES.md`, `DEVELOPMENT_NOTES.md`, `MEMORY.md`
  - Use for:
    - short-term continuity
    - recent implementation history
    - crash/handoff recovery
- `regex_corpus_bundle/README.md` and `regex_corpus_bundle/docs/regex_corpus_plan.md`
  - Use for:
    - PCRE2-first external regex hardening doctrine
    - corpus acquisition / normalization layout
    - the maintained difference between text-safe widening and compile-oracle measurement lanes
- `rust/docs/COVERAGE_GAP_TRIAGE.md`
  - Use for:
    - branch-level closure debugging on the remaining HDL seams
    - joining gap-report, coverage, and grammar-AST artifacts into one readable triage surface
    - avoiding more blind `@sample` / lexical hint sweeps when a family is down to a small stubborn target set

Operational rule:
- If a Rust task raises a question about doctrine, status, semantics, or workflow, reach for the matching repo doc instead of trying to infer everything from code alone.
- The code explains implementation; these docs explain whether the current implementation is aligned with project intent.

## Executive Summary
- PGEN's Rust codebase is not just a parser implementation. It is a parser-generation and parser-proof platform.
- The center of gravity is the AST pipeline in `rust/src/ast_pipeline/`, especially:
  - `mod.rs`
  - `ast_based_generator.rs`
  - `stimuli_generator.rs`
  - `annotation_validator.rs`
  - `semantic_runtime.rs`
- The generated parser path, stimuli/coverage closure path, semantic-steering path, and proof/gate path are deeply integrated rather than loosely bolted together.
- The strongest quality of the Rust codebase is coherence around determinism, observability, and machine-checkable proof.
- The main architectural risk is concentration of complexity in a few very large modules and a few repeated adapter seams.
- The newest downstream-trust expansion for `regex` is no longer just synthetic or narrative: `regex_corpus_bundle/` now feeds a maintained compile-oracle lane through `normalize_pcre2_compile_oracle.py`, `regex_corpus_probe`, `regex_pcre2_compile_oracle_gate.sh`, and a dedicated post-parse compile-contract layer in `rust/src/regex_compile_validation.rs`; the generated regex host path also retains a larger bounded worker stack plus widened generated recursion guard for legal deep PCRE2 conformance inputs.

## Snapshot Metrics
- Rust maintained source surface inspected in this pass: about `44k` lines.
- Biggest source hotspots:
  - `rust/src/ast_pipeline/stimuli_generator.rs`: `7907` lines
  - `rust/src/ast_pipeline/ast_based_generator.rs`: `7046` lines
  - `rust/src/ast_pipeline/annotation_validator.rs`: `4014` lines
  - `rust/src/main.rs`: `3183` lines
  - `rust/src/ast_pipeline/mod.rs`: `2920` lines
  - `rust/src/ast_pipeline/semantic_runtime.rs`: `2522` lines
  - `rust/src/ast_pipeline/unified_return_ast.rs`: `2625` lines
- Rust-owned shell gate scripts under `rust/scripts/`: `58`

## What The Rust Codebase Actually Is
- A grammar-to-parser pipeline:
  - `grammars/*.ebnf -> raw AST / JSON -> generated/*.rs`
- A parser generator:
  - AST-based Rust parser emission via `syn`, `quote`, and `prettyplease`
- A typed annotation platform:
  - return annotations
  - semantic annotations
  - validation and runtime-steering layers
- A stimuli-generation and coverage-closure platform:
  - in-memory stimuli generation
  - gap reporting
  - target planning
  - replay/closure-oriented telemetry
- A public integration surface:
  - parser registry
  - embedding API
  - grammar-profile-aware parse entrypoints
- A proof/gate system:
  - build orchestration
  - closure/status/contract sidecars
  - release/SOTA aggregate gates

## Major Architectural Layers

### 1. Core AST Pipeline And Grammar Normalization
Primary files:
- `rust/src/ast_pipeline/mod.rs`
- `rust/src/ast_pipeline/return_annotation_handler.rs`
- `rust/src/ast_pipeline/grouped_quantifier_parser.rs`
- `rust/src/ast_pipeline/mutual_recursion_handler.rs`
- `rust/src/ast_pipeline/ast_return_transform.rs`

Role:
- Defines the central IR for grammar transformation and parse-tree handling.
- Normalizes raw AST into the grammar tree used downstream.
- Handles branch/rule annotations.
- Performs left-recursion elimination and related normalization work.
- Provides shared runtime types:
  - parse node/content types
  - recursion/memoization machinery
  - trace/logging support

Assessment:
- This is the real heart of the crate.
- A lot of project doctrine is encoded here, not just in docs.
- It is powerful, but `mod.rs` itself is large enough that understanding the full transform pipeline now requires careful re-reading.

### 2. Parser Code Generation
Primary files:
- `rust/src/ast_pipeline/ast_based_generator.rs`
- `rust/src/ast_pipeline/ast_code_generator.rs`
- `rust/src/ast_pipeline/ast_generator_direct.rs`

Role:
- Turns normalized grammar AST into generated Rust parser source.
- Uses AST/token generation instead of raw string concatenation.
- Emits parser implementations with:
  - memoization and recursion guards
  - recovery telemetry
  - coverage-target telemetry
  - negative-case telemetry
  - deterministic-partition telemetry
  - semantic-runtime hooks

Assessment:
- The project is not generating “simple recognizers”; it is generating instrumented parsing systems.
- `ast_based_generator.rs` is one of the most important files in the repo.
- The AST-based codegen approach is a real strength because it reduces syntax-generation fragility.
- The downside is that too much emitted-parser policy is encoded in one giant generator module.

### 3. Stimuli, Coverage, Debt, And Closure Planning
Primary file:
- `rust/src/ast_pipeline/stimuli_generator.rs`

Role:
- Generates stimuli from grammar AST.
- Tracks coverage metrics across rules and branches.
- Computes reachable vs unreachable debt.
- Builds target plans for closure work.
- Supports recovery-biased and negative-ish generation modes.
- Integrates semantic steering into generation decisions.

Assessment:
- This is not a side tool; it is a second core engine beside the parser generator.
- It explains why PGEN should be thought of as a parser-proof platform, not only a parser generator.
- The module is very capable, but at nearly eight thousand lines it is a major maintainability hotspot.

### 4. Typed Annotation Model, Validation, And Runtime Semantics
Primary files:
- `rust/src/ast_pipeline/unified_return_ast.rs`
- `rust/src/ast_pipeline/unified_semantic_ast.rs`
- `rust/src/ast_pipeline/annotation_validator.rs`
- `rust/src/ast_pipeline/semantic_directive_registry.rs`
- `rust/src/ast_pipeline/semantic_runtime.rs`
- `rust/src/ast_pipeline/semantic_transform.rs`

Role:
- Parses and normalizes typed return/semantic annotation payloads.
- Validates annotation contracts against grammar structure.
- Defines semantic directive parsing and capability rules.
- Compiles semantic runtime annotations and executes them transactionally during parse.

Assessment:
- Return-annotation support appears deeper and more mature than the typed semantic AST layer.
- Semantic support is still substantial, but more directive-oriented and more spread across registry/runtime/validator seams.
- The validator is far beyond a “lint” layer; it is grammar-aware and contract-bearing.
- The semantic runtime is a meaningful subsystem in its own right.

### 5. Grammar-Specific Subsystems
Primary files:
- `rust/src/ebnf_frontend.rs`
- `rust/src/sv_preprocessor.rs`

Role:
- `ebnf_frontend.rs` provides a Rust-native `.ebnf -> raw_ast` frontend path.
- `sv_preprocessor.rs` implements a policyful SystemVerilog preprocessing stage with:
  - macro handling
  - include resolution
  - conditional compilation
  - diagnostics
  - source maps
  - event logging

Assessment:
- The Rust EBNF frontend is a real parser/tokenizer subsystem, not a small adapter.
- The SV preprocessor is substantial enough to deserve treatment as its own engine.
- The SV preprocessor’s explicit policies and observability surfaces are a strength.

### 6. Public Consumer Surfaces
Primary files:
- `rust/src/parser_registry.rs`
- `rust/src/embedding_api.rs`
- `rust/src/lib.rs`

Role:
- `parser_registry.rs` centralizes grammar-name dispatch across generated/bootstrap/profile-aware parsers.
- `embedding_api.rs` exposes a stable, versioned consumer contract with limits, result shapes, and AST-dump modes.
- `lib.rs` controls feature-gated exposure of the major subsystems.

Assessment:
- The embedding API is one of the cleaner and more disciplined pieces of the codebase.
- The registry layer is intentionally small and useful.
- There is still some repeated grammar/backend/profile branching across registry/API/binaries that could be unified further.

### 7. CLI, Build, And Operational Proof Layer
Primary files:
- `rust/src/main.rs`
- `rust/build.rs`
- `rust/Makefile`
- `rust/src/bin/*.rs`
- `rust/scripts/*.sh`

Role:
- `main.rs` is the large orchestration CLI for the core pipeline modes.
- `build.rs` resolves generated parser include paths at build time and emits `cfg` flags for available grammars.
- `rust/Makefile` coordinates bootstrap vs normal parser-generation flows.
- `rust/scripts/*.sh` provide the proof/gate ecosystem used for closure and release-grade validation.

Assessment:
- The build/gate layer is a major part of the product, not an afterthought.
- `build.rs` is strategically important because it lets the crate tolerate optional/generated parser availability.
- `main.rs` is functionally rich but overly large.
- The shell-gate surface is now big enough that architecture comprehension requires understanding both Rust and shell proof plumbing together.

## End-To-End Artifact Spine
1. Grammar/source input
   - Typical starting artifacts:
     - `grammars/*.ebnf`
     - generated grammar JSON inputs
     - real parser input samples
     - raw SystemVerilog source for preprocessing mode
2. Frontend / ingestion layer
   - `rust/src/ebnf_frontend.rs` can produce raw-AST envelopes directly from `.ebnf`
   - older or compatibility flows may still enter from precomputed JSON instead of live `.ebnf`
   - SystemVerilog preprocessing can branch here and emit expanded source plus source-map/diagnostic metadata before parsing
3. Normalization / transformation layer
   - `RustASTPipeline` in `rust/src/ast_pipeline/mod.rs` turns raw AST into the normalized grammar tree used for downstream generation
   - Important intermediate artifacts:
     - transformed / generation-input AST JSON
     - annotation metadata
     - normalization statistics
4. Generation layer
   - `ast_based_generator.rs` turns normalized grammar AST into generated Rust parser source
   - `stimuli_generator.rs` turns normalized grammar AST into:
     - in-memory stimuli
     - stimuli modules
     - coverage JSON
     - parseability reports
     - gap reports
     - target-drive telemetry
5. Runtime / consumer layer
   - Generated parser source becomes runtime parser modules through `build.rs` + `lib.rs`
   - Those runtime surfaces are then consumed by:
     - `parser_registry.rs`
     - `embedding_api.rs`
     - `parseability_probe`
     - `test_runner`
     - `perf_bench`
     - grammar-specific operational binaries
   - Parser-backed AST dumps can reappear here as a second artifact family, distinct from generation-input AST dumps
6. Proof / release layer
   - `rust/scripts/*.sh` collect upstream artifacts and emit machine-readable sidecars such as:
     - `summary.txt`
     - `summary.json`
     - `summary.csv`
   - Higher-level status / contract / combined-telemetry / SOTA gates then aggregate those sidecars into the project’s executable proof surface

Operational reading rule:
- Many bugs show up one stage later than where they originate.
- If a proof gate or parser runtime looks wrong, first identify which artifact family is wrong:
  - raw/frontend AST
  - normalized generation-input AST
  - generated parser source
  - runtime parser output
  - stimuli/coverage telemetry
  - proof sidecar summaries

## Artifact Persistence Classes
- Hand-authored source-of-truth artifacts
  - Examples:
    - `rust/src/**/*.rs`
    - `grammars/*.ebnf`
    - repo docs such as `README.md`, `docs/reference/PGEN_SOTA_IMPLEMENTATION_ROADMAP.md`, and this analysis doc
  - How to treat them:
    - edit directly when changing architecture, grammar intent, or authored implementation
- Tracked generated artifacts
  - Examples:
    - checked-in generated parser sources under `generated/`
    - especially tracked annotation/runtime-support artifacts like `generated/return_annotation_parser.rs` and `generated/semantic_annotation_parser.rs`
  - How to treat them:
    - do not hand-edit unless the project explicitly treats that file as maintained source
    - prefer regenerating from the real upstream grammar/pipeline and then committing the regenerated result when that tracked file is part of the intended contract
- Build-discovered parser artifacts
  - Examples:
    - parser files reached through `PGEN_*_PARSER_PATH`
    - files whose availability is surfaced via `build.rs` as `has_generated_*` cfgs
  - How to treat them:
    - confirm whether the current task expects a checked-in parser source, a locally generated file, or a gate/workdir-produced file
    - these are often runtime-available without being repository source-of-truth
- Ephemeral operational artifacts
  - Examples:
    - `rust/target/**`
    - temporary AST dumps
    - parseability reports
    - stimuli outputs
    - gate `summary.txt` / `summary.json` sidecars
  - How to treat them:
    - use them for proof, debugging, and validation
    - do not treat them as authoring surfaces unless a task is explicitly about emitted proof artifacts
- Consumer-visible but derived contract artifacts
  - Examples:
    - generated parser-backed AST JSON output
    - proof summaries consumed by higher gates
    - machine-readable reports consumed by status/aggregate layers
  - How to treat them:
    - these may be operationally important even when they are not checked in
    - fix the producer or the upstream source-of-truth first; do not “repair” the artifact by hand unless the task is specifically about the artifact schema/text itself

Operational rule:
- Before editing an artifact, classify it first:
  - authored source,
  - tracked generated file,
  - build-discovered runtime file,
  - or ephemeral proof/debug output
- A lot of accidental drift in this repo comes from patching a derived artifact when the real intended change belonged one layer upstream.

## Operational Vocabulary
- Raw AST
  - The frontend-oriented grammar structure coming directly from `.ebnf` ingestion or equivalent JSON import.
  - This is earlier and looser than the normalized generation-input AST.
- Generation-input AST
  - The normalized grammar tree actually consumed by parser generation and stimuli generation.
  - When this shape changes, parser output and stimuli output often both move.
- Generated parser source
  - The emitted Rust parser code produced from the generation-input AST before it becomes a compiled runtime module.
- Parser-backed AST dump
  - An AST or parse-tree-shaped output produced by a runtime parser surface such as `parseability_probe` or embedding APIs.
  - This is a runtime artifact, not the same thing as the generation-input AST.
- Parseability report
  - A machine-readable report about whether generated samples or corpus inputs are accepted by the relevant parser surface.
  - In this repo, parseability is often the bridge between generation logic and proof gates.
- Coverage / gap report
  - The stimuli-side telemetry that says what rules/branches were covered, which reachable targets remain, and what target-drive work is still open.
- `summary.txt`
  - The human-readable proof sidecar for a gate or contract layer.
  - Usually a flat `key: value` compatibility surface for quick inspection and shell-level checks.
- `summary.json`
  - The machine-readable proof sidecar for the same gate or contract layer.
  - This is the preferred surface for higher-level gates when the proof chain needs structured consumption.
- Family status / family status contract
  - Mid-layer proof artifacts that summarize a parser family’s current closure state and then freeze that state into a parity-checked contract surface.
- Combined telemetry
  - A higher aggregate layer that collects family-contract, family-status, and contract-sidecar evidence into one family-level proof view.
- SOTA / `sota_exit_gate`
  - The top aggregate proof layer that rolls family-level evidence into the project’s highest-level executable status surface.

## Canonical Source-Of-Truth Map
- Cargo binary and feature surface
  - Canonical source: `rust/Cargo.toml`
  - Use this first when the question is “does this binary/feature even exist?”
- Generated parser path resolution and `has_generated_*` cfg emission
  - Canonical source: `rust/build.rs`
  - Use this first when the question is “why is this generated parser path available or unavailable?”
- Feature-gated parser module exposure inside the crate
  - Canonical source: `rust/src/lib.rs`
  - Use this first when the question is “which generated modules are actually compiled into the library?”
- Grammar/profile parse dispatch and parser availability
  - Canonical source: `rust/src/parser_registry.rs`
  - Use this first when the question is “which parser surface does this grammar/profile name resolve to?”
- Embedder-facing parse, dump, and result contract
  - Canonical source: `rust/src/embedding_api.rs`
  - Use this first when the question is “what is the supported host-facing Rust/API behavior?”
- Main orchestration modes and generation/stimuli/preprocess CLI wiring
  - Canonical source: `rust/src/main.rs`
  - Use this first when the question is “what does the main Rust pipeline CLI do in this mode?”
- Stimuli, coverage, gap, and target-drive behavior
  - Canonical source: `rust/src/ast_pipeline/stimuli_generator.rs`
  - Use this first when the question is “why did generation/coverage/targeting behave this way?”
- EBNF frontend behavior
  - Canonical source: `rust/src/ebnf_frontend.rs`
  - Use this first when the question is “what raw AST did the Rust frontend mean to produce?”
- Proof-sidecar schema and aggregate proof flow
  - Canonical source: the emitting gate in `rust/scripts/*.sh`, with `summary.json` as the preferred structured contract surface
  - Use this first when the question is “which proof fields are actually promised here?”
- Aggregate annotation proof composition
  - Canonical source: `rust/Makefile`
  - Use this first when the question is “which top-level annotation/semantic/return proof targets are supposed to run together?”
- Annotation proof obligations and semantic intent
  - Canonical source: `docs/reference/PGEN_ANNOTATION_NORMATIVE_SPEC.md`
  - Use this first when the question is “what annotation behavior and proof obligations are we actually claiming?”
- Operator-facing annotation gate map
  - Canonical source: `PGEN_USER_GUIDE.md`
  - Use this first when the question is “which aggregate annotation gates are meant to be run or read by humans?”
- Current architecture/risk/steering snapshot
  - Canonical source: `docs/reference/RUST_CODEBASE_ANALYSIS.md`
  - Use this first when the question is “what is the current repo-level understanding of the Rust codebase?”

Operational rule:
- When two layers disagree, fix the upstream source of truth first, then bring downstream consumers back into parity.
- In this repo, a lot of wasted time comes from patching adapters, sidecars, or aggregate readers before confirming which layer is actually authoritative.

## Public Contract Surface Map
- Embedder-facing Rust API contract
  - Main surface:
    - `rust/src/embedding_api.rs`
  - Compatibility promise:
    - host-visible parse, AST-dump, input-limit, and error/result behavior for Rust callers and bindings
- Grammar/profile dispatch contract
  - Main surface:
    - `rust/src/parser_registry.rs`
  - Compatibility promise:
    - which grammar/profile names resolve to which parser-backed behaviors and whether those paths are available
- Generated-parser availability contract
  - Main surface:
    - `rust/build.rs`
    - `rust/src/lib.rs`
  - Compatibility promise:
    - whether generated parser modules are actually compiled/exposed for a given feature/path configuration
- Machine-readable proof contract
  - Main surface:
    - emitting gates in `rust/scripts/*.sh`
    - especially `summary.json`
  - Compatibility promise:
    - structured proof fields consumed by family-status, combined-telemetry, SOTA, and local-CI parity readers
- Aggregate annotation proof contract
  - Main surface:
    - `rust/Makefile`
    - `annotation_contract_gate`
    - `semantic_full_contract_gate`
    - `return_annotation_support_gate`
    - `annotation_stimuli_quality_gate`
  - Compatibility promise:
    - which aggregate annotation proof targets compose the repo’s practical closure claims for validator coverage, runtime/round-trip evidence, return support, and annotation stimuli closure
- Human-readable proof compatibility surface
  - Main surface:
    - gate `summary.txt`
  - Compatibility promise:
    - stable shell-readable `key: value` summaries that remain in parity with the JSON sidecar where the shipped proof spine requires it
- Parseability/AST probe contract
  - Main surface:
    - `rust/src/bin/parseability_probe.rs`
  - Compatibility promise:
    - the narrow machine-facing parseability and AST-dump behavior used by external checks and higher proof layers

Operational rule:
- If a task changes one of these surfaces, treat it as a compatibility change even when the code edit feels local.
- Validate not only the producer, but also the nearest real consumer that relies on that contract.
- Do not assume internal parser-registry or probe availability automatically means the same family already has a public embedding contract.
  - In this repo, internal capability can be ahead of the stable embedder-facing surface.

## Symptom-To-Layer Triage Shortcuts
- Symptom: Cargo can build or list a binary, but the expected parser/runtime path is still missing
  - First likely seam:
    - `rust/build.rs`
    - `rust/src/lib.rs`
  - Usually verify:
    - feature enablement
    - `PGEN_*_PARSER_PATH` resolution
    - matching `has_generated_*` cfg emission
- Symptom: A grammar looks fine at frontend/raw-AST level, but generated parser behavior is still wrong
  - First likely seam:
    - normalized generation-input AST in `rust/src/ast_pipeline/mod.rs`
    - then `rust/src/ast_pipeline/ast_based_generator.rs`
  - Common mistake:
    - assuming the raw AST already reflects the shape the generator actually consumes
- Symptom: `parseability_probe` / embedding behavior disagrees with expectations from generation or registry changes
  - First likely seam:
    - `rust/src/parser_registry.rs`
    - `rust/src/embedding_api.rs`
    - then the generated parser availability layer
  - Common mistake:
    - debugging only the CLI surface without checking the dispatch/availability contract under it
- Symptom: Generated samples, coverage, or target-drive behavior looks wrong while parser acceptance seems fine
  - First likely seam:
    - `rust/src/ast_pipeline/stimuli_generator.rs`
    - then the parseability-report and gap-report consumers in `rust/src/main.rs` and `rust/scripts/*.sh`
  - Common mistake:
    - treating stimuli/coverage problems as pure parser-codegen problems
- Symptom: Annotation-focused unit tests or leaf suites pass, but the repo-level annotation proof still feels wrong or incomplete
  - First likely seam:
    - the nearest aggregate annotation proof surface:
      - `annotation_contract_gate`
      - `semantic_full_contract_gate`
      - `return_annotation_support_gate`
      - `annotation_stimuli_quality_gate`
    - then the leaf validator/runtime/stimuli seam that feeds it
  - Common mistake:
    - stopping at the leaf suite that passed instead of checking which aggregate proof claim the repo is actually making
- Symptom: EBNF frontend output and generated-parser behavior drift apart
  - First likely seam:
    - `rust/src/ebnf_frontend.rs`
    - `rust/src/bin/ebnf_dual_run_diff.rs`
  - Common mistake:
    - debugging only downstream parser behavior without checking `parse` vs `parse_full` or frontend drift first
- Symptom: Gate summaries, family contracts, or aggregate proof outputs disagree even though lower layers “passed”
  - First likely seam:
    - the emitting gate in `rust/scripts/*.sh`
    - then the next aggregate reader above it
  - Common mistake:
    - patching only the aggregate reader when the emitting sidecar schema or TXT/JSON parity is actually wrong
- Symptom: A change compiles and unit tests pass, but the repo still feels inconsistent
  - First likely seam:
    - the next real artifact consumer, not the producer you already changed
  - Common mistake:
    - stopping validation at compile/test success instead of crossing the seam that the changed artifact feeds

## Safe Intervention Order
When a bug spans multiple layers, prefer these patch orders over ad hoc consumer-first edits.

- Build / availability problem
  - Preferred order:
    - `rust/Cargo.toml`
    - `rust/build.rs`
    - `rust/src/lib.rs`
    - `rust/src/parser_registry.rs` / `rust/src/embedding_api.rs`
    - CLI binaries / gates
  - Reason:
    - if feature wiring or cfg emission is wrong, every downstream layer will only produce misleading symptoms
- Parser-shape or acceptance problem
  - Preferred order:
    - frontend/raw-AST layer when applicable
    - normalized generation-input AST
    - parser generator
    - runtime parser consumers
    - proof/gate readers
  - Reason:
    - generator/runtime fixes applied before the normalized grammar contract is correct usually create brittle compensations
- Stimuli / coverage / target-drive problem
  - Preferred order:
    - normalized grammar contract
    - `stimuli_generator.rs`
    - CLI/report emission in `rust/src/main.rs`
    - nearest consuming gate or contract layer
  - Reason:
    - the proof artifacts are downstream of generation policy, not the source of it
- Annotation proof / closure problem
  - Preferred order:
    - typed annotation parse/validation/runtime layer
    - nearest leaf annotation contract or typed-AST suite
    - nearest aggregate annotation proof surface
      - `semantic_full_contract_gate`
      - `return_annotation_support_gate`
      - `annotation_stimuli_quality_gate`
      - `annotation_contract_gate`
    - doc / local-CI routing if the operator-facing proof map changed
  - Reason:
    - patching the top-level proof claim before the nearest annotation seam is correct tends to hide whether the real drift is semantic behavior, closure evidence, or just the proof map
- Registry / embedding disagreement
  - Preferred order:
    - generated-parser availability layer
    - `parser_registry.rs`
    - `embedding_api.rs`
    - CLI probes / external consumers
  - Reason:
    - embedder behavior should not be patched around registry/build drift
- Proof-sidecar disagreement
  - Preferred order:
    - emitting gate
    - direct reader one layer above
    - higher aggregates
    - local CI regression guard
  - Reason:
    - if the emitted proof surface is wrong, fixing only the aggregate consumer tends to multiply schema drift

Operational rule:
- Patch the earliest layer that can truthfully explain the symptom.
- Only patch a downstream layer first when the upstream contract is already verified correct and the bug is strictly in the consumer.

## Main Rust Executables And Roles
- `ast_pipeline` / `ast_pipeline_bootstrap`
  - Both are wired to `rust/src/main.rs` via Cargo features.
  - This is the main orchestration CLI for:
    - AST transformation
    - parser generation
    - stimuli generation
    - stimuli-module generation
    - generation-input AST dumps
    - SystemVerilog preprocessing
  - If a task sounds like “run the Rust pipeline on a grammar or source file,” this is usually the first executable to inspect.
- `test_runner`
  - The main round-trip and suite-running harness for bootstrap/generated parser validation.
  - Important when the task is test-suite behavior, normalization in tests, or parser-family regression coverage.
- `parseability_probe`
  - The compact machine-facing probe for “does this grammar/profile parse this input?” and “dump the AST for this parse.”
  - This is one of the cleanest executable surfaces for external parseability contracts and AST-dump behavior.
- `ebnf_dual_run_diff`
  - A specialist diagnostic tool for the generated EBNF parser path.
  - It compares `parse` vs `parse_full` behavior and emits structured diagnostics for unconsumed tails and frontend drift.
- `perf_bench`
  - Benchmark/threshold executable for bootstrap-vs-generated parser throughput and latency.
  - Relevant when performance changes need proof, not just anecdotal timing.
- `pgen_ast`
  - A focused AST-based codegen CLI that reads transformed AST JSON and emits parser source.
  - It is narrower than `ast_pipeline`, but still useful for direct generator work or compatibility testing around AST-based emission.
- `return_annotation_generated_audit`
  - A small audit executable for generated return-annotation typed-AST serialization over sample lists.
  - Useful as a niche contract checker, not as a primary day-to-day workflow surface.
- `pgen`
  - An older parser smoke-test CLI for semantic/return/regex parser inputs with log-file output.
  - It is not the main modern operational surface, but it still exists and should be treated as a legacy-adjacent utility rather than deleted-by-assumption.

Assessment:
- Not every Rust executable here is equally strategic.
- The practical “primary” binaries are:
  - `ast_pipeline` / `ast_pipeline_bootstrap`
  - `test_runner`
  - `parseability_probe`
  - `ebnf_dual_run_diff`
  - `perf_bench`
- The smaller `pgen_ast`, `return_annotation_generated_audit`, and `pgen` executables are better thought of as specialist or legacy-support utilities.

## Canonical-Vs-Legacy Surface Map
- Canonical day-to-day Rust operational surfaces
  - `ast_pipeline` / `ast_pipeline_bootstrap`
  - `test_runner`
  - `parseability_probe`
  - `ebnf_dual_run_diff`
  - `perf_bench`
  - Why:
    - these are the surfaces most likely to reflect the current intended build/runtime/proof contract
- Specialist but current surfaces
  - `pgen_ast`
  - `return_annotation_generated_audit`
  - selected grammar-specific operational flows that are narrow but still intentionally maintained
  - Why:
    - these are not the first place to look for most tasks, but they still represent real maintained seams
- Legacy-adjacent or carryover surfaces
  - `pgen`
  - older test-layer pieces like `rust/src/test_registry.rs` and `rust/src/test_discovery.rs`
  - Why:
    - they still exist and can matter for compatibility or historical behavior, but they should not be assumed to define the repo’s main modern workflow
- Canonical proof/verification surfaces
  - `rust/scripts/*.sh` gates on the shipped proof spine
  - family status / family status contract / combined telemetry / `sota_exit_gate`
  - `rust/scripts/ci_workflow_local_gate.sh`
  - Why:
    - these are the main executable proof contracts that preserve and validate the Rust-produced artifacts

Operational rule:
- If a newer canonical surface and an older carryover surface disagree, debug the canonical one first unless the user explicitly asks about compatibility or historical behavior.
- Use legacy-adjacent surfaces as corroborating evidence, not as the primary definition of current repo truth.

## Rust-To-Shell Contract Seams
- Parser availability / registry seam
  - Main Rust producers:
    - `rust/build.rs`
    - `rust/src/lib.rs`
    - `rust/src/parser_registry.rs`
    - `rust/src/embedding_api.rs`
    - `rust/src/bin/parseability_probe.rs`
  - Main shell-side consumers:
    - grammar-quality and family-contract gates under `rust/scripts/*.sh`
  - Typical failure mode:
    - a Rust cfg/registry change looks like a gate/proof failure because the shell layer only sees “adapter unavailable”, missing parseability support, or missing downstream report fields
- Parseability / stimuli / gap-report seam
  - Main Rust producers:
    - `rust/src/main.rs`
    - `rust/src/ast_pipeline/stimuli_generator.rs`
    - `rust/src/bin/parseability_probe.rs`
  - Main shell-side consumers:
    - family-contract, family-status, and aggregate proof gates that ingest parseability reports, coverage reports, gap reports, and target-drive summaries
  - Typical failure mode:
    - a Rust-side artifact schema or counting rule changes and the nearest shell consumer starts failing parity checks or status derivation
  - Regex-specific note:
    - the regex family now also depends on deterministic parseability-counterexample triage artifacts derived from the parser-backed stimuli report, so report-shape/counting changes can now break family-contract, SOTA, and combined-telemetry parity even when headline totals still look stable
    - it now also depends on an explicit regex formal-exhaustive-closure sidecar layered on top of those family artifacts, so broader-corpus proof promotion work now has a named gate/schema contract rather than a placeholder blocker string
- Summary sidecar seam
  - Main producers:
    - emitting gates under `rust/scripts/*.sh`
    - Rust tools whose outputs are re-packaged into `summary.txt` / `summary.json`
  - Main consumers:
    - family-status
    - family-status-contract
    - combined telemetry
    - `sota_exit_gate`
    - `ci_workflow_local_gate`
  - Typical failure mode:
    - the artifact still exists, but the structured contract changed and higher readers disagree on path, key, or parity expectations
- Frontend / dual-run seam
  - Main Rust producers:
    - `rust/src/ebnf_frontend.rs`
    - `rust/src/main.rs`
    - `rust/src/bin/ebnf_dual_run_diff.rs`
  - Main shell-side consumers:
    - `rust/scripts/ebnf_*`
    - grammar-quality gates that depend on frontend parity or generated-parser freshness
  - Typical failure mode:
    - an ingestion/frontend change first appears as an `ebnf` gate drift or a generated-parser proof mismatch rather than a direct Rust test failure
- Aggregate annotation proof seam
  - Main Rust / Makefile producers:
    - `rust/Makefile`
    - `rust/src/ast_pipeline/annotation_validator.rs`
    - `rust/src/ast_pipeline/unified_return_ast.rs`
    - `rust/src/ast_pipeline/unified_semantic_ast.rs`
  - Main shell-side / doc-side consumers:
    - `rust/scripts/ci_workflow_local_gate.sh`
    - `README.md`
    - `PGEN_USER_GUIDE.md`
    - `docs/reference/PGEN_ANNOTATION_NORMATIVE_SPEC.md`
  - Aggregate proof spine:
    - `annotation_contract_gate`
      - validator + built-in bootstrap suites
      - shared bootstrap/generated annotation suites
      - SC leaf contract gates
      - `semantic_full_contract_gate`
      - `return_full_contract_gate`
      - `annotation_robustness_gate`
      - `annotation_stimuli_quality_gate`
    - `semantic_full_contract_gate`
      - `semantic_runtime_contract_gate`
      - `semantic_ast_roundtrip_gate`
      - `semantic_differential_regression_gate`
    - `return_annotation_support_gate`
      - sits above `return_full_contract_gate`
      - also requires the dedicated exhaustiveness proof
  - Typical failure mode:
    - the leaf tests still pass, but the operator-facing proof map or aggregate gate composition drifts and CI/humans stop running the intended top-level evidence path

Operational rule:
- When a Rust change affects artifact names, paths, schemas, or count semantics, inspect the nearest shell consumer above that artifact before assuming the bug is isolated to Rust.
- The meaningful validation seam is usually:
  - the emitted Rust artifact,
  - plus the next gate that consumes it.

## Where To Start By Task Type

### If the task is figuring out which Rust executable owns a workflow
Start here:
- `rust/Cargo.toml`
- `rust/src/main.rs`
- `rust/src/bin/test_runner.rs`
- `rust/src/bin/parseability_probe.rs`
- `rust/src/bin/ebnf_dual_run_diff.rs`
- `rust/src/bin/perf_bench.rs`
- `docs/reference/RUST_CODEBASE_ANALYSIS.md` section `Main Rust Executables And Roles`

Reason:
- Cargo wiring matters in this repo because feature-gated binaries share entrypoints.
- The fastest way to stop wandering is to identify whether a task belongs to the main pipeline CLI, a validation harness, a parseability contract tool, a frontend diagnostic, or a specialist audit utility.

### If the task is grammar normalization or parser-shape behavior
Start here:
- `rust/src/ast_pipeline/mod.rs`
- `rust/src/ast_pipeline/grouped_quantifier_parser.rs`
- `rust/src/ast_pipeline/mutual_recursion_handler.rs`
- `rust/src/ast_pipeline/return_annotation_handler.rs`

Reason:
- This is where raw grammar structure becomes the normalized grammar tree that the rest of the system depends on.
- Changes here can affect parser generation, stimuli generation, annotation validation, and closure metrics all at once.

### If the task is generated parser behavior or parser code shape
Start here:
- `rust/src/ast_pipeline/ast_based_generator.rs`
- `rust/src/ast_pipeline/ast_code_generator.rs`
- `rust/src/ast_pipeline/ast_generator_direct.rs`

Reason:
- This is the emitted-parser contract layer.
- Parser runtime telemetry, semantic-runtime ownership, recovery behavior, and branch ordering all converge here.

### If the task is stimuli, gap reports, or coverage closure
Start here:
- `rust/src/ast_pipeline/stimuli_generator.rs`

Then usually inspect:
- `rust/src/ast_pipeline/mod.rs`
- `rust/src/ast_pipeline/semantic_runtime.rs`

Reason:
- Stimuli generation is highly coupled to normalized grammar shape and semantic steering.
- Coverage/debt behavior is not a thin report layer; it is part of how closure work is directed.

### If the task is return/semantic annotation parsing or validation
Start here:
- `rust/src/ast_pipeline/unified_return_ast.rs`
- `rust/src/ast_pipeline/unified_semantic_ast.rs`
- `rust/src/ast_pipeline/annotation_validator.rs`
- `rust/src/ast_pipeline/semantic_directive_registry.rs`
- `rust/src/ast_pipeline/semantic_runtime.rs`

Then usually inspect:
- `rust/Makefile`
- `docs/reference/PGEN_ANNOTATION_NORMATIVE_SPEC.md`
- `PGEN_USER_GUIDE.md`

And pick the nearest aggregate proof surface:
- `annotation_contract_gate`
- `semantic_full_contract_gate`
- `return_annotation_support_gate`
- `annotation_stimuli_quality_gate`

Reason:
- The typed annotation model, validator rules, directive registry, and runtime behavior are split across these files.
- It is easy to fix one layer and accidentally leave the others inconsistent.
- In this repo, annotation work is not fully real until it still fits the aggregate proof spine that operators and CI actually run.

### If the task is external integration or embedder-facing API behavior
Start here:
- `rust/src/embedding_api.rs`
- `rust/src/parser_registry.rs`
- `rust/src/lib.rs`
- `rust/build.rs`

Reason:
- Consumer-visible behavior depends on both runtime dispatch and build-time generated-parser availability.
- Feature/cfg/build-path interactions matter here as much as function signatures do.

### If the task is SystemVerilog preprocessing
Start here:
- `rust/src/sv_preprocessor.rs`

Then usually inspect:
- `rust/src/main.rs`
- relevant `rust/scripts/sv_*` gates

Reason:
- The SV preprocessor is its own subsystem with policies, diagnostics, event logs, and source maps.
- Its behavior is not just a helper phase before parsing.

### If the task is EBNF frontend conversion
Start here:
- `rust/src/ebnf_frontend.rs`
- `rust/src/main.rs`
- `rust/Makefile`
- relevant `rust/scripts/ebnf_*` gates

Reason:
- The Rust EBNF frontend sits at the start of the build/proof spine, not only as a parsing helper.
- Changes here can affect raw-AST conversion, dual-run differentials, and the generated-parser pipeline.

### If the task is CLI mode behavior or top-level orchestration
Start here:
- `rust/src/main.rs`
- `rust/src/bin/*.rs`
- `rust/Makefile`

Reason:
- The codebase has one large orchestration entrypoint plus several smaller utility binaries.
- The main risk is changing mode behavior without aligning the supporting build/gate surface.

### If the task is proof plumbing, contract sidecars, or release-gate behavior
Start here:
- `rust/scripts/*.sh`
- `rust/Makefile`
- `rust/src/bin/parseability_probe.rs`
- `rust/src/parser_registry.rs`
- `rust/src/embedding_api.rs`

For SystemVerilog external-corpus proof normalization, narrow quickly to:
- `rust/scripts/sv_external_corpus_triage_gate.sh`
- `sv_formal_exhaustive_closure_gate` when the task is SystemVerilog external-corpus proof normalization
- `rust/scripts/sv_formal_exhaustive_closure_gate.sh` when the task is SystemVerilog external-corpus proof normalization
- `rust/scripts/sv_parser_family_status_gate.sh`
- `rust/scripts/sv_parser_family_status_contract_gate.sh`
- `rust/scripts/sota_exit_gate.sh` when the task is aggregate proof-surface propagation
- `rust/scripts/sv_combined_telemetry_contract_gate.sh` when the task is aggregate parity over retained SystemVerilog sidecars

For SystemVerilog-preprocessor formal-closure proof normalization, narrow quickly to:
- `sv_preprocessor_formal_exhaustive_closure_gate` when the task is SystemVerilog-preprocessor formal-closure proof normalization
- `rust/scripts/sv_preprocessor_formal_exhaustive_closure_gate.sh` when the task is SystemVerilog-preprocessor formal-closure proof normalization
- `rust/scripts/sv_preprocessor_syntax_closure_gate.sh`
- `rust/scripts/sv_preprocessor_aggregate_contract_gate.sh`
- `rust/scripts/sv_preprocessor_reachability_closure_gate.sh`
- `rust/scripts/sv_parser_family_status_gate.sh` when the task is retained family-status propagation for the preprocessor formal-closure seam
- `rust/scripts/sv_parser_family_status_contract_gate.sh` when the task is source-side contract validation for that propagated seam
- `rust/scripts/sota_exit_gate.sh` when the task is aggregate proof-surface propagation for the preprocessor formal-closure seam
- `rust/scripts/sv_combined_telemetry_contract_gate.sh` when the task is aggregate parity over the retained preprocessor formal-closure sidecar paths

For annotation-specific proof plumbing, narrow quickly to:
- `annotation_contract_gate`
- `semantic_full_contract_gate`
- `return_annotation_support_gate`
- `annotation_stimuli_quality_gate`

Reason:
- A large amount of project truth now lives in the shell-gate layer and the artifacts it consumes/emits.
- These tasks often require understanding both machine-readable sidecars and the Rust producer/consumer seams behind them.

## High-Risk Change Zones
- `rust/src/ast_pipeline/mod.rs`
  - high blast radius because it changes the normalized grammar contract used by both parser and stimuli generation.
- `rust/src/ast_pipeline/ast_based_generator.rs`
  - high blast radius because emitted parser behavior, runtime telemetry, and semantic hooks all converge here.
- `rust/src/ast_pipeline/stimuli_generator.rs`
  - high blast radius because closure metrics, target planning, and semantic steering are co-located here.
- `rust/src/main.rs`
  - high coordination cost because many modes share one orchestration entrypoint.
- `rust/build.rs`
  - easy to underestimate; build-time parser-availability bugs can look like runtime/parser bugs elsewhere.
- `rust/src/embedding_api.rs` and `rust/src/parser_registry.rs`
  - small files relative to the engines, but they sit on public integration seams where drift is expensive.
- `rust/scripts/sota_exit_gate.sh` and sibling family aggregate/status gates
  - not Rust code, but they are part of the effective Rust-owned product contract.

## Review Hotspots And Common Regression Types
- `rust/src/ast_pipeline/mod.rs`
  - First review for:
    - normalized rule-shape drift
    - left-recursion or grouping rewrites that silently change generator/stimuli inputs
    - raw-AST vs generation-input-AST confusion
- `rust/src/ast_pipeline/ast_based_generator.rs`
  - First review for:
    - emitted parser behavior drift
    - semantic-runtime hook drift
    - telemetry/counterexample/recovery output regressions
- `rust/src/ast_pipeline/stimuli_generator.rs`
  - First review for:
    - reachable/unreachable accounting drift
    - target-drive or closure-priority drift
    - parseability/coverage report shape changes
- `rust/src/main.rs`
  - First review for:
    - mode-selection drift
    - output-path / artifact-emission drift
    - wiring mismatches between CLI behavior and downstream consumer expectations
- `rust/build.rs`
  - First review for:
    - generated-parser path discovery regressions
    - missing or incorrect `has_generated_*` cfg emission
    - “binary exists, runtime path unavailable” failure shapes
- `rust/src/parser_registry.rs` and `rust/src/embedding_api.rs`
  - First review for:
    - parser availability/dispatch disagreements
    - embedder-facing compatibility drift
    - fixes that patch consumers instead of the upstream availability/source-of-truth layer
- `rust/scripts/*.sh` on the shipped proof spine
  - First review for:
    - `summary.txt` / `summary.json` parity drift
    - field/path/schema renames that higher readers were not updated for
    - aggregate layers preserving consumed provenance instead of dropping it

Operational rule:
- In this repo, “high risk” often means “easy to preserve compile success while breaking a contract.”
- Review the contract outputs and next consumers, not only the local code diff.

## Modules That Tend To Change Together
- Grammar normalization cluster
  - Typical files:
    - `rust/src/ast_pipeline/mod.rs`
    - `rust/src/ast_pipeline/grouped_quantifier_parser.rs`
    - `rust/src/ast_pipeline/mutual_recursion_handler.rs`
    - `rust/src/ast_pipeline/ast_based_generator.rs`
    - `rust/src/ast_pipeline/stimuli_generator.rs`
    - `rust/src/ast_pipeline/annotation_validator.rs`
  - Why they move together:
    - normalized rule shape leaks into parser generation, typed-annotation handling, and closure/stimuli behavior faster than the file boundaries suggest
- Generated-parser availability cluster
  - Typical files:
    - `rust/Cargo.toml`
    - `rust/build.rs`
    - `rust/src/parser_registry.rs`
    - `rust/src/embedding_api.rs`
    - `rust/src/bin/parseability_probe.rs`
  - Why they move together:
    - build-time parser discovery and runtime parser exposure are separate layers, so “parser unavailable” bugs often span both
- Semantic annotation cluster
  - Typical files:
    - `rust/src/ast_pipeline/unified_return_ast.rs`
    - `rust/src/ast_pipeline/unified_semantic_ast.rs`
    - `rust/src/ast_pipeline/annotation_validator.rs`
    - `rust/src/ast_pipeline/semantic_directive_registry.rs`
    - `rust/src/ast_pipeline/semantic_runtime.rs`
    - `rust/src/ast_pipeline/ast_based_generator.rs`
  - Why they move together:
    - annotation parse shape, validation policy, emitted parser hooks, and runtime execution semantics are distributed rather than owned by one module
- EBNF/bootstrap ingestion cluster
  - Typical files:
    - `rust/src/ebnf_frontend.rs`
    - `rust/src/main.rs`
    - `rust/Makefile`
    - `rust/src/bin/ebnf_dual_run_diff.rs`
    - relevant `rust/scripts/ebnf_*` gates
  - Why they move together:
    - changes at the EBNF/raw-ingestion edge often propagate into bootstrap generation, dual-run proofs, and build orchestration
- Proof/consumer cluster
  - Typical files:
    - `rust/src/parser_registry.rs`
    - `rust/src/embedding_api.rs`
    - `rust/src/bin/parseability_probe.rs`
    - relevant `rust/scripts/*.sh` gates
  - Why they move together:
    - externally visible truth in this repo is shared between Rust artifact producers and shell-side proof consumers

Operational rule:
- If one file in a cluster changes, scan the rest of that cluster before deciding your validation scope.
- If a task touches more than one cluster, validate at the first downstream artifact where those clusters converge rather than validating each layer in isolation.

## Change-Impact Checklist
Use this as a first-pass companion-check map, not as a complete proof checklist.

- If you change grammar normalization or core AST pipeline shape
  - Typical primary files:
    - `rust/src/ast_pipeline/mod.rs`
    - `rust/src/ast_pipeline/grouped_quantifier_parser.rs`
    - `rust/src/ast_pipeline/mutual_recursion_handler.rs`
  - Usually re-check:
    - `rust/src/ast_pipeline/ast_based_generator.rs`
    - `rust/src/ast_pipeline/stimuli_generator.rs`
    - `rust/src/ast_pipeline/annotation_validator.rs`
    - generation-input AST dump behavior in `rust/src/main.rs`
    - round-trip / parseability surfaces that implicitly depend on normalized rule shape
- If you change parser code generation
  - Typical primary files:
    - `rust/src/ast_pipeline/ast_based_generator.rs`
    - `rust/src/ast_pipeline/ast_code_generator.rs`
    - `rust/src/ast_pipeline/ast_generator_direct.rs`
  - Usually re-check:
    - generated parser compileability and include-path assumptions
    - `rust/src/parser_registry.rs`
    - `rust/src/embedding_api.rs`
    - `rust/src/bin/parseability_probe.rs`
    - `rust/src/bin/test_runner.rs`
    - `rust/src/bin/perf_bench.rs`
- If you change stimuli, coverage, or gap logic
  - Typical primary file:
    - `rust/src/ast_pipeline/stimuli_generator.rs`
  - Usually re-check:
    - `rust/src/main.rs` stimuli CLI/report wiring
    - parseability validation report behavior
    - any derived counterexample-triage artifacts that shell gates summarize from parseability reports
    - coverage / gap / target-drive JSON artifacts
    - grammar-quality and family-contract gate expectations in `rust/scripts/*.sh`
- If you change annotation parsing, validation, or semantic runtime behavior
  - Typical primary files:
    - `rust/src/ast_pipeline/unified_return_ast.rs`
    - `rust/src/ast_pipeline/unified_semantic_ast.rs`
    - `rust/src/ast_pipeline/annotation_validator.rs`
    - `rust/src/ast_pipeline/semantic_runtime.rs`
    - `rust/src/ast_pipeline/semantic_directive_registry.rs`
  - Usually re-check:
    - generated parser conversion paths
    - `test_runner` bootstrap vs generated parity
    - annotation-focused suites and typed-AST consumers
    - any docs or gates that currently treat return-annotation support as closed and semantic support as still more fluid
    - the nearest aggregate annotation proof surface:
      - `annotation_contract_gate` for shared annotation contract drift
      - `semantic_full_contract_gate` for semantic runtime/round-trip/regression drift
      - `return_annotation_support_gate` for return-annotation closure drift
      - `annotation_stimuli_quality_gate` when stimuli/coverage closure is part of the change
- If you change build-script or generated-parser availability behavior
  - Typical primary files:
    - `rust/build.rs`
    - `rust/src/lib.rs`
    - `rust/src/parser_registry.rs`
  - Usually re-check:
    - Cargo feature combinations
    - `PGEN_*_PARSER_PATH` resolution behavior
    - `has_generated_*` cfg guards
    - binaries gated by `generated_parsers` or `ebnf_dual_run`
    - embedder-facing availability behavior in `embedding_api.rs`
- If you change embedder-facing or registry-facing parse surfaces
  - Typical primary files:
    - `rust/src/embedding_api.rs`
    - `rust/src/parser_registry.rs`
  - Usually re-check:
    - `rust/src/bin/parseability_probe.rs`
    - AST dump contract behavior
    - feature/cfg fallback behavior
    - any gates or tests that rely on registry exposure or parser support checks
- If you change EBNF frontend behavior
  - Typical primary files:
    - `rust/src/ebnf_frontend.rs`
    - `rust/src/main.rs`
    - `rust/src/bin/ebnf_dual_run_diff.rs`
  - Usually re-check:
    - raw-AST export behavior
    - dual-run drift reports
    - `ebnf_dual_run` build assumptions
    - readiness/quality gates that now rely on the Rust frontend path
- If you change SystemVerilog preprocessing behavior
  - Typical primary files:
    - `rust/src/sv_preprocessor.rs`
    - SV preprocess wiring in `rust/src/main.rs`
  - Usually re-check:
    - source-map and diagnostics behavior
    - strict-warning policy handling
    - downstream parseability expectations on preprocessed output
    - SV quality/aggregate proof gates in `rust/scripts/`
- If you change proof-sidecar shape or release-gate aggregation
  - Typical primary files:
    - `rust/scripts/*.sh`
    - sometimes `rust/src/bin/parseability_probe.rs` or `rust/src/embedding_api.rs`
  - Usually re-check:
    - `summary.txt` / `summary.json` parity
    - `ci_workflow_local_gate.sh`
    - higher aggregate readers like family-status, combined telemetry, and SOTA exit
    - `docs/reference/RUST_CODEBASE_ANALYSIS.md` if the effective operational contract changed

## Build And Feature Model
- The crate is feature-gated around bootstrap, normal, generated-parser, and EBNF-dual-run modes.
- Generated parser modules are not hardwired; `rust/build.rs` resolves them from environment-configured paths and only enables grammar-specific `cfg`s when files actually exist.
- This is a strength because it supports:
  - bootstrap cycles
  - clean checkout behavior
  - relocatable worktrees
  - partial grammar availability
- It also means more conditional complexity and more chances for path/feature divergence.

Feature/build matrix:
- `normal`
  - unlocks the `ast_pipeline` binary from `rust/src/main.rs`
  - represents the standard non-bootstrap orchestration path
- `bootstrap`
  - unlocks the `ast_pipeline_bootstrap` binary from the same `rust/src/main.rs` entrypoint
  - exists so the pipeline can keep functioning when generated-parser availability is intentionally reduced or absent
- `generated_parsers`
  - unlocks binaries and code paths that depend on the generated parser registry and generated parser includes
  - directly gates:
    - `parseability_probe`
    - `perf_bench`
    - generated-parser branches in the embedding/test surfaces
- `ebnf_dual_run`
  - unlocks the generated-EBNF differential tooling
  - directly gates:
    - `ebnf_dual_run_diff`
    - Rust-frontend/generated-frontend comparison flows in the CLI/build ecosystem

Build-time availability model:
- `rust/build.rs` does two distinct jobs:
  - resolves include paths from environment variables like `PGEN_SYSTEMVERILOG_PARSER_PATH` and `PGEN_VHDL_PARSER_PATH`
  - emits grammar-specific `cfg`s like `has_generated_systemverilog_parser` only when the resolved file actually exists
- That means feature enablement alone is not enough for every generated-parser behavior.
- In practice there are two layers of availability:
  - Cargo feature enabled
  - matching generated parser file found by `build.rs`

Generated parser env/cfg map:
- `PGEN_EBNF_PARSER_PATH`
  - resolved by `build.rs` into:
    - `PGEN_EBNF_PARSER_PATH_RESOLVED`
    - `PGEN_EBNF_PARSER_PATH_RESOLVED_BIN`
  - used by the `ebnf_dual_run` surface
  - important nuance: there is no `has_generated_ebnf_parser` cfg; EBNF availability is handled differently from the other grammar families
- `PGEN_JSON_PARSER_PATH`
  - drives `has_generated_json_parser`
  - controls `generated_parsers::json` and related parser-registry exposure
- `PGEN_REGEX_PARSER_PATH`
  - drives `has_generated_regex_parser`
  - controls `generated_parsers::regex`, related parser-registry exposure, and regex generated-backend availability inside `embedding_api.rs`
  - the repo now also carries the default tracked artifact at [generated/regex_parser.rs](generated/regex_parser.rs), so a normal checkout no longer needs an ad hoc env override just to make regex’s generated backend exist
- `PGEN_SYSTEMVERILOG_PARSER_PATH`
  - drives `has_generated_systemverilog_parser`
  - controls generated SystemVerilog registry, embedding, and parseability paths
- `PGEN_SYSTEMVERILOG_PREPROCESSOR_PARSER_PATH`
  - drives `has_generated_systemverilog_preprocessor_parser`
  - controls generated SV-preprocessor parser availability
- `PGEN_VHDL_PARSER_PATH`
  - drives `has_generated_vhdl_parser`
  - controls generated VHDL registry, embedding, and parseability paths
- `PGEN_RTL_CONST_EXPR_PARSER_PATH`
  - drives `has_generated_rtl_const_expr_parser`
  - controls generated RTL-const-expr parser availability

Important asymmetries:
- `return_annotation` and `semantic_annotation`
  - live under `generated_parsers`, but are included from tracked generated sources rather than `build.rs` env-driven grammar-path discovery
- `ebnf`
  - uses `build.rs`-resolved include paths, but not the same `has_generated_*` cfg pattern used by the other generated grammar families
- `systemverilog`, `vhdl`, and the other env-driven grammar families
  - usually require both:
    - `feature = "generated_parsers"`
    - matching `has_generated_*` cfg emitted by `build.rs`

Operational takeaway:
- If a binary or API path appears to “exist but not really work,” check both:
  - the Cargo feature set
  - the relevant `PGEN_*_PARSER_PATH` resolution and resulting `has_generated_*` cfg
- A surprising amount of apparent parser/runtime breakage in this repo can actually be feature/build-shape drift.

## Bootstrap-Vs-Generated Boundary Map
- `ast_pipeline`
  - The normal orchestration binary from `rust/src/main.rs`
  - Usually the right entrypoint when generated-parser-backed behavior is intended to be available
- `ast_pipeline_bootstrap`
  - The bootstrap-mode sibling from the same `rust/src/main.rs`
  - Important when the pipeline must remain usable even while generated-parser availability is intentionally constrained
- `test_runner`
  - A hybrid validation surface
  - It can exercise bootstrap behavior, generated-parser behavior, and parity between them depending on feature set and suite path
- `parseability_probe`
  - A generated-parser-oriented runtime probe
  - If this surface is unavailable or behaving oddly, suspect generated-parser availability and registry exposure before assuming generic parser bugs
- `return_annotation` / `semantic_annotation`
  - Important boundary exception
  - They sit under `generated_parsers`, but come from tracked generated sources rather than the same env-resolved grammar-family path model used by the larger HDL/regex families
- `ebnf`
  - Another important boundary exception
  - Dual-run EBNF uses build-script-resolved include paths, but not the same `has_generated_*` cfg pattern as the other env-driven grammar families
- `parser_registry.rs` / `embedding_api.rs`
  - These are the main mixed-boundary consumers
  - They are where bootstrap-vs-generated availability differences become host-visible behavior

Operational rule:
- When debugging a bootstrap-vs-generated mismatch, do not start from the consumer alone.
- First confirm:
  - which side of the boundary you are actually on
  - whether the relevant generated parser is truly available
  - whether the mismatch is in parser behavior itself or only in boundary wiring/exposure

## Grammar-Family Asymmetry Map
- `systemverilog`
  - A major env-driven generated-parser family
  - It also carries unusually heavy surrounding proof surface through aggregate, status, semantic-scope, roundtrip, and failure-context consumers
  - It now also has a dedicated formal-exhaustive-closure proof sidecar whose provenance is preserved end to end through family-status, family-status-contract, aggregate SOTA telemetry, and combined telemetry
  - The current machine-checked family-status row remains `Mostly Done` with `7` total closure criteria, `4` satisfied, `3` unsatisfied
  - The current retained blockers are now explicit and should drive the next SystemVerilog slice directly:
    - `syntax_closure_gate_status=fail failure_count=2`
    - `shadow_parser_rejections_total=3 > 0`
    - `focused_replay_target_count=2550 > 0`
- `systemverilog_preprocessor`
  - Also env-driven, but not just a smaller copy of parser-only families
  - Its runtime semantics include macro expansion, include handling, conditional policy, source mapping, diagnostics, and strict-promotion-adjacent behavior
  - The latest retained closure reduction came from one parser-side fix plus two shared parser+stimuli-safe grammar tightenings:
    - [ast_based_generator.rs](rust/src/ast_pipeline/ast_based_generator.rs) now disables auto layout skipping for regex tokens in the `systemverilog_preprocessor` generated parser family, using the generator's normalized PascalCase grammar identifier instead of only the raw underscore file stem
    - [systemverilog_preprocessor.ebnf](grammars/systemverilog_preprocessor.ebnf) now requires `condition_expr` in `pp_elsif_branch`, which closes illegal bare `` `elsif`` generation without narrowing valid preprocessor syntax
    - that same grammar now routes the line-oriented non-conditional directives `undef`, `include`, `timescale`, `default_nettype`, `celldefine`, and `endcelldefine` through `directive_comment_tail := inline_trivia line_comment?` instead of the broader `directive_tail`
  - Root causes:
    - preprocessor directive rules already encode same-line trivia explicitly via `inline_trivia`
    - the generated parser was still auto-skipping layout before regex tokens, so line-oriented regex rules like `directive_tail` could hop across a newline and swallow the following directive line
    - the grammar also still allowed `pp_elsif_branch` with no condition expression, which let the stimuli lane emit syntactically invalid bare `` `elsif`` lines
    - even after those two fixes, the broader `directive_tail` allowance on line-oriented non-conditional directives still left a narrower same-line directive-chaining seam that needed a more surgical shared-tail contract
  - Fresh focused proof on the retained shape now records `parseability_attempts_total=37`, `parseability_accepted_total=36`, `parseability_rejected_total=1`, `parseability_parser_rejections_total=1`, `parseability_counterexamples_captured_total=1`, `stage0_target_count=3`, `stage1_target_count=2`, and `final_targets=0` in the preprocessor aggregate contract lane
  - The higher-level proof readers are now aligned on that same retained aggregate: formal closure, family status, lightweight SOTA telemetry, and combined telemetry all now mirror the `1`-reject / `final_targets=0` baseline instead of the older stale `2`-reject snapshot
  - The direct minimal parser repros now pass on the rebuilt generated adapter too:
    - `/*a*/\`ifdef A`
    - `/*a*//*b*/\`ifdef A`
    - `/*a*/ /*b*/\`ifdef A`
  - The focused sample corpus no longer emits bare `` `elsif`` lines
  - Important reuse nuance: the stable main-SV aggregate reuse surface is `rust/target/sv_parser_aggregate_contract_gate_json_proof`, and the lightweight SOTA reuse surfaces for failure-context and roundtrip are the `*_json_proof` directories, not the plain gate directories
  - Two tempting shared parser+stimuli changes were explicitly rejected during this slice:
    - excluding backticks from `directive_tail` made the measured rejection surface worse, so future work should not retry that narrowing blindly
    - the broader all-directives `directive_line_tail := inline_trivia line_comment?` refactor also made the measured rejection surface worse, so future work should keep the narrower `directive_comment_tail` scope instead of broadening it again by default
- `vhdl`
  - An env-driven generated-parser family with a comparatively cleaner parser-family seam than SV
  - In practice it is strongly coupled to quality/parseability, strict-promotion, and now a dedicated formal-exhaustive-closure proof surface
  - The current machine-checked family-status row is now `Done`, with `10/10` closure criteria satisfied and no remaining tracked blocker debt. The refreshed direct quality, family-contract, formal-closure, family-status, family-status-contract, and combined-telemetry sidecars now all agree on the retained closed baseline.
  - Fresh direct VHDL quality proof on the retained Rust-side slice now records:
    - `closed_loop_initial_targets=247`
    - `closed_loop_replay_targets=0`
    - `closed_loop_parseability_shadow_parser_rejections_total=0`
    - `quality_parseability_generation_parser_rejections_total=0`
  - The refreshed machine-checked family blocker surface is now empty:
    - `quality_closed_loop_replay_targets=0`
    - `strict_promotion_primary_blocker=none`
    - `strict_promotion_trial_passed=3`
  - The parser-backed generation side of the current canonical family-quality surface is now green (`attempts=8`, `accepted=8`, `rejected=0`), realistic-corpus parity is now `13` expected pass / `1` expected fail with matching observed totals, and strict promotion is still green (`trial_passed=3`, `primary_blocker=none`)
  - The new VHDL formal-closure gate is now green off the checked-in external-corpus-backed proof surface, and the family-status / family-status-contract / SOTA / combined-telemetry stack preserves that provenance end to end
  - Recent VHDL work also reinforced the general EBNF steering rule: grammar changes must be classified as parser-only, stimuli-only, or shared parser+stimuli changes before they are kept. The retained VHDL slice (`--enforce-word-boundary-spacing`, `trivia` priority bias, newline-terminated line comments, and explicit `end process` / `end procedure` / `end function` body endings) improved the shared family-quality surface; a temporary `wait until` grammar broadening was intentionally reverted because it improved one parser-facing case while worsening replay-shadow parser debt
  - Two additional directions are now explicitly rejected rather than merely “not yet landed”:
    - a shared `stimuli_generator.rs` direct-probe rebias worsened replay debt from `11` to `30`
    - broader VHDL branch-steering experiments either worsened replay debt (`11 -> 17`) or made replay materially more expensive without yielding a keepable result
  - The current preferred tactic for the remaining VHDL replay targets is now the new branch-level triage tool plus targeted generator-side interventions instead of more blanket sample-hint sweeps:
    - [coverage_gap_triage.rs](rust/src/bin/coverage_gap_triage.rs) joins the gap report, coverage report, and grammar AST into one readable triage surface
    - the verified current VHDL run shows:
      - `trivia#line_comment` is a real selection-bias seam
      - `actual_parameter_element#range_expression` is part of a shared `range_expression` dependency failure
      - `actual_part#expression` is part of a shared `expression` dependency failure
    - the latest retained generator-side win now reflects that diagnosis directly:
      - dependency-blocked target branches are no longer failure-throttled before their still-targeted referenced rules record any success history
      - still-targeted OR branches that fail only on local depth exhaustion now get one temporary depth-slack retry during plain target driving
      - that retry is explicitly disabled during validation-aware target driving so replay-shadow parseability stays on the stricter canonical surface
      - `priority_first` branch selection now gives unresolved unseen target branches a one-shot probe bias without altering ordinary priority ordering
    - operational consequence:
      - the tracked VHDL contract is now closed
      - future VHDL work should preserve this closed family/status/aggregate baseline unless the contract is intentionally widened
  - The proof-plumbing caveat from the first `9`-target refresh attempt is now resolved in the retained gate path:
    - [vhdl_stimuli_quality_gate.sh](rust/scripts/vhdl_stimuli_quality_gate.sh) now isolates a state-local `CARGO_TARGET_DIR`
    - nested strict-promotion refreshes no longer clobber the adapter-backed generated `ast_pipeline` / `parseability_probe` binaries
    - future VHDL work should spend its effort on reducing the remaining replay targets, not on this normalized refresh seam
- `regex`
  - An env-driven generated-parser family, but operationally closer to the EBNF frontend world than the HDL families
  - Dual-run/frontend/stimuli closure surfaces matter a lot here, so parser-family work often crosses into ingestion and diagnostic tooling
  - It now also has a dedicated regex-only family stimuli contract (`rust/test_data/grammar_quality/regex_family_stimuli_contract.json`), so the canonical regex family/status/aggregate sidecars refresh from a family-local parser-backed baseline instead of piggybacking on the broader shared non-annotation stimuli contract
  - The current published regex family proof stack is now on the refreshed parser-family contract baseline (`5911` attempts, `5197` accepted, `714` diagnostic target-drive parser rejections, `804 -> 0` targets after `6526` target-drive attempts), and the machine-checked regex family status computes `Done` with `8/8` closure criteria satisfied
  - It now also has deterministic parser-rejection triage sidecars in its family-contract and aggregate proof stack, so regex closure work can talk about dominant failing sample/error/location buckets instead of only total rejection counts
  - It now also has a checked-in broader-corpus proof gate over the regex stress corpus (`44` executed, `44` pass, `0` fail in the current measured slice), and the formal-exhaustive-closure gate is now green because that broader-corpus proof surface exists
  - It now also has a maintained external-corpus acquisition starter under `regex_corpus_bundle/`; that bundle is deliberately separate from the closed regex family-status math and should be treated as the canonical future widening lane for PCRE2-first external hardening rather than as a replacement for the current checked-in `stress_tests.json` proof slice
  - The downstream regex host contract is now materially stronger than the first release slice: `embedding_api.rs` now publishes a regex parser release version, a regex integration-contract version, explicit generated-backend requirement metadata, a regex AST-dump schema version, and machine-localizable parse-failure locations through `ParseDiagnostic.location`
  - The current RGX handoff is parser release `1.1.29` / integration contract `1.1.31`; legal deep PCRE2 repros such as `80` nested captures plus `\80` and recursive named-group interpolation patterns still parse through the generated host path without stack abort, and the latest source-derived audit adds grammar coverage for PCRE2 short Unicode property escapes, quoted class literals, quoted class range endpoints, escaped quoted-literal body characters, bounded variable-length lookbehind, Unicode capture names, orphan class `\E`, dedicated `\C` single-code-unit escape transport, callout-prefixed conditional assertions, PCRE2 POSIX word-boundary aliases, UTF width start-option aliases, and callout/lookaround/script-run/scan-substring/modifier/escape forms while tightening compile-contract false accepts around unbounded lookbehind, malformed names, empty substantive class bodies, plain class `\N`, nonliteral class range endpoints, decoded escaped class-range endpoint ordering including bare octal `\NNN`, false unboundedness from declarative `DEFINE` conditionals inside lookbehind, and scan-substring references that truly do not exist in the whole pattern. The AST schema remains `1`
  - PCRE2-conformance fixes should follow the source-of-truth workflow captured in `docs/reference/REGEX_BOOTSTRAP_ARCHITECTURE.md`: prose docs first, `pcre2_compile.c` for edge cases, and PCRE2 `testdata/testinput*` for executable regression truth
  - Recent real-world regex follow-ups showed why this family is so frontend-coupled: fixing quoted-terminal escape decoding in `ebnf_frontend.rs`, widening `literal_char` just enough for `:` and `/`, deliberately allowing an empty top-level regex, and then disabling implicit layout skipping in generated regex parsers were enough to turn the checked-in `url_pattern`, `empty_regex`, and leading-whitespace quantifier false negatives green without changing the higher-level proof architecture
  - The final regex blocker turned out to be in the stimuli engine rather than the parser itself: alternate-entry helper probes inside `generate_until_targets_with_filter` now retain helper-rule coverage progress even when those helper outputs fail the primary-entry parseability filter, so regex target driving no longer spins by rolling back legitimate helper coverage
  - It now also has a public embedding seam in `embedding_api.rs`, but that public surface should not be mistaken for complete parser-family closure by itself
  - It now also has a dedicated downstream integration contract doc plus a regex-specific host contract gate layered on top of the generic embedding API gate, so downstream consumers no longer have to reconstruct the regex host promise from the generic embedding contract alone; RGX-style consumers can now pin both parser release version and contract version explicitly when integrating or reporting bugs
- `ebnf`
  - Not just another generated runtime parser family
  - It sits at the ingestion/frontend edge, uses build-script-resolved include paths, and has its own dual-run diagnostic shape rather than the standard `has_generated_*` family contract
- `return_annotation` / `semantic_annotation`
  - Important tracked-generated exceptions
  - They live under `generated_parsers`, but come from checked-in generated sources and typed-annotation workflows rather than the env-driven grammar-family path model
- `json` / `rtl_const_expr`
  - Supporting generated-parser families
  - They matter for build/registry completeness, but they are not the main day-to-day closure-driving families in the same way as SV, VHDL, and regex

Operational rule:
- Do not assume one family’s build, validation, or proof-plumbing shape generalizes cleanly to another.
- Before copying a fix pattern across families, check whether the source family is:
  - env-driven parser runtime
  - frontend/ingestion
  - tracked generated annotation support
  - or a supporting parser family with lighter operational ownership

## Known Traps And False Assumptions
- “If Cargo lists the binary, the runtime path must be available.”
  - False here.
  - A binary can exist in Cargo while the relevant generated parser path is still unavailable because the matching `has_generated_*` cfg was never emitted.
- “Raw AST and generation-input AST are basically the same artifact.”
  - False here.
  - The normalization layer is substantial; many downstream changes only make sense once you know which side of that boundary is wrong.
- “Parser-backed AST dumps and generation-input AST dumps are the same debugging surface.”
  - False here.
  - One is a runtime parser output, the other is a pre-generation pipeline artifact.
- “A successful compile is enough validation for a Rust change.”
  - Usually false here.
  - In this repo, the next real consumer of the artifact is often the meaningful validation seam.
- “If `parseability_rejected_total=0`, the parseability counterexample sidecar must be empty.”
  - False here.
  - Counterexample triage can still retain target-drive filter evidence from rejected alternate-entry helper probes even when the canonical primary-entry parseability totals are fully green.
- “If an annotation leaf suite or one SC gate passes, the repo-level annotation proof claim is done.”
  - False here.
  - Aggregate annotation claims usually live one layer higher in `annotation_contract_gate`, `semantic_full_contract_gate`, `return_annotation_support_gate`, or `annotation_stimuli_quality_gate`.
- “Shell gates are just wrappers around the real Rust product.”
  - False here.
  - The shell proof layer is part of the effective product contract because it defines and preserves the executable proof surfaces.
- “`summary.txt` is just for humans and `summary.json` is optional.”
  - False for the shipped proof spine.
  - Higher layers increasingly depend on the JSON sidecar, while TXT/JSON parity remains part of the contract.
- “Every generated grammar family follows the same include-path and cfg rules.”
  - False here.
  - `ebnf` and the tracked annotation parsers are important exceptions.
- “The modern surface is always in `rust/src/main.rs`.”
  - Not always.
  - A lot of live operational behavior sits in `rust/src/bin/*.rs`, `rust/scripts/*.sh`, and feature/cfg wiring rather than the main orchestration CLI alone.

## Testing And Verification Shape
- The test surface is not only `cargo test`.
- The codebase relies on:
  - unit/integration tests in Rust modules
  - JSON-driven round-trip suites
  - parser-family quality and contract gates
  - SOTA/aggregate proof surfaces
- `rust/src/test_runner/round_trip_tests.rs` is the more modern JSON-driven testing spine.
- `rust/src/test_registry.rs` and `rust/src/test_discovery.rs` look older and more limited by comparison.

Assessment:
- The repo is very strong on proof surfaces.
- The downside is that the test ecosystem is mixed-generation and not fully consolidated behind one obvious canonical layer.

## Validation Ladder By Change Type
Treat this as a representative ladder, not a claim that every task needs every step.

- Docs-only or continuity-only change
  - usually enough:
    - `git diff --check`
- Build-shape or feature/cfg change
  - start with:
    - `cargo check --manifest-path rust/Cargo.toml --bins`
  - then usually add the relevant feature shape, for example:
    - `cargo check --manifest-path rust/Cargo.toml --features generated_parsers --bin ast_pipeline`
    - `cargo check --manifest-path rust/Cargo.toml --features generated_parsers --bin parseability_probe`
    - `cargo check --manifest-path rust/Cargo.toml --features "generated_parsers ebnf_dual_run" --bins`
- Parser-registry or embedder-facing change
  - usually re-check:
    - `cargo test --manifest-path rust/Cargo.toml --features generated_parsers --lib parser_registry`
    - `cargo test --manifest-path rust/Cargo.toml --features generated_parsers --lib embedding_api`
- Parser behavior or typed-AST conversion change
  - usually re-check:
    - focused `test_runner` suites
    - focused `parseability_probe --parse ...`
    - any generated-vs-bootstrap parity path that the touched parser family relies on
  - if the parser is `return_annotation` or `semantic_annotation`, usually also add:
    - `make -C rust annotation_contract_gate`
    - and the nearest annotation aggregate proof surface above the touched seam
- EBNF frontend or dual-run change
  - usually re-check:
    - `cargo test --manifest-path rust/Cargo.toml --features ebnf_dual_run --lib ebnf_frontend::tests`
    - `cargo check --manifest-path rust/Cargo.toml --features "generated_parsers ebnf_dual_run" --bin ebnf_dual_run_diff`
    - focused `ebnf_dual_run_diff` execution if the issue is parse/full-parse drift
- Stimuli, coverage, or gap-report change
  - usually re-check:
    - focused `ast_pipeline --generate-stimuli ...`
    - parseability report generation when the change affects parser-backed validation
    - the nearest grammar-quality or family-contract gate that consumes those artifacts
  - for annotation-focused stimuli work, usually also add:
    - `make -C rust annotation_stimuli_quality_gate`
    - and, if annotation semantics changed too, `make -C rust annotation_contract_gate`
- Proof-sidecar or gate change
  - usually re-check:
    - the touched gate directly
    - `bash rust/scripts/ci_workflow_local_gate.sh`
  - and when the change affects aggregate proof flow:
    - the nearest family-status / family-status-contract / combined-telemetry / SOTA reader above it
  - for annotation-proof changes, the practical aggregate readers are usually:
    - `annotation_contract_gate`
    - `semantic_full_contract_gate`
    - `return_annotation_support_gate`
    - `annotation_stimuli_quality_gate`

Operational rule:
- Prefer the smallest validation slice that still crosses the seam you changed.
- In this repo, “build passes” is often weaker than “the next consumer of the artifact still agrees.”

## Strengths
- Strong architecture around determinism, observability, and machine-checkable proof.
- Clear Rust-first integration posture with explicit bootstrap escape hatches rather than hidden hand-written exceptions.
- Stable consumer-facing API design in `embedding_api.rs`.
- Sophisticated stimuli/coverage/gap machinery that matches the project’s closure doctrine.
- Good generated-parser integration model in `build.rs`.
- Real policyfulness in the SV preprocessor instead of a shallow text-prepass design.

## Main Risks And Technical Debt
- Complexity concentration in:
  - `stimuli_generator.rs`
  - `ast_based_generator.rs`
  - `annotation_validator.rs`
  - `mod.rs`
  - `main.rs`
- Repeated grammar/backend/profile adapter logic across:
  - `parser_registry.rs`
  - `embedding_api.rs`
  - selected binaries / CLI surfaces
- Bootstrap/generated duality remains necessary but expensive to reason about.
- Semantic support is powerful but distributed across several coupled files, which raises the cost of safe changes.
- The shell-gate layer is large enough that “the Rust codebase” now effectively includes a substantial Bash proof system.

## Steering Implications
- Future implementation should keep treating parser generation, stimuli closure, and proof/gate output as one system.
- Effort spent only on parser acceptance without preserving observability and proof surfaces will fight the project’s actual architecture.
- Refactors should aim to reduce concentration without weakening the current proof doctrine.

## Architectural Invariants Worth Preserving
- Explicit bootstrap-vs-generated boundaries
  - Keep parser availability visible through features, env-driven paths, and `build.rs`/cfg wiring rather than hidden fallback behavior.
- Stage-distinct artifacts
  - Preserve the distinction between:
    - raw/frontend AST
    - normalized generation-input AST
    - generated parser source
    - runtime parser output
    - proof sidecars
  - A lot of bugs come from collapsing these into one mental bucket.
- Machine-readable proof contracts
  - Preserve `summary.json` as a first-class contract surface alongside `summary.txt`, especially on the shipped proof spine.
- Upstream source-of-truth repair
  - Prefer fixing the emitting layer or canonical producer over patching an aggregate consumer to “make the drift disappear.”
- Seam-crossing validation
  - Keep validating at the next real consumer of the changed artifact; compile success alone is often not enough in this repo.
- Parser behavior plus observability
  - Avoid changes that improve acceptance or throughput by quietly weakening telemetry, counterexample capture, gap reporting, or proof surfaces.
- Shell proof layer as product surface
  - Treat `rust/scripts/*.sh` and their emitted sidecars as part of the effective Rust-owned contract, not as optional wrappers around “the real code.”

## Refactor Patterns That Fit This Codebase
- Split by artifact boundary, not by raw line count
  - Good split examples:
    - parser emission vs semantic-runtime emission vs telemetry/report emission
    - raw/frontend AST handling vs normalized-AST handling
    - report production vs report aggregation
  - Why:
    - this repo’s real seams are artifact and contract boundaries, not just file size
- Stabilize outputs before moving orchestration
  - Prefer extracting helpers around stable outputs first:
    - generated parser source
    - parseability/coverage reports
    - `summary.txt` / `summary.json`
  - Why:
    - if outputs stay stable, you can move internals without multiplying downstream drift
- Refactor one contract seam at a time
  - Prefer:
    - `build.rs` + one direct consumer
    - one report producer + one immediate gate consumer
    - one dispatch layer + one public API layer
  - Avoid broad simultaneous rewrites across:
    - feature/cfg shape
    - registry exposure
    - proof-gate consumers
  - Why:
    - this repo has enough coupled seams that multi-seam rewrites get hard to validate honestly
- Replace repeated branching with narrow shared adapters
  - Best targets:
    - repeated grammar/backend/profile dispatch
    - repeated report-path / proof-surface extraction
    - repeated parser-availability checks
  - Why:
    - duplicated branch logic is a bigger long-term risk here than small helper indirection
- Add proof before deleting carryover paths
  - If a refactor replaces a consumer or proof seam, lock the new behavior first with the nearest meaningful validation or local-CI assertion before removing the old path
  - Why:
    - subtractive cleanup is safer once the replacement seam is already machine-checked
- Keep the next consumer in the loop while refactoring
  - When extracting or moving code, validate at the first downstream consumer that depends on the moved artifact rather than stopping at compile success
  - Why:
    - many regressions here are contract regressions, not syntax or type-check failures

## Recommended Refactor Priorities
- Split `rust/src/main.rs` into subcommand or mode-focused modules.
- Break `rust/src/ast_pipeline/stimuli_generator.rs` into smaller policy/reporting/runtime units.
- Break `rust/src/ast_pipeline/ast_based_generator.rs` into emitter-focused submodules:
  - parser struct/runtime emission
  - semantic runtime emission
  - recovery/coverage telemetry emission
  - per-rule method emission
- Reduce repeated dispatch logic by introducing a more unified grammar/backend adapter layer shared by:
  - `parser_registry.rs`
  - `embedding_api.rs`
  - CLI/binary consumers
- Clarify which test layers are canonical and which are legacy carryovers.

## Open Architecture Questions
- How far can the giant engine files be split without weakening artifact/proof seams?
  - Main hotspots:
    - `rust/src/main.rs`
    - `rust/src/ast_pipeline/mod.rs`
    - `rust/src/ast_pipeline/ast_based_generator.rs`
    - `rust/src/ast_pipeline/stimuli_generator.rs`
- Can parser dispatch be unified further without hiding important feature/cfg/build-path realities?
  - Main seam:
    - `rust/src/parser_registry.rs`
    - `rust/src/embedding_api.rs`
    - CLI consumers
- Which older/carryover test layers still provide unique value, and which are just maintenance drag?
  - Main seam:
    - `test_runner` versus `rust/src/test_registry.rs` / `rust/src/test_discovery.rs`
- How much of the proof spine should remain in shell versus moving into Rust?
  - Main seam:
    - `rust/scripts/*.sh`
    - Rust-produced artifacts and machine-readable reports
- Which family asymmetries are permanent design facts versus candidates for normalization?
  - Main seam:
    - `ebnf`
    - tracked annotation parsers
    - env-driven generated families
    - companion crates like `rtl_const_expr` / `rtl_frontend`
- What is the long-term relationship between the main `rust/` crate and the companion RTL crates?
  - Main seam:
    - `rust/`
    - `rtl_const_expr/`
    - `rtl_frontend/`

Operational rule:
- Treat these as active design questions, not settled doctrine.
- If a future task materially resolves one of them, refresh this section instead of letting the answer stay implicit in scattered commits.

## What To Re-Check At The Start Of A New Session
- Whether the hotspot files and their responsibilities have materially shifted.
- Whether new grammar families or generated parser integrations changed the build/registry shape.
- Whether bootstrap vs generated boundaries moved.
- Whether the public consumer seam changed:
  - embedding API
  - parser registry
  - grammar-profile coverage
- Whether the proof/gate layer changed enough that this document’s description of the operational surface is stale.
- Whether the main current risks are still:
  - concentrated module size
  - repeated adapter seams
  - bootstrap/generated maintenance cost
  - mixed-generation testing layers

## Session-Start Sanity Probes
Use these as cheap orientation probes before deeper Rust work, not as a replacement for task-specific validation.

- `git status --short`
  - Confirms whether unrelated dirt, generated artifacts, or untracked directories are already present before you start attributing odd behavior to the code.
- `rg -n "^\\[\\[bin\\]\\]|^\\[features\\]" rust/Cargo.toml`
  - Re-checks whether the binary and feature surface still matches this document’s assumptions.
- `rg -n "PGEN_[A-Z_]+_PARSER_PATH|has_generated_" rust/build.rs rust/src/lib.rs`
  - Re-checks the generated-parser availability contract quickly without re-reading the full files.
- `rg --files rust/src/bin`
  - Re-confirms the active Rust utility-binary surface.
- `sed -n '1,120p' docs/reference/RUST_CODEBASE_ANALYSIS.md`
  - Fast check that the live analysis doc still presents the same top-level structure and hasn’t fallen behind a major architectural shift.
- If the task is proof/gate-heavy:
  - `rg -n "summary\\.json|summary\\.txt|sota_exit_gate|combined_telemetry|family_status" rust/scripts`
  - Quick way to confirm whether the proof-sidecar vocabulary or aggregate-gate surface has drifted materially since the last session.

## Limits Of This Snapshot
- This assessment came from a deep source read and structural review.
- It is not a benchmark report.
- It is not a full dynamic validation run of all Rust binaries and all gates.
- It should therefore be refreshed when runtime evidence materially changes the picture.

## Current Closure Steering Notes
- The active HDL closure lane is now main `systemverilog`, not `vhdl` or `systemverilog_preprocessor`.
  - `vhdl` is on a clean retained baseline.
  - `systemverilog_preprocessor` is on a clean retained baseline.
  - `regex` is closed for the currently published contract.
- Main SystemVerilog aggregate proof now has a better retained counterexample surface.
  - `rust/scripts/sv_parser_aggregate_contract_gate.sh` now summarizes:
    - unique/dominant `primary_entry_rule`
    - unique/dominant `generation_entry_rule`
    - unique/dominant `entry_mode`
    - for both generation and replay-shadow counterexample triage
  - important reuse rule:
    - prefer the top-level `rust/target/sv_stimuli_quality_gate` state as the canonical cheap reuse surface
    - older nested aggregate artifacts are not the default reuse source anymore
  - important compatibility rule:
    - current reusable `sv_stimuli_quality_gate` reports may be lean and omit embedded `counterexamples`
    - aggregate proof now normalizes those omissions to zero/empty, so `<none>` / `0` on the new entry-context fields is acceptable on lean artifacts
- Downstream SOTA/combined-telemetry proof refreshes are now aligned with that incremental aggregate surface.
  - `rust/scripts/sota_exit_gate.sh` and `rust/scripts/sv_combined_telemetry_contract_gate.sh` now default missing main-SV entry-context fields from older aggregate summaries to:
    - `0` for counts
    - `<none>` for dominant-value fields
  - practical effect:
    - cheap reuse-backed proof refreshes stay viable even when the aggregate summary predates the new entry-context keys
    - do not waste a session rebuilding the full main-SV aggregate proof solely to make downstream summaries stop showing `<none>` / `0`
- A fresh main-SV generator shortcut was tried and rejected.
  - rejected idea:
    - force a newline before any non-line-start `//...` segment in `StimuliGenerator::append_generated_segment(...)`
  - retained evidence:
    - adapter-backed direct probe regressed from `112/180 accepted, 68 parser rejects`
    - to `111/180 accepted, 69 parser rejects`
  - steering consequence:
    - do not reopen the broad line-comment normalization path
    - the remaining main-SV parser debt still needs a narrower seam than “newline all same-line comments”
- The current best retained main-SV stimuli seam is `escaped_identifier`, not global comment handling.
  - retained change:
    - `grammars/systemverilog.ebnf`
    - `@sample: "\\foo "` above `escaped_identifier := trivia /\\[!-~]+/`
  - retained evidence:
    - adapter-backed direct probe improved from `111/180 accepted, 69 parser rejects`
    - to `123/181 accepted, 58 parser rejects`
  - steering consequence:
    - the next honest main-SV step is a full `sv_stimuli_quality_gate` rerun to see whether that focused win survives the retained proof lane
    - until that rerun lands, treat this as a focused direct-lane improvement, not a refreshed main-SV status row
- VHDL still has useful local triage tooling if it ever needs to be revisited.
  - non-default `--entry-rule` plus `--validate-parseability` stays intentionally rejected in `ast_pipeline`; full-entry validation only
  - `coverage_gap_triage` plus `top_failure_reasons` remains the right resume surface if VHDL closure is reopened later

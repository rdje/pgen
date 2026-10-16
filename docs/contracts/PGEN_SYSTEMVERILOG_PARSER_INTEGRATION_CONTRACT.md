# docs/contracts/PGEN_SYSTEMVERILOG_PARSER_INTEGRATION_CONTRACT.md

## Purpose
Define the downstream integration contract for PGEN's main `systemverilog` parser family.

This is the document downstream projects such as Nexsim should read first when deciding how to embed the PGEN systemverilog parser.

> **Current-state note (2026-10-16, `SV-CORPUS-GRAD.50` — GRAMMAR + PINS + ANSWER-KEY tier, release `1.0.253`, ledger `SV-0138`, **schema `34` unchanged**, accept set NARROWS under `verilog_2005` by two shapes; `sv_2017` / `sv_2023` / `sv_2023_tolerant` untouched on every instrument).** Under `verilog_2005` an instance's EMPTY parameter value assignment (`mod #() m1(…)`; IEEE 1364-2005 A.4.1.1 `# ( list_of_parameter_assignments )`, the list never empty, `docs/verilog/2005/txt/section-Annex_A-normative-formal-syntax-definition.txt:399-402`) and a SECOND packed range on a reg, a named-block reg, a function range, an ANSI port, an output reg port or a task port (A.2.1.3 `reg [ signed ] [ range ]` `docs/verilog/2005/txt/section-Annex_A-normative-formal-syntax-definition.txt:172`, A.2.1.2 `:133-139`, A.2.6 `:268`, A.2.7 `:296-305`; `range` `:252`) are REFUSED from `1.0.253`; IEEE 1800 derives both (`docs/systemverilog/2017/md/section-41-data-read-api.md:1795`; `:842`, `:867`), which is why the same text is accepted under the SV profiles. The node of a legal 1364 instance or declaration is the one you already have: `parameter_value_assignment_v2005` renders `{params}` under `kind: "sv_2017"` (the shape's name) and `packed_dimension_list_v2005` renders the same `dims` list as the repetition did (byte-identical on the controls and on every accepted corpus tree — AST sweeps `ast_moved=0` on all three lanes). Ten verilog_2005-lane corpus rows the vendored iverilog had keyed `must_accept` (`-g2005` accepts both constructs, a tool extension) now stop on the construct and are re-keyed `must_reject` by the adjudicator's EDITION-PACKED-DIMENSION pass with the clause in their basis; the lane's bar stays 0. Pins: two flipped, five new invalid (one per site), three controls, one `.3.48` control narrowed to the IEEE 1800 profiles; one sv2v row (`test/core/string_byte_order.v`, a second packed range behind an `include`) pinned by the clause and deferred to the pipeline reading as a chained row.
>
> **Current-state note (2026-10-09, `SV-CORPUS-GRAD.48` — GRAMMAR + PINS tier, release `1.0.252`, ledger `SV-0137`, **schema `34` unchanged**, accept set NARROWS under `verilog_2005` by one DECLARATION shape; `sv_2017` / `sv_2023` / `sv_2023_tolerant` untouched).** A module's parameter port list under `verilog_2005` is `#( parameter … { , parameter … } )` and nothing else — IEEE 1364-2005 A.1.3 `module_parameter_port_list ::= # ( parameter_declaration { , parameter_declaration } )` (`docs/verilog/2005/txt/section-Annex_A-normative-formal-syntax-definition.txt:38`), A.2.1.1 `parameter_declaration ::= parameter [ signed ] [ range ] list_of_param_assignments | parameter parameter_type list_of_param_assignments` (`:126-128`): a declaration without `parameter` (`#(A = 1)`, `#(integer A = 1)`, `#(parameter real A = 1.0, integer B = 2)`), a `localparam` inside the list and an empty `#()` are REFUSED from `1.0.252` (ledger `SV-0137`; through `1.0.251` the one `parameter_port_list` rule carried IEEE 1800-2017 A.1.3's production on every profile). The node is unchanged: `{kind: "declarations", items: [parameter_port_declaration …]}` — the `verilog_2005` twin renders the same kind and the same items, and every tree the profile still accepts is byte-identical (`AST-IDENTITY-SWEEP: profile=verilog_2005 files=2943 both_accepted=2558 identical=2558 ast_moved=0 verdict_moved=3 both_nonaccepting=382`). **Consumers of `sv_2017` / `sv_2023` / `sv_2023_tolerant`: nothing to change** — verilog_2005 pass_to_fail=3 fail_to_pass=0; sv_2017 0 / 0; sv_2023 0 / 0; `AST-IDENTITY-SWEEP: profile=sv_2017 files=16336 both_accepted=10022 identical=10022 ast_moved=0 verdict_moved=0 both_nonaccepting=6314`; `AST-IDENTITY-SWEEP: profile=sv_2023 files=16336 both_accepted=10048 identical=10048 ast_moved=0 verdict_moved=0 both_nonaccepting=6288`; `PIPELINE-MOVERS[sv_2017] rows_before=16336 rows_after=16336 moved=0 added=0 lost=0`; `PIPELINE-MOVERS[sv_2023] rows_before=16336 rows_after=16336 moved=0 added=0 lost=0`. **Consumers of `verilog_2005`**: text only IEEE 1800 admits in a parameter port list is refused (`MOVED verilog_2005 ('iverilog', 'stimuli/sv/subs/iverilog/ivtest/ivltests/parameter_omit1.v') pass -> fail`; `MOVED verilog_2005 ('iverilog', 'stimuli/sv/subs/iverilog/ivtest/ivltests/parameter_omit2.v') pass -> fail`; `MOVED verilog_2005 ('iverilog', 'stimuli/sv/subs/iverilog/ivtest/ivltests/parameter_omit3.v') pass -> fail` — the three ivtest `parameter_omit*.v` rows, each headed *"In Verilog this is not allowed"*). Pins `ADJUDICATION-REPROS: checked=1997 armed=236 listed=972 multi_profile_rows=752 preprocess_rows=41 tolerated_rows=8 superset_rows=753 failures=0`. The recognized certificate union `canonical UNKNOWN=14, union UNKNOWN=0`; the Verilog-2005 conformance gate `passed`. ⛔ Still admitted and routed: an EMPTY parameter VALUE assignment on an instance (`mod #() m1(…)`; 1364-2005 A.4.1.1 `:399-402`) — `SV-CORPUS-GRAD.50`, watched by `accepts_invalid_v2005_instance_empty_parameter_value_assignment.sv`.
>
> **Current-state note (2026-10-07, `SV-CORPUS-GRAD.47` — GRAMMAR + PINS tier, release `1.0.251`, ledger `SV-0136`, **schema `34` unchanged**, accept set WIDENS under `sv_2023` (and `sv_2023_tolerant`) by one LEXICAL shape; `sv_2017` / `verilog_2005` untouched).** A sized literal whose SIZE begins with `0` — `01'b0`, `007'h5`, `04'b1` — parses under `sv_2023`: IEEE 1800-2023 §5.7 Syntax 5-2 and A.8.7 read `size ::= unsigned_number` (`docs/systemverilog/2023/md/section-5-lexical-conventions.md:213`, `docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:3200`) and the edition defines no `non_zero_unsigned_number`; its §5.7.1 prose keeps the size nonzero (`:258`), so a ZERO size (`0'b1`, `0_'b1`) stays rejected on every profile. IEEE 1800-2017 §5.7 / A.8.7 and 1364-2005 §3.5 / A.8.7 read `size ::= non_zero_unsigned_number`, so `sv_2017` and `verilog_2005` keep rejecting it — the `1.0.235` note below stands for those two profiles and is CORRECTED for `sv_2023` (it had read 1800-2023 as agreeing with 2017; ledger `SV-0120`, corrected forward by `SV-0136`). The node is unchanged: `{kind: "number", body: {kind: "integral", body}}` with the source text as `body`, leading zeros kept — no new kind. ⛔ NOT a tolerance: the 2026-10-06 session had designed this spelling as the opt-in profile's sixth member on an ecosystem reading (`SV-CORPUS-GRAD.46`); the pins' citations, checked against the 2023 text, refuted that before a reading was spent, and the opt-in list is unchanged at five. **Consumers of `sv_2017` / `verilog_2005`: nothing to change** — zero rows moved on any raw lane, `AST-IDENTITY-SWEEP: profile=sv_2017 files=16336 both_accepted=10022 identical=10022 ast_moved=0 verdict_moved=0 both_nonaccepting=6314`, `AST-IDENTITY-SWEEP: profile=verilog_2005 files=2606 both_accepted=2283 identical=2283 ast_moved=0 verdict_moved=0 both_nonaccepting=323`, `PIPELINE-MOVERS[sv_2017] rows_before=16336 rows_after=16336 moved=0 added=0 lost=0`. **Consumers of `sv_2023`**: a previously refused file carrying this spelling now parses (friscv `rtl/friscv_alu.sv`, the one corpus file: `PIPELINE-MOVERS[sv_2023] rows_before=16336 rows_after=16336 moved=1 added=0 lost=0 MOVED (('friscv', 'stimuli/sv/subs/friscv/rtl/friscv_alu.sv'), 'fail', 'pass')`; `2023_only_pass=41 2017_only_pass=0`); every file accepted before yields a byte-identical tree (`AST-IDENTITY-SWEEP: profile=sv_2023 files=16336 both_accepted=10048 identical=10048 ast_moved=0 verdict_moved=0 both_nonaccepting=6288`). Pins `ADJUDICATION-REPROS: checked=1938 armed=236 listed=946 multi_profile_rows=733 preprocess_rows=38 tolerated_rows=8 superset_rows=734 failures=0`. The recognized certificate union `canonical UNKNOWN=14, union UNKNOWN=0`; the Verilog-2005 conformance gate `passed`.
>
> **Current-state note (2026-10-05, `SV-CORPUS-GRAD.45` — GRAMMAR + PINS tier, release `1.0.250`, ledger `SV-0135`, **schema `34` unchanged**, the OPT-IN profile `sv_2023_tolerant` gains its FIFTH spelling; the edition profiles unmoved).** Under `sv_2023_tolerant` — and under no other profile — a hierarchical NAME of three or more parts is accepted as the operand of `binsof(…)`: `binsof(cfg.vif.sig) intersect {1}`. It is the reference form of the dotted cross item the profile has admitted since `1.0.249`: a select expression has no other way to name an implicitly covered cross item, and every OpenTitan statement that writes it names an item of its own cross. Admitted by a call the director delegated (2026-10-05, the addendum of `docs/decisions/project_sv_opt_in_tolerance_profile.md`), on the ruling of `SV-CORPUS-GRAD.44` (IEEE 1800-2017 §19.6.1 / 1800-2023 A.2.11 derive a variable name, or a coverpoint name with an optional bin name, and nothing longer). It arrives under `bins_expression` as `{kind: "tolerated_hierarchical", body: {path}}` — `path` a standard `hierarchical_identifier`, the SAME shape the tolerated cross item carries. A TWO-part name inside `binsof` keeps its standard `cover_point` reading under every profile (by syntax it is `cover_point . bin`); an expression, a trailing select and a call inside `binsof` stay rejected under the tolerant profile, each a pin. **Consumers of `sv_2017` / `sv_2023` / `verilog_2005`: nothing to change** — zero rows moved on any raw lane (10 022 / 10 048 / 2 283 pass, unchanged), the corpus AST identity sweep `identical=10022 / 10048 / 2283, ast_moved=0, verdict_moved=0`, the shape contract 60 / 60 aligned, the binding parse-cost counters byte-identical (the alternative is one profile-gated rule), both strict readings of the pipeline lane `moved=0`. Under the tolerant reading exactly three rows move, the three files named before the measurement: `sysrst_ctrl_env_pkg.sv`, `core_ibex_fcov_if.sv`, `core_ibex_pmp_fcov_if.sv` (`tolerant_only_pass` 15 → 18, package files 6 → 7, published packages 282 → 283, members reached 315 → 337, `lost_tolerant=0`). Pins `ADJUDICATION-REPROS: checked=1881 armed=236 listed=919 multi_profile_rows=714 preprocess_rows=16 tolerated_rows=8 superset_rows=713 failures=0`. The certificate's recognized union is unchanged; under its own base the tolerant profile reads `total=1564 proof=23 witness=1532 UNKNOWN=9` — the new rule and the three `solve` rules are not witnessed by generation, for one measured reason (the witness planner's minimal body does not satisfy the rule's leading lookahead; `ENGINE-UNIVERSAL-SERVICES.120`), and are exercised by the pins and the corpus. ⛔ The `1.0.249` note below gave another reason for the three `solve` rules; it did not match the certificate's debug probe and is corrected here.
>
> **Current-state note (2026-10-03, `SV-CORPUS-GRAD.43` — GRAMMAR + FAMILY-COMPONENT + PINS tier, release `1.0.249`, ledger `SV-0134`, **schema `34` unchanged**, a NEW OPT-IN PROFILE `sv_2023_tolerant`; the edition profiles unmoved).** By director decision (2026-10-02, `docs/decisions/project_sv_opt_in_tolerance_profile.md`) the parser carries one opt-in profile: `sv_2023` plus a closed list of four simulator-tolerated spellings — `foreach (a[i][j])`, `solve … before` directly inside a `foreach` constraint, a dotted name as a `cross` item, a lifetime in a class method prototype — and nothing else. Ask for it by name (`parse_grammar_profile_named("systemverilog", "sv_2023_tolerant", …)`, `--profile sv_2023_tolerant`); it has no alias and is never a default. Each tolerated spelling arrives under its own `kind` (`foreach_tolerated_select`, `foreach_tolerated_solve`, `foreach_tolerated_select_solve`, `tolerated_hierarchical`, `pure_virtual_tolerated_lifetime`, `extern_tolerated_lifetime`; the shapes are tabled in the SV book's *Json Carrier* chapter). **Consumers of `sv_2017` / `sv_2023` / `verilog_2005`: nothing to change** — zero rows moved on any raw lane (sv_2017 10 022 / sv_2023 10 048 / verilog_2005 2 283 pass, unchanged), the corpus AST identity sweep `identical=10022 / 10048 / 2283, ast_moved=0, verdict_moved=0` on each, the shape contract 60 / 60 aligned, the parse-cost counters FLAT under the strict profiles (`Δentries 0 Δcommitted 0 Δmemo_hits 0`: each tolerated alternative is one profile-gated rule, asked for nothing where it is gated off). Under the tolerant profile the reading of the pipeline lane passes 15 rows and 6 OpenTitan package files that `sv_2023` rejects (`sec_cm_pkg`, `tl_agent_pkg`, `adc_ctrl_env_pkg`, `flash_ctrl_env_pkg` ×3), publishes four more packages and reaches 315 members, and loses nothing `sv_2023` accepts (`2023_only_pass=0`). Pins `listed=908 failures=0`, with a superset sweep (`superset_rows=711`) that reads every `sv_2023` pin under the tolerant profile. Held by doctrine 46 `SV-TOLERANT-PROFILE-SUPERSET`. The certificate's recognized union is unchanged (its universe is the rules live under `sv_2017`); under its own base the tolerant profile reads `total=1563 proof=23 witness=1532 UNKNOWN=8`, the three `solve`-tolerance rules among the unknowns and exercised by the pins instead.
>
> **Current-state note (2026-10-02, `SV-CORPUS-GRAD.39` — GRAMMAR tier, release `1.0.248`, ledger `SV-0133`, **schema `34` unchanged**, accept set WIDENS on `sv_2017` / `sv_2023` by one shape — a method call whose receiver is a `local::`-qualified name, `q.size() == local::q.size()` inside `randomize() with { … }` (OpenTitan `spi_host_seq.sv:25`, `i2c_base_seq.sv:181`); `verilog_2005` untouched).** `split_callable_receiver_scope` — the one rendering of A.8.4's `[ class_qualifier \| package_scope ]` prefix a CALLABLE receiver reaches (`.13c.2j`'s per-site attribution) — gains the `[ local :: ]` half of `class_qualifier`, the arm `primary_hier_scope_prefix` has carried since `.3.37`: IEEE 1800-2017 A.8.2 `method_call ::= method_call_root . method_call_body`, `method_call_root ::= primary`, A.8.4 `primary ::= [ class_qualifier \| package_scope ] hierarchical_identifier select`, `class_qualifier ::= [ local :: ] [ implicit_class_handle . \| class_scope ]`, §18.7.1 — derivable in both editions, no deviation. `local::x`, `local::q[i]`, `local::q.size` and `local::g()` parsed before and parse the same; the typed receiver carries `scope: {kind: "local_qualifier", scope: …}` where the package and class scopes already sat. The fifteen declarative separator `@sample`s that `ENGINE-UNIVERSAL-SERVICES.98` made redundant (fourteen `` `name `` directive keywords, `" / "`, `` `FOO\n ``) are removed — stimuli only, the language unmoved. Measured on the final bytes: `ADJUDICATION-REPROS: checked=1820 … listed=891 … failures=0`; the raw lanes `pass_to_fail=0 fail_to_pass=0`; ONE typed AST moves — a `local::`-qualified dotted name (`local::e.c2_f`, verilator `t_randomize_method_with_scoping.v:69`) now renders through the split callable receiver, the shape package- and class-qualified dotted names already had — every other accepted file byte-identical; one package `fail` → `pass` under the pipeline.
>
> **Current-state note (2026-10-01, `SV-CORPUS-GRAD.37` — GRAMMAR tier, release `1.0.247`, ledger `SV-0132`, **schema `34` unchanged**, accept set WIDENS on `sv_2017` / `sv_2023` by one shape — a `const`-first class property with a constructor initializer, `const local uvm_event_pool events = new("events");` (UVM 2020.3.1 `src/base/uvm_transaction.svh:443`); `verilog_2005` untouched; `SVPP-EXPANSION.5`(d) RULED AND CLOSED).** `class_property`'s const arm reads `( assign ( class_new | constant_expression ) )?`, `class_new` first. IEEE 1800-2017 A.1.9 gives that arm only a `constant_expression` (`const { class_item_qualifier } data_type const_identifier [ = constant_expression ] ;`) and `new(…)` is none — while the `local const …` order is derivable through `data_declaration`'s `class_new` initializer, an asymmetry the standard carries and every simulator smooths over; §8.19's prose reads "global constant class properties include an initial value as part of their declaration". PGEN accepted the UVM spelling through `1.0.218` only because `local` lexed as an identifier; from the Annex B reserved-word fix (`1.0.219`, 2026-09-04) it rejected `uvm_pkg.sv` — and everything built on UVM — at that line. The director's 2026-08-10 bounding ruling (the operative test first: a change must not make the parser reject input the ecosystem actually writes) decides it; the arm is a DOCUMENTED TOOL EXTENSION, on by default, recorded in the SV book's pipeline chapter and changelog. Typed shape unchanged: `{kind: "const", qualifiers, data_type, name, init}` with `init` the `class_new` object; six shape-neutral controls' typed ASTs byte-identical across the change. Measured on the final bytes: `ADJUDICATION-REPROS: checked=1814 … listed=888 … failures=0`; the raw lanes `pass_to_fail=0 fail_to_pass=0` and `ast_moved=0 verdict_moved=0` on all three; five UVM units `fail` → `pass` under the pipeline; the external-corpus proof surface `parse_pass_total: 14` / `parse_fail_total: 0`.
>
> **Current-state note (2026-09-30, `ENGINE-UNIVERSAL-SERVICES.73` — ENGINE tier (the family component), NO release, contract identity `1.0.246` unchanged, embedding API `2.0.0 → 2.1.0`, accept set UNMOVED on any tree that holds its bundle): THE STANDARD-LIBRARY BUNDLE IS EMBEDDED IN THE BINARY.** The `std::` predefined classes were preloaded from `<compile-time repo root>/parser_libs/sv_<profile>_std/`, and a missing directory was a silent `Ok(())` — so a binary run after its tree moved parsed with no stdlib and REJECTED `mailbox m;` (measured: `15.4--mailbox-{blocking,non-blocking}.sv`, both `must_accept`, went `parse_full passed` → `Parser did not consume full input` on the same binary with the bundle hidden). The bundle is now `include_str!`-embedded from the tracked source, `PGEN_PARSER_LIBS_DIR` is the one override, and a bundle missing there is a refusal by name. **Consumers.** Nothing to change; a deployment that copied `parser_libs/` beside the checkout to keep the stdlib working may stop doing so. See § "Build / Availability Requirements".
>
> **Current-state note (2026-09-28, `ENGINE-UNIVERSAL-SERVICES.111` + `SV-CORPUS-GRAD.33` — ENGINE + GRAMMAR tier, release `1.0.246`, ledger `SV-0131`, **schema `34`
> unchanged**, accept set WIDENS on `sv_2017` / `sv_2023` by the two module-scope wildcard-escape shapes; ZERO rows moved on any lane (sv_2017 10 022 / sv_2023 10 048 / verilog_2005 2 283 pass, unchanged) — the module-scope shapes the release makes legal occur in no corpus file that reaches them; the fix is proven by the two pins): THE WRITE EPOCH IS A CONTENT IDENTITY, AND THE WILDCARD ESCAPES REACH MODULE SCOPE.**
> IEEE 1800-2017 A.1.9 `class_type ::= ps_class_identifier [ parameter_value_assignment ] { :: class_identifier [ parameter_value_assignment ] }`, A.2.2.1 `data_type`, A.9.3 `ps_type_identifier`, §26.3 / §26.4 (docs/systemverilog/2017/md/section-41-data-read-api.md). Through `1.0.245` a parameterized class type (`uvm_tlm_fifo#(int) fifo;`) or a class-scoped type (`C#(int)::width_t x;`) at MODULE scope whose class is unresolved under an open `import pkg::*` was REJECTED: the `.28` / `.31` escapes were confined away from every site where `ident #(` can open a module item, because at such a site the class route consumed the parameter list, failed at the instance name, rolled back, and the instantiation route parsed the same list AGAIN — the semantic runtime's store write epoch advanced on a discarding rollback and evicted every store-tainted memo entry, including the list's, whose consulted facts all still stood (`.28`'s first candidate: entries +5.18 %). `1.0.246` makes the epoch an IDENTITY OF STORE CONTENT (a fresh value from a monotone allocator on every mutation; a discarding rollback restores the checkpoint's value, because the content is the checkpoint's again) — measured alone on the pinned sample: entries −0.05 %, memo_hits −0.07 %, committed flat, the generated parser byte-identical — and then admits the escapes at every `class_type` and at the four type-position prefixes; the two pinned residuals flip to ACCEPT. The bare `hierarchical_identifier` reading of `method_call_receiver` loses its never-committing package-qualifier arm (`SV-CORPUS-GRAD.33`; `qualifier: null` keeps the node shape). Rules `1734 → 1734` (one prefix twin added, the qualifier rule deleted). Every previously accepted tree byte-identical (the corpus AST identity sweeps read ast_moved=0 on every lane — 10 022 / 10 022 / 2 283 accepted trees byte-identical against the 1.0.245 probe, verdict_moved=0); entries −0.57 % / memo_hits −0.67 % / committed unchanged (7,090,581), both falls promoted by rebaseline — the deleted qualifier probe alone −1.24 M entries.
>
> **Current-state note (2026-09-23, `SV-CORPUS-GRAD.32` — GRAMMAR tier, release `1.0.245`, ledger `SV-0130`, **schema `34`
> unchanged**, accept set NARROWS on `sv_2017` / `sv_2023` by two pinned invalid block-local shapes; no corpus file moves): THE BLOCK SITE'S PERMISSIVE CLASS-TYPE ARM IS DELETED.**
> IEEE 1800-2017 A.2.2.1 `data_type`, A.2.8 `block_item_declaration`, §8.25, §8.23 / §26.3, §6.18 (docs/systemverilog/2017/md/section-41-data-read-api.md). `block_data_type` carried, beside the typedef-excluded `known_unscoped_block_class_type`, a permissive twin gated on `has_fact(type_name)` alone (kept since 2026-05-30 "for class-internal scoped types"), so `typedef int T; … begin T #(int) x; end` and `T::P x;` were ACCEPTED as block locals through `1.0.244` while the data_type site refused them. Measured on `1.0.244`: the shapes the arm was kept for select `scoped_block_type_identifier` (branch 9), and the corpus liveness census over the `sv_2017` lane found no file whose accepted tree passed through the arm. From
> `1.0.245` the rule and its arm are gone (`1735 → 1734` rules). **Consumers.** Nothing to change: a block local whose type is a declared class keeps `known_unscoped_block_class_type`'s `{head, params, scope_chain}` node; every previously accepted tree is byte-identical (the corpus AST identity sweeps read `ast_moved=0` on every lane (10 022 / 10 048 / 2 283 files both binaries accept, every one identical; `verdict_moved=0` everywhere)). Corpus: ZERO rows moved on any lane (`sv_2017` 10 022 / `sv_2023` 10 048 / `verilog_2005` 2 283 pass, unchanged) — the fix is proven by the pins (the two invalid shapes REJECT, the two identity controls byte-identical), not by a corpus mover
> (sv_2017: match=6069 unexplained=0; sv_2023: match=6061 unexplained=0; verilog_2005: match=2345 unexplained=0 — unchanged); bars 0 / 0 / 0 (`divergence:unexplained` on all three adjudication manifests: every bar stays EMPTY). Pins `ADJUDICATION-REPROS: checked=1770 armed=225 listed=865 multi_profile_rows=672 preprocess_rows=13 failures=0` (865 rows: the routed `accepts_invalid_block_typedef_parameterized.sv` flipped to `invalid` / REJECT — the RED run before the flip read EXACTLY its two arms `expect=ACCEPT got=REJECT` (`checked=1770 … failures=2`) and nothing else — `invalid_block_typedef_scope_chain.sv` REJECT, the two identity controls ACCEPT, every other pin as keyed); lanes `LANE-MOVERS lane=sv_2017 rows_before=16336 rows_after=16336 added=0 lost=0 pass_to_fail=0 fail_to_pass=0 other=0`; `LANE-MOVERS lane=sv_2023 rows_before=16336 rows_after=16336 added=0 lost=0 pass_to_fail=0 fail_to_pass=0 other=0`; `LANE-MOVERS lane=verilog_2005 rows_before=2606 rows_after=2606 added=0 lost=0 pass_to_fail=0 fail_to_pass=0 other=0`; certificates union 1531 → 1530 (the deleted rule; UNKNOWN 15 canonical / 0 union unchanged), `verilog_2005` 1246 → 1245 (the same rule, a proof rule there; UNKNOWN 15 unchanged); closure
> closure rc 0 (`unreachable_rules: 1`, `unreachable_branches: 2` — the pre-existing set, unchanged; the contract re-stamped on the new digest with `defined_rule_count=1702 reachable_rules=1733` — the one deleted rule gone from both counts, nothing newly unreachable; the five family umbrellas read EXACTLY `1.0.244`'s pre-existing `.24` conditions and nothing new — (a) the EBNF dual-run divergence `systemverilog` 211 against the ceiling 155, unchanged (its row's rule count 1703 → 1702); (b) the preprocessor syntax-closure contract carrying no identity block, refusing `sv_formal_exhaustive_closure_gate`, `sv_preprocessor_formal_exhaustive_closure_gate` and `sv_preprocessor_syntax_closure_gate` alike; (c) the VHDL realistic corpus' observed pass total 14 against the contract's 13 — while the annotation-differential, return and stimuli-module parity gates, the `rtl_const_expr` certificate gate, the parse-harness equivalence and semantic gates and the preprocessor reachability closure are GREEN); parse cost entries −0.01 %, committed −132, memo_hits −0.01 % — a FALL (exactly six rules fell: the deleted arm, its identifier chain and its memo-served asks; none rose), promoted by rebaseline, `✅ parse-cost ratchet holds`; share 2.688 % reproduces exactly; the pipeline lane pass 12 486 / fail 3 573 / pp_fail 277, unchanged (`moved=0`), timeout 0; the tuples union `1530 13 1502 1517` (UNKNOWN 15 / 0), `verilog_2005` `1245 397 833 15` — adopted with `adopt_certs.py`; the book marker block regenerated; the release distance `SV-RELEASE-DISTANCE: items=21 (legs_unmet=3 bar_rows=0 [sv_2017=0 sv_2023=0 verilog_2005=0] open_leaves=18 [sv=5 engine=13]) corpus_pass_pct=37.2 known_defects=0 | trailing rate=0.25 items/release over 5 rows, ETA≈84.0 releases ≈ 27.5 days (ESTIMATE)`.
>
> **Current-state note (2026-09-23, `SV-CORPUS-GRAD.13c.2m` — GRAMMAR tier, release `1.0.244`, ledger `SV-0129`, **schema
> 33 → 34**, accept set NARROWS on every profile by the nine pinned invalid arms and WIDENS on `sv_2017` / `sv_2023` by the `local ::` type position; no corpus file moves): THE LRM-EXTRACTION PSEUDO-KEYWORDS ARE DELETED.**
> IEEE 1800-2017 A.8.4 `class_qualifier ::= [ local :: ] [ implicit_class_handle . | class_scope ]`, A.9.3 `ps_type_identifier ::= [ local :: | package_scope | class_scope ] type_identifier` (footnote 43 / 1800-2023 footnote 48; §18.7.1), Annex B (docs/systemverilog/2017/md/section-41-data-read-api.md; docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:3097). The extractor had welded the nonterminal `class_qualifier` into a literal keyword (the source markdown prints its head with ONE colon) and the footnote superscripts `43` / `48` into tokens, so through `1.0.243` `y = class_qualifier m;`, `y = null class_qualifier:=;` and the LRM's own production text `y = null class_qualifier:=local::43;` / `…::48;` were ACCEPTED (nine pinned arms across the three profiles) while `local::T'(1)` inside `randomize() with { … }` was REJECTED. From
> `1.0.244` the three token rules, the four welded `null class_qualifier := …` alternatives and the literal arm of the three scope-prefix copies are gone (`1738 → 1735` rules; `primary_hier_scope_prefix` SV-gated), and the type-scope twins render the LRM's `local ::` as `{kind: "local_qualifier"}`. **Consumers.** Three kinds leave the schema — `"null_class"` (`method_call_receiver_sv_2017` / `_sv_2023`), `"null_class_assign"` (`primary_sv_2017` / `_sv_2023`), `"class_qualifier"` (`primary_hier_scope_prefix`) — none was ever produced by SystemVerilog text, so a consumer that matched them may delete the arm; a `ps_type` node whose scope is `local ::` now carries `scope: {kind: "local_qualifier"}` (a package- or class-scoped type keeps its tree); every previously accepted tree is byte-identical (the corpus AST identity sweeps read `ast_moved=0` on every lane (10 022 / 10 048 / 2 283 files both binaries accept, every one identical; `verdict_moved=0` everywhere)). Corpus: ZERO rows moved on any lane (`sv_2017` 10 022 / `sv_2023` 10 048 / `verilog_2005` 2 283 pass, unchanged) — the fix is proven by the pins (seven invalid arms REJECT, four `local::T` arms ACCEPT), not by a corpus mover
> (sv_2017: match=6069 unexplained=0; sv_2023: match=6061 unexplained=0; verilog_2005: match=2345 unexplained=0 — unchanged); bars 0 / 0 / 0 (`divergence:unexplained` on all three adjudication manifests: every bar stays EMPTY). Pins `ADJUDICATION-REPROS: checked=1764 armed=225 listed=862 multi_profile_rows=669 preprocess_rows=13 failures=0` (862 rows: the five `accepts_invalid_*` pins flipped to `invalid` / REJECT — the RED run before the flip read EXACTLY their seven arms `expect=ACCEPT got=REJECT` (`checked=1764 … failures=7`) and nothing else — the two `local::T` type controls ACCEPT from `1.0.244`, the two identifier controls ACCEPT, every other pin as keyed); lanes `LANE-MOVERS lane=sv_2017 rows_before=16336 rows_after=16336 added=0 lost=0 pass_to_fail=0 fail_to_pass=0 other=0`; `LANE-MOVERS lane=sv_2023 rows_before=16336 rows_after=16336 added=0 lost=0 pass_to_fail=0 fail_to_pass=0 other=0`; `LANE-MOVERS lane=verilog_2005 rows_before=2606 rows_after=2606 added=0 lost=0 pass_to_fail=0 fail_to_pass=0 other=0`; certificates union 1534 → 1531 (the three deleted tokens; UNKNOWN 15 canonical / 0 union unchanged), `verilog_2005` 1250 → 1246 (the literal, the `43` token, the SV-gated prefix and the `48` token, an un-gated token rule declared under every profile although only 1800-2023 text ever reached it — the certificate population is the set DECLARED under the profile, which the pre-registration had read as the REACHABLE set (1247 predicted; the declared-set delta of the two grammars is exactly these four); UNKNOWN 15 unchanged); closure
> closure rc 0 (`unreachable_rules: 1`, `unreachable_branches: 2` — the pre-existing set, unchanged; the contract re-stamped on the new digest with `defined_rule_count=1703 reachable_rules=1734` — the three deleted tokens gone from both counts, nothing newly unreachable; the five family umbrellas read EXACTLY `1.0.243`'s pre-existing `.24` conditions and nothing new — (a) the EBNF dual-run divergence `systemverilog` against the ceiling 155 — 214 → 211, three fewer divergent rows, the three deleted token rules, still over the ceiling; (b) the preprocessor syntax-closure contract carrying no identity block, refusing `sv_formal_exhaustive_closure_gate`, `sv_preprocessor_formal_exhaustive_closure_gate` and `sv_preprocessor_syntax_closure_gate` alike; (c) the VHDL realistic corpus' observed pass total 14 against the contract's 13 — while the annotation-differential, return and stimuli-module parity gates, the `rtl_const_expr` certificate gate, the parse-harness equivalence and semantic gates and the preprocessor reachability closure are GREEN); parse cost entries −1.64 %, committed unchanged, memo_hits −1.90 % — a FALL (exactly three rules fell, the literal, `null` and `trivia`; none rose), promoted by rebaseline, `✅ parse-cost ratchet holds`; share 2.648 → 2.688 % (the corpus total fell, the family unchanged); the pipeline lane pass 12 486 / fail 3 573 / pp_fail 277, unchanged (`moved=0`), timeout 0; the tuples union `1531 13 1503 1518` (UNKNOWN 15 / 0), `verilog_2005` `1246 398 833 15` — adopted with `adopt_certs.py`; the book marker block regenerated; the release distance `SV-RELEASE-DISTANCE: items=22 (legs_unmet=3 bar_rows=0 [sv_2017=0 sv_2023=0 verilog_2005=0] open_leaves=19 [sv=6 engine=13]) corpus_pass_pct=37.2 known_defects=0 | trailing rate=0.00 items/release over 5 rows, ETA≈n/a (no net closure over the window) releases ≈ n/a days (ESTIMATE)`.
>
> **Current-state note (2026-09-23, `SV-CORPUS-GRAD.31` — GRAMMAR tier, release `1.0.243`, ledger `SV-0128`, **schema `33`
> unchanged**, accept set WIDENS on `sv_2017` / `sv_2023` inside wildcard-import files (expression, statement, parameter and out-of-block sites); `verilog_2005` byte-identical): THE WILDCARD ESCAPE AT THE CLASS SCOPE.**
> IEEE 1800-2017 A.1.9 `class_scope ::= class_type ::`, A.8.4 / A.8.5 / A.8.6 / A.9.3, §8.24, §18.5.1 (docs/systemverilog/2017/md/section-41-data-read-api.md); §26.3 / §26.4. Every class-scope head demanded a declared class, so the UVM `uvm_config_db#(int)::set(…)` call, `C#(int)::count = 1;`, `x = C#(int)::count`, `C#(int)::type_id::create(…)`, `localparam int W = C#(int)::WIDTH;`, `C#(int)::A`, `C#(int)::arr[0]`, `C#(int)::new()`, and the out-of-block `function void C#(int)::m();` / `constraint C#(int)::cc { … }` were refused under an open `import pkg::*` through `1.0.242`. From
> `1.0.243` the escape is `class_scope`'s LAST alternative under an `ordered` policy — tried only where the known arm fails, a provably identical choice — at every one of its sites but the four type-position prefixes, which take `class_scope_plain`, and `class_scoped_call_prefix` gains an ordered last arm with the same head: the parameter list CONSUMED (a bare `ident ::` never takes it — a bare unresolved scope may be a package scope), a declared class keeps the known arm by tie order. The four type-position prefixes keep plain `class_scope`: a module-scope `C#(int)::t x;` stays refused (routed `ENGINE-UNIVERSAL-SERVICES.111`, like the module-scope member). **Consumers.** Nothing to change: a `kind: "class_scope"` node whose class is unresolved renders the same `{body: {head: {body}, params, scope_chain}}` object a declared class renders (measured byte-identical under the interpreter); a declared class keeps its tree (the corpus AST identity sweeps read `ast_moved=0` on every lane (10 016 / 10 042 / 2 283 files both binaries accept, every one identical; `verdict_moved=6` per SV profile — the pre-registered rows)). Corpus: EXACTLY the six pre-registered rows `fail → pass` on both SV profiles (`sv_2017` 10 016 → 10 022, `sv_2023` 10 042 → 10 048: the svaunit package and five opentitan AES error-injection wrappers), 0 the other way, `verilog_2005` 0 rows moved (2 283)
> (sv_2017: match=6069 unexplained=0; sv_2023: match=6061 unexplained=0; verilog_2005: match=2345 unexplained=0 — unchanged); bars 0 / 0 / 0 (`divergence:unexplained` on all three adjudication manifests: every bar stays EMPTY). Pins `ADJUDICATION-REPROS: checked=1756 armed=225 listed=858 multi_profile_rows=665 preprocess_rows=13 failures=0` (858 rows: the three `.31` defect pins flipped to `fixed` / ACCEPT on both SV profiles — the RED run on the twin bytes had read exactly those six arms `expect=REJECT got=ACCEPT` — the two shapes controls and two identity controls, one invalid, and the module-scope type residual, REJECT, routed `.111`); lanes `LANE-MOVERS lane=sv_2017 rows_before=16336 rows_after=16336 added=0 lost=0 pass_to_fail=0 fail_to_pass=6 other=0`; `LANE-MOVERS lane=sv_2023 rows_before=16336 rows_after=16336 added=0 lost=0 pass_to_fail=0 fail_to_pass=6 other=0`; `LANE-MOVERS lane=verilog_2005 rows_before=2606 rows_after=2606 added=0 lost=0 pass_to_fail=0 fail_to_pass=0 other=0`; certificates union 1532 → 1534 (the two introduced rules witnessed under both configurations; UNKNOWN 15 canonical / 0 union unchanged), `verilog_2005` unchanged (1250 / 399 / 836 / 15); closure
> closure rc 0 (`unreachable_rules: 1`, `unreachable_branches: 2` — the pre-existing set, unchanged; the contract re-stamped on the new digest with `defined_rule_count=1706 reachable_rules=1737` — the two introduced rules reachable; the five family umbrellas read EXACTLY `1.0.242`'s pre-existing `.24` conditions and nothing new — (a) the EBNF dual-run divergence `systemverilog` 214 against the ceiling 155, unchanged (its row's rule count 1704 → 1706); (b) the preprocessor syntax-closure contract carrying no identity block, refusing `sv_formal_exhaustive_closure_gate`, `sv_preprocessor_formal_exhaustive_closure_gate` and `sv_preprocessor_syntax_closure_gate` alike; (c) the VHDL realistic corpus' observed pass total 14 against the contract's 13 — while the annotation-differential, return and stimuli-module parity gates, the `rtl_const_expr` certificate gate, the parse-harness equivalence and semantic gates and the preprocessor reachability closure are GREEN); parse cost entries +1.27 %, committed +0, memo_hits +0.95 % — accepted under `reached_files_and_contained_guards` (74 of 192 sampled files reached; nothing outside the introduced sub-graph moved on the others), rebaselined, `✅ parse-cost ratchet holds`; share 2.678 → 2.648 %; the twin (+1.43 %) and the gate-first (+1.82 %) spellings measured and refused; the pipeline lane pass 12 381 → 12 486 (+105: every mover `fail → pass` — 86 stopped on a class scope in their own text, 19 on a macro whose expansion carries one; the six raw-lane movers among them, the other 99 raw-`fail` rows deferred or explained by preprocessing) / fail 3 573 / pp_fail 277, timeout 0; the tuples union `1534 14 1505 1520` (UNKNOWN 15 / 0), `verilog_2005` unchanged — adopted with `adopt_certs.py`; the book marker block regenerated; the release distance `SV-RELEASE-DISTANCE: items=22 (legs_unmet=3 bar_rows=0 [sv_2017=0 sv_2023=0 verilog_2005=0] open_leaves=19 [sv=6 engine=13]) corpus_pass_pct=37.2 known_defects=0 | trailing rate=-0.25 items/release over 5 rows, ETA≈n/a (no net closure over the window) releases ≈ n/a days (ESTIMATE)`.
>
> **Current-state note (2026-09-23, `SV-CORPUS-GRAD.30` — GRAMMAR tier, release `1.0.242`, ledger `SV-0127`, **schema `33`
> unchanged**, accept set WIDENS on `sv_2017` / `sv_2023` inside wildcard-import files (block items) and NARROWS by one illegal shape; `verilog_2005` byte-identical): THE PARAMETERIZED-CLASS WILDCARD ESCAPE AT THE BLOCK ITEM's TYPE.**
> IEEE 1800-2017 A.2.8 `block_item_declaration ::= { attribute_instance } data_declaration` and A.1.9 `class_type` (docs/systemverilog/2017/md/section-41-data-read-api.md); §26.3 / §26.4. `.28` put the escape on `class_type`, which `data_type` reaches; a block item's type is read by `block_data_type`, whose class-type route carried no escape, so a function-body, task-body, constructor or `begin`-`end` local `uvm_tlm_b_target_socket_base #(int) bt;` under an open `import pkg::*` was refused on both SV profiles through `1.0.241` (Surelog `amiq_eth_ve_top.v` line 10592). From
> `1.0.242` `wildcard_escape_block_class_type` is the LAST arm of `block_data_type` — the parameter list CONSUMED, no scope gate (a block item's position admits no instantiation), rendering `known_unscoped_block_class_type`'s exact `{head, params, scope_chain}`; and the shared escape head `wildcard_escape_class_type_identifier` now carries the typedef exclusion `known_unscoped_block_class_type` carries, so `typedef int T; class c; T #(int) x; endclass` under an open import is REJECTED as it is without the import (§26.3 + §8.25; ACCEPTED by `1.0.240` / `1.0.241`). **Consumers.** Nothing to change: a block-local `block_data` body whose class is unresolved renders the bare `{head: {body}, params, scope_chain}` object a declared class renders (measured byte-identical under the interpreter); a declared class keeps the known arm and its tree (the corpus AST identity sweeps read `ast_moved=0` on every lane (10 015 / 10 041 / 2 283 files both binaries accept, every one identical; `verdict_moved=1` per SV profile — the amiq row)). Corpus: EXACTLY the amiq row `fail → pass` on both SV profiles (`sv_2017` 10 015 → 10 016, `sv_2023` 10 041 → 10 042), 0 the other way, `verilog_2005` 0 rows moved (2 283)
> (sv_2017: match=6069 unexplained=0; sv_2023: match=6061 unexplained=0; verilog_2005: match=2345 unexplained=0 — unchanged); bars 0 / 0 / 0 (`divergence:unexplained` on all three adjudication manifests: every bar stays EMPTY); the amiq row PARSES IN FULL on both SV profiles from `1.0.242` — the residual dark row `.27` re-keyed `PARSER-DEFECT` at line 907, `.28` at line 8925 and `.29` at line 10592 leaves `RESIDUAL_ROWS.tsv` (16 → 15 rows) and the dark worklist (`PARSER-DEFECT 0`, TOTAL 57); the lanes read it `fail → pass` on `sv_2017` and `sv_2023`, the pipeline lane too. Pins `ADJUDICATION-REPROS: checked=1744 armed=225 listed=852 multi_profile_rows=659 preprocess_rows=13 failures=0` (852 rows: the two `.30` defect pins flipped to `fixed` / ACCEPT on both SV profiles — the RED run before the flip read exactly those four arms `expect=REJECT got=ACCEPT` — the shapes control and two identity controls, two invalids, one `accepts_invalid` (routed `.32`), and the three routed `.31` defect pins — REJECT — with their declared control); lanes `LANE-MOVERS lane=sv_2017 rows_before=16336 rows_after=16336 added=0 lost=0 pass_to_fail=0 fail_to_pass=1 other=0`; `LANE-MOVERS lane=sv_2023 rows_before=16336 rows_after=16336 added=0 lost=0 pass_to_fail=0 fail_to_pass=1 other=0`; `LANE-MOVERS lane=verilog_2005 rows_before=2606 rows_after=2606 added=0 lost=0 pass_to_fail=0 fail_to_pass=0 other=0`; certificates union 1531 → 1532 (the one introduced rule witnessed under both configurations; UNKNOWN 15 canonical / 0 union unchanged), `verilog_2005` unchanged (1250 / 399 / 836 / 15); closure
> closure rc 0 (`unreachable_rules: 1`, `unreachable_branches: 2` — the pre-existing set, unchanged; the contract re-stamped on the new digest with `defined_rule_count=1704 reachable_rules=1735` — the one introduced rule reachable; the family umbrellas' pre-existing RED conditions stay routed to `.24`; no new condition); parse cost entries +0.01 %, committed +0, memo_hits +0.01 % — accepted under `contained_in_introduced_subgraph` (6 rules rose, 0 fell, 0 escaped the introduced sub-graph), rebaselined, `✅ parse-cost ratchet holds`; share 2.679 → 2.678 %; the pipeline lane pass 12 380 → 12 381 (+1, the amiq row) / fail 3 678 / pp_fail 277, timeout 0; the tuples union `1532 14 1503 1518` (UNKNOWN 15 / 0), `verilog_2005` unchanged — adopted with `adopt_certs.py`; the book marker block regenerated; the release distance `SV-RELEASE-DISTANCE: items=23 (legs_unmet=3 bar_rows=0 [sv_2017=0 sv_2023=0 verilog_2005=0] open_leaves=20 [sv=7 engine=13]) corpus_pass_pct=37.2 known_defects=0 | trailing rate=-0.25 items/release over 5 rows, ETA≈n/a (no net closure over the window) releases ≈ n/a days (ESTIMATE)`. ⛔ ROUTED + OWNED: `.31` — the CLASS SCOPE of a static call / static member under an open wildcard import (`uvm_config_db#(int)::set(…)`, the UVM idiom) is still refused, three defect pins; `.32` — the block site's permissive `provisional_unscoped_block_class_type` admits a typedef'd non-class name with a parameter list with NO import (`accepts_invalid_block_typedef_parameterized.sv`).
>
> **Current-state note (2026-09-22, `SV-CORPUS-GRAD.29` — GRAMMAR tier, release `1.0.241`, ledger `SV-0126`, **schema `33`
> unchanged**, accept set WIDENS on `sv_2017` (and on `sv_2023` for a multi-dimensional select only); `verilog_2005` byte-identical): THE `implicit_class_handle .` CLASS_QUALIFIER AT THE PRIMARY.**
> IEEE 1800-2017 A.8.4 `primary ::= [ class_qualifier | package_scope ] hierarchical_identifier select` with `class_qualifier ::= [ local :: ] [ implicit_class_handle . | class_scope ]` (docs/systemverilog/2017/md/section-41-data-read-api.md:3126 / :3140, the same in 1800-2023): `id = this.arr[0];` is ONE primary. Through `1.0.240` `primary_hier_scope_prefix` had no `implicit_class_handle .` arm, so a `this.`-rooted expression was read only by `call_primary` (`this.arr` as a paren-less method call) and the sv_2017 twin — which, LRM-faithfully, carries no trailing select after a call (1800-2017 A.8.4 `function_subroutine_call` bare; 1800-2023 adds `[ [ range_expression ] ]`) — refused the `[0]`. From
> `1.0.241` the prefix's LAST arm is `implicit_class_handle_prefix_sv_only := implicit_class_handle dot -> {kind: "instance", handle: $1}` (sv-only), rendering the node `instance_or_class_scope` / `variable_lvalue_scope` already render for the same bytes. **Consumers.** A `this.`-rooted member chain ending in a select is now a `"hierarchical"` primary `{scope: {kind: "instance", handle: {kind: "this" | "super" | "this_super"}}, name, select}` under `sv_2017` (it had no tree before); under `sv_2023` the same bytes keep their `"call"` tree with its `select` field (the earlier arm keeps the tie) except a multi-dimensional select (`this.m[0][1]`), new on both profiles as the hierarchical primary. Every previously accepted tree is byte-identical on every profile (the corpus AST identity sweeps read `ast_moved=0` on every lane (10 015 / 10 041 / 2 283 files both binaries accept, every one identical)) — the prefix loses every tie to the `call` arm by declaration order. Corpus: ZERO rows moved on any lane (`sv_2017` 10 015 / `sv_2023` 10 041 / `verilog_2005` 2 283 pass, unchanged) — as the population pre-screen predicted; the fix is proven by the pins and by the amiq row's stop moving from line 8925 to line 10592
> (sv_2017: match=6069 unexplained=0; sv_2023: match=6061 unexplained=0; verilog_2005: match=2345 unexplained=0 — unchanged); bars 0 / 0 / 0 (`divergence:unexplained` on all three adjudication manifests: every bar stays EMPTY); the amiq row stays RESIDUAL — its stop moved from line 8925 (the `this.member[select]` rvalue, now parsed) to line 10592, `uvm_tlm_b_target_socket_base #(T) bt;`, a parameterized-class-typed LOCAL of a static function under the open wildcard import (the `.28` escape sits on `class_type`, which `data_type` reaches; a block item's type is read by `block_data_type`, whose class-type twins carry no escape): `RESIDUAL_ROWS.tsv` re-keyed `PARSER-DEFECT` → `defect_wildcard_escape_parameterized_tf_local.sv`, routed `.30`; the dark worklist rc 0. Pins `ADJUDICATION-REPROS: checked=1724 armed=225 listed=842 multi_profile_rows=649 preprocess_rows=13 failures=0` (842 rows: the `.28` defect pin flipped to `fixed` / ACCEPT on both SV profiles, the three `.29` controls and one invalid, and the two `.30` defect pins — REJECT — routed); lanes `LANE-MOVERS lane=sv_2017 rows_before=16336 rows_after=16336 added=0 lost=0 pass_to_fail=0 fail_to_pass=0 other=0`; `LANE-MOVERS lane=sv_2023 rows_before=16336 rows_after=16336 added=0 lost=0 pass_to_fail=0 fail_to_pass=0 other=0`; `LANE-MOVERS lane=verilog_2005 rows_before=2606 rows_after=2606 added=0 lost=0 pass_to_fail=0 fail_to_pass=0 other=0`; certificates union 1530 → 1531 (the one introduced rule witnessed under both configurations; UNKNOWN 15 canonical / 0 union unchanged), `verilog_2005` unchanged (1250 / 399 / 836 / 15); closure
> closure rc 0 (`unreachable_rules: 1`, `unreachable_branches: 2` — the pre-existing set, unchanged; the contract re-stamped on the new digest with `defined_rule_count=1703 reachable_rules=1734`; the family umbrellas' pre-existing RED conditions stay routed to `.24`; no new condition); parse cost entries +0.23 %, committed +0, memo_hits +0.27 % — accepted under `contained_in_introduced_subgraph` (3 rules rose, 0 fell, 0 escaped the introduced sub-graph), rebaselined, `✅ parse-cost ratchet holds`; share 2.684 → 2.679 %; the pipeline lane pass 12 380 / fail 3 679 / pp_fail 277, unchanged, timeout 0; the tuples union `1531 14 1502 1517` (UNKNOWN 15 / 0), `verilog_2005` unchanged — adopted with `adopt_certs.py`; the book marker block regenerated; the release distance `SV-RELEASE-DISTANCE: items=22 (legs_unmet=3 bar_rows=0 [sv_2017=0 sv_2023=0 verilog_2005=0] open_leaves=19 [sv=6 engine=13]) corpus_pass_pct=37.2 known_defects=0 | trailing rate=0.25 items/release over 5 rows, ETA≈88.0 releases ≈ 115.0 days (ESTIMATE)`.
>
> **Current-state note (2026-09-22, `SV-CORPUS-GRAD.28` — GRAMMAR tier, release `1.0.240`, ledger `SV-0125`, **schema `33`
> unchanged**, accept set WIDENS on `sv_2017` / `sv_2023` inside wildcard-import files only; `verilog_2005` byte-identical): THE WILDCARD ESCAPE FOR A PARAMETERIZED CLASS TYPE.**
> IEEE 1800-2017 A.1.9 `class_type ::= ps_class_identifier [ parameter_value_assignment ] { :: class_identifier [ parameter_value_assignment ] }` (docs/systemverilog/2017/md/section-41-data-read-api.md) is syntactically ANY identifier; §26.3 / §26.4 make every declaration of a wildcard-imported package a candidate for the importing scope, and resolution is elaboration's. `.27` admitted an undeclared bare identifier at every gated type position but not a `class_type` with a `#( … )`; from
> `1.0.240` an sv-only escape gated on the `wildcard_import_open` fact CONSUMES the parameter list at the two sites where
> `ident #(` is never an instantiation — a `class_type` arm inside a class body (a new `class_body` scope), a `param_expression` arm (type arguments) and the
> class-alias `typedef` — rendering `{head: {kind: "class", body}, params, suffix}` as the known-class route does; a module-scope class-typed
> member under a wildcard import stays refused (routed `ENGINE-UNIVERSAL-SERVICES.111`). **Consumers.** Nothing to change: a bare name
> keeps its `.27` route and tree (the corpus AST identity sweeps read `ast_moved=0` on every lane (10 013 / 10 039 / 2 283 files both binaries accept, every one identical)); a `class_type` node's head may now carry an unresolved class name under
> `kind: "class"` — exactly what a declared class produces — and elaboration resolves it. Corpus: `sv_2017` / `sv_2023`: EXACTLY the two pre-registered rows `fail → pass` each, 0 the other way; `verilog_2005`: 0 rows moved
> (sv_2017: match=6069 unexplained=0; sv_2023: match=6061 unexplained=0; verilog_2005: match=2345 unexplained=0); bars 0 / 0 / 0 (`divergence:unexplained` on all three adjudication manifests: every bar stays EMPTY); the amiq row stays RESIDUAL — its stop moved from line 907 (the parameterized class type, now parsed) to line 8925, `m_id = this.x_port_names[m_port_name];`, a `this.member[select]` RVALUE refused under `sv_2017` only: `RESIDUAL_ROWS.tsv` re-keyed `PARSER-DEFECT` → `defect_this_member_select_rvalue.sv`, routed `.29`; the dark worklist rc 0. Pins `ADJUDICATION-REPROS: checked=1711 armed=225 listed=836 multi_profile_rows=642 preprocess_rows=13 failures=0` (837 rows: the four `.28` pins — the `.27` defect pin flipped to `fixed`, one control, one invalid, one routed defect — and the three `.29` pins); lanes `LANE-MOVERS lane=sv_2017 rows_before=16336 rows_after=16336 added=0 lost=0 pass_to_fail=0 fail_to_pass=2 other=0`; `LANE-MOVERS lane=sv_2023 rows_before=16336 rows_after=16336 added=0 lost=0 pass_to_fail=0 fail_to_pass=2 other=0`; `LANE-MOVERS lane=verilog_2005 rows_before=2606 rows_after=2606 added=0 lost=0 pass_to_fail=0 fail_to_pass=0 other=0`; certificates union 1524 → 1530 (+6 introduced rules, every one witnessed; UNKNOWN 15 canonical / 0 union unchanged), `verilog_2005` unchanged; closure
> closure rc 0 (the family umbrellas' pre-existing RED conditions stay routed to `.24`; no new condition); parse cost entries +0.25 %, committed +20 (the class-body markers), memo_hits +0.14 % — accepted under `reached_files_and_contained_guards` (81 of 192 sampled files reached; nothing outside the introduced sub-graph moved elsewhere), rebaselined, `✅ parse-cost ratchet holds`; share 2.693 → 2.684 %; the pipeline lane pass 12 324 → 12 380 (+56), fail 3 735 → 3 679, pp_fail 277; the tuples union `1530 14 1501 1516` (UNKNOWN 15 / 0), `verilog_2005` unchanged — adopted with `adopt_certs.py`; the book marker block regenerated; the share anchors `2.693 → 2.684` moved by their script; the release distance `SV-RELEASE-DISTANCE: items=22 (legs_unmet=3 bar_rows=0 [sv_2017=0 sv_2023=0 verilog_2005=0] open_leaves=19 [sv=6 engine=13]) corpus_pass_pct=37.2 known_defects=0 | trailing rate=0.25 items/release over 5 rows, ETA≈88.0 releases ≈ 104.9 days (ESTIMATE)`.
>
> **Current-state note (2026-09-18, `SV-CORPUS-GRAD.27` — GRAMMAR tier, release `1.0.239`, ledger `SV-0124`, **schema `33`
> unchanged**, accept set WIDENS on `sv_2017` / `sv_2023` inside wildcard-import files only; `verilog_2005` byte-identical): THE
> WILDCARD ESCAPE REACHES TYPE POSITIONS.** IEEE 1800-2017 A.1.9 `class_type ::= ps_class_identifier …` and A.2.2.1 `data_type ::= … | [ class_scope | package_scope ] type_identifier { packed_dimension }` are syntactically ANY identifier; §26.3 / §26.4 make every declaration of a wildcard-imported package a candidate for the importing scope, and resolution is elaboration's — the parser cannot prove an identifier undeclared while an unresolved `pkg::*` import is open (SV-PARSE-STRICT.2, sound-not-complete, the nettype precedent). Through `1.0.238` the escape SV-PARSE-STRICT.2 introduced covered nettype
> identifiers alone, so `class test extends uvm_test;` and `uvm_component parent` under an open `import uvm_pkg::*` stopped at the
> store gate. From `1.0.239` two sv-only escapes, each gated on the `wildcard_import_open` fact and each rendering what the known
> rule renders, sit LAST in `base_class_type_head` and in the two unscoped type use-sites — and YIELD where the same bytes
> already had a resolution-free reading: `nettype unknown_t x;` stays the nettype alias (A.2.1.3's second production) and a
> direction-less port `foo_t a` stays an interface port, both as through `1.0.238` (the first candidate had moved both trees —
> the pins pre-screen read `ast_moved=2` before any lane ran; `nettype_alias_shape_ahead`, `wildcard_escape_interface_port_shape`) —
> and NEVER steal a name: at an implicit-capable site (`data_type_or_implicit`: tf ports, coverpoint labels, property formals) a lone
> unresolved identifier stays the NAME it was through `1.0.238` (`wildcard_escape_lone_name_shape`; the first candidate had lost three
> corpus files on a coverpoint label and moved untyped tf-port trees silently — the population pre-screen and seven probes, before any lane ran; a scoped type's package name is not that shape either — the second population pre-screen; a randsequence production's name is not either — the third candidate's lanes on slang `all.sv`).
> **Consumers.** Nothing to change: a
> declared name keeps its branch and its tree (identity controls SAME; the corpus AST identity sweeps read `ast_moved=0` on every lane (9 843 / 9 869 / 2 283 files both binaries accept, every one identical)); a consumer sees a `class` / `known_unscoped`
> node whose name may be unresolved — exactly what the net-alias escape already produces — and elaboration resolves it. Corpus:
> `sv_2017` / `sv_2023`: EXACTLY the 170 pre-registered rows `fail → pass` each, 0 the other way; `verilog_2005`: 0 rows moved (sv_2017: match=6069 unexplained=0; sv_2023: match=6061 unexplained=0; verilog_2005: match=2345 unexplained=0); `t_uvm_hello.v` leaves `unexplained_rejects_valid` for `divergence:explained_svpp_macro_use` on both SV manifests — refused ON `` `uvm_component_utils(test) ``, text the preprocessor alters, as pre-registered; the pipeline lane is where its pass is measured; the SV axis-2 bar 2 → 0 rows (`divergence:unexplained` on `adjudication_manifest.tsv`: 0 — the SV axis-2 bar is EMPTY), the `sv_2023` bar 1 → 0 rows (`divergence:unexplained` on `adjudication_manifest_sv2023.tsv`: 0 — EMPTY; `verilog_2005` stays 0).
> Pins `ADJUDICATION-REPROS: checked=1700 armed=225 listed=830 multi_profile_rows=637 preprocess_rows=13 failures=0` (830 rows: the 21 `.27` wildcard-escape pins and the four RESIDUAL-adjudication pins); lanes `LANE-MOVERS lane=sv_2017 rows_before=16336 rows_after=16336 added=0 lost=0 pass_to_fail=0 fail_to_pass=170 other=0`; `LANE-MOVERS lane=sv_2023 rows_before=16336 rows_after=16336 added=0 lost=0 pass_to_fail=0 fail_to_pass=170 other=0`; `LANE-MOVERS lane=verilog_2005 rows_before=2606 rows_after=2606 added=0 lost=0 pass_to_fail=0 fail_to_pass=0 other=0`; certificates union 1517 → 1524 (+7 rules; UNKNOWN 15 canonical / 0 union unchanged), `verilog_2005` 1248 → 1250 (UNKNOWN 15 unchanged); closure closure rc 0; parse cost entries +0.80 %, committed +2.83 %, memo_hits +0.81 % — accepted under the fifteenth invariant `reached_files_and_contained_guards` (74 of 192 sampled files reached; nothing outside the introduced sub-graph moved elsewhere), rebaselined, `✅ parse-cost ratchet holds`; share 2.643 → 2.693 %; the pipeline
> lane pass 12 200 → 12 324 (+124), fail 3 859 → 3 735, pp_fail 277; the tuples union `1524 14 1495 1510`, `verilog_2005` `1250 399 836 15`; the release distance `SV-RELEASE-DISTANCE: items=21 (legs_unmet=3 bar_rows=0 [sv_2017=0 sv_2023=0 verilog_2005=0] open_leaves=18 [sv=6 engine=12]) corpus_pass_pct=37.2 known_defects=0 | trailing rate=0.75 items/release over 5 rows, ETA≈28.0 releases ≈ 10.4 days (ESTIMATE)`.
>
> **Current-state note (2026-09-17, `SV-CORPUS-GRAD.26` — GRAMMAR tier, release `1.0.238`, ledger `SV-0123`, **schema `33`
> unchanged** (an additive `primary` kind), accept set WIDENS on `sv_2017` / `sv_2023` by exactly one sanctioned comparison;
> `verilog_2005` byte-identical): A TYPE COMPARISON IS AN EXPRESSION OPERAND.** IEEE 1800-2017 §6.23 (`docs/systemverilog/2017/md/section-65536-216-bits.md:1612-1613`) = IEEE 1800-2023 §6.23 (`docs/systemverilog/2023/md/section-6-data-types.md:2938-2939`): a type reference "can also be used in equality/inequality and case equality/inequality comparisons with other type references, and such comparisons are considered to be constant expressions"; Annex A lists `type_reference` in `constant_primary` ONLY (footnote 40 / 45 — "illegal … with any operators except the equality/inequality and case equality/inequality operators", `docs/systemverilog/2017/md/section-41-data-read-api.md:3560-3562`, `docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:3525-3527`) and never in `primary`. Through `1.0.237` `if (type(b) == type(a))` had no derivation on either SV profile
> (the constant route `localparam bit T = type(a) == type(b);` always had one). From `1.0.238` `primary` carries a last
> arm `{kind: "type_comparison", body: {left, op, right}}` — two type references and one of `===`, `!==`, `==`, `!=` —
> on the SV profiles. **Consumers.** A new additive kind in `primary`: a consumer that switches on `primary.kind`
> gains one case; nothing else moves (every previously accepted tree byte-identical: the corpus AST identity sweeps read `ast_moved=0` on every lane (9 843 / 9 868 / 2 283 files both binaries accept, every one identical)). Corpus:
> `sv_2023`: slang `all.sv` `fail → pass` (`9 868 → 9 869`); `sv_2017` / `verilog_2005`: 0 rows moved (the `sv_2023` lane's unexplained rows `2 → 1`, the `sv_2017` lane unchanged); the `sv_2023` bar 2 → 1 (`unexplained_rejects_valid` 2 → 1: slang `all.sv` now matches; `t_uvm_hello.v` remains — the same file as the SV axis-2 bar's one row, `.27`). Pins ADJUDICATION-REPROS: checked=1648 armed=225 listed=805 multi_profile_rows=612 preprocess_rows=13 failures=0 (the first run read failures=1: the `verilog_2005` control was written as `.v` and the runner lists pins by a `*.sv` glob — renamed, a runner limitation worth knowing: a pin's extension is `.sv` whatever its profile); lanes `LANE-TOTALS: lane=sv_2017 rows=16336 pass=9843 fail=6493 timeout=0` (unchanged), `LANE-TOTALS: lane=sv_2023 rows=16336 pass=9869 fail=6467 timeout=0` (was 9868 / 6468), `LANE-TOTALS: lane=verilog_2005 rows=2606 pass=2283 fail=323 timeout=0` (unchanged); certificates
> union `1515/10/1490/15` + `1505/0` → `1517/10/1492/15` + `1507/0` (the two sv-only rules witnessed), `verilog_2005` unchanged `1248/397/836/15`; closure `unreachable_rules: 1`, re-stamped; parse cost family share `2.648 → 2.643` adopted at every anchor; `entries` +0.24 % and `memo_hits` +0.28 % on the pinned sample ACCEPTED under `contained_in_introduced_subgraph` (two rules rose: the new arm and the `type_reference` it enters; `committed` flat), rebaselined, `parse-cost-ratchet: OK`; the pipeline lane `PIPELINE-LANE: files=16336 pass=12200 fail=3859 pp_fail=277`, the row file byte-identical to `1.0.237`'s (0 movers); the tuples
> raw `7485/2606/6245/4039/1` and pipeline `9139/2606/4591/3311/1` unchanged; the release distance `SV-RELEASE-DISTANCE: items=22 (legs_unmet=3 bar_rows=2 [sv_2017=1 sv_2023=1 verilog_2005=0] open_leaves=17 [sv=5 engine=12]) corpus_pass_pct=37.1 known_defects=1 | trailing rate=1.25 items/release over 5 rows, ETA≈17.6 releases ≈ 4.0 days (ESTIMATE)` (the `1.0.238` row recorded; bar rows 3 → 2).
>
> **Current-state note (2026-09-17, `SV-CORPUS-GRAD.3.69` + `ENGINE-UNIVERSAL-SERVICES.108` — ENGINE + GRAMMAR + ANSWER-KEY tier,
> release `1.0.237`, ledger `SV-0122`, **schema `33` unchanged**, accept set NARROWS on `verilog_2005` ALONE; `sv_2017` /
> `sv_2023` byte-identical): THE HEADER-LESS ANSI DEFAULT IS GATED ON THE LATEST GOVERNING HEADER.** IEEE 1364-2005 A.2.1.2 `output_declaration ::= … | output reg [ signed ] [ range ] list_of_variable_port_identifiers | output output_variable_type list_of_variable_port_identifiers`, `list_of_variable_port_identifiers ::= port_identifier [ = constant_expression ] { , port_identifier [ = constant_expression ] }`, `list_of_port_identifiers ::= port_identifier { , port_identifier }`
> (`docs/verilog/2005/txt/section-Annex_A-normative-formal-syntax-definition.txt:137-141`, `:224-228`): a header-less item is one more identifier of the previous declaration, and only `output reg` /
> `output integer|time` lists admit `= expr`. Through `1.0.236` the `verilog_2005` twin admitted a default on any
> header-less item (`inout [31:0] x, y = 1`, the `.3.66` residual): a rule cannot read its earlier siblings, and the
> store's existence queries answer for every header ever seen. From `1.0.237` the shared semantic runtime provides
> `latest_fact_attribute_equals(kind, attribute, value)` — true iff the most recently emitted, still-standing fact of `kind` at any depth up to the current one carries `attribute = value`; every `verilog_2005` header arm re-emits `ansi_port_header{defaults}` at branch
> start, the list opener resets it, and the header-less arm's default is gated on the latest one. **Consumers.** The
> engine grows one predicate (the roster in `docs/book/src/semantic-store.md` is fifteen); no generated parser but
> `systemverilog` changes a byte (`make regenerate_generated_parsers` on the new runtime: of the ten generated parsers only `generated/systemverilog_parser.rs` changed (`44dc76eb… → 697f363e…`, the consumer grammar); `ebnf`, `json`, `regex`, `return_annotation`, `rtl_const_expr`, `rtl_frontend`, `scratch`, `semantic_annotation` and the rest byte-identical (`shasum` before / after) — the primitive is runtime-only, no codegen changed); the interpreter and every generated parser agree on the new construct
> (`cargo test --features "generated_parsers ebnf_dual_run" --lib`: `test result: ok. 3 passed; 0 failed` — `ast_pipeline::semantic_runtime::tests::latest_fact_attribute_equals_reads_the_nearest_standing_fact`, `parse_harness_semantic_suite::gate::every_semantic_construct_is_byte_identical` (37/37, the new `sem_latest_attr_gate` among them), `parse_harness_semantic_suite::gate::semantic_construct_coverage_is_complete`; `parse_harness_equivalence_gate`: `test result: ok. 4 passed` — every registered grammar CERTIFIED byte-identical between the interpreter and its generated parser on the stimuli corpus at seeds 0/7/42, the deferred ratchet and completeness held; `parse_harness_semantic_gate`: `test result: ok. 2 passed` (37/37 constructs byte-identical, coverage complete)); nothing to change unless a consumer feeds `verilog_2005` a header-less default behind
> `input` / `inout` / a net `output`. Corpus: `verilog_2005`: `module_inout_port_list_def.v` `pass → fail` (`2 284 → 2 283`), 0 the other way; `sv_2017` / `sv_2023`: 0 rows moved (`v2005: match=2345 unexplained=0` (was 2344 / 1), the SV lanes unchanged); the bar 1 → 0 (`accepts_invalid` 1 → 0: `module_inout_port_list_def.v` matches; the `.8c.2` family is closed and the `verilog_2005` bar is EMPTY) — the last
> `.8c.2` row leaves it. Pins ADJUDICATION-REPROS: checked=1629 armed=225 listed=795 multi_profile_rows=603 preprocess_rows=13 failures=0; lanes `LANE-TOTALS: lane=sv_2017 rows=16336 pass=9843 fail=6493 timeout=0` (unchanged), `LANE-TOTALS: lane=sv_2023 rows=16336 pass=9868 fail=6468 timeout=0` (unchanged), `LANE-TOTALS: lane=verilog_2005 rows=2606 pass=2283 fail=323 timeout=0` (was 2284 / 322); certificates union unchanged `1515/10/1490/15` + `1505/0`; `verilog_2005` `1246/397/835/14` → `1248/397/836/15` (the gated default helper UNKNOWN by construction, `ENGINE-UNIVERSAL-SERVICES.109`); closure `unreachable_rules: 1`, re-stamped; parse
> cost family share `2.648` and its counts unchanged, no binding rise, rebaselined, `parse-cost-ratchet: OK`; the pipeline lane `PIPELINE-LANE: files=16336 pass=12200 fail=3859 pp_fail=277`, the row file byte-identical to `1.0.236`'s (0 movers); the tuples raw `7485/2606/6245/4039/1` and pipeline `9139/2606/4591/3311/1` unchanged.
>
> **Current-state note (2026-09-17, `SV-CORPUS-GRAD.3.68` — GRAMMAR + ANSWER-KEY tier, release `1.0.236`, ledger `SV-0121`,
> **schema `33` unchanged**, accept set NARROWS on `verilog_2005` ALONE; `sv_2017` / `sv_2023` byte-identical): A NET
> DECLARATION UNDER `verilog_2005` FOLLOWS IEEE 1364-2005 A.2.1.3.** IEEE 1364-2005 A.2.1.3 `net_declaration ::= net_type [ signed ] [ delay3 ] list_of_net_identifiers ; | net_type [ drive_strength ] [ signed ] [ delay3 ] list_of_net_decl_assignments ; | net_type [ vectored | scalared ] [ signed ] range [ delay3 ] list_of_net_identifiers ; | net_type [ drive_strength ] [ vectored | scalared ] [ signed ] range [ delay3 ] list_of_net_decl_assignments ; | trireg …`
> (`docs/verilog/2005/txt/section-Annex_A-normative-formal-syntax-definition.txt:153-176`, `range` `:252`; IEEE 1800-2017 A.2.1.3 `net_declaration ::= net_type [ drive_strength | charge_strength ] [ vectored | scalared ] data_type_or_implicit [ delay3 ] list_of_net_decl_assignments ; | …` `docs/systemverilog/2017/md/section-41-data-read-api.md:805-810`, A.2.2.1 `implicit_data_type ::= [ signing ] { packed_dimension }` `:867`) — no data type, ONE range. Through `1.0.235` the sv_2017 twin served both
> profiles and read 1800's `data_type_or_implicit`, so `wire integer x;`, `wire real r;` and `wire [3:0][3:0] a;` parsed under
> `verilog_2005`. From `1.0.236` a `verilog_2005` twin behind the `net_declaration` dispatcher renders the sv twin's six fields
> — the implicit carrier `{kind: "implicit", body}` over `( signed )? ( range )?`, one range as the one-element list — so
> every tree both releases accept is byte-identical (the corpus AST identity sweeps read `ast_moved=0` on every lane (9 843 / 9 868 / 2 284 files both binaries accept, every one identical)); the sv_2017 twin's gate narrows to `sv_2017` and the
> SV profiles do not move (`LANE-MOVERS lane=sv_2017 … pass_to_fail=0 fail_to_pass=0`, `sv_2023` the same, `LANE-MOVERS lane=verilog_2005 … pass_to_fail=11 fail_to_pass=0` — EXACTLY the eleven pre-registered rows (iverilog `br_gh156.v`, `ca_time_real.v`, `delayed_sfunc.v`, `pr1864110a.v`, `pr1864110b.v`, `pr1864115.v`, `module_nonansi_fail12.v`, `br_gh497a.v`; sv2v `interface_array_multi.v`, `interface_delay_1.v`, `interface_delay_2.v`), 0 the other way, 0 on the SV lanes). **Consumers.** Nothing to change unless a consumer feeds the `verilog_2005`
> profile a net with a data type or a second packed range: those are refused from `1.0.236`. Corpus: EXACTLY the 11
> pre-registered `verilog_2005` rows moved `pass → fail` (six Icarus real-net extension tests, `module_nonansi_fail12.v`,
> four multi-range nets in ivtest and sv2v goldens), each re-keyed `must_reject` by the EDITION-NET-DECLARATION answer-key
> pass (the pass re-keyed the 11 `must_reject`; `v2005: match=2344 unexplained=1`, the SV lanes unchanged); the bar 1 → 1 (`accepts_invalid` 1: the routed residual `module_inout_port_list_def.v`, `ENGINE-UNIVERSAL-SERVICES.108`; the eleven movers are `match`). Pins ADJUDICATION-REPROS: checked=1605 armed=225 listed=783 multi_profile_rows=595 preprocess_rows=13 failures=0 (the first run read failures=2: `control_v2005_net_bit_name.sv` declared on every profile while `bit` is a keyword under the 1800 profiles — the manifest row corrected to `verilog_2005` alone, the edition contrast it pins); lanes `LANE-TOTALS: lane=sv_2017 rows=16336 pass=9843 fail=6493 timeout=0` (unchanged), `LANE-TOTALS: lane=sv_2023 rows=16336 pass=9868 fail=6468 timeout=0` (unchanged), `LANE-TOTALS: lane=verilog_2005 rows=2606 pass=2284 fail=322 timeout=0` (was 2295 / 311); certificates union unchanged `1515/10/1490/15` + `1505/0`, `verilog_2005` `1243/397/832/14` → `1246/397/835/14`; closure
> `unreachable_rules: 1`, re-stamped; parse cost family share `2.648` and its counts unchanged, no binding rise, rebaselined, `parse-cost-ratchet: OK`; the pipeline lane `PIPELINE-LANE: files=16336 pass=12200 fail=3859 pp_fail=277`, the row file byte-identical to `1.0.235`'s (0 movers); the tuples raw `7485/2606/6245/4039/1` and pipeline `9139/2606/4591/3311/1` unchanged.
>
> **Current-state note (2026-09-17, `SV-CORPUS-GRAD.3.67` — GRAMMAR tier, release `1.0.235`, ledger `SV-0120`,
> **schema `33` unchanged**, accept set NARROWS on EVERY profile, every accepted tree byte-identical): THE BASED
> LITERAL'S LEXICAL STRICTNESS.** IEEE 1364-2005 A.8.7 = IEEE 1800-2017 A.8.7 = IEEE 1800-2023 A.8.7 agree on the shape of a based literal
> (`docs/verilog/2005/txt/section-Annex_A-normative-formal-syntax-definition.txt:978-1005`, `docs/systemverilog/2017/md/section-41-data-read-api.md:3253-3289`): the size is a
> `non_zero_unsigned_number` (a non-zero first digit), each base has its own digit set and the value starts with a digit,
> and a decimal value is an `unsigned_number` or ONE `x_digit` / `z_digit` followed by underscores. Through `1.0.234` the
> `integral_number` terminal took one value charset for every base and any digit string as a size, so `0'b0`, `04'b1`,
> `4'b_1`, `4'b2`, `4'o8`, `4'b1a` and `32'dx_1` all parsed as numbers on every profile. From `1.0.235` the same terminal
> slot reads `/([1-9][0-9_]*G)?'[sS]?(?:[bB]G[01xXzZ?][01xXzZ?_]*|[oO]G[0-7xXzZ?][0-7xXzZ?_]*|[dD]G(?:[0-9][0-9_]*|[xXzZ?]_*)|[hH]G[0-9a-fA-FxXzZ?][0-9a-fA-FxXzZ?_]*)/`
> (G the `.3.56` white-space-or-comment gap, kept verbatim) and every one of those is refused on `sv_2017`, `sv_2023` and
> `verilog_2005`; `4'b1_`, `'b1`, `4'sb1`, `1_0'b1`, `4'dx_`, `4'd?`, `8'h?F`, `4 'b 1` and every other legal spelling
> parse to the SAME `{kind: "integral", body}` node as before (the carrier is untouched: 8/8 legal-probe arms SAME under
> the interpreter; sweeps the corpus AST identity sweeps read `ast_moved=0` on every lane (9 843 / 9 868 / 2 295 files both binaries accept, every one identical)). **Consumers.** Nothing to change unless a consumer feeds a literal Annex A
> does not derive: the new language is a SUBSET of the old, so no previously refused file is admitted and no accepted
> file's tree moves. The corpus scanned with comments and strings blanked holds 7 files carrying such a literal; EXACTLY
> 3 moved, accept → reject (`sv_2017` / `sv_2023`: `br_gh60a.v`, `br_gh79.v`, `t_const_dec_mixed_bad.v` `pass → fail`; `verilog_2005`: the two ivtest rows; 0 the other way): the two ivtest rows of the `verilog_2005` bar (`br_gh60a.v` `0'b0`,
> `br_gh79.v` eight `_`-led values) and verilator's `t_const_dec_mixed_bad.v` (`32'dx_1`, which its own driver refuses —
> re-keyed `must_reject` by A.8.7). Under the pipeline `PIPELINE-LANE: files=16336 pass=12200 fail=3859 pp_fail=277` — the four pre-registered rows `pass → fail` (friscv `friscv_alu.sv` among them), nothing else: friscv `rtl/friscv_alu.sv` carries 27 leading-zero
> sizes (`01'b0`), a spelling every tool lexes leniently and Annex A does not derive — ONE real design falls to the
> letter of the LRM; a leniency would be a deviation from the standard and is the director's decision, surfaced here.
> Slice 0 deletes the two profile-dead helpers `constant_member_chain_sv_only` / `generate_scope_step_sv_only` (gated
> sv-only, referenced only by the `verilog_2005` twins as `[]`-rendering optionals) and spells the two fields literally
> `[]`: behaviour-neutral, `1715 → 1713` rules, the union certificate's proof returns `12 → 10` — the `1.0.234` note's
> reading "ungated, misnamed" was false and is corrected forward. Pins ADJUDICATION-REPROS: checked=1547 armed=225 listed=757 multi_profile_rows=576 preprocess_rows=13 failures=0; lanes `LANE-TOTALS: lane=sv_2017 rows=16336 pass=9843 fail=6493 timeout=0` (was 9846 / 6490), `LANE-TOTALS: lane=sv_2023 rows=16336 pass=9868 fail=6468 timeout=0` (was 9871 / 6465), `LANE-TOTALS: lane=verilog_2005 rows=2606 pass=2295 fail=311 timeout=0` (was 2297 / 309); adjudicator `v2005: match=2344 unexplained=1` (was 2342 / 3), the SV lanes' counts unchanged; certificates union `1517/12/1490/15` + `1505/0` → `1515/10/1490/15` + `1505/0` (the two profile-dead helpers deleted), `verilog_2005` unchanged `1243/397/832/14`; closure
> `unreachable_rules: 1`, re-stamped; parse cost family share `2.648` unchanged (the family's and the corpus' entries both fall with the three refused files), no binding rise, rebaselined, `parse-cost-ratchet: OK`; the tuples raw `7485/2606/6245/4039/1` unchanged, under the pipeline `9140/2606/4590/3310/1 → 9139/2606/4591/3311/1`.
>
> **Current-state note (2026-09-16, `SV-CORPUS-GRAD.3.66` — GRAMMAR tier, release `1.0.234`, ledger `SV-0115` … `SV-0119`,
> **schema `33` unchanged**, accept set NARROWS on `verilog_2005` ALONE; `sv_2017` / `sv_2023` byte-identical): THE
> DECLARATION TIER OF THE 1364-2005 BAR.** Eight productions carried IEEE 1800's shape on the 1364-2005 profile. **(a)** Port
> declarations, ANSI and non-ANSI (IEEE 1364-2005 A.2.1.2 `:133-141`, `docs/verilog/2005/txt/section-Annex_A-normative-formal-syntax-definition.txt`;
> `list_of_port_identifiers` `:224`, `list_of_variable_port_identifiers` `:228`, `output_variable_type ::= integer | time` `:181`):
> `input` / `inout` take `[ net_type ] [ signed ] [ range ]` and bare identifiers — no variable type, no default, no
> dimension; `output` takes the net form, `reg [ signed ] [ range ]` or `integer` / `time` with a default; there is no
> `real` port — so `input reg a`, `inout integer c`, `inout [31:0] x, y = 1`, `input a = 1`, `output a = 1`, `input real r`,
> `input a [0:3]` and the non-ANSI `input reg a;` now REJECT on `verilog_2005`, while `output reg [3:0] q = 0`,
> `output integer c = 1`, `input a, b`, `input a; reg a;` keep parsing. **(b)** A `parameter` inside a generate scope (A.4.2
> `:452-454`; A.1.4 `:88-92` is `parameter_declaration`'s only home; `module_or_generate_item` `:65` admits `localparam`)
> now REJECTs there — a lookahead-guarded generate item; a module-level `parameter` never passes through it. **(c)** A dotted name in
> a constant (A.8.4 `:911-920`: `parameter_identifier [ [ constant_range_expression ] ]`; A.9.3 `parameter_identifier ::=
> identifier`; A.8.2 `constant_function_call` `:848-851`, a plain call with parentheses) — `dut.WIDTH`, `a.b` — now REJECTs on
> all three 1800 readings the grammar had (a member chain, a generate-scoped parameter, and the METHOD chain the tree actually
> took); `P[3:0]`, `P[1]`, `$clog2(8)`, `f(3)` keep parsing, and the `verilog_2005` twin is dispatched under the same
> `{kind: "sv_2017"}` tag the tree has always carried. **(d)** A system FUNCTION call carries at least one expression (A.8.2
> `system_function_call` `:854-855`) — `r = $time()` now REJECTs — while a LATER slot may be empty (IEEE 1364-2005 §17.2.4's own
> `code = $fread( mem, fd, , count);`, `docs/verilog/2005/md/section-17-system-tasks-and-functions.md:1029`, the clause read against
> Annex A as `.3.34` read §12.6.1) and the system task enable `$display()` (A.6.9 `:624`) is untouched: a CONTEXT SPLIT at the
> expression-site reader. **(e)** A function's items and ports are inputs only (A.2.6
> `:263-266`) — `function f; output o; …`, `function f(output o)` now REJECT; a task keeps every direction. Each production
> is a pass-through dispatcher over the sv twin (the `1.0.233` rule VERBATIM) and a `verilog_2005` twin with the SAME
> template fields; three carrier facts were measured under the interpreter before a twin was written (an untyped rule
> renders `{body: child}`; a rule whose whole body is one optional renders it as a list; a header-less ANSI item carries
> a header object because the sv optional header matches empty), so every accepted tree is byte-identical — the pins
> pre-screen `ast_moved=0` and the corpus AST identity sweeps `ast_moved=0` on every lane. `1668 → 1715` rules. Corpus:
> the three lanes moved EXACTLY the nine pre-registered `verilog_2005` rows (four `.8c.2` rows and the five re-keyed CE rows; `pass 2 306 → 2 297`), all `pass → fail`, 0 the other way, and ZERO rows on `sv_2017` / `sv_2023` (totals identical to `1.0.233`'s); the corpus AST identity sweeps read `ast_moved=0` on every lane; the pins `ADJUDICATION-REPROS: checked=1469 armed=225 listed=731 multi_profile_rows=550 preprocess_rows=13 failures=0`; the `verilog_2005` bar 7 → 3. ⛔ **Consumers on `verilog_2005`**: Verilog text that only a tool's extension admits (a typed or
> defaulted input port, a `parameter` in a generate block, `dut.WIDTH` in a constant, `$time()`, an output on a function)
> is refused from `1.0.234`; the SV profiles are unchanged. ⛔ RESIDUAL, stated and ROUTED: a header-less ANSI item with a
> default — `inout [31:0] x, y = 1`, the `.8c.2` row `module_inout_port_list_def.v` — is legal iff the GOVERNING header is
> `output reg` / `output integer|time`, state across siblings the semantic store cannot express; pinned `accepts_invalid`, owned by
> `ENGINE-UNIVERSAL-SERVICES.108` (a latest-fact predicate), the row stays on the bar. Answer key: five ivtest CE rows
> (`module_nonansi_fail5.v` … `_fail9.v`) whose `.8c.2` key read `input integer x;` / `output real x;` as BNF-valid are re-keyed
> `must_reject` by A.2.1.2. Detail: the five ledger rows; `docs/tasks/SV-CORPUS-GRAD.md` `.3.66`.

> **Current-state note (2026-09-16, `SV-CORPUS-GRAD.3.65` — GRAMMAR + ANSWER-KEY tier, release `1.0.233`, ledger
> `SV-0112` / `SV-0113` / `SV-0114`, **schema `33` unchanged**, accept set NARROWS on `verilog_2005` ALONE; `sv_2017` /
> `sv_2023` byte-identical): THE STATEMENT TIER OF THE 1364-2005 BAR.** Three productions carried IEEE 1800's shape on
> the 1364-2005 profile. **(a)** A user tf-call: IEEE 1364-2005 A.8.2 `function_call ::= hierarchical_function_identifier
> { attribute_instance } ( expression { , expression } )` and A.6.9 `task_enable ::= hierarchical_task_identifier
> [ ( expression { , expression } ) ] ;` (`docs/verilog/2005/txt/section-Annex_A-normative-formal-syntax-definition.txt:852-853`,
> `:625`) put at least one expression inside the parentheses and know no named argument, where IEEE 1800-2017 A.8.2
> `tf_call ::= … [ ( list_of_arguments ) ]` (`docs/systemverilog/2017/md/section-41-data-read-api.md:2913`, `:2935-2938`)
> admits an empty list and `.name(expr)` — `f()`, `t();`, `t(.x(1))` now REJECT on `verilog_2005`; a system task enable
> (`$display()`, A.6.9 `:624`) is untouched. **(b)** A task body holds exactly ONE `statement_or_null` and a function
> body at least one item declaration (or a non-empty port list) and exactly one `function_statement`, with no end label
> (A.2.7 `:275-283`, A.2.6 `:254-262`, `:266`; 1800-2017 A.2.7 `:1124-1131`, A.2.6 `:1091-1100`) — a two-statement or
> empty body, `function f();`, `function f; f = 1; endfunction`, `endtask : t`, `endfunction : f` now REJECT there.
> **(c)** A block admits `block_item_declaration`s only BEHIND a label, lists `{ statement }` (never the null `;`,
> which is `statement_or_null`, A.6.4 `:555-556`) and has no end label (A.6.3 `:537-538`, `:529-530`; 1800-2017 A.6.3
> `:2064-2074`) — `begin integer x; … end`, `fork integer x; … join`, `begin ; end`, `end : b` now REJECT there. Each
> production is a pass-through dispatcher over the sv twin (the `1.0.232` rule VERBATIM) and a `verilog_2005` twin with
> the SAME template fields, so every accepted tree is byte-identical (measured under the interpreter on a synthetic
> grammar before a byte of SV was edited, then by the pins pre-screen `ast_moved=0` and the corpus AST identity sweeps);
> `1652 → 1668` rules. Corpus: the three lanes moved EXACTLY the ten pre-registered `verilog_2005` rows (the six `.8c.2` rows and the four `t()` rows; `pass 2 316 → 2 306`), all `pass → fail`, 0 the other way, and ZERO rows on `sv_2017` / `sv_2023` (totals identical to `1.0.232`'s); the corpus AST identity sweeps read `ast_moved=0` on every lane (every accepted file's typed AST byte-identical); the pins `ADJUDICATION-REPROS: checked=1343 armed=225 listed=667 multi_profile_rows=508 preprocess_rows=13 failures=0`; the `verilog_2005` bar 13 → 7. Answer key: a fourth edition pass (EDITION-EMPTY-CALL, the `.19`
> shape on the `)` of an empty user call, re-keyed `must_reject` iff `sv_2017` accepts the bytes) rules iverilog
> `pr1682887.v` and sv2v `logic_cond_tb.v` / `tf_block.v` / `top_tf.v`; eight control pins whose `verilog_2005` claim
> rested on 1800-only text are corrected. ⛔ **Consumers on `verilog_2005`**: text that IEEE 1364-2005 does not derive
> and a Verilog tool compiles as an extension (an empty user call, a multi-statement task body, a declaration in an
> unnamed block) is refused from `1.0.233`; the SV profiles are unchanged. Detail: the three ledger rows;
> `docs/tasks/SV-CORPUS-GRAD.md` `.3.65`.

> **Current-state note (2026-09-16, `SV-CORPUS-GRAD.3.64` — GRAMMAR tier, DELETIONS ONLY, release `1.0.232`, ledger
> `SV-0111`, **schema `33` unchanged**, accept set UNCHANGED on every profile, every typed AST byte-identical):
> THE FOUR GENERAL TERMINALS `1.0.231` ORPHANED ARE DELETED.** `1.0.231` moved `z_or_x` onto the boundary-free
> `edge_sym_*` terminals and left `kw_x_11f6ad8e` / `kw_X_c032adc1` / `kw_z_395df8f7` / `kw_Z_909f99a7` — the general
> single-letter keyword terminals it had been the only reader of — unreferenced: definition-only in the grammar
> (measured by `grep -cw`), unreachable from the entry (the syntax-closure gate listed them, `1 → 5`), PROOF in both
> certificates, and still compiled into the generated parser (15 mentions each). `1.0.232` deletes the four
> definitions (`1656 → 1652` rules) and the syntax-closure allow-list returns to ONE (`casting_type`) by the
> good-news branch — the `.3.30` → `.3.33` shape, one release apart. Nothing a consumer reads moves: no rule
> referenced the four, so no input could reach them, and that is measured on the final bytes rather than argued —
> the three lanes moved ZERO rows in either direction (totals identical to `1.0.231`'s: `sv_2017` 9 846 / 6 490, `sv_2023` 9 871 / 6 465, `verilog_2005` 2 316 / 290), the corpus AST identity sweeps read `ast_moved=0 verdict_moved=0` on every lane (every accepted file's typed AST byte-identical), the pins `ADJUDICATION-REPROS: checked=1265 … failures=0`, the closure gate `unreachable_rules: 1`. Consumers of `1.0.231` need do nothing. Detail: ledger `SV-0111`;
> `docs/tasks/SV-CORPUS-GRAD.md` `.3.64`.

> **Current-state note (2026-09-15, `SV-CORPUS-GRAD.3.63` — GRAMMAR tier, release `1.0.231`, ledger `SV-0108` / `SV-0109` /
> `SV-0110`, **schema `33` unchanged**, accept set WIDENS on every profile by the bracketed timing-check forms and on
> `verilog_2005` alone by the two-argument `$width`; NARROWS on every profile by one spelling no edition derives):
> ANNEX A'S BOLD BRACKETS ARE TOKENS.** Three timing-check productions had been transcribed with IEEE's literal
> brackets read as EBNF optionality — the PDF-to-text extraction drops the bold — and one carried the 1800 production
> where 1364-2005 differs. **(a)** `edge_control_specifier ::= edge [ edge_descriptor { , edge_descriptor } ]`
> (IEEE 1364-2005 A.7.5.3, `docs/verilog/2005/txt/section-Annex_A-normative-formal-syntax-definition.txt:813-820`;
> IEEE 1800-2017 A.7.5.3, `docs/systemverilog/2017/md/section-41-data-read-api.md:2846`; the clauses' own examples
> `edge[01, 0x, x1] clr`, §15.4 / §31.5): the rule now takes `lbrack` / `rbrack` tokens and `descriptors` is the list of
> `edge_descriptor` nodes; the two-character descriptors `0x`, `x1`, `1x`, `z0` are ONE token each, so `z_or_x` /
> `zero_or_one` sit on six boundary-free terminals `edge_sym_0` / `_1` / `_x` / `_x_upper` / `_z` / `_z_upper` (the general
> `kw_x_11f6ad8e := trivia /x\b/` could never match a half followed by a word character — the class `.3.30` measured on
> the UDP table symbols, the same remedy). A bracket-less `edge 01 CL`, accepted through `1.0.230`, is REJECTED: no
> edition derives it. **(b)** `delayed_data` / `delayed_reference ::= terminal_identifier | terminal_identifier
> [ constant_mintypmax_expression ]` (A.7.5.2, `:786-791` / `:2732`): the bracketed alternative — `with_expr`, the
> kind the contract has published since `1.0.64` but which was unreachable behind the bare identifier — comes first,
> with `lbrack` / `rbrack`; `$setuphold(…, C,,,D, BL_X[0]);` parses. **(c)** `$width ( controlled_reference_event ,
> timing_check_limit [ , threshold [ , notifier ] ] ) ;` — the threshold is OPTIONAL under IEEE 1364-2005 A.7.5.1
> (`:770-772`) and REQUIRED under IEEE 1800-2017 / 1800-2023 A.7.5.1 (`:2783-2786`; 2023 `section-Annex_A-normative-formal-syntax.md:2760-2763`):
> a `@profiles: ["verilog_2005"]` sub-rule `sv_dollar_width_timing_check_no_threshold_v2005` admits the two-argument
> form on the 1364-2005 profile ONLY, as `sv_dollar_width_timing_check -> {kind: "no_threshold", body: {controlled_reference_event,
> limit, tail}}`; the three-argument form keeps its shape byte-for-byte on every profile, and `sv_2017` / `sv_2023` still
> refuse the two-argument form (pinned). ⛔ **Consumers on `verilog_2005`**: a `system_timing_check` of kind `width` may
> now carry a body with no `threshold` field (the `no_threshold` wrapper) — read `body.kind` first. Measured on the
> final bytes: `sv_2017` pass 9 844 → 9 846 / fail 6 490, `sv_2023` 9 869 → 9 871 / 6 465, `verilog_2005` 2 313 → 2 316 / 290 — exactly the three predicted rows (`pr1520314.v`, `pr1867332.v` on every lane; `test_15_04_00_1.v` on `verilog_2005`), every one `fail → pass`, 0 the other way; the corpus AST identity sweeps read `ast_moved=0` on every lane (every file both binaries accept has a byte-identical typed AST) with `verdict_moved` 2 / 2 / 3, the predicted rows; the pipeline lane exactly 2 rows `fail → pass` (iverilog `pr1520314.v`, `pr1867332.v`), 0 the other way, `pass 12 202 → 12 204`; the `verilog_2005` bar 16 → 13 (`rejects_valid` 3 → 0 — the three rows this leaf was opened on; the thirteen `.8c.2` accepts-invalid pins are all that remain);
> pins `ADJUDICATION-REPROS: checked=1265 armed=231 listed=625 multi_profile_rows=480 preprocess_rows=13 failures=0`; `1649 → 1656` rules. Previous note (2026-09-14, `.3.60` + `.3.61`) follows.
> **Current-state note (2026-09-14, `SV-CORPUS-GRAD.3.60` + `.3.61` — GRAMMAR + ANSWER-KEY tier, release `1.0.230`, ledger
> `SV-0103` / `SV-0104` / `SV-0105` / `SV-0106`, **schema `32 → 33`**, accept set WIDENS on both SV profiles by six shapes — a name
> shadowing a `$unit` typedef as a net, a function, an ANSI port, a non-ANSI port declaration, a tf port and a parameter port
> typed after a comma (a typedef or a package-scoped type, after a parameter or a type assignment); and FOUR
> AST bindings a lookahead had shifted are corrected): THE COMPILATION-UNIT SCOPE AND THE MODULE ARE DIFFERENT NAME SPACES, AND
> A LOOKAHEAD IS A CHILD.** IEEE 1800-2017 3.13 (`docs/systemverilog/2017/md/section-2-normative-references.md:478`, `:492`) puts a
> `$unit` typedef in the compilation-unit scope name space and a same-named net, function, port or parameter in the module's, so
> `wire T = 1'b1;`, `function U;`, `module m(input T);`, `input T;` and `parameter int P = 5, T typed_value = 8'ha5` each declare a NEW
> name under an IMPLICIT type (A.2.1.3 `net_declaration`, A.2.6 `function_body_declaration`, A.1.3 `ansi_port_declaration` /
> `net_port_type`, A.2.7 `tf_port_declaration`, A.2.4 `param_assignment` footnote 18 — `section-41-data-read-api.md:805, :1091,
> :292, :900, :1160, :1002`). Through `1.0.229` each read only `data_type_or_implicit`, whose `data_type` arm consumed the typedef
> name and found no declarator (the five iverilog `sv_type_identifier_*.v` rows). `1.0.230` admits the typed reading only where a DECLARATOR follows —
> `data_type_or_implicit_declared := data_type &( declaration_identifier | hash ) | implicit_data_type`, one new rule (`1648 → 1649`) read
> by the net, net-port (ANSI and the non-ANSI `input` / `output` / `inout` of A.2.1.2) and tf-port declarations, the same guard on
> `var_data_type` and `function_data_type_or_implicit` — so the typedef name before `=` `;` `,` `)` is the declarator under the
> implicit reading of the same rule (no twin alternative: the engine derives every alternative, and a twin re-derived each typed
> declaration it also matched — `committed +3.41 %` on the refused first candidate). The default-less parameter arm carries
> `&( comma | rparen | semi )` — a default-less assignment is COMPLETE only there, so `T typed_value` and `p::PT pkg_value` after the
> comma are the next declaration; the `.3.35` type-list guard generalises the same way.
> **And the audit that the first of those guards provoked** (`.3.61`): in the engine a lookahead is a positional child that binds
> `[]`, so `$N` counts it and `$N::K*` reads the K-th child of each iteration guard included. Ten templates in the shipped grammar
> did not count it and stayed green under every gate (the accept set never moved; the identity sweep compares a release with the
> one before; the certificate reads reachability): `type_assignment`'s four twins bound `{name: $1, value: $2}` behind `!kw_type`,
> so **every `parameter type T = int` from `1.0.79` read `name: []`, `value: T` and carried NO default type** (`SV-0104`); the
> `.3.35` type lists (`1.0.210`) sliced the guard's comma in place of every assignment after the first (`SV-0105`); the `.13c.2s`
> argument lists (`1.0.191`) filled `ordered_tail` with the guard's empty nodes (`SV-0106`); the mixed string parameter port list
> bound two guard-carrying envelopes raw. `1.0.230` reads `{name: $2, value: $3}`, `[$1, $2::3*]`, `ordered_tail: [$2::3*]`,
> `type_tail: [$3::3*]` / `decl_tail: [$7::2*]`. ⛔ **Consumers — schema `33`**: a `type_assignment` node's `name` is now the
> declared type identifier and `value` the optional `( assign data_type )?` envelope (`[]` or `[{kind: "assign"}, <data_type>]`);
> a type list carries every assignment; a property / sequence instance's `ordered_tail` carries its further positional arguments;
> `mixed_string_parameter_port_list`'s `type_tail` / `decl_tail` are clean arrays. Measured two-sided on the final bytes: the
> corpus AST identity sweeps read `AST-IDENTITY-SWEEP: profile=sv_2017 files=16336 both_accepted=9847 identical=9651 ast_moved=196 verdict_moved=0 both_nonaccepting=6489`; `AST-IDENTITY-SWEEP: profile=sv_2023 files=16336 both_accepted=9872 identical=9668 ast_moved=204 verdict_moved=0 both_nonaccepting=6464`; `AST-IDENTITY-SWEEP: profile=verilog_2005 files=2606 both_accepted=2313 identical=2313 ast_moved=0 verdict_moved=0 both_nonaccepting=293` — the movers are 196 accepted files on `sv_2017` and 204 on `sv_2023`, every one a `.3.61` shape and nothing else (`rust/target/s360/classify_movers.py`, every mover's two trees diffed node by node: 193 / 201 files carry a `type_assignment` whose `name` and `value` are now bound — 822 / 838 nodes — 9 / 9 carry a type list whose second and later assignments replace a `comma` node — 26 / 26 elements — and 3 / 3 carry a property or sequence instance's `ordered_tail` — 19 / 19 arguments; 0 unclassified), the other 9 651 / 9 668 accepted trees byte-identical, `verilog_2005` untouched (2 313 identical); `TWO-SIDED sv_2017: ast_moved=196 predicted_ast_movers=178 ast_predicted_but_unmoved=0 ast_moved_but_not_predicted=18 … moved_but_not_predicted=0`, `sv_2023: ast_moved=204 predicted_ast_movers=186 … ast_moved_but_not_predicted=18`, `verilog_2005` all zero — every predicted mover moved; the 18 the grep-seeded prediction missed per profile were read tree by tree (`rust/target/s360/unpred/`): 14 default-less class type parameters (`class C #(type T)`, the `.3.61` binding), 4 property / sequence instance ordered tails — no other accepted tree moved a byte. Lanes `LANE-TOTALS: lane=sv_2017 rows=16336 pass=9847 fail=6489 timeout=0`, `LANE-TOTALS: lane=sv_2023 rows=16336 pass=9872 fail=6464 timeout=0`, `LANE-TOTALS: lane=verilog_2005 rows=2606 pass=2313 fail=293 timeout=0` — every count identical to 1.0.229's, 0 rows moved on any lane (the five iverilog rows sit behind a multi-line `` `define ``; the raw lane never reaches the shapes);
> 0 raw rows moved on any lane (the five iverilog rows sit behind a multi-line `` `define ``: the raw lane stops at the macro on every release, the shapes come after it). The PIPELINE lane: 16 336 rows, `pipeline pass 12 131 → 12 136`, exactly FIVE movers, all `fail → pass`, exactly the five rows the slice was opened on — iverilog `ivtest/ivltests/sv_type_identifier_net_name.v`, `_function_name.v`, `_port_name.v`, `_parameter_port_name.v`, `_task_function_argument_name.v` (each `[origin=…]` at the shadowing token on 1.0.229; `_port_name.v` and `_parameter_port_name.v` had needed the non-ANSI `input T;` and the `p::PT` continuation beyond the first candidate); 0 the other way; 0 same-verdict origin moves; the library pass unchanged (166 packages published, 5 074 member rows); the guard build and the refused twin build read the lane byte-identically (0 movers between them). Pins `ADJUDICATION-REPROS: checked=1173 armed=231 listed=589 multi_profile_rows=445 failures=0` (sixteen new: eight fixed, eight controls — five of the controls pin an ARM
> that is ABSENT on `1.0.229` and PRESENT on `1.0.230`, because a verdict cannot see a binding). Recurrence guard: the new doctrine
> `LOOKAHEAD-SLOT-TEMPLATES` (`scripts/check_lookahead_slot_templates.sh`, every commit; `TOOLBOX.md` §1.5i); the engine-side lint
> is routed to `ENGINE-UNIVERSAL-SERVICES.106`. Certificates union GREEN `1502/10/1477/15` + `1492/0` (+1 rule — `data_type_or_implicit_declared` — in the canonical population, +1 canonical witness, +1 union witness; canonical UNKNOWN 15, union UNKNOWN 0, residual `[]`, deterministic across seeds 0/7/42, `sample_parse_failures=0`), `verilog_2005` GREEN `1191/372/805/14` (`spf=0`; the twin candidate had read `1194/371/809/14` — a rule moved out of proof — the guard reads proof unchanged) — identities re-stamped; closure `sv_syntax_closure_gate` GREEN (allow-list unchanged, identity re-stamped); shape `[ast_shape_contract][systemverilog] samples=57 aligned=57 drift=0` (no shape sample carries a type parameter); reproducibility
> byte-identical; parse cost the twin candidate REFUSED — `entries +389,336 (+0.10 %)`, `committed 6,865,967 → 7,099,854 (+3.41 %)`, `memo_hits +275,561`: the tournament derives every alternative, so the implicit twins re-derived each typed declaration they also matched (`br_gh330.v` `committed +127,080` on `entries +245`, both function bodies parsed twice — read per rule with `--dump-rule-outcome-counts-json` on the 1.0.229 and the twin probes); the guard candidate `entries 402,201,215 → 402,211,106 (+0.00 %)`, `committed 6,865,967 → 6,867,224 (+0.02 %)`, `memo_hits 174,826,951 → 174,832,885 (+0.00 %)` — a RENAME (`data_type_or_implicit_declared +2,355 / +1,664` against `data_type_or_implicit −2,355 / −1,664` exactly), the guard's lookups (`declaration_identifier +2,141` entries, `non_keyword_identifier +2,141` memo hits; `comma` / `rparen` / `semi` / `hash` tries) and the kept pushes of the successful positive lookahead (`committed +2,507` outside the introduced reading, within the guards' 10,355 lookups) — ACCEPTED under the new coded invariant `guarded_typed_reading` (`ENGINE-UNIVERSAL-SERVICES.105`, three rows in `accepted_rises.tsv`, re-derived on the run: rename exact, no sampled file regressed, 8 rules rose in entries and 7 in memo hits all inside the 25-rule guard sub-graph or on the three declared hosts, 11 fell in entries and 28 in committed) and the ratchet rebaselined; the LR-family share `2.665` unchanged.
> **Current-state note (2026-09-10, `SV-CORPUS-GRAD.3.59` — GRAMMAR + ANSWER-KEY tier, release `1.0.229`, ledger `SV-0102`,
> schema `32` UNCHANGED, accept set WIDENS by two shapes — a queue element selected with `$` as a method call's receiver on
> both SV profiles, and the relative tolerance range `[A +%- B]` on `sv_2023`): A SELECT ON A RECEIVER IS AN EXPRESSION, AND
> `$` IS A PRIMARY.** IEEE 1800-2017 A.8.4 `primary ::= … | hierarchical_identifier select`, `bit_select ::= { [ expression ] }`,
> `$` among the primaries (7.10.4: `q[$]`, `q[$-1]`) and A.8.2 `method_call_root ::= primary | implicit_class_handle`
> (`docs/systemverilog/2017/md/section-41-data-read-api.md:3126, :3157, :2942`). Through `1.0.228` the chains' index operand
> (`hierarchical_index_operand`, the rule `.3.31` reserved for exactly that `bit_select`) carried only `constant_primary`, so
> `q[$-1].delete` rejected at the `$` while `q[0].delete` and `x = q[$-1]` parsed (verilator `t_dynarray_multid.v:201`).
> `1.0.229` adds `kw_dollar -> {kind: "dollar"}` as its LAST alternative (a `$` is never a constant_primary, so every earlier
> alternative keeps its tree). And IEEE 1800-2023 11.4.13 defines the relative tolerance token `+%-` beside `+/-`
> (`docs/systemverilog/2023/md/section-11-operators-and-expressions.md:1470-1473`; A.8.3 `value_range ::= … | [ expression +%-
> expression ]`, A.6.7.1 the covergroup twin — `section-Annex_A-normative-formal-syntax.md:2270, :1673`): `1.0.229` adds
> `lbrack expression plus percent minus expression rbrack -> {kind: "relative_tolerance", center, tol}` to `value_range_sv_2023`
> and `-> {kind: "relative_tolerance", body}` to `covergroup_value_range_sv_2023` (the `sv_2017` rules have no tolerance
> range at all, pinned). 1648 rules unchanged; no engine, no codegen, no Rust. ⛔ **Consumers**: on both SV profiles an index
> operand node may now read `{kind: "dollar"}` inside a chain's `{bits: […]}` select; on `sv_2023` a `value_range` /
> `covergroup_value_range` node may read `kind: "relative_tolerance"` — ADDITIVE kinds, no node the earlier releases emitted
> changes shape (the corpus AST identity sweeps read `AST-IDENTITY-SWEEP: profile=sv_2017 files=16336 both_accepted=9847 identical=9847 ast_moved=0 verdict_moved=0 both_nonaccepting=6489`; `AST-IDENTITY-SWEEP: profile=sv_2023 files=16336 both_accepted=9872 identical=9872 ast_moved=0 verdict_moved=0 both_nonaccepting=6464`; `AST-IDENTITY-SWEEP: profile=verilog_2005 files=2606 both_accepted=2313 identical=2313 ast_moved=0 verdict_moved=0 both_nonaccepting=293`). Lanes `LANE-TOTALS: lane=sv_2017 rows=16336 pass=9847 fail=6489 timeout=0`, `LANE-TOTALS: lane=sv_2023 rows=16336 pass=9872 fail=6464 timeout=0`, `LANE-TOTALS: lane=verilog_2005 rows=2606 pass=2313 fail=293 timeout=0` — every count identical to 1.0.228's, 0 rows moved on any lane (both shapes sit behind macros in the raw corpus). The
> PIPELINE lane, where the corpus spells the forms: 16 336 rows, `pipeline pass 12 130 → 12 131`, exactly ONE mover, `fail → pass`, exactly the predicted row — verilator `test_regress/t/t_dynarray_multid.v` (was `[origin=…:201:13]`, the `$` receiver); 0 the other way; 0 same-verdict origin moves (the twenty other `[$…].` files of the candidate list stay behind an earlier barrier — UVM); the library pass unchanged (166 packages published). Pins `ADJUDICATION-REPROS: checked=1141 armed=219 listed=573 multi_profile_rows=429 failures=0` (six new: the fixed queue receiver, its
> constant-index control, the relative tolerance on `sv_2023` with its `sv_2017` refusal twin, the streaming slice-size lock
> and its control). **The answer key**: verilator `t_stream_unpack.v:89` `{ >> bit[1:0] {arr2}} = bit6;` pinned `must_reject`
> — A.8.1 `slice_size ::= simple_type | constant_expression`, A.2.2.1 `simple_type` has no packed dimension (`:2896`, `:910`);
> the `.6b.4` reading had bucketed the legal streaming lvalue. Certificates union GREEN `1501/10/1476/15` + `1491/0` (unchanged — no rule added; canonical UNKNOWN 15, union UNKNOWN 0, residual `[]`, deterministic across seeds 0/7/42), `verilog_2005` GREEN `1190/372/804/14` (`spf=0`; corpus 369 checks / 0 mismatches) — identities re-stamped; closure `sv_syntax_closure_gate` GREEN (allow-list unchanged, identity re-stamped); shape `[ast_shape_contract][systemverilog] samples=57 aligned=57 drift=0`;
> reproducibility byte-identical; parse cost `entries` +185,338 (402,015,877 → 402,201,215, +0.05 %) and `memo_hits` +92,669 (174,734,282 → 174,826,951, +0.05 %), `committed` +0 — `kw_dollar` +92,669 entries / +0 memo and `trivia` +92,669 entries / +92,669 memo, every other rule flat: one net-new terminal alternative × 92,669 attempted positions × (1 body + 1 memoized trivia), entries exactly twice the memo hits — ACCEPTED under the coded `unmatched_terminal_alternatives` invariant (two rows in `accepted_rises.tsv`; the standard's own `$`, eliminating it changes the accept set) and the ratchet rebaselined; the LR-family share `2.666 → 2.665` adopted in its constant and anchors.
> **Current-state note (2026-09-09, `SV-CORPUS-GRAD.3.58` — GRAMMAR tier, release `1.0.228`, ledger `SV-0101`,
> schema `32` UNCHANGED, accept set WIDENS on `sv_2017` and `verilog_2005` by one shape — the implicit event
> expression with white space inside its parentheses): THE STANDARD'S NOTATION SEPARATES TOKENS BY WHITE SPACE.**
> IEEE 1800-2017 A.6.5 writes `event_control ::= … | @*  | @ (*)` (`docs/systemverilog/2017/md/section-41-data-read-api.md:654-658`),
> IEEE 1364-2005 A.6.5 the same (`docs/verilog/2005/md/section-Annex_A-normative-formal-syntax-definition.md:709`), and
> IEEE 1800-2023 A.6.5 spells the parenthesised form out as three tokens, `@ ( * )`
> (`docs/systemverilog/2023/md/section-9-processes.md:669-671`). Through `1.0.227` the `sv_2017` / `verilog_2005` rule
> `event_control_sv_2017` carried `@*` (`at_sign star`) and the glued `@(*)` (`at_sign attr_open rparen` — `(*` read as
> the attribute opener, so `@ (* )` parsed while `@( * )`, `@ ( * )` and `@ ( *)` rejected at the `*`); the `sv_2023` rule
> already carried `at_sign lparen star rparen`. `1.0.228` adds that alternative to the 2017/2005 rule
> (`{kind: "wildcard_paren"}`, the kind the 2023 rule already emits) and factors the `@` out of the six alternatives —
> `event_control_sv_2017 := at_sign event_control_after_at_sv_2017 -> $2`, a pass-through — so a position where no
> event control stands costs one `at_sign` attempt for the whole rule instead of one per alternative (the parse-cost
> ratchet measured the un-factored sixth alternative at +78,986 `at_sign` entries over the corpus and refused it;
> factored, the corpus cost FALLS: `entries` FELL 402,647,195 → 402,015,877 (−0.16 %) and `memo_hits` FELL 175,049,941 → 174,734,282 (−0.18 %) — `at_sign` 402,604 → 86,660 entries, one attempt per position instead of one per alternative — PROMOTED (the ratchet rebaselined and tightened); `committed` +88 (6,865,879 → 6,865,967), exactly the introduced wrapper's one frame per commit of the body it wraps, accepted under the coded `introduced_wrapper_frames` invariant (a row in `accepted_rises.tsv`, scope `event_control_after_at_sv_2017`); the LR-family share `2.659 → 2.666` adopted in its constant and anchors (the family's 23,552,489 entries unchanged over a smaller total). The FIRST candidate — the sixth alternative added un-factored — had been REFUSED: `entries` +158,397 (+0.04 %), `memo_hits` +79,271, `at_sign` +78,986 and `trivia` +79,126, one per failing position; no coded invariant covers a reducible shape, so it was eliminated rather than recorded); 1647 → 1648 rules; no engine, no codegen, no Rust.
> ⛔ **Consumers on `sv_2017` / `verilog_2005`**: an `event_control_sv_2017` node may now carry `kind: "wildcard_paren"`
> (bare, like `"wildcard"` / `"wildcard_alt"`) — an ADDITIVE kind, no node the earlier releases emitted changes shape
> (the corpus AST identity sweep on the final bytes reads `AST-IDENTITY-SWEEP: profile=sv_2017 files=16336 both_accepted=9847 identical=9847 ast_moved=0 verdict_moved=0 both_nonaccepting=6489`; `AST-IDENTITY-SWEEP: profile=sv_2023 files=16336 both_accepted=9872 identical=9872 ast_moved=0 verdict_moved=0 both_nonaccepting=6464`; `AST-IDENTITY-SWEEP: profile=verilog_2005 files=2606 both_accepted=2313 identical=2313 ast_moved=0 verdict_moved=0 both_nonaccepting=293` — the AST movers are
> the files that now parse and nothing else). The raw lanes: `LANE-TOTALS: lane=sv_2017 rows=16336 pass=9847 fail=6489 timeout=0`, `LANE-TOTALS: lane=sv_2023 rows=16336 pass=9872 fail=6464 timeout=0`, `LANE-TOTALS: lane=verilog_2005 rows=2606 pass=2313 fail=293 timeout=0` — every count identical to 1.0.227's, 0 rows moved on any lane (the spaced form sits behind macros in the raw
> corpus, so the raw lanes are byte-inert by construction — measured, not argued: the interpreter pre-screen read
> `PRESCREEN-PINS: jobs=1130 verdict_moved=2 ast_moved=0` — the two movers the new fixed pin's own arms — and
> `PRESCREEN-CORPUS: files=10 arms=20` with no mover). The
> PIPELINE lane, where the corpus actually spells the form: 16 336 rows, `pipeline pass 12 121 → 12 130`, exactly 9 movers, all `fail → pass`, exactly the predicted rows — sv2v `test/basic/sense_star.sv` (was `[origin=…:8:15]`) and the eight OpenTitan `hw/top_{earlgrey,darjeeling}/ip/ast/rtl/{vcaon,vcc,vcmain,vio}_pgd.sv` (were `:29:10` / `:29:26`); 0 the other way; 0 same-verdict origin moves; the library pass unchanged (166 packages published). Pins `ADJUDICATION-REPROS: checked=1130 armed=219 listed=567 multi_profile_rows=425 failures=0` (three new: the fixed shape on every
> profile, the glued control byte-identical, a `*` beside an operand rejected). Certificates union GREEN `1500/10/1475/15 → 1501/10/1476/15` + `1490/0 → 1491/0` (the introduced `event_control_after_at_sv_2017` witnessed canonically and in the union; UNKNOWN 15 / 0 unchanged, residual `[]`, deterministic across seeds 0/7/42), `verilog_2005` GREEN `1189/372/803/14 → 1190/372/804/14` (`spf=0`; corpus 369 checks / 0 mismatches) — both contracts adopted by name in their notes and re-stamped; closure `sv_syntax_closure_gate` GREEN (allow-list unchanged, identity re-stamped);
> shape `[ast_shape_contract][systemverilog] samples=57 aligned=57 drift=0`; reproducibility byte-identical; parse cost `entries` FELL 402,647,195 → 402,015,877 (−0.16 %) and `memo_hits` FELL 175,049,941 → 174,734,282 (−0.18 %) — `at_sign` 402,604 → 86,660 entries, one attempt per position instead of one per alternative — PROMOTED (the ratchet rebaselined and tightened); `committed` +88 (6,865,879 → 6,865,967), exactly the introduced wrapper's one frame per commit of the body it wraps, accepted under the coded `introduced_wrapper_frames` invariant (a row in `accepted_rises.tsv`, scope `event_control_after_at_sv_2017`); the LR-family share `2.659 → 2.666` adopted in its constant and anchors (the family's 23,552,489 entries unchanged over a smaller total). The FIRST candidate — the sixth alternative added un-factored — had been REFUSED: `entries` +158,397 (+0.04 %), `memo_hits` +79,271, `at_sign` +78,986 and `trivia` +79,126, one per failing position; no coded invariant covers a reducible shape, so it was eliminated rather than recorded.
> **Current-state note (2026-09-09, `SV-CORPUS-GRAD.3.56` — GRAMMAR + ANSWER-KEY, release `1.0.227`, ledger `SV-0100`,
> schema `32` UNCHANGED, accept set WIDENS on every profile by two lexical shapes — a multi-line `` `define `` and a
> based literal whose tokens are separated by newlines or comments): A LEXICAL FACT LIVES IN THE TERMINAL'S REGEX.**
> IEEE 1800-2017 §22.5.1 — the newline preceded by a backslash continues the macro text, the first newline not so
> preceded ends it (`docs/systemverilog/2017/md/section-22-compiler-directives.md:171-174`) — and §5.7.1 — a based
> literal is three tokens, the unsigned number "optionally preceded by white space", the apostrophe and the base
> inseparable (`docs/systemverilog/2017/md/section-1024-characters-if-an-identifier-exceeds-the-implementation-specific-length-limit-an-error-shall-be-reported.md:191-210`),
> §5.3 white space being spaces, tabs, newlines and formfeeds. Through `1.0.226` the opaque directive line ended at its
> first newline, so a multi-line macro's continuation lines were parsed as items (500 failing lane files carry the
> shape), and a based literal admitted only spaces and tabs between its tokens. `1.0.227` spells both in the terminals
> (`opaque_directive_line`, `integral_number`); 1647 rules unchanged. And, exposed by that widening, `time_literal`'s magnitude is restricted to `unsigned_number`/`real_number`
> (A.8.7 `time_literal ::= unsigned_number time_unit | fixed_point_number time_unit`), dropping a based-literal
> over-acceptance (0 corpus rows; the value node byte-identical for every real time literal). ⛔ **Consumers on every profile**: a
> `{kind: "opaque", text}` directive node now carries the WHOLE macro text of a multi-line `` `define ``, backslashes and
> newlines included, where `1.0.226` gave you its first line and then the continuation lines as whatever they parsed as
> — the three accepted corpus files carrying the shape are the only AST movers (friscv `rtl/friscv_h.sv`, sv-tests `tests/generic/preproc/preproc_test_8.sv`, uvm-core `src/deprecated/macros/uvm_sequence_defines.svh`; the corpus AST identity
> sweep on the final bytes reads `AST-IDENTITY-SWEEP: profile=sv_2017 files=16336 both_accepted=9771 identical=9768 ast_moved=3 verdict_moved=76 both_nonaccepting=6489`; `AST-IDENTITY-SWEEP: profile=sv_2023 files=16336 both_accepted=9796 identical=9793 ast_moved=3 verdict_moved=76 both_nonaccepting=6464`; `AST-IDENTITY-SWEEP: profile=verilog_2005 files=2606 both_accepted=2310 identical=2310 ast_moved=0 verdict_moved=3 both_nonaccepting=293` — on each SV lane exactly 3 accepted trees move (the three files whose multi-line `` `define `` bodies fold into the directive's opaque text) and 76 files newly parse; 0 accepted-tree moves on `verilog_2005`); a based literal's `{kind: "integral", body}` keeps the source text verbatim,
> newlines and comments included when they were written (no accepted file had them). The verdict movers: 76 corpus files `fail → pass` on each SV lane (macro-heavy headers and number-whitespace tests across opentitan, black-parrot, friscv, sv2v, Surelog, verilator) and 3 on `verilog_2005` (sv2v `default_nettype.v`, `pattern_revised.v`, `typename_deep.v`), 0 the other way; the three AST movers are the already-accepted define files friscv `friscv_h.sv`, sv-tests `preproc_test_8.sv`, uvm-core `uvm_sequence_defines.svh`.
> **The answer key** re-keys twelve bar rows `must_reject` by production, each with an isolation reproducer: bare
> `begin … end` blocks as module items and inside generate regions (A.1.4, A.4.2 — six verilator rows), a statement as
> a generate item (A.4.2, Surelog), a bare brace pair in a covergroup (A.2.11), a backslash-newline in a UDP table row
> (A.5.4), free text between pragma comments (§5.4), a pipe-filter placeholder (A.1.2), and `` `default_nettype `` inside
> a module (§22.8: outside design elements only). Lanes `LANE-TOTALS: lane=sv_2017 rows=16336 pass=9847 fail=6489 timeout=0` (was 9771 / 6565 at 1.0.226 — 76 files `fail → pass`, 0 the other way), `LANE-TOTALS: lane=sv_2023 rows=16336 pass=9872 fail=6464 timeout=0` (was 9796 / 6540 — the same 76), `LANE-TOTALS: lane=verilog_2005 rows=2606 pass=2313 fail=293 timeout=0` (was 2310 / 296 — 3 files, the two lexical facts 1364-2005 shares); pins `ADJUDICATION-REPROS: checked=1121 armed=219 listed=564 multi_profile_rows=422 failures=0` (was 1078 / 219 / 548 / 406; the sixteen new pins green on the final bytes); union `sv_cert_recognized_union_gate` GREEN — canonical `1500/10/1475/15` UNCHANGED from `1.0.226` (no rule added), union witness `1490`, union UNKNOWN `0`, residual `[]`, `sample_parse_failures=0`, deterministic across seeds 0 / 7 / 42; identity re-stamped (the baseline was stale on the grammar digest and every expectation held). ⛔ The FIRST candidate (before the `time_literal` restriction) REFUSED with `sample_parse_failures` non-deterministic across seeds (3 / 2 / 0): the widened `integral_number` let the witness generator compose a based-literal time literal (`timeprecision'D 9F? fs`) whose interior layout the glued unit must refuse; restricting `time_literal` to `unsigned_number` / `real_number` (IEEE 1800-2017 A.8.7) cleared it deterministically; `verilog_2005_conformance_gate` GREEN — cert `1189/372/803/14` UNCHANGED, `corpus matrix: 369 file×profile checks, 0 mismatches`; identity re-stamped; `sv_syntax_closure_gate` GREEN — `reachable_rules: 1646 unreachable_rules: 1 unreachable_branches: 2`, the allow-list unchanged at ONE (`casting_type`), identity re-stamped; `ast_shape_contract_gate` rc 0 (`18 passed`); `[ast_shape_contract][systemverilog] samples=57 aligned=57 drift=0 regression_lock_failures=0`; NO re-anchor — the `time_literal` value node is byte-identical for every real time literal and the H.11.5 locks use plain-number glued time literals;
> `check_generated_reproducibility.sh --rebaseline` — TIER 2, every family re-derives byte-identically from HEAD (`systemverilog` 0 sites, every other family 0 sites); `generated_reproducibility_v0.json` re-stamped: `systemverilog` `parser_sha` `→ 423042d2…`, `input_sha` `→ 968033d4…`; `sv_parse_cost_family_share` derived `2.659 %` (`23,552,489 of 885,794,193` entries over 16 336 files) against the carried `2.660` — ADOPTED (`CORPUS_FAMILY_SHARE_PCT` + the four prose anchors; `2.659` reproduces exactly on re-run); `sv_parse_cost_rebaseline` PROMOTED — `entries 403,243,744 → 402,647,195 (−0.15 %)` an improvement (the `time_literal → unsigned_number` re-spelling makes 149 sampled files cheaper), `committed 6,865,447 → 6,865,879 (+432)` and `memo_hits 175,043,676 → 175,049,941 (+6,265)` ACCEPTED as attributed (`accepted_rises.tsv`, invariant `advanced_files_only`): the rise is confined to the three sampled files the opaque-directive-line widening let parse further (`default_nettype.v` and `uvm_recorder_defines.svh` flip no→yes, `alert_handler_base_vseq.sv` advances furthest 334→1475), no non-advanced file rose; `sv_parse_cost_ratchet` GREEN on the promoted baseline (wall-clock advisory `+8.7 %` within the ±50 % band, machine-dependent). ⛔ `advanced_files_only` was EXTENDED this slice (`ENGINE-UNIVERSAL-SERVICES.104`) to admit a non-scoped PURE IMPROVEMENT — a file that only got cheaper cannot hide a rise — with a self-test arm (12/12).
>
> **Current-state note (2026-09-08, `SV-CORPUS-GRAD.3.55` — ENGINE + GRAMMAR + ANSWER-KEY, release `1.0.226`,
> ledger `SV-0099`, schema `32` UNCHANGED, accept set WIDENS on `sv_2017` / `sv_2023` by exactly one shape — a
> number, white space and a unit-spelled identifier after a delay — and NARROWS by the spaced time literal no
> edition admits): A TIME UNIT AFTER WHITE SPACE IS AN IDENTIFIER.** IEEE 1800-2017 A.8.7 `time_literal ::=
> unsigned_number time_unit | fixed_point_number time_unit` with Annex A footnote 44 — "the unsigned number or
> fixed-point number in time_literal shall not be followed by white_space"
> (`docs/systemverilog/2017/md/section-41-data-read-api.md:3149-3153`, `:3567`; 1800-2023 footnote 49,
> `docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:3101-3105`, `:3532`). Through `1.0.225`
> the unit followed the number through the engine's layout skip, so `1 s` WAS the time literal `1s`: `#1 s = 0;`
> lost its variable and `trans ##1 s` its sequence (ispras `16.08_04.sv`, `16.09.11_01.sv` — §16.8 / §16.9.11's own
> examples). `1.0.226` spells the footnote with the new rule-level `@glued: true` directive
> (`ENGINE-UNIVERSAL-SERVICES.103`: the rule's regex terminals accept no leading layout) on
> `time_unit_glued := /s\b/ | /ms\b/ | /us\b/ | /ns\b/ | /ps\b/ | /fs\b/`, reached by `time_literal := number
> time_unit_glued`; the `` `timescale `` directive keeps its layout-skipping `time_unit` (§22.7 allows `1 ns / 1 ps`);
> `1646 → 1647` rules. ⛔ **Consumers on every profile — nothing moves for any file you already read**: the time
> literal keeps its `{value: {kind: "integral" | "real", body}, unit: {kind}}` node byte for byte (the corpus AST
> identity sweep on the final bytes reads `AST-IDENTITY-SWEEP: profile=sv_2017 files=16336 both_accepted=9767 identical=9767 ast_moved=0 verdict_moved=4 both_nonaccepting=6565`; `AST-IDENTITY-SWEEP: profile=sv_2023 files=16336 both_accepted=9792 identical=9792 ast_moved=0 verdict_moved=4 both_nonaccepting=6540`; `AST-IDENTITY-SWEEP: profile=verilog_2005 files=2606 both_accepted=2310 identical=2310 ast_moved=0 verdict_moved=0 both_nonaccepting=296` — every tree both probes accept is byte-identical on every lane, the 240 accepted files with glued time literals among them — `ast_moved=0` on every lane); a delay or cycle delay you
> never received because its operand was spelled like a unit arrives as the delay and the identifier (the verdict
> movers: ispras `16.08_04.sv` and `16.09.11_01.sv`, Surelog `FSMBsp13/top.v`, iverilog `pr2785294.v` — `fail → pass` on `sv_2017` and `sv_2023`, 0 the other way; `verdict_moved=0` on `verilog_2005`); `timeprecision 1 ps;` now REJECTS at the `ps` (0 corpus rows carried it). **The
> answer key** re-keys six bar rows `must_reject` by production, each with an isolation reproducer and a control
> pin: an initializer on a non-ANSI NET output port (A.2.1.2 `list_of_port_identifiers`, verilator), a default on a
> non-ANSI input port (A.2.1.2, sv2v), a modport inside a generate loop (A.1.4 `interface_or_generate_item`,
> sv2v), a bare `begin … end` inside `generate` (A.4.2 `generate_item`, sv2v), and a port declaration in the body
> of a module whose header lists no ports (A.1.2: such a header is ANSI and its body admits
> `non_port_module_item` only — iverilog, and an ispras NEGATIVE test the `.8b.2` reading had keyed on its other
> illegality). Lanes `LANE-TOTALS: lane=sv_2017 rows=16336 pass=9771 fail=6565 timeout=0` (was 9767 / 6569 at 1.0.225 — the four pre-registered rows `fail → pass`), `LANE-TOTALS: lane=sv_2023 rows=16336 pass=9796 fail=6540 timeout=0` (was 9792 / 6544 — the same four), `LANE-TOTALS: lane=verilog_2005 rows=2606 pass=2310 fail=296 timeout=0` (byte-identical — time literals are IEEE 1800's); pins `ADJUDICATION-REPROS: checked=1078 armed=219 listed=548 multi_profile_rows=406 failures=0` (was 1050 / 219 / 534 / 392; the fourteen new rows green on the final bytes); union `sv_cert_recognized_union_gate` — canonical `1499/10/1474/15 → 1500/10/1475/15` (+1 total, +1 witness: `time_unit_glued`, witnessed by the canonical config's own time-literal samples; proof 10 and the canonical UNKNOWN 15-rule set unchanged, identical), union witness `1489 → 1490`, union UNKNOWN 0, residual `[]`, deterministic across seeds 0 / 7 / 42. The FIRST run on the final bytes refused twice over: the totals (`seed=0 canonical total=1500 (expected 1499)`) and `sample_parse_failures=10 / 10 / 8` on seeds 0 / 7 / 42 — the witness generator's sequence join wrote `4 us`, `6 ms`, `1 ps`, which the glued unit rightly refuses; the generator's join now appends a glued rule reference with no separator (`append_sequence_segment`, test `a_glued_rule_reference_renders_without_a_separator`), the expectations were stamped from the per-seed `summary.json` with a `.3.55` baseline note, and the second run confirmed them with `sample_parse_failures=0` and re-stamped the identity; `verilog_2005_conformance_gate` — cert `1188/371/803/14 → 1189/372/803/14` (`CERTIFICATE-COVERAGE: grammar='systemverilog' entry='systemverilog_file' samples=40 total=1189 proof=372 witness=803 UNKNOWN=14 fully_certified=false (sample_parse_failures=0, proof_reverify_failures=0)`: +1 total and +1 proof on `time_unit_glued`, reached only through the SV-only `time_literal` and so provably unreachable under `verilog_2005` exactly as `time_literal` is; witness and UNKNOWN unchanged, deterministic across seeds 0 / 7 / 42), `corpus matrix: 369 file×profile checks, 0 mismatches` unchanged; the gate's first run refused (`seed=0 cert total=1189 (expected 1188)` + STALE), stamped from the three-seed census with a `.3.55` note and confirmed by a second GREEN run that re-stamped the identity; `sv_syntax_closure_gate` GREEN — `reachable_rules: 1645 → 1646 unreachable_rules: 1 unreachable_branches: 2` (the glued unit rule), the allow-list unchanged at ONE (`casting_type`), identity re-stamped; `ast_shape_contract_gate` rc 0; `[ast_shape_contract][systemverilog] samples=57 aligned=57 drift=0 regression_lock_failures=0 aligned_assertion_failures=0 samples_per_status={"aligned": 57}` (no new sample: no template changed — a time literal keeps its node); the gate's FIRST run on the final bytes failed ONE regression lock, `systemverilog_hash_token_is_not_stolen_by_comment_arms_during_speculation` (GRAMMAR-WELLFORMED.H.11.5), whose four samples spelled `timeunit 09 ns` / `/633 s` — spaced time literals the pre-1.0.226 grammar accepted against footnote 44; the samples are respelled `09ns` / `633s` (the lock's subject, the `#` arm during speculation over the comment tail, is unchanged) and the gate is GREEN on the re-anchored lock; `check_generated_reproducibility.sh --rebaseline` — TIER 2, every family re-derives byte-identically from HEAD (`systemverilog` 0 sites and every other family 0 sites), the regenerated parser carrying the glued unit's no-skip atoms; `generated_reproducibility_v0.json` re-stamped: `systemverilog` `parser_sha` `bde02432… → e1e5a9d8…`, `input_sha` `be0d073b… → b49eb8d7…`, `verified_at_commit` `89fb21d3 → ac575805`; `sv_parse_cost_family_share` derived `2.660 %` (`23,503,030 of 883,497,827` entries over 16 336 files; the corpus lost 4.9 M entries) against the carried `2.645` — ADOPTED (`CORPUS_FAMILY_SHARE_PCT` and the four prose anchors; `--verify-family-share` fresh); the rebaseline read every binding metric FALLING — `entries 405,659,946 → 403,243,744 (−0.60 %)`, `committed 6,865,461 → 6,865,447 (−14)`, `memo_hits 176,251,757 → 175,043,676 (−0.69 %)` — attributed per rule (`rule_costs.tsv` HEAD vs promoted, 15 rows moved): each of the 201,348 time-literal attempts on the pinned sample used to enter `time_unit` and then SIX keyword-rule frames (`kw_s_a0f1490a` … `kw_fs_3f4bb586`, −201,348 entries each) with their six `trivia` frames (`trivia` −1,208,088 entries, −1,208,081 memo hits); it now enters ONE `time_unit_glued` frame (+201,348 entries, +7 commits — the seven time literals the sample commits, whose unit / keyword / trivia commits are the −14); promoted on the adopted share (`cost.md` re-derived on the `.3.55` digest, share `2.660`); `sv_parse_cost_ratchet` GREEN on the promoted baseline; the wall clock advisory `46674 ms → 45551 ms (−2.4 %)` within the ±50 % band, machine-dependent;
> `parse_harness_semantic_gate` — `2 passed` (`✅ Parse-harness semantic-directive orchestration suite passed`, every suite pin `CLEAN … diverge=0 anchor_miss=0`); `parse_harness_equivalence_gate` — `4 passed` (`✅ Parse-harness differential-equivalence gate passed`: the generated parser and the parse-harness interpreter byte-identical on every CERTIFIED family over the stimuli corpus at seeds 0 / 7 / 42, the regenerated SystemVerilog parser included — the `@glued` decision is read by both through one function); `parse_harness_combinator_gate` — `2 passed` (`✅ Parse-harness structural combinator suite passed`, every combinator pin `CLEAN`).
>
> **Current-state note (2026-09-08, `SV-CORPUS-GRAD.3.54` — GRAMMAR + ANSWER-KEY, release `1.0.225`, ledger `SV-0098`,
> schema `32` UNCHANGED, accept set WIDENS on `sv_2017` / `sv_2023` by exactly one shape — a declared package used as a
> scope while a typedef of the same name is visible — and moves nowhere else): A PACKAGE NAME LIVES IN ITS OWN NAME
> SPACE.** IEEE 1800-2023 §3.13 (`docs/systemverilog/2023/md/section-3-design-and-verification-building-blocks.md:420-432`) — the 1800-2017
> sentence at `docs/systemverilog/2017/md/section-3-park-avenue.md:432`, a vendored file whose only headings are the address page, cited by line — and A.9.3 `package_scope ::=
> package_identifier :: | $unit ::` (`docs/systemverilog/2017/md/section-41-data-read-api.md:3390`) make `T::VALUE` a
> package scope whenever `package T` is declared; §26.3 (`docs/systemverilog/2017/md/section-26-packages.md:206-210`)
> keeps a wildcard-imported typedef `T` merely potentially visible. Through `1.0.224` the package arm every `pkg::` site
> reaches (`non_typedef_package_scope`) carried one gate — "the name is no typedef" — so a typedef of the name refused
> the scope and every class-scope arm refused too (`T` is no class): iverilog `sv_type_identifier_package_name.v`
> stopped at the `VALUE` after `T::`. `1.0.225` makes that exclusion branch-local and adds a second alternative admitting
> the scope by its `package_name` fact under `@branch_policy: ordered` (tried only when the first alternative fails, its prefix answered from the memo); both
> alternatives carry the same template, 1646 rules unchanged. ⛔ **Consumers on every profile — nothing moves for any
> file you already read**: the corpus AST identity sweep on the final bytes reads `AST-IDENTITY-SWEEP: profile=sv_2017 files=16336 both_accepted=9767 identical=9767 ast_moved=0 verdict_moved=0 both_nonaccepting=6569`; `AST-IDENTITY-SWEEP: profile=sv_2023 files=16336 both_accepted=9792 identical=9792 ast_moved=0 verdict_moved=0 both_nonaccepting=6544`; `AST-IDENTITY-SWEEP: profile=verilog_2005 files=2606 both_accepted=2310 identical=2310 ast_moved=0 verdict_moved=0 both_nonaccepting=296` — every tree both probes accept is byte-identical and no verdict moved on any lane — `ast_moved=0` on every
> lane, `verdict_moved=0` on every lane (`verdict_moved=0` on `sv_2017`, `sv_2023` and `verilog_2005`; the iverilog row `sv_type_identifier_package_name.v` advances `247 → 331` on both SV lanes and stays a non-accept); a `pkg::` scope you never received arrives as the same
> `{kind: "package", name}` node `non_typedef_package_scope` has always carried, in the same slots; a typedef that names
> no package still refuses (pinned). **The answer key** re-keys six bar rows `must_reject` by production, each with an
> isolation reproducer and a control pin: a member select or an indexed member as a `std::randomize` argument (18.12
> Syntax 18-11 `variable_identifier_list`, two verilator rows), an `extern function` prototype with no return type in a
> class and in an interface (A.2.6 `function_prototype`, two verilator rows), a bodiless `virtual` method with neither
> `pure` nor `extern` (A.1.9 `class_method`, an ispras NEGATIVE test the `.8b.2` reading had keyed on its other
> illegality), and a trailing comma in a named port-connection list (A.4.1.1, a verible kythe fixture whose class
> `environment` no fixture file declares — planted, the parse advances to the comma). Lanes `LANE-TOTALS: lane=sv_2017 rows=16336 pass=9767 fail=6569 timeout=0`, `LANE-TOTALS: lane=sv_2023 rows=16336 pass=9792 fail=6544 timeout=0`, `LANE-TOTALS: lane=verilog_2005 rows=2606 pass=2310 fail=296 timeout=0` — every total identical to 1.0.224's (the one file carrying the shape advances and stays a non-accept); pins `ADJUDICATION-REPROS: checked=1050 armed=219 listed=534 multi_profile_rows=392 failures=0` (was 1022 / 219 / 520 / 378; the fourteen new rows green on the final bytes);
> union `sv_cert_recognized_union_gate` — the FIRST run on the final bytes REFUSED with the planner flattening the new branch gate: canonical `1499/10/1461/28`, union witness `1482`, union UNKNOWN **7** on every seed (`class_scoped_tf_call`, `extern_constraint_declaration`, `extern_constraint_declaration_sv_2017`, `kw_implements_e133e2cb`, `method_call_receiver_qualifier`, `scoped_base_class_type_identifier`, `scoped_interface_class_type_identifier` — the `pkg::`-witnessed rules); on the branch-local planner (`BRANCH-PREDICATE-LOCALITY.3`) the second run read canonical `1499/10/1474/15` (the IDENTICAL 15-rule UNKNOWN set), union witness `1489`, union UNKNOWN 0, residual `[]`, deterministic across seeds 0 / 7 / 42 — every number 1.0.224's, no rule added or removed — the baseline STALE on `stimuli_generator.rs` and every expectation holding, so the gate re-stamped the identity; `verilog_2005_conformance_gate` — cert `1188/371/803/14` UNCHANGED (`CERTIFICATE-COVERAGE: grammar='systemverilog' entry='systemverilog_file' samples=40 total=1188 proof=371 witness=803 UNKNOWN=14 fully_certified=false (sample_parse_failures=0, proof_reverify_failures=0)`, deterministic across seeds 0 / 7 / 42; the rule is SV-only, so nothing enters or leaves that profile's universe), `corpus matrix: 369 file×profile checks, 0 mismatches`, aliases 2 — GREEN on the first run of the final bytes and again on the branch-local planner, each time STALE on a declared input and every criterion holding, so the gate re-stamped the identity; `sv_syntax_closure_gate` GREEN — `reachable_rules: 1645 unreachable_rules: 1 unreachable_branches: 2` unchanged (1646 rules, no rule added), the allow-list unchanged at ONE (`casting_type`), identity re-stamped twice (on the final bytes, and again STALE on `stimuli_generator.rs` after the planner fix); `ast_shape_contract_gate` rc 0; `[ast_shape_contract][systemverilog] samples=57 aligned=57 drift=0 regression_lock_failures=0 aligned_assertion_failures=0 samples_per_status={"aligned": 57}` (no new sample: no template changed — the package scope keeps its `{kind: "package", name}` node in every slot); `check_generated_reproducibility.sh --rebaseline` — TIER 2, every family re-derives byte-identically from HEAD (`systemverilog` 0 sites and every other family 0 sites), run twice (on the final bytes, and again after the grammar's comment edit moved the input hash); `generated_reproducibility_v0.json` re-stamped: `systemverilog` `parser_sha` `fc9962e0… → bde02432…`, `input_sha` `de4c7c04… → be0d073b…`, `verified_at_commit` `85e2bdbd → 89fb21d3`; `sv_parse_cost_family_share` derived `2.645 %` (`23,500,802 of 888,378,865` entries over 16 336 files) against the carried `2.650` — ADOPTED (`CORPUS_FAMILY_SHARE_PCT` and the four prose anchors in `TOOLBOX.md`, `DOCTRINE_ENFORCEMENT.md`, `docs/book/src/the-enforced-doctrines.md`, `docs/knowledge/a-deterministic-counter-cannot-see-a-per-entry-cost-rise.md`; `--verify-family-share` fresh); the first rebaseline REFUSED two binding rises — `entries 404,943,513 → 405,659,946 (+0.18 %)`, `memo_hits 175,533,047 → 176,251,757 (+0.41 %)`, `committed` conserved exactly — attributed per rule (`rule_costs.tsv` HEAD vs fresh, 44 rows moved): ONE riser, `package_scope` `+721,482` entries = `+721,482` memo hits, committed +0 — wherever alternative 1 FAILS for lack of an `identifier ::` (721,482 of the rule's 731,273 fresh evaluations on the pinned sample) the ordered choice tries alternative 2 and the engine answers its `package_scope` from the memo — and forty-three small fallers (`non_typedef_package_scope` −873 / −1,528 memo, `non_keyword_identifier` −781, `declaration_identifier` −465, `trivia` −395, `class_scope` −299 …) from the memo answering what it used to re-ask; no rule introduced, no sampled file moved its verdict or furthest (the one corpus file the change touches is not in the sample); two rows recorded in `accepted_rises.tsv` under `lookahead_replay_occurrences` (introduced `!package_scope ~non_typedef_package_scope`) with the WHY — a dispatcher helper would enter one frame per attempt (7,232,230 on this sample, ten times this rise), a terminal lookahead would trade one replay for another, the predicate language has no disjunction — and the predicate's docstring, self-test (`arms=16 → 18`) and the book name the SECOND SHAPE: an ordered choice whose alternatives share a prefix replays that prefix from the memo, the same clauses reading the same artefact without a lookahead in the spelling (the `.100` record carries the addendum); the second rebaseline read both rises `ACCEPTED as attributed … under lookahead_replay_occurrences (SV-CORPUS-GRAD.3.54)` with the invariant RE-DERIVED on the run (`` `package_scope` entries +721,482 = memo_hits +721,482, committed +0; every sampled file keeps its verdict and furthest; 0 rule(s) committed more (+0) …; 1 rule(s) rose in entries (+721,482), all reachable from the introduced / un-gated rules; 30 fell (-5,049) — an occurrence-count artefact, not re-done work ``) and PROMOTED (`cost.md` re-derived on the `.3.54` digest, share `2.645`); `sv_parse_cost_ratchet` GREEN on the promoted baseline; the wall clock advisory `44569 ms → 46674 ms (+4.7 %)` within the ±50 % band, machine-dependent.
>
> **Current-state note (2026-09-08, `SV-CORPUS-GRAD.3.53` — GRAMMAR, release `1.0.224`, ledger `SV-0097`, schema `32`
> UNCHANGED, accept set WIDENS on `sv_2017` / `sv_2023` by the parenthesised operator assignment as a chain OPERAND and
> NARROWS on `verilog_2005` by the same form at the top of an expression): THE PARENTHESISED OPERATOR ASSIGNMENT IS AN
> EXPRESSION, HENCE AN OPERAND — AND IT IS IEEE 1800'S, NOT 1364'S.** IEEE 1800-2017 A.8.3 `expression ::= … |
> ( operator_assignment ) | expression binary_operator { attribute_instance } expression`
> (`docs/systemverilog/2017/md/section-41-data-read-api.md:3026-3034`; §11.4.2's own example `j = i++ + (i = i - 1);`,
> `docs/systemverilog/2017/md/section-11-operators-and-expressions.md:502`; 1800-2023
> `docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:2988-2996`) — through `1.0.223` the grammar
> admitted the form only as an alternative of `expression_base`, so the LRM's example REJECTED at the `(` after `+`
> (three bar rows: ispras `11.04.02_01.sv`, sv-tests `11.3.6--two_assign_in_expr.sv` and its `-sim` twin), and the
> alternative carried no `@profiles`, so `j = (i = i - 1);` PARSED under `verilog_2005`, whose IEEE 1364-2005 A.8.3
> `expression` (`docs/verilog/2005/md/section-Annex_A-normative-formal-syntax-definition.md:1070-1074`) has no assignment
> inside an expression. `1.0.224` spells `paren_operator_assignment_operand := lparen operator_assignment rparen ->
> {kind: "paren_op_assign", body: $2}` under `@profiles: ["sv_2017", "sv_2023"]`, reached from every chain step through
> the ORDERED pass-through `expression_chain_operand := expression_operand -> $1 | paren_operator_assignment_operand ->
> $1` (the two alternatives are disjoint, so `ordered` changes no verdict), as a chain head with
> `expression_chain_step+`, and from `hierarchical_index_operand`; the top-level alternative passes the same node through
> (`1644 → 1646` rules). ⛔ **Consumers on `sv_2017` / `sv_2023` — nothing moves for any file you already read**: the
> top-level form keeps its `{kind: "paren_op_assign", body}` node at the base of the expression and every chain step's
> triple carries the node it did; the corpus AST identity sweep on the final bytes reads `AST-IDENTITY-SWEEP: profile=sv_2017 files=16336 both_accepted=9764 identical=9764 ast_moved=0 verdict_moved=3 both_nonaccepting=6569`; `AST-IDENTITY-SWEEP: profile=sv_2023 files=16336 both_accepted=9789 identical=9789 ast_moved=0 verdict_moved=3 both_nonaccepting=6544`; `AST-IDENTITY-SWEEP: profile=verilog_2005 files=2606 both_accepted=2310 identical=2310 ast_moved=0 verdict_moved=0 both_nonaccepting=296` — every tree both probes accept is byte-identical on every lane, the thirteen top-level-form files and `t_nba_assign_on_rhs.v` among them — `ast_moved=0` on
> every lane; the verdict movers are exactly the pre-registered rows (ispras `11.04.02_01.sv`, sv-tests `11.3.6--two_assign_in_expr.sv` and `-sim`, `fail → pass` on `sv_2017` and `sv_2023`, `verdict_moved=0` on `verilog_2005`). A parenthesised assignment you
> never received as an OPERAND arrives as `{kind: "paren_op_assign", body: {…operator_assignment…}}` in the `first` slot
> of an `operand_chain` or as the third element of a chain step's triple — the kind documented for the base since
> schema 26, in a slot it never occupied before; no template changed, schema `32` stands. **Consumers on
> `verilog_2005`**: `( lvalue = expr )` inside an expression now REJECTS at the `=` (the lane's 2 606 rows carried no
> such text; the sweep confirmed it). Lanes `LANE-TOTALS: lane=sv_2017 rows=16336 pass=9767 fail=6569 timeout=0` (was 9764 / 6572 at 1.0.223 — the three pre-registered rows `fail → pass`), `LANE-TOTALS: lane=sv_2023 rows=16336 pass=9792 fail=6544 timeout=0` (was 9789 / 6547 — the same three), `LANE-TOTALS: lane=verilog_2005 rows=2606 pass=2310 fail=296 timeout=0` (byte-identical — the lane's 2 606 rows carry neither form); pins `ADJUDICATION-REPROS: checked=1022 armed=219 listed=520 multi_profile_rows=378 failures=0` (was 1017 / 219 / 517 / 376; the three new rows green on the final bytes); union `sv_cert_recognized_union_gate` — canonical `1497/10/1472/15 → 1499/10/1474/15` (+2 total, +2 witness: `paren_operator_assignment_operand` and `expression_chain_operand`, witnessed by the canonical config's own expression samples; proof 10 and the canonical UNKNOWN 15-rule set unchanged, identical), union witness `1487 → 1489`, union UNKNOWN 0, residual `[]`, deterministic across seeds 0 / 7 / 42; the gate's first run refused (`seed=0 canonical total=1499 (expected 1497)` + STALE), the expectations were stamped from the per-seed `summary.json` with a `.3.53` baseline note and confirmed by a second GREEN run that re-stamped the identity; `verilog_2005_conformance_gate` — cert `1187/371/802/14 → 1188/371/803/14` (`CERTIFICATE-COVERAGE: grammar='systemverilog' entry='systemverilog_file' samples=40 total=1188 proof=371 witness=803 UNKNOWN=14 fully_certified=false (sample_parse_failures=0, proof_reverify_failures=0)`: +1 total and +1 witness on `expression_chain_operand`, the unprofiled dispatcher counted in this profile's universe and witnessed through its first alternative exactly as `expression_operand` is; `paren_operator_assignment_operand` sits outside the profile — the gating; proof and UNKNOWN unchanged, deterministic across seeds 0 / 7 / 42), the gate's first run refused (`seed=0 cert total=1188 (expected 1187)` + STALE), stamped from the three-seed census with a `.3.53` note and confirmed by a second GREEN run that re-stamped the identity; the conformance corpus gains `reject/v2005_paren_operator_assignment.sv` (`initial j = (i = i - 1);` — REJECT `verilog_2005` @42, ACCEPT `sv_2017` / `sv_2023`, the `.3.50` precedent), `corpus matrix: 369 file×profile checks, 0 mismatches` (was 366); `sv_syntax_closure_gate` GREEN — `reachable_rules: 1643 → 1645 unreachable_rules: 1 unreachable_branches: 2`, the allow-list unchanged at ONE, identity re-stamped; `ast_shape_contract_gate` rc 0; `[ast_shape_contract][systemverilog] samples=57 aligned=57 drift=0 regression_lock_failures=0 aligned_assertion_failures=0 samples_per_status={"aligned": 57}` (no new sample: no template changed — the `paren_op_assign` node is the 1.0.223 node, now also emitted inside an `operand_chain`'s `rest` and as its `first`); `check_generated_reproducibility.sh --rebaseline` — TIER 2, every family re-derives byte-identically from HEAD (`systemverilog` 0 sites, `systemverilog_preprocessor` / `vhdl` / `rtl_const_expr` / `rtl_frontend` / `scratch` / `ebnf` / `json` / `regex` / `return_annotation` / `semantic_annotation` 0 sites); `generated_reproducibility_v0.json` re-stamped: `systemverilog` `parser_sha` `597f7655… → fc9962e0…`, `input_sha` `b2d15d53… → de4c7c04…`, `verified_at_commit` `2fa6fe36 → 85e2bdbd`;
> `sv_parse_cost_family_share` derived `2.650 %` (`23,500,736 of 886,751,315` entries over 16 336 files) against the carried `2.655` — ADOPTED (`CORPUS_FAMILY_SHARE_PCT` and the four prose anchors in `TOOLBOX.md`, `DOCTRINE_ENFORCEMENT.md`, `docs/book/src/the-enforced-doctrines.md`, `docs/knowledge/a-deterministic-counter-cannot-see-a-per-entry-cost-rise.md`; `--verify-family-share` fresh); the first rebaseline REFUSED three binding rises — `entries 404,060,933 → 404,943,513 (+0.22 %)`, `committed 6,835,468 → 6,865,461 (+0.44 %)`, `memo_hits 175,122,427 → 175,533,047 (+0.23 %)` — attributed per rule (`rule_costs.tsv` HEAD vs fresh): EXACTLY four rules moved — `paren_operator_assignment_operand` NEW `+915,081` entries (496,001 memo replays, 0 commits: the pinned sample carries no such text), `expression_chain_operand` NEW `+52,880` entries / `+29,993` commits (one pass-through frame per chain-step operand the sample commits, bounded by `expression_operand`'s 250,130), `lparen` `−84,636` and `operator_assignment` `−745` (the two retired inline arms: the memoized helper tries `lparen` once per position where `expression_base` and `hierarchical_index_operand` each tried it) — 915,081 + 52,880 − 84,636 − 745 = +882,580 and 496,001 − 84,636 − 745 = +410,620 exactly, no pre-existing rule rose; three rows recorded in `accepted_rises.tsv` under `introduced_wrapper_frames` (introduced `paren_operator_assignment_operand expression_chain_operand`) with the WHY — `expression_base` is a longest-match tournament with NO first-byte prune guard (the guard is emitted only for `@whitespace_sensitive` grammars, `ast_based_generator.rs` `emit_first_set_guard = top_level && layout_sensitivity().terminals`; this grammar skips layout before every terminal), so the `(`-decided helper is TRIED at every expression start by both alternatives that begin with it (two node shapes, the `.3.45` rule forbids the merging reorder), at every index operand and at every chain-step operand `expression_operand` refused; a `&lparen` lookahead would trade one helper entry for one `lparen` entry — and THE LEAF THAT REMOVES IT, `ENGINE-UNIVERSAL-SERVICES.102` (a layout-aware FIRST-set prune guard; `.96` inline typing retires the dispatcher's frames); the second rebaseline read the three rises `ACCEPTED as attributed … under introduced_wrapper_frames (SV-CORPUS-GRAD.3.53)` with the invariant RE-DERIVED on the run (`introduced paren_operator_assignment_operand expression_chain_operand commit 29,993 frames, each bounded by the rules it wraps; 0 commits retired elsewhere (Δcommitted +29,993); no pre-existing rule commits more; pre-existing entry rises +0 covered by 85,381 retired entries`) and PROMOTED (`cost.md` re-derived on the `.3.53` identity, share `2.650`); `sv_parse_cost_ratchet` GREEN on the promoted baseline; the wall clock advisory `46092 ms → 47898 ms (+3.9 %)` within the ±50 % band, machine-dependent.
>
> **Current-state note (2026-09-08, `SV-CORPUS-GRAD.3.49` — GRAMMAR, release `1.0.223`, ledger `SV-0096`, schema `32`
> UNCHANGED, accept set WIDENS on `sv_2017` / `sv_2023` by the forward / cross-file CHECKER INSTANTIATION alone; `verilog_2005`
> untouched): A STORE GATE THAT DECIDES NO PARSE IS A RESOLUTION CHECK ANNEX A NEVER ASKED FOR.** Through `1.0.222` the
> unscoped checker name carried `@predicate: { name: has_fact, args: [checker_name, $body], phase: post }` — the 2026-03-21
> semantic-fact PILOT (`3c63cf9b`, landed to *"prove the semantic-runtime fact model is not trapped in type_name-only
> usage"*) — so `my_checker check(p2(a, b, c), @($global_clock) a[*1:$] ##1 b);` REJECTED at the second actual's `@`
> whenever `checker my_checker … endchecker` followed the module (Surelog `tests/CheckerInst/dut.sv`, ispras
> `14.14_05.sv`) or lived in another file, and `test dut(foo);` inside a `program` (iverilog `program5b.v`) rejected at
> `test`. IEEE 1800-2017 §3.2 (`docs/systemverilog/2017/md/section-2-normative-references.md:99-105`) makes a checker a
> design element; §23.3.2 (`docs/systemverilog/2017/md/section-23-modules-and-hierarchy.md:869-870`) lets an instantiated
> name be *"previously declared or one declared later"*; A.4.1.4 / A.9.3 (`docs/systemverilog/2017/md/section-41-data-read-api.md:1835`)
> derive `checker_instantiation` from a bare `identifier`. The gate disambiguated nothing: for the plain `ident inst(args);`
> form the tournament tie goes to `module_instantiation` by order whether or not a checker of that name was declared
> (`rust/target/s349/20_declared_plain.sv` dumps `{"kind":"module_instantiation"}`), so the only parses the checker
> alternative can win are those whose actuals are sequence / property syntax no module port admits. `1.0.223` deletes the
> annotation (the `checker_name` fact is still emitted at the declaration, the scoped form untouched) and, where an
> instantiation-shaped alternative competes (`module_common_item`, `bind_instantiation`), reaches a BESIDE-MODULE twin
> `checker_instantiation_beside_module := !interface_instantiation ps_checker_identifier name_of_instance ( … ) ;` — the
> ceremony's first pass had measured the ungated alternative re-reading every plain-actual instantiation as property actuals
> (`committed +33.56 %` on the pinned sample, refused by the parse-cost ratchet); `interface_instantiation` derives exactly
> the language `module_instantiation` does and is tried at every such position, so the lookahead is a memo lookup that
> commits nothing (`1641 → 1644` rules; the twins carry the same kinds, so every tree is byte-identical).
> ⛔ **Consumers — nothing moves for any file you already read**: the corpus AST identity sweep on the final bytes reads
> `AST-IDENTITY-SWEEP: profile=sv_2017 files=16336 both_accepted=9761 identical=9761 ast_moved=0 verdict_moved=3 both_nonaccepting=6572`; `AST-IDENTITY-SWEEP: profile=sv_2023 files=16336 both_accepted=9786 identical=9786 ast_moved=0 verdict_moved=3 both_nonaccepting=6547`; `AST-IDENTITY-SWEEP: profile=verilog_2005 files=2606 both_accepted=2310 identical=2310 ast_moved=0 verdict_moved=0 both_nonaccepting=296` — every tree the parser accepted before is byte-identical on every lane — `ast_moved=0` on every lane; the verdict movers are exactly the pre-registered rows (Surelog `tests/CheckerInst/dut.sv`, ispras `ieee-1800-2012/14/14.14_05.sv`, iverilog `ivltests/program5b.v` — `fail → pass` on `sv_2017` and `sv_2023`, 3 / 3 / 0 movers, 0 `pass → fail` anywhere). A
> checker instantiation you never received before arrives as the `concurrent_assertion_item` / `procedural_assertion_statement`
> / `bind_instantiation` branch `{kind: "checker_instantiation", body: {name, instance, connections}}` documented since
> schema 26 — no new kind, no template changed, schema `32` stands. Lanes `sv_2017` pass 9761 → 9764 / fail 6575 → 6572 / timeout 0 of 16336 rows (the sweep's `verdict_moved=3`, every one `fail → pass`, every one pre-registered); `sv_2023` pass 9786 → 9789 / fail 6550 → 6547 / timeout 0 (`verdict_moved=3`, the same three); `verilog_2005` pass 2310 / fail 296 / timeout 0 of 2606 rows, `verdict_moved=0` ⇒ `transitions: NONE` — **0 `pass → fail` on any lane**; pins `ADJUDICATION-REPROS: checked=881 armed=219 listed=457 multi_profile_rows=317 failures=0` (was 874 / 219 / 453 / 314; the four new rows green on the final bytes); union `sv_cert_recognized_union_gate` — canonical `1494/10/1469/15 → 1497/10/1472/15` (+3 total, +3 witness: the three beside-module twins, witnessed by the canonical config's own module items; proof 10 and canonical UNKNOWN 15 unchanged, the identical set), union witness `1484 → 1487`, union UNKNOWN 0, residual `[]`, deterministic across seeds 0 / 7 / 42; the gate's first run refused (`seed=0 canonical total=1497 (expected 1494)` + STALE), the expectations were stamped from the per-seed `summary.json` and confirmed by a second GREEN run that re-stamped the identity; `verilog_2005_conformance_gate` — cert `1186/370/802/14 → 1187/371/802/14` (`CERTIFICATE-COVERAGE: grammar='systemverilog' entry='systemverilog_file' samples=40 total=1187 proof=371 witness=802 UNKNOWN=14 fully_certified=false (sample_parse_failures=0, proof_reverify_failures=0)`: +1 total and +1 proof on `checker_instantiation_beside_module`, the unprofiled twin, provably unreachable under `verilog_2005` exactly as its original is; witness and UNKNOWN unchanged, deterministic across seeds), stamped from the three-seed census and confirmed by a second GREEN run that re-stamped the identity; `corpus matrix: 366 file×profile checks, 0 mismatches` (unchanged from `.3.50`); `sv_syntax_closure_gate` GREEN — `reachable_rules: 1640 → 1643 unreachable_rules: 1 unreachable_branches: 2`, the allow-list unchanged at ONE, identity re-stamped;
> `ast_shape_contract_gate` rc 0; `[ast_shape_contract][systemverilog] samples=57 aligned=57 drift=0 regression_lock_failures=0 aligned_assertion_failures=0 samples_per_status={"aligned": 57}` (no new sample: no template changed); `check_generated_reproducibility.sh --rebaseline` — every family re-derives byte-identically (`systemverilog` `0 sites`), the SV parser's new digest recorded; `sv_parse_cost_family_share` — `derived family share 2.655 % (23,500,265 of 885,215,887 entries over 16336 files, 0 no-dump)`; the carried constant `2.658 → 2.655 %` ADOPTED (`stimuli/sv/corpus_parse_cost.py` + its four prose anchors: `TOOLBOX.md`, `DOCTRINE_ENFORCEMENT.md`, `docs/book/src/the-enforced-doctrines.md`, the deterministic-counter knowledge card; the ungated first spelling had read `2.639 %`); `sv_parse_cost_rebaseline` — binding `entries 403,768,688 → 404,060,933 (+0.07 %)`, `committed 6,763,138 → 6,835,468 (+1.07 %)`, `memo_hits 174,980,252 → 175,122,427 (+0.08 %)` ACCEPTED as attributed under the NEW `lookahead_replay_occurrences` (`ENGINE-UNIVERSAL-SERVICES.100`; three rows in `accepted_rises.tsv`, scope `assertion_item_beside_module concurrent_assertion_item_beside_module checker_instantiation_beside_module !interface_instantiation ~checker_instantiation`), the invariant RE-DERIVED on the run: `lookahead replay: `interface_instantiation` entries +72,025 = memo_hits +72,025, committed +0; every sampled file keeps its verdict and furthest; 80 rule(s) committed more (+72,330), all inside the 847-rule sub-graph the guard replays or the un-gated branch reaches or one of 3 ancestor frame(s) of the un-gated rule within its rise (+32); 122 rule(s) rose in entries (+605,993), all reachable from the introduced / un-gated rules or 9 loop-continuation tries within the un-gated rise; 11 fell (-313,748)` — an occurrence-count artefact (C3-B counts a memo replay and a losing success with multiplicity), not re-done work: `interface_instantiation` is a pure lookup, the twins commit nothing, the whole `committed` rise sits on four accepted sample files with the widest port lists whose typed ASTs are byte-identical (`t_math_wallace_mul.v` +63,594); ⛔ the ungated first spelling read `committed +33.56 %` and was REFUSED; the arm was refused twice on its own data before its ancestor-frame and loop-continuation clauses were bounded; `sv_parse_cost_ratchet` `parse-cost-ratchet: OK (identity fresh for: generated parser, grammar raw ast, instrument, sample inputs; 192 pinned sample files)`.
>
> **Current-state note (2026-09-08, `SV-CORPUS-GRAD.3.50` — GRAMMAR, release `1.0.222`, ledger `SV-0095`, schema `32`
> UNCHANGED, accept set UNCHANGED on `sv_2017` / `sv_2023` and NARROWED on `verilog_2005` by the IEEE 1800 unpacked SIZE form
> alone): A PROFILE-BLIND ARM IS AN OVER-ACCEPTANCE ON EVERY PROFILE WHOSE STANDARD LACKS IT.** Through `1.0.221`
> `unpacked_dimension := lbrack constant_range rbrack | lbrack constant_expression rbrack` carried no `@profiles` on its
> second alternative, so `reg x [2];` PARSED under `verilog_2005` — IEEE 1364-2005 A.2.5 `dimension ::=
> [ dimension_constant_expression : dimension_constant_expression ]`
> (`docs/verilog/2005/md/section-Annex_A-normative-formal-syntax-definition.md:309`) and `range ::=
> [ msb_constant_expression : lsb_constant_expression ]` (`:310`) admit a RANGE alone; the size form is IEEE 1800-2017 A.2.5
> `unpacked_dimension ::= [ constant_range ] | [ constant_expression ]` (`docs/systemverilog/2017/md/section-41-data-read-api.md:1046-1050`;
> 1800-2023 `docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:1079-1083`; §7.4.2). Found by
> `SV-CORPUS-GRAD.3.48`'s own two-sided control (the 1364-2005 twin of `control_unpacked_dimension_size.sv` was written
> expecting REJECT and ACCEPTED), filed `accepts_invalid` and fixed here. `1.0.222` lifts the size arm into
> `unpacked_dimension_size_sv_only := lbrack constant_expression rbrack -> {kind: "expression", body: $2}` under
> `@profiles: ["sv_2017", "sv_2023"]` (the `_sv_only` helper idiom, the `-0289` named-rule ruling; `1640 → 1641` rules) and
> `unpacked_dimension`'s second alternative passes the helper's node through (`-> $1`). ⛔ **Consumers on `sv_2017` /
> `sv_2023` — nothing moves**: the size form keeps its `{kind: "expression", body}` node and every accepted tree is
> byte-identical; the corpus AST identity sweep on the final bytes reads `AST-IDENTITY-SWEEP: profile=sv_2017 files=16336 both_accepted=9761 identical=9761 ast_moved=0 verdict_moved=0 both_nonaccepting=6575`; `AST-IDENTITY-SWEEP: profile=sv_2023 files=16336 both_accepted=9786 identical=9786 ast_moved=0 verdict_moved=0 both_nonaccepting=6550`; `AST-IDENTITY-SWEEP: profile=verilog_2005 files=2606 both_accepted=2310 identical=2310 ast_moved=0 verdict_moved=0 both_nonaccepting=296` — every tree the parser accepted before is byte-identical on every lane — `ast_moved=0` and `verdict_moved=0`
> on both SV lanes. **Consumers on `verilog_2005`**: `x [N]` now REJECTS at the `[` (the pin
> `accepts_invalid_v2005_unpacked_dimension_size.sv`, flipped to `REJECT` / `invalid`); the lane's 2 606 rows carried no
> such declaration (the census by token read 0, the sweep confirmed it). Lanes `sv_2017` pass 9761 / fail 6575 / timeout 0 of 16336 rows (unchanged from `1.0.221`); `sv_2023` pass 9786 / fail 6550 / timeout 0 (unchanged); `verilog_2005` pass 2310 / fail 296 / timeout 0 of 2606 rows (unchanged, `transitions: NONE`) — **0 `pass → fail` and 0 `fail → pass` on any lane**; pins `ADJUDICATION-REPROS: checked=874 armed=219 listed=453 multi_profile_rows=314 failures=0` (the counts unchanged; the flipped row green as `REJECT` / `invalid`, its stop `furthest_position=800` — the `[`'s byte after the pin's two comment lines); union `sv_cert_recognized_union_gate` — canonical `1493/10/1468/15 → 1494/10/1469/15` (+1 total, +1 witness: `unpacked_dimension_size_sv_only`, witnessed by the canonical config's own declaration samples; proof 10 and canonical UNKNOWN 15 unchanged, the identical set), union witness `1483 → 1484`, union UNKNOWN 0, residual `[]`, deterministic across seeds 0 / 7 / 42; the gate's first run refused (`seed=0 canonical total=1494 (expected 1493)` + STALE) and the expectations were stamped from the per-seed `summary.json` and confirmed by a second GREEN run that re-stamped the identity; `verilog_2005_conformance_gate` — cert `1186/370/802/14` UNCHANGED (`CERTIFICATE-COVERAGE: grammar='systemverilog' entry='systemverilog_file' samples=40 total=1186 proof=370 witness=802 UNKNOWN=14 fully_certified=false (sample_parse_failures=0, proof_reverify_failures=0)`; the helper is gated out of the profile), ⛔ its corpus matrix refused ONE case on the first run: `accept/event_trigger_array.v` (`event e[4]; initial -> e[0];`, expected ACCEPT under `verilog_2005`) now REJECTS at the `[` — the curated accept case had ENCODED the over-acceptance (IEEE 1364-2005 A.2.3 `list_of_event_identifiers ::= event_identifier { dimension }`, `docs/verilog/2005/md/section-Annex_A-normative-formal-syntax-definition.md:270`, with `dimension ::= [ a : b ]`, `:309`); respelled `event e[0:3];` so the case keeps locking the `-> e[0];` trigger it exists for (its expectation triple unchanged and measured on the final probe: `verilog_2005` accept, `sv_2017` reject at the trigger's select, `sv_2023` accept), and the size form locked by a NEW reject case `reject/v2005_unpacked_size_dimension.sv` (`event e[4];`: `verilog_2005` reject at the `[`, `sv_2017` / `sv_2023` accept); the gate's second run GREEN and re-stamped: `corpus matrix: 366 file×profile checks, 0 mismatches` (the first run: 363 checks, 1 mismatch);
> `sv_syntax_closure_gate` GREEN — `reachable_rules: 1639 → 1640 unreachable_rules: 1 unreachable_branches: 2`, the allow-list unchanged at ONE, identity re-stamped; `ast_shape_contract_gate` rc 0; `[ast_shape_contract][systemverilog] samples=57 aligned=57 drift=0 regression_lock_failures=0 aligned_assertion_failures=0 samples_per_status={"aligned": 57}` (no new sample: no template changed); `check_generated_reproducibility.sh --rebaseline` — every family re-derives byte-identically (`systemverilog` `0 sites`), the SV parser's new digest recorded; `sv_parse_cost_family_share` — `derived family share 2.658 % (23,499,775 of 884,195,201 entries over 16336 files, 0 no-dump)`, the carried constant `2.658 %` reproduces exactly (no adoption); `sv_parse_cost_rebaseline` — binding `entries 403,707,725 → 403,768,688 (+0.02 %)` and `committed 6,763,013 → 6,763,138 (+0.00 %)` ACCEPTED as attributed under `introduced_wrapper_frames` (two rows in `accepted_rises.tsv`, introduced `unpacked_dimension_size_sv_only`: one frame per `unpacked_dimension` attempt reaching the second alternative, its commits the size-form commits, no pre-existing rule committing differently in either direction — the chain's first rebaseline run had refused both rises before the rows existed), `memo_hits 174,980,300 → 174,980,252` (an improvement, promoted); `sv_parse_cost_ratchet` `OK (identity fresh for: generated parser, grammar raw ast, instrument, sample inputs; 192 pinned sample files)`; advisory wall clock `47373 → 47897 ms (+1.1 %)`, within the band. Schema `32` stands: no template changed and no kind was added.
>
> **Current-state note (2026-09-08, `SV-CORPUS-GRAD.3.47` — GRAMMAR + ANSWER-KEY, release `1.0.221`, ledger `SV-0094`, schema `32`
> UNCHANGED, accept set WIDENS on `sv_2017` / `sv_2023` by the §5.9 continuation (and on `sv_2023` by the 1800-2023 triple-quoted
> string) and NARROWS on every profile by the raw newline inside a string; `verilog_2005` otherwise untouched): THE STRING LITERAL
> IS SPLIT BY EDITION, AND MEASURED IN BOTH DIRECTIONS.** One terminal, `/"([^"\\]|\\.)*"/`, had served every profile since
> `1.0.0` and was wrong for each standard in a different direction. IEEE 1800-2017 §5.9
> (`docs/systemverilog/2017/md/section-32-h-12ab-f001.md:101-102`; 1800-2023 §5.9
> `docs/systemverilog/2023/md/section-5-lexical-conventions.md:463-464`) — *"a string literal shall be contained in a single line
> unless the newline character is immediately preceded by a \ (backslash); in this case, the backslash and the newline character
> are ignored"* — REJECTED at the backslash on every profile, because the regex crate's `.` in `\\.` does not match a newline
> (five bar rows: Surelog `AssumeProp/dut.sv:6`, ispras `05.09_01.sv:9`, sv-tests `5.9-string-broken-line.sv:18`, verible
> `forbid_line_continuations.sv:3`, verilator `t_display.v:175`); a RAW newline inside a string — which both 1800 editions
> (1800-2023's Syntax 5-3: `quoted_string_item ::= any_ASCII_character except \ or newline or "`, `:447-452`) and IEEE 1364-2005
> §3.6 / A.8.8 (`docs/verilog/2005/md/section-3-lexical-conventions.md:299`,
> `docs/verilog/2005/md/section-Annex_A-normative-formal-syntax-definition.md:1218`) forbid — parsed on every profile (`[^"\\]`
> admits it; 0 accepting corpus rows carried one). `1.0.221` spells `string_literal := string_literal_sv_2017 |
> string_literal_sv_2023 | string_literal_v2005` under `@profiles` (`1637 → 1640` rules): the SV twins admit `\` + any character
> (the newline, a CRLF pair, or an escape) and refuse a bare newline; the `sv_2023` twin adds IEEE 1800-2023 §5.9's TRIPLE-QUOTED
> string `"""…"""` (`:436-452`: any character except `\` inside — a bare `"` and a bare newline included; the regex spells "no
> `"""` inside" without look-around) as `{kind: "triple_quoted", body}`; the 1364-2005 twin refuses both the continuation and
> the raw newline. ⛔ **Consumers — nothing moves for any string you already read**: every double-quoted twin carries the
> shipped `{kind: "double_quoted", body}` template and `body` is the SOURCE TEXT verbatim, quotes excluded, continuation
> included (§5.9's *ignored* is your reading of the text, not a rewrite of it); the corpus AST identity sweep on the final
> bytes reads `AST-IDENTITY-SWEEP: profile=sv_2017 files=16336 both_accepted=9756 identical=9756 ast_moved=0 verdict_moved=5 both_nonaccepting=6575`; `AST-IDENTITY-SWEEP: profile=sv_2023 files=16336 both_accepted=9780 identical=9780 ast_moved=0 verdict_moved=6 both_nonaccepting=6550`; `AST-IDENTITY-SWEEP: profile=verilog_2005 files=2606 both_accepted=2310 identical=2310 ast_moved=0 verdict_moved=0 both_nonaccepting=296` — every tree the parser accepted before is byte-identical on every lane. The `triple_quoted` kind appears only on `sv_2023` and only on text no earlier release accepted, so
> schema 32 stands. **Answer key**: iverilog `ivltests/string12.v` — a continuation — is `must_reject` on the `verilog_2005`
> lane (§3.6 / A.8.8; the parser was right, the ivtest key records the iverilog dialect); verilator `t_display_qqq.v` — a
> triple-quoted string — is `must_reject` on the 2017 lane as an EDITION CONTRAST and `must_accept` on the 2023 lane, where it
> now parses. Lanes `sv_2017` pass 9756 → 9761 / fail 6580 → 6575 / timeout 0 of 16336 rows (the sweep's `verdict_moved=5`, every one `fail → pass`, every one pre-registered); `sv_2023` pass 9780 → 9786 / fail 6556 → 6550 / timeout 0 (`verdict_moved=6`); `verilog_2005` pass 2310 / fail 296 / timeout 0 of 2606 rows, `verdict_moved=0` ⇒ `transitions: NONE` — **0 `pass → fail` on any lane** (the raw-newline refusal moved no accepted row); pins `ADJUDICATION-REPROS: checked=865 armed=219 listed=449 multi_profile_rows=311 failures=0` (was 853 / 213 / 443 / 307; ⛔ the chain's first run read `failures=1` — `invalid_string_line_continuation_v2005.v: listed in the manifest, absent on disk` — because `run_adjudication_repros.py` enumerates `*.sv` alone, so the Verilog-2005 pin was renamed `.sv` and the ratchet re-run before any number was quoted); certificates union `sv_cert_recognized_union_gate` — canonical `1492/10/1467/15 → 1493/10/1468/15` (+1 total, +1 witness: `string_literal_sv_2017`, the sv_2017 twin; the sv_2023 / verilog_2005 twins sit outside the sv_2017 total), union witness `1482 → 1483`, union UNKNOWN `0`, residual `[]`, deterministic across seeds `[0,7,42]` — ALL-POSITIVE, adjudicated: the first run read every expectation +1 on the two count fields and unchanged everywhere else and refused to self-stamp (the `.3.43` shape), the contract stamped from those lines and the gate re-run GREEN; `verilog_2005_conformance_gate` — cert `1185/370/801/14 → 1186/370/802/14` (+1 total, +1 witness: `string_literal_v2005`), `sample_parse_failures=0, proof_reverify_failures=0`, `lint orphans=0, corpus 363 checks/0 mismatches, aliases 2`, deterministic, stamped and re-run GREEN.

> **Current-state note (2026-09-07, `SV-CORPUS-GRAD.3.46` — GRAMMAR + SHAPE-CONTRACT, release `1.0.220`, ledger `SV-0093`,
> schema `31 → 32` (a REPLACEMENT), accept set unchanged on every profile EXCEPT one LRM-illegal nettype form): `block_data_type`
> ALTERNATIVE 1 CARRIES `data_type`'s OWN `kind`, AND THE NETTYPE ALIAS-HEAD GUARD REFUSES A `with` CONTINUATION.** `1.0.219`
> typed the partial-typing residue and measured two items on the way out. (1) The block-scoped integer VECTOR declaration —
> `reg [7:0] v;` / `logic [3:0] t;` inside a `begin … end`, a function or a task body; the `body` of `{kind: "block_data", body}` —
> had been typed by SV-Slice-96 with the field set `{base, signing, dims}` and NO `kind`, the one integer-vector shape in the
> grammar without one after its four multi-element siblings took `data_type`'s kinds in `1.0.219`. It is now
> `{kind: "integer_vector", base, signing, dims}`, `data_type`'s own template. ⛔ **Consumers — a REPLACEMENT, schema 32:** a
> reader that recognised the vector form by the ABSENCE of `kind` on the `block_data` body reads `kind == "integer_vector"`; a
> `data_type` reader reads the body unchanged; every other alternative is byte-identical. Pre-registered by the `.3.45` census
> on the `1.0.219` probe (`RAW-ENVELOPE-CENSUS:`, every lane row) — **712 nodes in 259 accepting files on `sv_2017` and
> `sv_2023`, 130 in 73 on the 1364-2005 lane** — and measured two-sided by the corpus AST identity sweep on the final bytes:
> `AST-IDENTITY-SWEEP: profile=sv_2017 files=16336 both_accepted=9756 identical=9497 ast_moved=259 verdict_moved=0 both_nonaccepting=6580`; `AST-IDENTITY-SWEEP: profile=sv_2023 files=16336 both_accepted=9780 identical=9521 ast_moved=259 verdict_moved=0 both_nonaccepting=6556`; `AST-IDENTITY-SWEEP: profile=verilog_2005 files=2606 both_accepted=2310 identical=2237 ast_moved=73 verdict_moved=0 both_nonaccepting=296`. One AST-shape contract sample (`block_data_type_integer_vector`) locks the `kind`; the vector pin's arm is
> `block_data>integer_vector`. (2) `nettype <declared nettype> <name> with <fn>;` — IEEE 1800-2017 §6.6.7's ALIAS form carrying
> a resolution function, a form A.2.1.3 gives NO production (the alias production `nettype [ scope ] net_type_identifier
> net_type_identifier ;` has no `with`; a nettype identifier is not a `data_type`,
> `docs/systemverilog/2017/md/section-41-data-read-api.md:828-833`, `docs/systemverilog/2017/md/section-6-data-types.md:617-619`
> and `:692`) — parsed down the data-type alternative of BOTH edition twins through `1.0.219`, with the nettype read as a
> `known_unscoped_data_type`, because the `.3.45` alias-head guard required the `;` after `[ scope ] <declared nettype>
> <identifier>` and `with` is not one. The guard now refuses the head whatever follows; the text falls to the alias alternative,
> meets `with` where it needs `;` and REJECTS. **0 corpus rows on any lane** (14 `with` declarations in the corpus, every base a
> typedef — measured by the census's alias detector and a token grep), so no lane row moves; every legal form is unchanged and
> byte-identical (the data-type form with `with` on a typedef base, scoped or not; the alias form; the wildcard-import alias).
> Pins `invalid_nettype_alias_with_resolution_function.sv` / `invalid_nettype_alias_with_scoped_base_resolution_function.sv`
> (REJECT, both SV profiles) with `control_nettype_data_type_scoped_with.sv` / `_sv2023.sv` (ACCEPT, the one-token twins). GRAMMAR
> tier, ZERO Rust / codegen / engine bytes, `1637` rules unchanged; certificates union `sv_cert_recognized_union_gate` GREEN — canonical `1492/10/1467/15` UNCHANGED (`expected_total 1492, expected_proof 10, expected_canonical_unknown 15`), union witness `1482` / UNKNOWN `0`, residual `[]`, deterministic across seeds `[0,7,42]`, the identity re-stamped on the adjudicated contract (`the baseline was STALE and every expectation still held, so this run re-stamped it`); `verilog_2005_conformance_gate` GREEN — cert `1185/370/801/14` UNCHANGED (`sample_parse_failures=0, proof_reverify_failures=0`), `lint orphans=0, corpus 363 checks/0 mismatches, aliases 2`, deterministic across seeds `[0,7,42]`, re-stamped; the three lanes `sv_2017` pass 9756 / fail 6580 / timeout 0 of 16336 rows, `verdict_moved=0` in the sweep ⇒ `transitions: NONE`; `sv_2023` pass 9780 / fail 6556 / timeout 0 of 16336 rows, `verdict_moved=0` ⇒ `transitions: NONE`; `verilog_2005` pass 2310 / fail 296 / timeout 0 of 2606 rows, `verdict_moved=0` ⇒ `transitions: NONE`; pins
> `ADJUDICATION-REPROS: checked=853 armed=213 listed=443 multi_profile_rows=307 failures=0` (was 847 / 211 / 439 / 305); the final-probe census `RAW=0 unclassified=0` on all three lanes with alternative 1's rows conserved as typed
> (`block_data_type:1:typed` 712 rows in 259 files on `sv_2017` and on `sv_2023`, 130 in 73 on `verilog_2005` — the HEAD census's rows and files exactly, every one now read through the new `kind`).

> **Current-state note (2026-09-07, `SV-CORPUS-GRAD.3.45` — GRAMMAR + INSTRUMENT + SHAPE-CONTRACT, release `1.0.219`, ledger `SV-0092`,
> schema `30 → 31`, accept set unchanged on every profile EXCEPT one LRM-illegal nettype form — a return template moves what the parser
> EMITS, never what it accepts, the one lookahead moves only which of two SUCCEEDING alternatives wins, and the one store gate added
> refuses an undeclared alias base): the PARTIAL-TYPING RESIDUE IS TYPED.** `1.0.218` closed with the
> class measured: `PARTIAL-TYPING-CENSUS:` read 38 rules typed on some alternatives and not others, 19 of the untyped alternatives raw
> `Sequence` envelopes, in eight rules; the corpus census on the `1.0.218` probe (`RAW-ENVELOPE-CENSUS:`, TOOLBOX §1.5h, every lane row,
> `docs/tasks/artifacts/sv_corpus_grad/partial_typing_residue/census.md`) read **2 319 raw envelopes in 1 088 accepting files on
> `sv_2017`** (1 779 / 819 on `sv_2023`, 217 / 120 on the 1364-2005 lane) — 873 the block-scoped integer atom (`integer i;`,
> `int unsigned x;`), 542 the parenthesised clocking event `@(posedge clk)`, 524 the structure-keyed assignment pattern `'{a: 1}`, 193 the
> two-value path delay, 60 the replication `'{2{0, 1}}`, 56 the array-keyed pattern, 27 the `global clocking` block, 16 + 15 the
> `case … matches` / `case … inside` statements, 4 + 5 + 2 the three- / six- / twelve-value path delays, 1 + 1 a block-scoped struct and
> virtual interface. ⛔ **Consumers — seventeen new templates, one rule for all of them: every newly typed alternative carries a `kind`,
> and every typed sibling keeps its shipped shape BYTE-IDENTICALLY, so `kind` absent is the form you already read.**
> `assignment_pattern`: `{kind: "structure" | "array", keys, values}` (`keys[i]` pairs with `values[i]`; the key node is the typed
> `structure_pattern_key` / `array_pattern_key`) and `{kind: "replication", count, exprs}`; the positional form stays `{exprs}`.
> `block_data_type` (the `body` of `{kind: "block_data", body}`): the integer-atom, struct/union, enum and virtual-interface forms take
> `data_type`'s OWN templates — `{kind: "integer_atom", base, signing}`, `{kind: "struct_union", header, packed_signing, members: {first,
> rest}, dims}`, `{kind: "enum", base_type, names, dims}`, `{kind: "virtual_interface", interface_keyword, name, params, modport}` — so a
> `data_type` reader reads them unchanged; the vector form stays `{base, signing, dims}`. `case_statement`: `{kind: "matches" | "inside",
> unique_priority, keyword, expr, items: {first, rest}}` with `case_pattern_item` / `case_inside_item` items (the `inside` form's
> `keyword` is the literal `{kind: "case"}`, the only keyword A.6.7 admits there); the plain form unchanged. `clocking_declaration`:
> `{kind: "global", name, event, end_label}` (no `items` — the production has none). `clocking_event_sv_2017`: `{kind: "expression",
> body}` — the `sv_2023` twin's shape; `@id` stays `{body}`. `list_of_path_delay_expressions` (the `delay.body` of a path declaration):
> `{kind: "two", trise, tfall}`, `{kind: "three", trise, tfall, tz}`, `{kind: "six", t01, t10, t0z, tz1, t1z, tz0}`, `{kind: "twelve",
> t01 … tzx}` — the LRM's own position names; one value stays `{body}`. `property_case_item`: `{kind: "default", body}`.
> `net_type_declaration_sv_2017`: `{kind: "nettype", scope, base, name}` for IEEE 1800-2017 §6.6.7's ALIAS form `nettype wTsum
> nettypeid2;`. ⭐ **That last one was not raw but SHADOWED, on both SV profiles**: both alternatives succeed on the LRM's own example (a
> declared nettype is a `type_name` fact, so the data-type form reads `wTsum` as a `known_unscoped_data_type`) and the longest-match tie
> went to declaration order (Protocol D: `selected branch 1/2` for `net_type_declaration_sv_2017` AND for the fully typed
> `nettype_declaration_sv_2023`), so `1.0.0`–`1.0.218` called the base nettype a data type. `1.0.219` breaks the tie in the LRM's favour by a
> NEGATIVE LOOKAHEAD on the data-type alternative — it refuses a text that reads as an alias head, `[ scope ] <declared nettype>
> <identifier> ;` — with the ORDER left as shipped, and routes the alias base `net_type_identifier` — a bare `declaration_identifier`
> through `1.0.218`, one ungated twin per edition — to `checked_nettype_identifier | wildcard_escape_nettype_identifier` (a declared
> nettype, or an undeclared name under a wildcard import: how `net_declaration` reads a net of a user-defined nettype, SV-PARSE-STRICT.2 /
> SV-0037). ⛔ Two spellings were refuted before this one, each by an instrument: reordering the UNGATED alternative read `nettype T wT;` (T
> a struct typedef, ispras `06.07.02_01.sv`) as an alias — the two-sided corpus sweep's one moved-but-not-predicted file — and reordering
> the GATED one passed every corpus instrument and broke the certificate's own nettype prelude (the generator renders an unplanned
> declaration with the first of two equally cheap alternatives, so the prelude became `nettype \foo \foo;` and the union certificate read
> `UNKNOWN=2` where `1.0.218` had 0). The winner now changes exactly on alias declarations (two per SV lane in the corpus: verilator `t_nettype.v`, ispras `06.06.07_01.sv`). ⚠️ **The one narrowing**: `nettype
> undeclared_t x;` with no wildcard import in scope — LRM-illegal (§6.6.7: the base must be an existing nettype), accepted through `1.0.218`
> down the ungated alias alternative — now REJECTS on both SV profiles; the census read 0 accepting corpus files down that alternative, and
> `invalid_nettype_alias_undeclared_base.sv` pins it (`furthest_position=32`, the base). **Migration:** a consumer that indexed a raw envelope reads the named fields — the elements are the same typed
> nodes in the same order with the punctuation and keyword tokens dropped (`pattern[2]`/`[4]` and each `[, key : expr]` iteration →
> `keys[i]` / `values[i]`; `body[0]`/`[1]` of a block integer atom → `base` / `signing`; the case envelope's `[3]`, `[6]`, `[7]` →
> `expr`, `items.first`, `items.rest`; the clocking envelope's `[2]`/`[3]`/`[6]` → `name` / `event` / `end_label`; the event envelope's
> `[2]` → `body`; the comma list of delays → the position names; the property default arm's `[2]` → `body`); a consumer of the
> `nettype` data-type form should expect the alias node where the data type used to be a nettype identifier. ⛔ **Two non-LRM
> alternatives are REMOVED** from `list_of_path_delay_expressions`, which had carried seven since the grammar's first promotion where IEEE
> 1364-2005, 1800-2017 and 1800-2023 A.7.4 all give five: a second six-value spelling that matched the same text as the LRM's and lost the
> longest-match tie to it on every input (`selected branch 4/7`), and a twelve-value spelling whose middle four positions were the regex
> terminal `/tx_path_delay_expression\b/` — the production NAME extracted as literal source text, matched by no corpus file (the extraction
> class `.13c.2m` owns through `class_qualifier`). Neither could produce a tree; both are gone with their orphaned keyword terminal, and the
> three ungated nettype identifier twins with them (`1641 → 1637` rules). Measured on the final bytes: `sv_2017` pass 9756 / fail 6580 / timeout 0 of 16336 rows, `verdict_moved=0` in the sweep ⇒ `transitions: NONE`; `sv_2023` pass 9780 / fail 6556 / timeout 0 of 16336 rows, `verdict_moved=0` in the sweep ⇒ `transitions: NONE`; `verilog_2005` pass 2310 / fail 296 / timeout 0 of 2606 rows, `verdict_moved=0` in the sweep ⇒ `transitions: NONE`; the corpus AST identity sweep (the `1.0.218` probe vs this release, every
> lane row) `AST-IDENTITY-SWEEP: profile=sv_2017 files=16336 both_accepted=9756 identical=8662 ast_moved=1094 verdict_moved=0 both_nonaccepting=6580`; `AST-IDENTITY-SWEEP: profile=sv_2023 files=16336 both_accepted=9780 identical=8959 ast_moved=821 verdict_moved=0 both_nonaccepting=6556`; `AST-IDENTITY-SWEEP: profile=verilog_2005 files=2606 both_accepted=2310 identical=2190 ast_moved=120 verdict_moved=0 both_nonaccepting=296` (the `verilog_2005` lane's 2 606 rows); the final-probe census reads `RAW=0 unclassified=0` on all three lanes and the grammar's partial-typing census
> `PARTIAL=31 … RAW_ENVELOPE=0` (the 31 are passthrough alternatives, typed one level down by design); `ADJUDICATION-REPROS: ADJUDICATION-REPROS: checked=843 armed=209 listed=437 multi_profile_rows=303 failures=0`
> (29 new `control_*.sv` pins, one per alternative, the typed siblings as the controls that cannot move, arms on the new `kind`s, two
> wildcard-import alias controls, plus the `must_reject` `invalid_nettype_alias_undeclared_base.sv`);
> `ast_shape_contract_gate` 18 / 18 with 56 SV samples (16 new, one per newly typed alternative parsed AS its rule; the nettype alias form
> takes none — its identifier is store-gated on a declared nettype — and is held by the two alias pins); certificates union `sv_cert_recognized_union_gate` GREEN — canonical `1492/10/1467/15` (total 1494 → 1492 and witness 1469 → 1467: the two deleted `sv_2017`-live rules, `kw_tx_path_delay_expression_7b2dee37` and `net_type_identifier_sv_2017`; proof 10 and canonical UNKNOWN 15 UNCHANGED — the identical set), union witness `1482` / UNKNOWN `0`, residual `[]`, `sample_parse_failures=0`, deterministic across seeds `[0,7,42]`, the identity re-stamped on the adjudicated contract (`the baseline was STALE and every expectation still held, so this run re-stamped it`); `verilog_2005_conformance_gate` GREEN — cert `1185/370/801/14` (total 1186 → 1185 and witness 802 → 801: the deleted terminal; proof 370 and UNKNOWN 14 unchanged; `sample_parse_failures=0, proof_reverify_failures=0`), `lint orphans=0, corpus 363 checks/0 mismatches, aliases 2, cert deterministic across seeds [0,7,42]`, re-stamped on the adjudicated contract (`every criterion still held`). See the
> SV parser book's [Typed Partial Alternatives](../systemverilog_parser_book/src/json-carrier.md#typed-partial-alternatives-schema-31).

> **Current-state note (2026-09-07, `SV-CORPUS-GRAD.3.44` — GRAMMAR + INSTRUMENT + SHAPE-CONTRACT, release `1.0.218`, ledger `SV-0091`,
> schema `29 → 30`, accept set UNCHANGED on every profile — a return template moves what the parser EMITS, never what it accepts):
> `function_body_declaration` and `task_body_declaration` are TYPED ON EVERY ALTERNATIVE.** IEEE 1800-2017 A.2.6 / A.2.7 give a function
> or task body two forms — the non-ported `[ interface_identifier . | class_scope ] identifier ; { tf_item_declaration } …` and the PORTED
> `… identifier ( [ tf_port_list ] ) ; { block_item_declaration } …` — each with an optional out-of-block prefix, four alternatives in the
> grammar (`docs/systemverilog/2017/md/section-41-data-read-api.md:1090-1135`; 1800-2023 Annex A `:1125-1175` reads the same). Through
> `1.0.217` only alternative 1 (`type f; …`, its ports declared inside the body) carried a return template; the ported form — the most
> common function in any corpus — and the two out-of-block forms arrived as the raw positional `Sequence` envelope (10 / 8 / 11 elements
> on a function, 9 / 7 / 10 on a task). Measured on the 1.0.217 probe by the AST-field census (`AST-FIELD-CENSUS:` with the new
> `--select-keys lifetime,body` selector + `TF-BODY-CLASSIFY:`, `docs/tasks/artifacts/sv_corpus_grad/tf_body_census/census.md`): **2 211 of
> the 3 411 function / task bodies the parser wrote on `sv_2017` were that envelope** (1 646 ported functions, 513 ported tasks, 30
> out-of-block functions, 22 out-of-block tasks), in 1 060 of the 1 958 accepting files that declare a function or task; 2 221 of 3 521 on
> `sv_2023`; 91 of 470 on the `verilog_2005` lane (the ported forms are legal IEEE 1364-2005). ⛔ **Consumers: every alternative now returns
> the typed object**, mirroring alternative 1's field set — `{return_type, name, ports, items, statements, end_label}` on a ported
> function, `{return_type, scope, name, items, statements, end_label}` on an out-of-block one, `{return_type, scope, name, ports, items,
> statements, end_label}` on an out-of-block ported one; the task twins without `return_type`. There is no `kind`: the discriminator is
> KEY PRESENCE, which is the standard's own distinction — `ports` is present exactly on the ported alternatives (the typed `tf_port_list`
> array, `[]` for `f()`), `scope` exactly on the out-of-block ones (the `out_of_block_tf_scope` node: the pair `[<interface_identifier>,
> {kind: "dot"}]` for an interface method, the `class_scope` node for a class method). `items` differs faithfully between the forms: a
> non-ported body's items are `tf_item_declaration` nodes (`{kind: "block_item" | "tf_port", body}`, where its ports live), a ported body's
> items are bare `block_item_declaration` nodes (A.2.6's second production admits no port declaration there). **Migration:** a consumer
> that indexed the raw envelope — `body[0]` return type, `body[3]` / `body[4]` port list, `body[6]` / `body[7]` / `body[8]` items — reads
> the named fields; the elements are the same typed nodes in the same order, with the `lparen` / `rparen` / `semi` / end-keyword tokens
> dropped as on alternative 1. Alternative 1 is BYTE-IDENTICAL across the release (the two in-block pins on all three profiles; every
> all-typed corpus file). Measured on the final bytes: the corpus AST identity sweep (the 1.0.217 probe vs this release, 16 336 files × 2 SV profiles + the 2 606-row `verilog_2005` lane) reads `ast_moved=1060 verdict_moved=0` on `sv_2017` (8696 of 9756 accepted trees byte-identical), `ast_moved=1064 verdict_moved=0` on `sv_2023`, `ast_moved=40 verdict_moved=0` on the `verilog_2005` lane — exactly the census's predicted files (`sv_2017`: predicted 1060, moved 1060, predicted-but-identical 0, moved-but-not-predicted 0; `sv_2023`: predicted 1064, moved 1064, predicted-but-identical 0, moved-but-not-predicted 0; `verilog_2005`: predicted 40, moved 40, predicted-but-identical 0, moved-but-not-predicted 0); `sv_2017` `transitions: NONE` (pass 9756 / fail 6580 of 16336 rows, 0 timeouts); `sv_2023` `transitions: NONE` (pass 9780 / fail 6556 of 16336 rows, 0 timeouts); `verilog_2005` `transitions: NONE` (pass 2310 / fail 296 of 2606 rows, 0 timeouts); pins `ADJUDICATION-REPROS: checked=781 armed=149 listed=407 multi_profile_rows=279 failures=0` (eight new
> `control_tf_body_*.sv`, one per alternative of each rule, arms `tf_port` / `!tf_port` pinning WHERE the ports live); six AST-shape
> contract samples, one per newly typed alternative, parsed AS the body rule (two test-only dispatch arms). ⭐ The CLASS is censused and
> owned: the new `PARTIAL-TYPING-CENSUS:` (every multi-alternative rule typed on some alternatives and not others) read 40 such rules
> with 25 raw-envelope alternatives on 1.0.217 and reads 38 / 19 after this release — the nineteen (`assignment_pattern`,
> `block_data_type`, `case_statement`'s `matches` / `inside` forms, `clocking_declaration`, `clocking_event_sv_2017`,
> `list_of_path_delay_expressions`, `net_type_declaration_sv_2017`, `property_case_item`) are `SV-CORPUS-GRAD.3.45`. See the SV parser
> book's [Typed Function and Task Bodies](../systemverilog_parser_book/src/json-carrier.md#typed-function-and-task-bodies-schema-30).

> **Current-state note (2026-09-07, `SV-CORPUS-GRAD.3.39` — GRAMMAR + ANSWER-KEY + INSTRUMENT, release `1.0.217`, ledger `SV-0090`,
> schema `28 → 29`, accept set NARROWS on `sv_2017` / `sv_2023` by exactly the compiler-directive argument forms IEEE 1800-2017 clause 22's
> syntax boxes refuse and WIDENS by white space between a directive's arguments; `verilog_2005` gains four rows and loses none): the thirteen
> compiler directives clause 22 gives a syntax box — `` `resetall ``, `` `undef ``, `` `undefineall `` (SV-only), `` `timescale ``,
> `` `default_nettype ``, `` `unconnected_drive ``, `` `nounconnected_drive ``, `` `celldefine ``, `` `endcelldefine ``, `` `pragma ``,
> `` `line ``, `` `begin_keywords ``, `` `end_keywords `` — are TYPED productions (`1593 → 1641` rules), transcribed box for box from
> §22.3–§22.14 (IEEE 1364-2005 clause 19 reads the same box for box; IEEE 1800-2023 Syntax 22-10 adds `"1800-2023"`, one profiled
> `keywords_version_specifier` token per edition).** Through `1.0.216` `compiler_directive.body` was the directive LINE as one string —
> `"`timescale 1ns / 1ps"` — at file top level and at the `.3.14b` in-scope item positions alike, and the parser accepted ANY arguments
> (`` `timescale frump ``, `` `unconnected_drive pull2 ``, `` `line 1 "f" 3 ``, a bare `` `pragma ``, `` `begin_keywords "VAMS-2.3" ``).
> ⛔ **Consumers: `body` is now a typed node** — `{kind: "timescale", unit: {value, unit: {kind}}, precision: {…}}`,
> `{kind: "default_nettype", value: {kind}}`, `{kind: "unconnected_drive", value: {kind: "pull0" | "pull1"}}`, `{kind: "undef", name}`,
> `{kind: "pragma", name, expressions}`, `{kind: "line", number, filename, level}`, `{kind: "begin_keywords", version}`, and the six bare
> kinds — and every other backtick line the parser tolerates at file top level (`` `include ``, `` `define ``, the conditional family, a
> text-macro use, a vendor directive) is `{kind: "opaque", text}` carrying the old string; switch on `body.kind`. The accept set
> NARROWS on the illegal forms only: **26 corpus rows on `sv_2017` and 23 on `sv_2023`** (each pre-registered by the directive census —
> `docs/tasks/artifacts/sv_corpus_grad/directive_census/census.md` — and re-keyed `must_reject` with the clause, or routed to the
> Verilog-AMS lane), **0 on `verilog_2005`**; and it WIDENS on one form: white space — newlines and comments — between a directive's
> arguments is §5.3 white space (no box states a line rule; only §22.12 binds `` `line `` to its line, which `line_end` asserts), so
> iverilog's GitHub-782 family (`br_gh782c/d/e/f.v`, `` `unconnected_drive /* … */ `` ⏎ `pull1`) goes `fail → pass` on all three lanes —
> four `must_accept` rows leave the `verilog_2005` residual. Three readings a consumer should know: a bare-name `` `pragma `` is bounded
> to its line (`pragma_directive` is `@branch_policy: ordered`; the grammar's default longest-match tournament would otherwise let the
> expression arm eat the next line's first identifier); the MAGNITUDE of a `` `timescale `` argument is syntax (`1`, `10`, `100`, §22.7's
> only definition of it) while Annex A's own `timeunit 2ps;` stays derivable, and the precision ORDERING is a value rule not judged;
> `` `begin_keywords "1800-2023" `` parses under `sv_2023` only, `"1800-2017"` rejects under `verilog_2005`, and under `verilog_2005`
> `` `undefineall `` is an `opaque` text-macro use. Measured on the final bytes: pins `ADJUDICATION-REPROS: checked=761 armed=129
> listed=399 multi_profile_rows=271 failures=0` (forty-one new: thirty `invalid_directive_*.sv` REJECT, eleven `control_directive_*.sv`
> with kind-chain arms); `sv_2017` pass 9 778 → 9 756 (26 `pass → fail` + 4 `fail → pass`), `sv_2023` 9 801 → 9 780, `verilog_2005`
> 2 306 → 2 310; (HEAD probe rebuilt in a detached worktree from the HEAD grammar with the 1.0.216 generator snapshot — regenerated parser BYTE-IDENTICAL to the shipped `94a661afd7dccef1…` — vs the release probe, 16 336 files × 2 SV profiles) reads `sv_2017` `both_accepted=9752 identical=8962 ast_moved=790 verdict_moved=30`, `sv_2023` `both_accepted=9776 identical=8982 ast_moved=794 verdict_moved=27`, and a node census over every fresh-pass file proves the extent TWO-SIDEDLY: the 794 accepted files carrying a `compiler_directive` node are exactly the 790 moved trees ∪ the 4 `fail → pass` rows, and the 8 962 without one are exactly the identical trees; union certificate (`sv_cert_recognized_union_gate`, six configs, seeds 0 / 7 / 42) canonical `1448/8/1425/15 → 1494/10/1469/15`, union witness `1440 → 1484`, union UNKNOWN 0, residual `[]`, `sample_parse_failures=0` at every seed — total +46 = forty-nine new rules − the deleted wrapper − the two edition tokens not satisfiable under `sv_2017` (measured by `--dump-rule-profiles` HEAD vs final), proof +2 = the three lookahead-only `lrm_directive_name*` rules PROOF-covered and `identifier` moved proof → witness (the gate's own accounting with `PGEN_CERT_COVERAGE_DUMP_ALL`, HEAD through the 1.0.216 binary vs final), the canonical UNKNOWN set IDENTICAL (the same fifteen pre-existing names); the 1364-2005 conformance gate (`verilog_2005_conformance_gate`) `1142/378/750/14 → 1186/370/802/14` — total +44 = forty-nine − the five not satisfiable under `verilog_2005`, proof −8 = +2 lookahead-only rules − 10 moved to witness (`time_unit`, its six unit tokens, the two `timeunit_separator_*` rules — reachable under `verilog_2005` only since `timescale_directive`, exactly what the keyword census's leg 4 read — and `identifier`), the UNKNOWN set IDENTICAL (the same fourteen), `lint orphans=0, corpus 363 checks/0 mismatches, aliases 2, cert deterministic across seeds [0,7,42]`; both contracts re-baselined from the gates' own seed headlines (`rust/target/s339/rebaseline_certs.py`, the previous notes chained) and re-stamped GREEN on the re-run (`step_cert_union_green.log`, `step_cert_v2005_green.log`: `the baseline was STALE and every expectation still held, so this run re-stamped it`). Full record: `SV-CORPUS-GRAD.3.39`; ledger `SV-0090`; the json-carrier's
> [Typed Compiler Directives](../systemverilog_parser_book/src/json-carrier.md#typed-compiler-directives-schema-29).

> **Current-state note (2026-09-06, `SV-CORPUS-GRAD.3.43` — GRAMMAR + ANSWER-KEY + INSTRUMENT, release `1.0.216`, ledger `SV-0089`,
> schema `28` UNCHANGED, accept set NARROWS on `sv_2017` / `sv_2023` by exactly the DPI import result types IEEE 1800-2017 §35.5.5 excludes, `verilog_2005` untouched):
> a DPI import function's RESULT type is `void`, `byte`, `shortint`, `int`, `longint`, `real`, `shortreal`, `chandle`, `string` or a
> SCALAR `bit` / `logic` (`docs/systemverilog/2017/md/section-35-direct-programming-interface.md:485-493`; Annex A footnote 21
> `docs/systemverilog/2017/md/section-41-data-read-api.md:3524`; 1800-2023 §35.5.5 `:493-501`, footnote 26). Through `1.0.215`
> `dpi_function_proto := function_prototype` reached `data_type_or_void` whole, so `import "DPI-C" function logic [7:0] f();`,
> `bit [1:0]`, `bit []`, `reg`, `integer`, `time`, `realtime`, an enum / struct / union literal, `event`, a virtual interface,
> `C #(int)` and `my_t [3:0]` were ACCEPTED as DPI results. `dpi_function_proto` now reaches `dpi_function_prototype` — a caller
> split of `function_prototype` whose return slot is `dpi_function_result_type`, the `data_type_or_void` shape restricted to
> §35.5.5 with `data_type`'s own templates (1587 → 1593 rules, six new). **AST impact: none.** Every
> legal import's typed tree is byte-identical (interpreter A/B on 38 probes × 2 profiles; the corpus AST identity sweep over
> 16 335 files × 2 SV profiles `ast_moved=0 verdict_moved=2` on each profile — 9 778 / 9 799 accepted trees byte-identical, the two verdict movers the pre-registered files); a consumer reading `import_function.proto.body.body.return_type` sees the
> `data_type_or_void` shapes it already knows, and the refused forms never reach it because the file no longer parses. A
> NAMED type without packed dimensions (typedef, `pkg::T`, `C::T`, `type(…)`) is admitted exactly as `data_type` admits it
> (semantic resolution); signing on the atom and vector types is admitted (§35.5.5 is silent on it); `export "DPI-C" function g;`
> carries no type and is unchanged. **Corpus**: the AST-field census (`AST-FIELD-CENSUS:`, the parser's own reading of the slot)
> read 1 298 DPI result slots in 172 files, 24 illegal in four files, exactly TWO accepting — sv2v `test/nosim/dpi.sv:35`
> `bit [15:0] getStimulus` (the LRM's OWN §35.5.4 example, `:476`, which contradicts the normative §35.5.5 one paragraph later
> and which 1800-2023 repeats unchanged beside the unchanged restriction — the normative text wins) and verilator
> `t_dpi_result_type_bad.v` (sixteen results); `sv_2017` and `sv_2023` each read exactly two rows `pass → fail` (the two pre-registered files) and 0 `fail → pass` (pass 9 780 → 9 778 / 9 801 → 9 799, 0 timeouts), `verilog_2005` `transitions: NONE`; both keyed `must_reject` with the production (and
> `t_dpi_import.v`, already rejecting, pre-registered). Verilator accepts ≤ 32-bit 2-state vector results as an extension;
> a consumer that relied on that extension through this parser now receives a rejection at the result type's `[`.
> Certificates: union `1448/8/1425/15 + 1440/0`, 1364-2005 `1142/378/750/14` unchanged; parse cost: `entries` -1,472 and `memo_hits` -512 PROMOTED (falls), `committed` conserved exactly at 6 741 854 — the only entries risers are the five new rules (+32 each), 39 pre-existing rules fell, the LR-family share `2.661 %` reproduces to the anchor. Fourteen REJECT pins
> (`stimuli/sv/adjudication_repros/invalid_dpi_result_*.sv`) and two controls guard the slot.**

> **Current-state note (2026-09-06, `SV-CORPUS-GRAD.3.42` — GRAMMAR + INSTRUMENT, release `1.0.215`, ledger `SV-0088`,
> schema `27 → 28` (a REPLACEMENT of corrupted nodes), accept set UNCHANGED on `sv_2017` / `sv_2023`, `verilog_2005` NARROWS by one non-LRM literal form): sixteen inline
> alternations `( a | b )` that sat inside groups of annotated rules are REMOVED — fifteen LIFTED into eight named,
> unannotated helper rules (`implicit_handle_or_class_scope`, `method_call_receiver_qualifier`, `nonrange_lvalue_scope`,
> `ps_or_hierarchical_array_scope`, `ps_type_identifier_scope_sv_2017` / `_sv_2023`, `split_callable_receiver_scope`,
> `out_of_block_tf_scope`; every helper `@profiles: ["sv_2017", "sv_2023"]`) and the sixteenth, `hierarchical_btf_identifier`'s
> third alternative, respelled as its only live form `class_scope method_identifier` (`1579 → 1587` rules).** Spelled inline,
> the group's branch N was typed with the ENCLOSING rule's return template N wherever one existed
> (`ENGINE-UNIVERSAL-SERVICES.96`; a one-template host shadows branch 0, an unannotated alternative keeps the corrupted
> element in its raw envelope), so through `1.0.214` the parser emitted the host's template wrapped around a scope
> prefix in ACCEPTING files: `foreach (this.arr[i])` carried `array.scope = {name: {kind: "dot"}, scope: {kind: "this"}}`;
> `solve this.x before this.y;`, `disable soft this.x;` and `this.arr = new[4];` carried
> `scope: {name: {kind: "dot"}, scope: {kind: "this"}, select: "<invalid_sequence_access>"}` (the `sv_2023` twin also
> `call: "<invalid_sequence_access>"`); `covergroup cg @@(begin C::m)` with class `C` declared carried
> `scope: {kind: "block", body: <class_scope>}`; `function void I.h();` put six `<invalid_sequence_access>` into the raw
> envelope of `function_body_declaration` alternative 3. ⛔ **Consumers: the affected fields now carry the prefix as
> ITS OWN node** — `this .` is the pair `[{kind: "this"}, {kind: "dot"}]`, a class scope the `class_scope` node, a
> package scope the `package_scope` / `non_typedef_package_scope` node, an interface prefix the pair
> `[<interface_identifier>, {kind: "dot"}]`; a consumer that tolerated `"<invalid_sequence_access>"` in `select` / `call` /
> `name` / `items` / `statements` / `end_label` no longer sees it. **Byte-identical everywhere else, measured over the
> whole corpus** (the AST identity sweep, TOOLBOX §1.5e, HEAD probe vs release probe): over 16 335 corpus files × 2 SV profiles the HEAD-vs-release AST identity sweep reads `ast_moved=4 verdict_moved=0` on each profile (9 776 / 9 797 accepted trees byte-identical), the four movers being exactly the shadowed-branch files the liveness census predicted (ispras `25.07.03_01.sv`, `25.07.04_01.sv`, verilator `t_modport_export_bad.v`, `t_cast_string.v`), each differing only in its scope element (24 `<invalid_sequence_access>` sentinels gone); the
> non-shadowed branches (`pkg::`, `C::`), every absent optional and `function void C::g();` do not move; 610 pin ×
> profile arms `verdict_moved=0 ast_moved=0` under the interpreter; the seeded 393-file corpus pre-screen
> moved **0** verdicts and **0** furthest positions over 786 arms on both the lifted candidate and the final grammar (`sv_2017` 53 accept/accept + 340 reject/reject, `sv_2023` 54 + 339). **No corpus verdict moves**, by construction (a lift changes no accepted language) and by
> measurement: all three lanes on the final bytes read `transitions: NONE` (`sv_2017` 9 780 / 6 556, `sv_2023` 9 801 / 6 535, `verilog_2005` 2 306 / 300; 0 timeouts) — 0 rows moved in either direction. ⭐ **AND ONE `verilog_2005` VERDICT NARROWS, on invalid text, as a side effect worth naming**: the `class_qualifier` literal (IEEE 1800 Annex A's NONTERMINAL, extracted as a keyword — `SV-CORPUS-GRAD.13c.2m`) reached the IEEE 1364-2005 profile through `split_hierarchical_callable_receiver`'s inline group, whose host carries no `@profiles`; the lifted `split_callable_receiver_scope` is SV-gated, so `module m; initial class_qualifier a.b(); endmodule` — the `V2005-KEYWORD-FAITHFULNESS` census's witness — goes `parse_full passed` → `furthest_position=33` under `verilog_2005` (HEAD probe vs new), and the census reads **`over_acceptances 1 → 0`, `gated 24 → 25`** — the last confirmed 1364-2005 keyword over-acceptance closed; the corpus spells the literal in 0 files (the `verilog_2005` lane moved 0 rows), and the PRIMARY-position form `y = class_qualifier m;` is unchanged on every profile (`sv_2017` still the `.13c.2m` accepts-invalid, `verilog_2005` REJECT at 648 before and after). Liveness of the lifted rules (`LIVENESS-CENSUS:`): of the eight helpers, three commit on **0** corpus files (`method_call_receiver_qualifier`, `implicit_handle_or_class_scope`, `nonrange_lvalue_scope`), and the four that commit (`ps_type_identifier_scope_*` 14 files, `split_callable_receiver_scope` 13, `out_of_block_tf_scope` 34 accepted, `ps_or_hierarchical_array_scope` 1) do so almost entirely through never-shadowed `pkg::` / `C::` branches — exactly **4** accepted files spell a shadowed branch (`t_cast_string.v`'s `foreach (this.m_counters[i])`, the three interface out-of-block definitions), and those are precisely the trees the sweep names. Pins:
> `checked=618 armed=103 listed=342 multi_profile_rows=218 failures=0` (four new controls, six new arm claims). Gates on the final bytes: `sv_cert_recognized_union_gate` GREEN — canonical `1443/8/1420/15` (identical at seeds `[0,7,42]`), union `1435` / UNKNOWN `0` on the SIX existing configs (the one new canonical UNKNOWN, `method_call_receiver_qualifier`, is a successful-but-losing branch from the canonical entry — Protocol A measured; the slice's first spelling, a ninth helper, had made the union seed-dependent and was respelled); `verilog_2005_conformance_gate` GREEN — `1142/378/750/14` UNCHANGED; `sv_syntax_closure_gate` GREEN (`unreachable_rules=1`, the allow-list ONE); `ast_shape_contract_gate` 18/18; reproducibility TIER 2 byte-identical; binaries byte-identical across the gates' rebuilds. Parse cost: parse cost: `entries` +0.15 % and `committed` +6 ACCEPTED under the existing `introduced_wrapper_frames` invariant (every riser a helper's own frame, zero pre-existing rules up in any counter, ten down), `memo_hits` −0.36 % promoted, the LR-family share `2.665 → 2.661 %` adopted at its anchors (family entries unchanged exactly). Instruments: the class
> census (`docs/tasks/artifacts/sv_corpus_grad/inline_alt_census/inline_alt_census.py`, `INLINE-ALT-CENSUS:` `16 → 0`
> on the shipped grammar, seven-control `--self-test`), the sweep + liveness pair (`…/inline_alt_lift/`), and the
> tracked grammar-candidate pre-screen scripts made runnable from their tracked location (they had derived the repo
> root one level short since `1.0.214`, so that release's pre-screen numbers came from scratch copies). Honest bound,
> named: no instrument pins a NESTED typed shape byte-exactly — the pin runner's `arm` and the AST-shape contract both
> key on `kind` — routed `ENGINE-UNIVERSAL-SERVICES.97`; the other families' census sites (`ebnf` 35,
> `semantic_annotation` 14, `vhdl` 2, `rtl_frontend` 1) and the engine fix are `ENGINE-UNIVERSAL-SERVICES.96`; typing
> the raw function/task body alternatives is `SV-CORPUS-GRAD.3.44`.

> **Current-state note (2026-09-05, `SV-CORPUS-GRAD.3.38` — GRAMMAR + ANSWER-KEY, release `1.0.214`, ledger `SV-0087`,
> schema `27` UNCHANGED, `sv_2017` + `sv_2023` NARROW by one construct family): IEEE 1800-2017 Annex A footnote 20 is
> grammar — an unsized PACKED dimension `[]` is legal only as the SOLE packed dimension of a formal in a DPI IMPORT
> declaration.** A.2.5 `packed_dimension20 ::= [ constant_range ] | unsized_dimension` carries footnote 20 (*"permitted
> only as the sole packed dimension in a DPI import declaration; see dpi_function_proto and dpi_task_proto"*), §35.5.6.1
> adds that exported functions cannot have open arrays and that *"an unsized dimension shall be the sole packed
> dimension of a formal argument"*, and 1800-2023 keeps both verbatim (footnote 24). Until `1.0.213` the grammar took
> the unsized arm at EVERY packed site, so `reg [] illegal;` parsed at module level (iverilog `br_ml20181012b.v`,
> Surelog `UnsizedPacked/dut.sv`), as did `input [] p` on an implicit-type port and `logic [7:0] [] x` /
> `logic [] [7:0] x` inside an import. Now the DPI spec string of an import declaration opens a `dpi_import` scope and
> the declaration's `;` closes it (two marker rules, the `.3.29` mechanism — on the spec string, because a marker on the
> shared `import` keyword would open a scope at every package import), the unsized arm of `packed_dimension` is
> reachable only through `dpi_open_packed_dimension` — gated `in_scope_kind(dpi_import)` as a `post` predicate, with
> `!lbrack` refusing a following `[` — and the range arm refuses a following `[]` (`1576 → 1579` rules). `verilog_2005`
> is untouched (no unsized dimension exists there, `SV-0033`). ⛔ **Consumers: NO shape moves.** The markers bind
> exactly the elements the return annotations already bind (`spec: $2`) or discard (the `;`), and the gated arm passes
> `unsized_dimension`'s node through, so a packed `dims` entry still reads `{kind: "unsized", body: {kind: "unsized"}}`
> — now only inside a DPI import. Every DPI open array of §35.5.6.1's own list (`bit[]`, `logic [] arrayNx3 [3:1]`,
> `bit [] arrayNxN []`, `logic [31:0] array32xN []`), a task import, an unnamed formal (footnote 23) and both export
> forms parse with a typed AST byte-identical to `1.0.213` (interpreter AST JSON); 593 pin × profile arms and 174
> corpus candidates (every DPI-import file plus every textual `[]` candidate) moved NO verdict and NO typed AST beyond
> the two rows named. The accept set NARROWS by exactly the footnote: an unsized packed dimension outside a DPI import,
> or beside a sized one. Corpus, per path, three lanes on the final bytes: `sv_2017` exactly TWO rows `pass → fail` — iverilog `ivltests/br_ml20181012b.v` (`unexplained_accepts_invalid → match`) and Surelog `tests/UnsizedPacked/dut.sv` (`match` under its corrected key) — and **0** `fail → pass`, pass `9 782 → 9 780` (16 336 rows, 0 timeouts, 0 crashes), match `5 961 → 5 962`, unexplained (the axis-2 bar) **`100 → 99`**; `sv_2023` the same two rows, pass `9 803 → 9 801`, match `5 953 → 5 954`, unexplained `100 → 99`, the Surelog ruling CARRIED OVER mechanically (`production_parity.py`: `packed_dimension:IDENTICAL variable_dimension:IDENTICAL`; `carried_over 47+5 → 48+5`, `inherited_reject_reread_owed 16+184` unchanged); `verilog_2005` **0** rows moved (2 306 pass / 300 fail); the sv_2017 × sv_2023 census `both_pass=9780 2023_only_pass=21 2017_only_pass=0 population_breaches=0`; verdict coverage `7556/2606/6174/4016/99`; key provenance `clause_cited 715 → 716`, `tool_testimony 2227 → 2226`, `bar_trust_bound=95/99`. Every moved row is one the pre-screen named before the parser was built. Answer key: Surelog
> `tests/UnsizedPacked/dut.sv` (`must_accept` by its golden log) re-keyed `must_reject` on the footnote — Surelog's
> parser takes the unsized arm at every site and its elaboration never checks it; the unit itself names the variable
> `illegal`. Gates on the final bytes: `sv_cert_recognized_union_gate` GREEN — canonical `1436/8/1414/14` (identical at seeds `[0,7,42]`, `sample_parse_failures=0`, `proof_reverify_failures=0`), union witness `1428` / UNKNOWN `0`, residual `[]`, re-stamped (24 m 34 s under the guard, peak 6 735 MB); the canonical `+1` UNKNOWN is `dpi_open_packed_dimension`, scope-excluded by construction — the planner routes it through a module-level `specparam []` (`[plannable-probe] … parsed=false witnessed_target=false sample="module m(input wire a);specparam[]\\foo =7413.78;endmodule"`, rendered correctly and refused by the scope gate) — and a SIXTH union config `dpi_import_export:sv_2017` witnesses it from the import declaration itself (`sample="import\"DPI-C\"function bit[]\\foo ;"`); expectations PRE-STAMPED from the seed-0 census (five configs: union UNKNOWN `1` = `[dpi_open_packed_dimension]`; six: `0`) and confirmed by the gate first time; `verilog_2005_conformance_gate` GREEN — cert `1142/378/750/14` UNCHANGED (the three new rules are SV-gated, the shape a profile-only narrowing must have; identical at seeds `[0,7,42]`, `spf=0 prf=0`), lint orphans 0, corpus 363 checks / 0 mismatches, aliases 2, re-stamped (32 m 27 s, peak 10 303 MB); ⚠️ both gates re-emitted the parser and rebuilt the binaries: the release probe (`788d7ade…`) and the generated parser (`48d68d91…`) came back BYTE-IDENTICAL to the binaries the pins and lanes were keyed on (`rust/target/s338/sha_after_build.txt` vs `sha_after_certs.txt`; the debug `ast_pipeline` did not, `ebc1d186… → 2e132cce…`, which nothing downstream is keyed on); `sv_syntax_closure_gate` GREEN — `defined_rule_count 1545 → 1548`, `reachable_rules 1575 → 1578`, `unreachable_rules=1` (the allow-list ONE, `casting_type`), `unreachable_branches=2`, re-stamped (1 m 40 s, peak 1 916 MB); `ast_shape_contract_gate` GREEN — `18 passed; 0 failed` (3 m 12 s, peak 9 375 MB); `check_generated_reproducibility.sh --rebaseline` TIER 2 — every family re-derives byte-identically, baseline rewritten (1 m 25 s, peak 2 011 MB); `check_corpus_key_integrity.sh` OK (499 keyed rows, 0 contradictions; provenance 716 clause-cited); `check_identity_carriers.sh` OK (`live=12 frozen=7 rows_equal=51`); `check_sv_corpus_denominator.sh` OK (`7556/2606/6174/4016/99`); `report_family_status.sh --check-all` OK (11 blocks equal a fresh derivation after re-splicing the cross-family and SystemVerilog pages); `report_grammar_certification.sh --markdown` re-spliced (`1436 rules · 1428 witness · 8 proof · 0 unknown (canonical unknown 14)`); dark audit + worklist re-derived, `WORKLIST 53` unchanged. Parse cost: share → rebaseline → ratchet: the derived LR-family share `2.665 %` REPRODUCES EXACTLY (23 504 878 of 882 119 470 entries over 16 336 files; the constant and its anchors untouched); the first rebaseline REFUSED on three breaches — BINDING `entries` `402,541,171 → 402,638,477` (+0.024 %), `memo_hits` `175,670,128 → 175,688,013` (+0.010 %), `committed` `6,741,840 → 6,741,848` (+8). Per rule (`rule_costs.tsv` HEAD vs fresh): `dpi_open_packed_dimension` +79,340 entries (one per attempt of the arm it fronts; +149 memo hits), `unsized_dimension` +17,832 entries of which +17,733 memo hits (the range arm's `!unsized_dimension` guard asks a memoized rule at positions the star had already asked; the 99 misses are single-optional packed sites — enum base types, specparams — and cost `lbrack` +99), the two markers +192 / +32 entries and +4 / +4 committed (one frame per DPI import the sample accepts, bounded by `dpi_spec_string`'s 5 and `semi`'s commits), and `dpi_spec_string` −96 / `trivia` −93 entries = the memo hits `dpi_import_spec_open` now answers in their place (+96); the hosts `packed_dimension` and `dpi_import_export` asked, answered and committed EXACTLY as before; no pre-existing rule commits differently. None of the ten coded invariants admits this shape (`contained_in_introduced_subgraph` refuses the +8 `committed` and any fall; `introduced_wrapper_frames` covers the `committed` half exactly but needs the pre-existing entry rises covered by retired entries — 17,931 against 189; `caller_split_containment` needs `committed` conserved; `host_memo_eviction` needs a host losing its memo; the three identities need shapes this is not), so the ratchet gained an ELEVENTH, `scope_gated_arm` (`scripts/parse_cost_containment.py`; 18 self-test arms, every refusal shape fired): the HOSTS are DERIVED from the two reference graphs and unchanged in all three counters, no pre-existing rule commits differently in either direction, Δcommitted == the markers' own frames each bounded by what it wraps, every pre-existing riser sits inside the sub-graph reachable from the introduced rules and their hosts and is bounded by its consulters' entries (the misses are REPORTED, +198, never thresholded), and every pre-existing faller sits inside the introduced sub-graph and is at most the introduced rules' memo hits (189 ≤ 245). Three rows in `accepted_rises.tsv`; the rebaseline RE-DERIVED it on the run — *hosts dpi_import_export packed_dimension asked/answered/committed exactly as before; introduced … commit 8 frames (= Δcommitted, each bounded by what it wraps), no pre-existing rule commits differently; 2 pre-existing riser(s) (+17,931) all inside the gated construct, each bounded by its consulters' entries, +198 of them misses; 2 faller(s) (−189) inside the introduced sub-graph, covered by 245 introduced memo hits* — and PROMOTED; `sv_parse_cost_ratchet` OK (identity fresh; advisory wall clock −3.7 %). ⛔⛔ THE FIRST EVALUATION OF THE NEW PREDICATE REFUSED ON 117 HOSTS, AND THAT WAS A REAL DEFECT IN THE GATE: the tracked `rule_graph.json` every scoped predicate read as `base_edges` described grammar `4ebcc61b…` (1 485 rules), committed once by `PGEN-SV-CORPUS-GRAD-0261` and — by the ratchet's own R1/R3 rule that a derivation is never promoted — never updated, against a baseline of `1e2ad24a…` (1 545 rules): twenty releases and sixty rules stale, unnoticed because the three readers that already consumed it used it only as a name SUPERSET. The stored copy is DELETED (`git rm`) and the gate now DERIVES the baseline graph at evaluation time from the committed grammar revision whose raw_ast digest the identity table records (`_baseline_edges`: `git show <rev>:grammars/systemverilog.ebnf` → `derive_graph`, digest-verified, REFUSING when no committed revision reproduces it); re-evaluated, the hosts are exactly the two rules the change re-spelled. GATE tier: `scripts/parse_cost_containment.py`, `scripts/check_parse_cost_ratchet.sh`; ZERO Rust. Honest bound, named not modelled:
> the scope is the whole import declaration, so a DPI RETURN type could carry `[]` by this grammar's letter; footnote
> 21 / §35.5.5 restrict result types to small values (scalar `bit` / `logic` only), so `import "DPI-C" function logic [7:0] f();`
> derives too — a further over-acceptance, owned by `SV-CORPUS-GRAD.3.43`.

> **Current-state note (2026-09-04, `SV-CORPUS-GRAD.3.41` — GRAMMAR, release `1.0.213`, ledger `SV-0086`,
> schema `26 → 27` (a REPLACEMENT), `sv_2017` + `sv_2023` WIDEN by one construct family): the set-membership operator
> `inside { … }` is a CHAIN ELEMENT of `expression_base`'s operand chain, so an inside expression can be the LEFT
> operand of a binary operator — `y = a inside {2'b10, 2'b11} & ~b;`, `… && b`, `x inside {1,2} || x inside {4,5}` —
> and `b & a inside {…}` is no longer grouped as `(b & a) inside {…}`.** IEEE 1800-2017 A.8.3 derives
> `expression binary_operator expression` with `inside_expression ::= expression inside { open_range_list }` on either
> side, and Table 11-2 (11.3.2, row 7 `< <= > >= inside dist`) puts `inside` above `==`, `&`, `^`, `|`, `&&`, `||`. Until
> `1.0.212` the grammar reached a grouped `inside_expression := expression_base inside { … }` only at the top of
> `expression`: the chain could not continue past the `}` (REJECT, `furthest_position=630`, the `&`), and on the accepted
> right-operand form the wrapper swallowed the whole left chain (a wrong tree on accepted text, measured with the
> interpreter's typed AST JSON). This grammar flattens every binary operator into ONE chain whose precedence is the
> consumer's; `inside` now obeys the same rule. `expression_base`'s chain is `expression_operand expression_chain_step*`
> where a step is the unchanged binary triple OR `inside_operator`; `expression`, `expression_or_cond_pattern` and the
> three `inside_expression` rules lose the grouped form (`1575 → 1576` rules). `verilog_2005` is untouched (`inside` is an
> identifier there — pinned). `dist` is unchanged: IEEE 1800-2017 18.5 rules *"dist expressions cannot appear in other
> expressions"*, so `expression_or_dist` stays its only surface. ⛔ **Consumers — schema 27, a REPLACEMENT:**
> REMOVED `expression.kind == "inside"`, `expression_or_cond_pattern.kind == "inside"` and the `inside_expression` shape
> `{expr, ranges}`; ADDED an `operand_chain.rest` entry that is the OBJECT `{kind: "sv_2017"|"sv_2023", body: {kind:
> "inside", ranges}}` beside the array triples `[binary_operator, [attribute_instance …], expression_operand]`.
> Migration: an object entry is the postfix `inside {set}` operator on Table 11-2's relational row; `body.ranges` is the
> same typed `open_range_list` / `range_list`; the left operand is the chain to the entry's left. Every `inside`-free tree
> is byte-identical HEAD vs `1.0.213` (the interpreter's AST JSON on an inside-free control and across the pin set:
> 586 pin × profile arms under the interpreter, HEAD grammar vs candidate (`rust/target/s341/prescreen_pins.py`, 5 jobs, 13 m): verdict moved **2** — the defect pin on both SV profiles, REJECT → ACCEPT; typed AST moved **2** — `defect_scope_randomize_host_contexts.sv` on both profiles, the one other pin whose text carries an `x inside {…}` expression (by design); every other arm's verdict AND typed-AST sha identical). Corpus, per path, three lanes on the final bytes: `sv_2017` 1 rows `fail → pass`, **0**
> `pass → fail`, pass `9 781 → 9 782`, match `5 960 → 5 961`, the axis-2 bar `101 → 100`; `sv_2023`
> 1 rows `fail → pass`, pass `9 802 → 9 803`; `verilog_2005` 0 rows moved. Pins `checked=593 armed=88 listed=330 multi_profile_rows=206 failures=0`.
> Parse cost: share → rebaseline → ratchet: the derived LR-family share `2.664 → 2.665 %` (23 504 782 of 881 837 627 entries over 16 336 files; total corpus entries −0.02 %, family +0.03 %; adopted by hand in the constant, `TOOLBOX.md`, `DOCTRINE_ENFORCEMENT.md`, the `a-deterministic-counter…` card and `cost.md`); the first rebaseline REFUSED on FIVE breaches — the stale share (four anchors) and BINDING `committed` ROSE `6 712 187 → 6 741 840` (+0.44 %) — while BINDING `entries` FELL `402 867 861 → 402 541 171` (−0.08 %) and `memo_hits` FELL `176 431 867 → 175 670 128` (−0.43 %), both promoted. Per rule (`rule_costs.tsv` HEAD vs fresh): the ONLY `committed` risers are the three introduced rules — `expression_chain_step` +29 697 (one wrapper frame per binary step the sample accepts), `inside_operator` +4, `inside_operator_sv_2017` +4 — against −4 on each of the thirteen pre-existing rules of the retired nested `inside_expression → expression_base` derivation (net +29 653, exact); NO pre-existing rule commits more; the retired grouped rule had re-parsed its whole left operand at every expression (`inside_expression` −812 254 / `inside_expression_sv_2017` −402 984 / `expression_base` −402 984 entries) against `kw_inside` +51 157 and `trivia` +51 795 (the set's terminal now tried at every chain step) and 31 binary-operator terminals +22 each. No coded invariant admits a `committed` rise that sits purely on introduced frames (the eight others need `committed` flat, conserved, or ≤ 0, or refuse any pre-existing rise), so the ratchet gained a TENTH, `introduced_wrapper_frames` (`scripts/parse_cost_containment.py` + `scripts/check_parse_cost_ratchet.sh`, 11 self-test arms, refusals for an empty scope / a declared pre-existing nonterminal / a name the grammar lacks / a wrapper that wraps nothing; failure arms for a pre-existing committed rise, an undeclared new rule, a wrapper committing more than the rules it wraps, a from/to that is not the table's arithmetic, uncovered pre-existing entry rises): every rule new to the measurement declared, no pre-existing rule commits more, each introduced nonterminal's commits bounded by the commits of the rules it wraps, `Δcommitted` == introduced frames net of commits retired elsewhere, pre-existing entry rises covered by retired entries. One row in `accepted_rises.tsv` (`committed 6712187 → 6741840`, scope `expression_chain_step inside_operator inside_operator_sv_2017 inside_operator_sv_2023`) naming the leaf that removes the rise: `ENGINE-UNIVERSAL-SERVICES.96` (the inline spelling retires every wrapper frame). Second rebaseline PROMOTED (`entries`/`memo_hits` down, `committed` ACCEPTED and RE-DERIVED on the run); `sv_parse_cost_ratchet` **OK** — identity fresh for the generated parser, grammar raw ast, instrument and 192 pinned sample files; wall-clock advisory +1.6 % within the ±50 % band. Gates on the final bytes: pins `checked=593 armed=88 listed=330 multi_profile_rows=206 failures=0`; `sv_cert_recognized_union_gate` GREEN (canonical `1433/8/1412/13`, union witness `1425` / UNKNOWN `0`, residual `[]`, seeds `[0,7,42]`, re-stamped); `verilog_2005_conformance_gate` GREEN (`1142/378/750/14`, `spf=0 prf=0`, lint orphans 0, corpus 363 / 0 mismatches, re-stamped); `sv_syntax_closure_gate` GREEN (`unreachable_rules=1`, the allow-list ONE, re-stamped); `ast_shape_contract_gate` 18/18; reproducibility TIER 2 byte-identical, baseline rewritten; parse-cost ratchet OK — `entries` −0.08 % and `memo_hits` −0.43 % promoted, `committed` +0.44 % accepted under the new `introduced_wrapper_frames` invariant (the lifted chain-step wrapper's own frames, removable by `ENGINE-UNIVERSAL-SERVICES.96`); the release probe and generated parser byte-identical across the gates' rebuilds. ⭐ The first spelling of this fix — an inline
> `( … | inside_operator )*` group — typed an inside-free chain's first step with `expression_base`'s alternative-0
> template (`{kind: "tagged_union", body: {kind: "bitwise_and"}}`); the lifted rule is exact and the defect is routed
> as `ENGINE-UNIVERSAL-SERVICES.96` with a five-rule reproducer.

> **Current-state note (2026-09-04, `SV-CORPUS-GRAD.3.40` + `.3.36` — GRAMMAR, release `1.0.212`, ledger `SV-0085`,
> schema `26` UNCHANGED, `sv_2017` + `sv_2023` WIDEN by one construct): `local::` before a SUBROUTINE CALL inside
> `randomize() with { … }` — `get_x() == local::get_x()`, `local::get_c().c1_f`, `local::K::m()`, `local::this.f()` —
> parses.** IEEE 1800-2017 §18.7.1 applies the qualifier to *an identifier within an in-line constraint* and lets it
> *qualify class scopes and type names*, while Annex A A.8.4 `class_qualifier` reaches `primary` and no `tf_call`
> production: an Annex gap, not a prohibition. Release `1.0.211` had spelled A.8.4 faithfully and therefore NARROWED
> this one legal text (until then `local` was an admissible identifier standing in for a package name, and the text
> parsed by accident); the scoped tf-name rule now carries the same `local :: [ this. | C:: ]` arm `primary` already
> had. In the same release the dead `block_data_type_or_implicit` (orphaned by `.3.35`) is deleted (`1576 → 1575` rules)
> and the syntax-closure allow-list returns to ONE. ⛔ **Consumers: one `kind` is ADDED and nothing is removed or
> renamed** — a tf-call name may now be `{kind: "local_scope", scope, name}` beside `class_scope` / `package_scope`, and
> only on text that had NO derivation before; `x < local::x` (primary position) and the unqualified call chain keep
> byte-identical trees, every legal control its kind multiset. Corpus: exactly ONE row `fail → pass` on `sv_2017` and on
> `sv_2023` (verilator `t_randomize_method_with_scoping.v`), **0** the other way, `verilog_2005` unmoved; the axis-2 bar
> `102 → 101`. Parse cost: `entries` +0.30 % and `memo_hits` +0.34 % with `committed` FLAT — exactly one unmatched
> terminal alternative (`Δentries == 2 × Δmemo_hits`), ACCEPTED under the coded `unmatched_terminal_alternatives`
> invariant; the LR-family share `2.671 → 2.664 %`. Gates on the final bytes: closure GREEN (allow-list ONE); `sv_cert_recognized_union_gate` GREEN — canonical `1432/8/1411/13`, union witness `1424` / UNKNOWN `0`, residual `[]`, deterministic across seeds `[0,7,42]`, `sample_parse_failures=0`, re-stamped (18 m 2 s under the guard, peak 6 756 MB; expectations set from the gate's own seed-0 run — `total 1433 → 1432`, `proof 9 → 8`: the deleted dead rule had counted as a PROOF rule since 1.0.210; witness and UNKNOWN unchanged);
> `verilog_2005_conformance_gate` GREEN — cert `1141/378/749/14` (samples 40, deterministic across seeds `[0,7,42]`, `spf=0 prf=0`), lint orphans 0, corpus `363` checks / `0` mismatches, aliases 2, re-stamped (39 m 30 s under the guard, peak 12 175 MB; expectations set from the gate's own seed-0 run — `total 1142 → 1141`, `proof 379 → 378`, the deleted dead rule, which had no `@profiles` and so counted as a proof rule under this profile too; the `local ::` arm is gated out of it and the lane moved 0 rows); shape `ast_shape_contract_gate` GREEN — `18 passed; 0 failed` (3 m 57 s under the guard, peak 8 500 MB); reproducibility `check_generated_reproducibility.sh --rebaseline` TIER 2 — every family re-derives byte-identically from HEAD, baseline rewritten (1 m 21 s, peak 1 954 MB); `sv_parse_cost_ratchet` OK — identity fresh for the generated parser, grammar raw ast, instrument and 192 pinned sample files; the wall-clock advisory +7.5 % within the ±50 % band on a loaded host.

> **Current-state note (2026-09-04, `SV-CORPUS-GRAD.3.37` + `.13z` — GRAMMAR, release `1.0.211`, ledger `SV-0084`,
> schema `26` UNCHANGED, `sv_2017` + `sv_2023` NARROW): the SV reserved-word set is IEEE 1800-2017 Annex B
> Table B.1 — 248 words, derived from the LRM text and held there by doctrine `SV-RESERVED-WORDS` — where the
> hand-typed list had held 173: `new`, `this`, `super`, `null`, `static`, `var`, `const`, `global`, `final`, every
> assertion and coverage word were admissible identifiers, so `reg global;`, `i.super.i = 1;`, `var final;` and a
> constructor written `function new; input x;` (an ordinary function NAMED `new`) all parsed.** A bare `null` gains the
> `primary` alternative A.8.4 names, and `local ::` (A.8.4 `class_qualifier`) the production it never had — both had
> parsed only as identifiers. Two more footnotes are grammar: no `automatic`
> on a data declaration outside a procedural context (footnote 10 — module, package, checker and class sites; the
> randsequence code block keeps its lifetime), and a body `parameter` carries its default (footnote 18's other half;
> the parameter port list still may omit it). ⛔ **Consumers: one derivation is corrected** — `d = null;` / `d == null`
> used to yield a hierarchical NAME node for `null` and now yields the `{kind: "null"}` primary; the verdict is
> unchanged, the tree is right. Every other legal input keeps its kind multiset. Corpus: `sv_2017` 53 rows `pass → fail` (footnote-18 body parameters, Annex B keywords bound as names in Verilog-2005 tests, A.2.7 constructor orderings, footnote-10 lifetimes, an A.2.4 `new[…]`, and one legal-by-prose `local::get_x()` owed by `.3.40`), 0 `fail → pass`; `sv_2023` the same plus the two forward-type rows footnote 22 rules; the axis-2 bar
> `112 → 102` (five rows refused by the grammar, four keys corrected by `.13z` with the production); `verilog_2005`
> not at all. Parse cost: `entries` +0.40 % and `memo_hits` +0.47 % — the `null` and `local::` terminal attempts net of the footnote re-spellings — ACCEPTED under a new coded invariant (`ENGINE-UNIVERSAL-SERVICES.94`); `committed` FELL −0.59 %; family share `2.679 → 2.671 %`; `sv_parse_cost_ratchet` OK.

> **Current-state note (2026-09-04, `SV-CORPUS-GRAD.3.35` — GRAMMAR, release `1.0.210`, ledger `SV-0083`,
> schema `26` UNCHANGED, `sv_2017` + `sv_2023` widen AND narrow): four corrections from the residual-bar
> triage by construct.** Widened: a port default followed by the next port (`clk = $inferred_clock,
> rst = $inferred_disable`; `string msg = "violation", coverage_level clevel = cover_all`) — the actual
> argument's event expression no longer absorbs the list's comma; and `type T = int, T x = 0` (§6.20.1's
> own example). Narrowed, by Annex A's own footnotes: an implicit data type in a data declaration requires
> `var` (footnote 6 — `a;`, `y = 1;`, `[1:0] x;`, `signed x;` stop parsing at item level), and a
> `localparam` always carries its default (footnote 18). ⛔ **Consumers: one derivation is corrected** — a
> block whose first item is a statement (`begin x = 1; …`) used to yield an implicit-type `block_data`
> declaration node for it and now yields the `statement` it is; the verdict is unchanged, the tree is right.
> Every other legal input keeps its kind multiset. Corpus: `sv_2017` 17 rows moved — seven `fail → pass`, ten `pass → fail` (seven the footnotes target, three illegal texts that had parsed only through the mis-derivation), the axis-2 bar `137 → 112`; the `sv_2023` lane moves identically, `verilog_2005` not at all. Parse cost: `entries` FELL `414 471 371 → 400 072 681` (−3.47 %), `committed` −0.12 %, `memo_hits` −5.61 %, promoted by the rebaseline; family share `2.744 → 2.679 %`; `sv_parse_cost_ratchet` OK.
> The unterminated-block-comment acceptance found on the way is the engine's layout skipper
> (`ENGINE-UNIVERSAL-SERVICES.93`), routed, not fixed here.

> **Current-state note (2026-09-04, `SV-CORPUS-GRAD.3.34` + `.3.33` — GRAMMAR, release `1.0.209`, ledger `SV-0082`,
> schema `26` UNCHANGED, `sv_2017` WIDENS): a parenthesised NESTED tagged pattern — `tagged Jmp (tagged JmpU .a)`,
> `tagged Jmp (tagged JmpC '{.c, .a})`, the form IEEE 1800-2017's own §12.6.1 examples write
> (`section-12-procedural-programming-statements.md:826`, `:828`, `:840`) — now parses under `sv_2017`.** 1800-2017 Annex A A.6.7.1 `pattern`
> (`section-41-data-read-api.md:2279-2286`) omits `( pattern )`, an erratum 1800-2023 corrects by spelling it
> first (`section-Annex_A-normative-formal-syntax.md:2273-2281`); `pattern_sv_2017` had followed Annex A
> literally. It gains the 2023 arm's own alternative — no new rule, no new `kind` on any input that parsed
> before; a newly-accepted parenthesised pattern carries `{kind: "parenthesized"}` as it always has under
> `sv_2023`. The ten single-letter keyword terminals the UDP fix of `1.0.207` orphaned are deleted (1564 → 1554
> rules; no input ever reached them). Corpus: `sv_2017` exactly five rows `fail → pass` (the predicted ones), `pass → fail` 0 on all three lanes, the axis-2 bar `142 → 137`; the `sv_2023` and `verilog_2005` lanes move nothing. Parse cost: `entries` +0.04 % / `memo_hits` +0.04 % with `committed` exactly unchanged — one sampled fixture now parses 2 216 bytes further; the other 191 files byte-identical; accepted under a new per-file ratchet invariant, `advanced_files_only`.

> **Current-state note (2026-09-03, `SV-CORPUS-GRAD.3.31` — GRAMMAR, release `1.0.208`, ledger `SV-0081`,
> schema `26` UNCHANGED, `sv_2017` + `sv_2023` WIDEN): an array-element member access whose INDEX is an
> `inc_or_dec_expression` or a parenthesised `operator_assignment` — `clist[b[0]++].m <= 1;`,
> `clist[(b[0] += 1)].m` — now parses.** IEEE 1800-2017 A.8.3 gives `expression`
> (`section-41-data-read-api.md:3026`) the two operands that `constant_expression` (`:2982`) lacks, and
> the index of an element whose member is then selected was the ONE index path the grammar spelled
> through `constant_bit_select` (A.9.3 `hierarchical_identifier`, `:3361`); the member-less index
> (`clist[b[0]++] = null;`) went through `bit_select` and always parsed. The fix is a caller split:
> `hierarchical_bit_select` (over an operand adding exactly those two alternatives) at the ten
> hierarchical-path sites, `constant_bit_select` and its constant-context callers untouched. Every input
> that parsed before yields a byte-identical typed AST; a newly-accepted index carries the existing
> `inc_or_dec_expression` / `operator_assignment` node shapes as the operand body. Corpus: `sv_2017`
> exactly one row `fail → pass` (the predicted one), `pass → fail` 0, the axis-2 bar `162 → 161`;
> `verilog_2005` 0 rows moved. Parse cost `entries` +0.33 % / `memo_hits` +0.38 % with `committed`
> conserved exactly — the split pays for the memo the two twins no longer share — accepted under a new
> coded ratchet invariant and routed for elimination (`ENGINE-UNIVERSAL-SERVICES.92`).

> **Current-state note (2026-09-03, `SV-CORPUS-GRAD.3.30` — GRAMMAR, release `1.0.207`, ledger `SV-0080`,
> schema `26` UNCHANGED, ALL THREE profiles WIDEN): a UDP state table written with adjacent symbols —
> `(01)`, `??`, `01`, `0x`, `(bx)` — now parses, on every profile.** IEEE 1800-2017 §29.3.4
> (`section-29-user-defined-primitives.md:196`) separates the input fields *from the output field by a
> colon* and says nothing about white space between symbols; the standard's own tables read
> `(01) (10) 0 : 0 : 1 ;` (`:204`; 1364-2005 §8.1.4 `:213`), and Annex A A.5.3 gives the symbols as single
> tokens. The grammar's three table rules referenced its general keyword terminals (`trivia /0\b/` …),
> whose `\b` cannot match a symbol followed by another symbol character, so every spaced table parsed and
> every touching one was refused. Fourteen boundary-free `udp_sym_*` terminals, referenced only by
> `level_symbol`, `output_symbol` and `edge_symbol`, replace them there; the general keywords keep their
> boundary everywhere else (`assert #0`, `$finish(0)`, `init_val`, `scalar_constant`, `z_or_x`).
>
> **What now parses that did not**: any UDP table whose symbols touch — edge indicators `(01)`, `(0x)`,
> `(bx)`, level runs `??`, `01`, `00`, `0x`, sequential and combinational alike. **What still rejects**: a
> table field no symbol production derives (`2 0 : 0 ;`, `0 a : 0 ;` — pinned as
> `invalid_udp_non_symbol_field.sv`).
>
> **AST impact — shape-neutral.** The three rules keep their `kind` values; every table that parsed before
> yields a byte-identical typed AST (measured on the spaced control, all three profiles), and an adjacent
> table yields the AST its spaced spelling always did (equal once source spans are stripped). ⛔ The accept
> set **widens** on every profile — text that used to fail now yields an AST.
>
> **Corpus proof, per-path and two-sided, both lanes**: `sv_2017` **17** rows `fail → pass` (8
> `unexplained_rejects_valid → match`, 9 already routed or deferred), none the other way; pass
> **9 813 → 9 830**, `match 5 851 → 5 861` since `1.0.205`, the axis-2 bar **220 → 212**; `verilog_2005`
> **6** rows `unexplained_rejects_valid → match`, pass **2 300 → 2 306**. Certificates: union basis
> `1424/18/1393/13`, union `1406/0` (the ten keyword terminals the tables no longer reference are now
> provably unreachable and move witness → proof); `verilog_2005` `1137/383/740/14` with the gate's own flags
> (its three union configs; union `753/1`), the identical 14 UNKNOWN — a bare run without them reads `394/3` because
> the entry-unreachability proof quantifies over the declared entry universe, an artifact of the command line. Cost: the parse-cost
> ratchet reads `entries`, `memo_hits` and `committed` exactly unchanged — each new terminal takes what its
> keyword twin lost.
> **Current-state note (2026-09-03, `SV-CORPUS-GRAD.3.29` — GRAMMAR, release `1.0.206`, ledger `SV-0079`,
> schema `26` UNCHANGED, `sv_2017` + `sv_2023` WIDEN, `verilog_2005` unchanged): a `cross` body's two implicit
> typedefs, `CrossValType` and `CrossQueueType`, now parse as type names inside the cross — and nowhere else.**
> IEEE 1800-2017 §19.6.1.3 (`section-19-functional-coverage.md:1307`) says *"The types are named CrossValType
> and CrossQueueType. The scope of the type names is the cross itself and the types are not accessible
> outside of this scope"* and `:1320` *"The cross types shall be considered as implicit typedefs in the body of
> the cross"*; Annex A declares them nowhere, so `function CrossQueueType myFunc1(int f_lim);` — the standard's
> own §19.6.1.4 example, vendored as ispras `ieee-1800-2012/19/19.06.01.04_02.sv` and Surelog
> `tests/CrossFunc/dut.sv` — was refused by the parser's sound store gate (`checked_type_identifier`'s
> `has_fact(type_name, …)` found no fact). The grammar opens a `cross` scope at the body's `{` and closes it at
> the `}` (`cross_body_open` / `cross_body_close`, the first scope this grammar opens), and each of the two
> unscoped type use-sites gains an ORDERED second branch — a terminal spelling the two names, gated `post` on
> `in_scope_kind(cross)` — tried only after the unchanged store gate has refused the name.
>
> **What now parses that did not**: `function CrossQueueType f(…); … endfunction` inside a cross, and
> `CrossValType v;` / `CrossQueueType q;` as local types inside such a function; `bins b = f(…)` using it.
> **What still rejects**: `CrossQueueType q;` anywhere outside a cross body (pinned:
> `stimuli/sv/adjudication_repros/invalid_cross_predefined_type_after_cross.sv`), and `int CrossValType;` at
> module level still parses as an ordinary variable — the two spellings are not keywords.
>
> **AST impact — shape-neutral.** A predefined name arrives exactly where a declared type does and with the same
> shape: `{type: {body: "CrossQueueType"}, dims: […]}` beside `{type: {body: "q_t"}, dims: […]}`; no `kind` is
> added, removed or renamed; the typed AST of every input that parsed before is byte-identical (measured on the
> two cross-body controls, both profiles). ⛔ The accept set **widens** on `sv_2017` and `sv_2023` — text that
> used to fail now yields an AST.
>
> **Corpus proof, per-path and two-sided, both lanes**: exactly the two rows above moved, `fail → pass` and
> `unexplained_rejects_valid → match`; `sv_2017` pass **9 811 → 9 813**, `match 5 851 → 5 853`, the axis-2 bar
> **222 → 220**; `verilog_2005` pass **2 300** unmoved, its rule set unchanged at 1 123. Certificate: the union
> gate gains a fifth entry `cross_body:sv_2017` (canonical 1410/8/1389/13, union 1402/0 — the two new
> scope-gated rules are UNKNOWN from the canonical entry by construction and witnessed from the cross body).
> Cost: `PARSE-COST-RATCHET` measured `entries` +634 818 (+0.15 %) confined to the four introduced rules
> (one recognizer entry per refused type attempt outside a cross), accepted under
> `contained_in_introduced_subgraph`; the engine-tier elimination is `ENGINE-UNIVERSAL-SERVICES.90`.
> **Current-state note (2026-09-03, `EBNF-FRONTEND-SILENT-TRUNCATION.6` — EBNF FRONTEND (engine), release
> `1.0.205`, ledger `SV-0078`, schema `26` UNCHANGED, ALL THREE profiles): from the entry
> `systemverilog_parseable_file` (and `parseable_source_item`, `--entry-rule`, TOOLBOX §1.2 — both proven
> UNREACHABLE from the registered entry by `rule_fire_partition`), an `import p::*;`, a `timeunit 1ns;` or a
> `` `timescale `` item no longer parses: `parseable_source_item := semi` is the whole rule, and the three lines
> that follow it in the grammar (`| package_import_declaration`, `| timeunits_declaration`,
> `| compiler_directive`) have been `//` comments since 2026-04-25.** `grammars/ebnf.ebnf:583-586` defines
> `//` as a line comment; the hand-written frontend knew only `#`, read those lines as live alternatives (each
> with an empty regex token from the `//`), and every release since carried them. The generated parser moves in
> that ONE rule (`generated/systemverilog_parser.rs` `62fec0ab… → c82018f9…`); the other ten generated parsers
> are byte-identical. **AST shape: UNCHANGED for every legal input; no `kind` added, removed or renamed.** The
> semantic digest also moves for a second, envelope-only reason: 14 inline return-annotation payloads in 11
> rules had carried the indented comment that followed them (classified `return_scalar`); the generator had
> always re-read the balanced payload, so no parser output changes there — but from this release a
> comment-only edit after an annotation no longer moves the digest this contract is keyed on (measured).
> **Current-state note (2026-09-02, `SV-CORPUS-GRAD.13p` — GRAMMAR, release `1.0.204`, ledger
> `SV-0077`, schema `26` UNCHANGED, ALL THREE profiles): a THIRD delay value on an n-input or
> n-output gate, a pass-enable switch or a UDP instance no longer parses.** IEEE 1800-2017 §28.4
> says of those gates *"The delay specification shall be zero, one, or two delays"* and Annex A
> A.3.1 / A.5.4 spell `[ delay2 ]` there; three delays belong to enable gates and switches
> (`[ delay3 ]`). The shipped grammar carried ONE `delay` rule with the three-value alternative at
> every site, because the LRM extractor's footnote canonicaliser merged Annex A's `delay2` and
> `delay3` into one rule — the right keywords in the wrong arity, invisible to any keyword census.
>
> **What no longer parses**: `nand #(1,2,3) g (o, a, b);` and its `and` / `or` / `nor` / `xor` /
> `xnor` / `buf` / `not` siblings, `tranif0 #(1,2,3)`-style pass-enable switches and
> `my_udp #(1,2,3) u (…)` — on every profile, because every edition forbids them. `bufif0 #(1,2,3)`
> (an enable gate), `assign #(1,2,3)` and `wire #(1,2,3)` keep parsing: Annex A gives them `delay3`.
>
> **AST impact — NONE for legal text.** A two-delay site yields the same
> `{body: {kind: "pair_optional" | "value", …}}` object it always did; `kind: "triple_optional"`
> simply becomes unreachable at the seven `delay2` sites, where it was never legal. No `kind`
> added, removed or renamed. ⛔ This **narrows** the accept set on `sv_2017` too: text that used to
> yield an AST now rejects — safe for a consumer relying on LRM-faithful rejection, a change for
> one that fed such text through.
>
> **Corpus proof, per-path and two-sided**: ZERO verdicts moved on either lane — 16 336 `sv_2017` rows and 2 606 `verilog_2005` rows, row sets identical, pass 9 811 / 2 300 unchanged — and the axis-2 bar moved **268 → 269** for a reason that is measured and routed: `verilator/test_regress/t/t_gate_basic.v` (already `fail`) went from `divergence:explained_svpp_conditional` to `divergence:unexplained_rejects_valid` because its parse now stops at line 25, `nand #(2,3,4)`, where no preprocessing can alter the text (the `.12a` rule), under a `must_accept` key that rests on the suite's driver (*"t_gate_basic.py expects success"*) and not on the standard, which forbids that text (§28.4). It is the first MAIN-lane member of `SV-CORPUS-GRAD.13k`'s answer-key class and is routed to `.13k`(a)+(c) with the citation; the key is not edited in a release slice, so the bar reports the honest number under the key as it stands. Tuple `7556/2606/6174/4363/268` → `…/269`, `check_sv_corpus_denominator.sh` OK after the anchor moved with it. Measured blast radius before the run:
> exactly ONE of the 16 336 vendored files spells a three-value delay on a two-delay gate
> (`verilator t_gate_basic.v:25`), and it already failed for an unrelated preprocessing reason.
> The reproducer and two controls are pinned on the two-sided ratchet, which went RED on the
> reproducer row when the fix landed and now guards it on all three profiles.
> **Cert census, adjudicated by name**: `verilog_2005_conformance_gate` `1121/373/734/14` → **`1123/373/736/14`** (the two `verilog_2005`-satisfiable new rules `delay2` and `delay2_sv_2017`, both witnessed; proof, UNKNOWN and both failure counts unchanged; corpus matrix 363 checks / 0 mismatches) and `sv_cert_recognized_union_gate` total **1404 → 1406**, canonical witness **1385 → 1387**, union witness **1396 → 1398** (`delay2_sv_2023` profile-excluded by construction; proof 8, canonical UNKNOWN 11, union UNKNOWN 0, residual `[]` unchanged), both identical at seeds 0 / 7 / 42 and both re-derived GREEN on the adjudicated contracts; `ast_shape_contract_gate` 18 / 18, schema 26; `PARSE-COST-RATCHET` read an **IMPROVEMENT** — `entries` **−53 390** (−0.01 %), `memo_hits` **−38 119** (−0.02 %), `committed` unchanged, no pre-existing rule dearer (`delay_sv_2017` lost 3 memo hits to `delay2_sv_2017`, and the seven sites no longer speculate a third value) — promoted; ⭐ `ENGINE-UNIVERSAL-SERVICES.88`(b)'s PRE-REGISTERED prediction held: no terminal-only rule gained a profile-conditional alternative, and no memo eviction occurred; `GENERATED-REPRODUCIBILITY` tier 2 re-derived and re-stamped; clippy source + generated clean.

> **Current-state note (2026-09-01, `SV-CORPUS-GRAD.13o` — GRAMMAR, release `1.0.203`, ledger
> `SV-0076`, schema `26` UNCHANGED, `verilog_2005` profile ONLY): `always @(edge c)` no longer
> parses under `verilog_2005`.** The shipped `edge_identifier` carried IEEE 1800's third member
> `edge` with no profile gate; IEEE 1364-2005 A.7.4 is `edge_identifier ::= posedge | negedge` and
> its A.6.5 `event_expression` admits only `posedge` / `negedge` forms. `edge` IS a 1364-2005 keyword
> (the timing-check `edge_control_specifier`, A.7.5.3), which is why the keyword-admission census
> could not see it — the keyword is legal, the POSITION is not. Found by the keyword-enumeration
> membership census (TOOLBOX 5.13) and confirmed by three probe legs: `rc 0` under `verilog_2005`,
> `rc 0` under `sv_2017`, and `rc 1` under `verilog_2005` once `edge` is replaced by a fresh
> identifier.
>
> **What no longer parses**: an `edge` event control (`@(edge x)`) under the `verilog_2005`
> profile — text IEEE 1364-2005 never admitted. Nothing changes under `sv_2017` / `sv_2023`: the
> `edge` branch now references the profile-gated terminal `kw_edge_sv_only` under
> `@profiles: ["sv_2017", "sv_2023"]` (the `.13e.7` alias spelling was measured dearer by the cost
> ratchet and rejected), so those profiles are unmoved by construction — and by measurement (0 of 16 336 `sv_2017` corpus verdicts moved).
>
> **AST impact — NONE.** No `kind` value is added, removed or renamed; `"edge"` simply becomes
> unreachable under `verilog_2005`, and under the SystemVerilog profiles the `edge_identifier` node
> is byte-identical (`{kind: "edge"}` re-measured on the regenerated parser). ⛔ This **narrows**
> the `verilog_2005` accept set: text that used to yield an AST there now rejects — safe for a
> consumer relying on LRM-faithful rejection, a change for one that fed IEEE 1800 sources through
> the 1364 profile.
>
> **Corpus proof, per-path and two-sided**: ZERO verdicts moved on either lane — 16 336 `sv_2017`
> rows and 2 606 `verilog_2005` rows, row sets identical on both sides — and the verdict-coverage
> tuple `7556/2606/6174/4363/268` is unchanged. 0 of the 2 606 v2005-lane files spell `@(edge`
> (24 of the 16 336 main-lane files do, every one an IEEE 1800 source), so this is exactly the
> over-acceptance a corpus never probes negatively. The reproducer and two controls are pinned on
> the two-sided ratchet, which went RED on the reproducer row when the fix landed and now guards it.
> **Cert census, adjudicated by name**: `sv_cert_recognized_union_gate` total **1403 → 1404**, canonical witness **1384 → 1385**, union witness **1395 → 1396** — the one new rule, witnessed — with proof 8, canonical UNKNOWN 11, union UNKNOWN 0 and residual `[]` unchanged, identical at seeds 0 / 7 / 42; `verilog_2005_conformance_gate` GREEN with its census `1121/373/734/14` UNCHANGED (the gated terminal is unsatisfiable under that profile) and its corpus matrix **360 → 363** checks / 0 mismatches (the new `reject/v2005_edge_event.sv` case × 3 profiles); `ast_shape_contract_gate` 18 / 18, schema 26; `GENERATED-REPRODUCIBILITY` tier 2 re-derived and re-stamped; clippy source + generated clean. `PARSE-COST-RATCHET`: `entries` **+260** (+0.00 %) and `memo_hits` **+104** on a 412 397 710-entry / 184 456 833-hit sample, `committed` unchanged — attributed rule by rule (`edge_identifier` lost its 52 memo hits with entries and commits unchanged, `kw_posedge`/`kw_negedge` re-ran +52 each, `trivia` +104, `kw_edge_a0f2c69a` released 2 508 entries to the introduced `kw_edge_sv_only`), ACCEPTED under the coded `host_memo_eviction` invariant (`ENGINE-UNIVERSAL-SERVICES.88`), which the gate RE-DERIVED on its own run, and the baseline promoted.

> **Current-state note (2026-09-01, `SV-CORPUS-GRAD.13n` — GRAMMAR, release `1.0.202`, ledger
> `SV-0075`, schema `26` UNCHANGED, ALL THREE profiles): `tri0` and `tri1` are net types again.**
> The `net_type` alternation shipped **ten** of the LRM's **twelve** branches since before the revision register's genesis — the BRANCH has never existed (`git log -S 'kw_tri0'` returns exactly one commit, the fix). ⚠️ The observable REJECTION is younger and its start date is measured: the two spellings entered the reserved-keyword regexes at `cd1dfa14` (2026-07-02, `verilog_2005`) and `436b64fa` (2026-07-04, `sv_2017`/`sv_2023`), and it is that registration — a token that lexes as a keyword with no production to land in — that makes the text unparseable. Whether `tri0 a;` parsed BEFORE those dates, when the spelling was an ordinary identifier, is **UNMEASURED**. Both spellings were in the reserved-keyword regexes, so they lexed as
> keywords and had nowhere to land: `module m; tri0 a; tri1 b; endmodule` was `rc 1` on `sv_2017`,
> `sv_2023` **and** `verilog_2005`, while `wire`, `triand` and `trior` from that same alternation
> were `rc 0`.
>
> **What now parses that did not**: any declaration using the `tri0` / `tri1` net types — pull-down
> and pull-up nets, which appear in ordinary vendor RTL and in every SystemVerilog edition. All
> three vendored editions define them (IEEE 1364-2005 Annex A A.2.2.1:219, IEEE 1800-2023
> Annex A:891, IEEE 1800-2017 clause 6 at `section-6-data-types.md:928`).
>
> **AST impact — ADDITIVE ONLY.** Two new `kind` values, `"tri0"` and `"tri1"`, join the ten a
> `net_type` node could already carry; no existing shape is replaced or renamed and the schema is
> unchanged. A consumer switching *exhaustively* on that field must add two arms; one that matches
> known values and falls through is unaffected. ⛔ This **widens** the accept set, so text that used
> to fail now yields an AST — safe for a consumer reading ASTs, and a change for one relying on
> rejection.
>
> **Corpus proof, per-path and two-sided**: `sv_2017` pass **9 794 → 9 811** with the axis-2
> divergence bar **274 → 268**; `verilog_2005` pass **2 291 → 2 300** with its arm **66 → 57**.
> **18** rows moved in the main lane, **9** in the v2005 lane, the row sets identical on both sides,
> and **18 of 18 contain `tri0` or `tri1`** — attribution measured, not inferred. ⭐ Nine of the
> eighteen are `deferred:v2005_profile_lane`, so a real fix is again partly invisible to the bar,
> which is `SV-CORPUS-GRAD.13`'s standing thesis.

> **Current-state note (2026-08-28, `ENGINE-UNIVERSAL-SERVICES.65` — ENGINE, grammar-agnostic,
> release `1.0.201`, ledger `SV-0074`, schema `26` UNCHANGED, ALL profiles): the 4096-frame
> recursion ceiling is gone.** It is replaced by a bound the parse thread's own stack supplies:
> `4096` survives only as the depth at which the guard first MEASURES how much stack it has spent,
> and the ceiling is whatever the remaining headroom can still carry.
>
> **What now parses that did not**: syntactically valid deeply-nested expressions. On the shipped
> default reservation, nested `int'(…)` casts go from **249** accepted source levels to **4 813**
> (19.3×); VHDL's nested parentheses leave the measurement harness's own 16 384-level cap entirely,
> against 681 before. Because constructs spend frames at very different rates, the practical effect
> is largest exactly where it mattered least before — machine-emitted RTL, where bracketing style
> alone could decide whether a design parsed.
>
> **How you can tell it is really host-derived and not just a bigger number**: vary the reservation
> and the boundary moves with it, linearly. At 96 / 128 / 192 / 256 MiB the deepest accepted nested
> cast is 1 807 / 2 410 / 3 611 / 4 813 — **18.82 levels per MiB, flat to 0.14 %**.
> ⭐ `.87`(d) added the first point ABOVE the reference and it holds: 384 MiB bisects to **7 211**
> (18.78 levels/MiB), so across 96–384 MiB the source-level fit is flat to **0.26 %** — wider than
> the 0.14 % this contract published for 96–256 alone, and stated rather than quietly reused.
> ⭐⭐ Measured on FRAME DEPTH instead, the same pair is flat to **0.010 %** (81 670 → 122 517 frames
> for a reservation ratio of exactly 1.5): the source-level figure carries a shape-dependent
> frames-per-level conversion (16.97 → 16.99) that the frame count does not. A constant ceiling
> would be **33 %** off. Set
> `PGEN_PARSE_STACK_BYTES` (bytes; floored at 96 MiB) to buy more depth on a host that can afford
> it. A constant of any size cannot produce that curve.
>
> ⚠️ **What this costs, stated up front.** Parse cost is **quadratic in depth**: the same cast shape
> takes 0.13 s at depth 250 and **22.24 s at depth 4 000** (each doubling ≈3.95×), and refusing
> input one level past the boundary now costs **37.7 s** where it used to cost 0.19 s. The refusal
> is still bounded and still carries the `.61`(a) ceiling label — but if you feed this parser
> thousands of nesting levels, plan for seconds-to-minutes, not milliseconds. Tracked as
> `ENGINE-UNIVERSAL-SERVICES.68`.
>
> ✅ **The last literal is GONE (2026-08-31, `ENGINE-UNIVERSAL-SERVICES.69`(a)): the reservation's
> DEFAULT is now derived from your host's physical memory**, not compiled in. ⭐⭐ **And since
> `.87`(d) (same day, embedding API `1.3.6`) it is derived in BOTH directions** — the upper clamp
> that held every host over 16 GiB at 256 MiB is gone, so the law is now
> `max(96 MiB, physical_memory / 64)`. Ask any build what it resolved and why:
> `parseability_probe --stack-reservation` prints the reservation, its provenance
> (`operator_override` / `host_memory_derived` / `reference_fallback`) and the host reading.
> ⭐ The `1/64` share is chosen so `256 MiB × 64 = 16 GiB` — the law returns the old literal
> EXACTLY on a 16 GiB host, which is the design point it explains rather than replaces. A SMALLER
> host is no longer handed a per-parse stack it may not be able to afford; a LARGER one is no
> longer held back to a 16 GiB machine's capacity.
>
> ⛔⛔ **AND SAY THE OTHER HALF PLAINLY, BECAUSE IT IS A BEHAVIOUR CHANGE YOU CAN SEE: ON A HOST
> BELOW 16 GiB THE ACCEPT SET NARROWS, PROPORTIONALLY.** The paragraph above first stated this
> effect only as a benefit; that was one-sided and is corrected here. The accept boundary is
> LINEAR in the reservation — this contract's own measured table, 18.82 levels per MiB flat to
> 0.14 % — so halving the reservation halves the deepest input that parses:
>
> | host physical memory | resolved reservation | deepest accepted nested `int'(…)` cast | how |
> |---|---|---|---|
> | 64 GiB | 1 GiB | ≈ 19 250 | **derived from the law**, not measured |
> | 24 GiB | 384 MiB | **7 211** | **BISECTED** — 7 211 accepts, 7 212 refuses. A 4 814-cast file that `1.3.5` rejected now parses |
> | **16 GiB** | **256 MiB** | **4 813** | bisected (`.65`) — the design point, unchanged since before `.69`(a) |
> | 12 GiB | 192 MiB | 3 611 | bisected (`.65`) |
> | 8 GiB | 128 MiB | 2 410 | bisected (`.65`) |
> | ≤ 6 GiB | 96 MiB (floor) | 1 807 | bisected (`.65`) |
>
> ⇒ **the same file can now parse on one host and be REFUSED on another — in EITHER direction from
> wherever you measured.** The refusal is the ordinary bounded `E_PARSE_FAILURE` with the ceiling
> label, never a crash. This is the capability mandate working as specified — the limit is the
> host's RAM — but it means **an accept-set claim is now a claim about a host**.
> ⛔⛔ **PINNING IS AN ESCAPE HATCH, NOT AN OBLIGATION, AND THIS PARAGRAPH USED TO IMPLY OTHERWISE.**
> The contract is that **a valid input parses for as long as the host's resources allow** — the
> default must simply work, and no embedder is required to configure anything to get it. Pin
> `PGEN_PARSE_STACK_BYTES` only if you positively WANT one identical accept set across machines
> (a fleet-wide or CI-wide determinism choice, not a correctness requirement).
> ⚠️ **THE HONEST GAP, STATED HERE RATHER THAN IN A TASK TREE.** *"As long as the host allows"* is
> not yet what the engine implements: it hands one parse a fixed **1/64** share of installed
> memory, so a 24 GiB host refuses at 7 211 nesting levels while ~23 GiB sits free. Raising that
> share is not free either — measured, the deepest ACCEPTED parse at a 384 MiB reservation costs
> **730 MB peak RSS** against a 377 MiB stack budget, i.e. total footprint runs ≈1.9× the
> reservation, so the reservation bounds the STACK and not the process. Deriving the share from a
> real footprint bound is `ENGINE-UNIVERSAL-SERVICES.84`'s measurement, and until it exists the
> 1/64 is a deliberately conservative policy rather than a limit anyone measured. ⚠️ For context on whether this can reach real input:
> no file in the 16 336-file tracked corpus comes near the boundary (p99.9 bracket nesting is 8),
> so this bites machine-emitted or adversarial input, not ordinary RTL.
>
> ⚠️ **AND THE COST SIDE, WHICH MOVES WITH IT.** Parse cost is quadratic in depth, so a bigger host
> also spends proportionally LONGER before refusing an over-deep input — the boundary it has to
> reach first is further away. The safety margin moves the other way and in your favour: above
> 256 MiB the guard's first stack sample stays at 4 096 frames while the budget keeps growing, so
> the margin on the worst measured cell goes **2.12× → 3.17×** at 384 MiB.
> ⭐ Per-frame census anchor: `debug 2.12x @ systemverilog/verilog_2005 · spread 14.81x · separation 5.55x · release 16.57x @ systemverilog/verilog_2005 · debug/release 7.83x @ systemverilog/verilog_2005 · measured 3d292d1a5 2026-08-30 (debug) / d5dc08fb1 2026-08-31 (release)`
> (`PER-FRAME-CENSUS-CURRENCY`, `ENGINE-UNIVERSAL-SERVICES.83` — the figure above is re-derived from
> the tracked census on every commit and this line is held equal to it, as are the platform book,
> the engine's doc comment and the knowledge card.)
> ⚠️ The derivation is PER-PARSE — this API spawns one thread per call, so `N` concurrent
> maximally-deep parses can commit up to `N/64` of host memory; set `PGEN_PARSE_STACK_BYTES` smaller
> if you run high parse concurrency. Measured for scale: 128 concurrent live parse threads at 16 GiB
> each spawn with no refusal, so the real ceiling is address space rather than a limit you will meet.
> ⚠️ A physical-memory reading outside `[512 MiB, 64 TiB)` is treated as UNKNOWN and takes the fixed
> 256 MiB reference — an unreadable or implausible `sysctl` is never allowed to become an outage.
>
> ⚠️ The derived bound applies to threads PGEN creates for parsing, which is every embedding-API
> entry and every CLI instrument; a parse driven on some other thread keeps the previous
> 4096-frame behaviour, deliberately, because this process cannot know that thread's stack size.
>
> **Current-state note (2026-08-28, `ENGINE-UNIVERSAL-SERVICES.66` — ENGINE, grammar-agnostic,
> release `1.0.200`, ledger `SV-0073`, schema `26` UNCHANGED, ALL profiles):** a per-quantifier
> **10 000-iteration cap** (`SAFETY_LIMIT`) is removed from every generated parser. It did not
> refuse — **it stopped iterating**, silently truncating the repetition, so the parse failed at the
> enclosing level as an ordinary *"did not consume full input"* with nothing naming the cause.
>
> **What now parses that did not**: any construct built from a repetition longer than 10 000 —
> measured, a **flattened-netlist module with 10 001 instances**, 10 001 wire declarations, and a
> `case` with 10 001 items; concatenation element lists and module port lists failed just above the
> same cap. All accept at 10 001, 12 000 and 20 000 after this release: **13 of 13 measured
> rejections eliminated, 0 introduced.** This is the limit a GENERATED or FLATTENED RTL flow was
> most likely to meet, and real ASIC gate-level netlists run 10⁵–10⁷ instances.
>
> **The replacement bound is the INPUT, not a bigger constant**: every counted iteration must advance
> the cursor, so a repetition cannot have more elements than the input has bytes. The guard still
> terminates a pathological zero-width loop; it can no longer refuse conforming source.
>
> **Nothing else moves.** External-corpus verdicts are **identical over 16 336 files** (`pass=9794
> fail=6542 timeout=0 crash=0`, 0 per-file differences, same `--profile sv_2017`); the recursion
> ceiling's accept boundaries are byte-for-byte unchanged (`(((…)))` 322/323, `a ? b : (…)` 272/273,
> `int'(…)` 249/250); the interpreter remains byte-identical to the generated parser
> (`parse_harness_equivalence_gate`, 4/4). No grammar bytes, so the SV grammar digest below is
> unchanged; the parser artifact is regenerated because codegen moved.
>
> ⚠️ **Read this bound before sizing a job**: removing the cap makes a large netlist *legal*, not yet
> *ingestible*. Parse memory measures **2 500×–8 000× the input size** and scales linearly — a 523 KB
> netlist peaks at ≈4.0 GB and a 1.1 MB one at ≈8.1 GB. The limit is now your host's RAM rather than
> a constant in PGEN's source, which is the intended shape, but it arrives sooner than it should.
> Tracked as `ENGINE-UNIVERSAL-SERVICES.67`.

> **Current-state note (2026-08-26, `SV-CORPUS-GRAD.13e.10`(c4) RESIDUE +
> `ENGINE-UNIVERSAL-SERVICES.50`(e)/`.55` — GRAMMAR + ENGINE, release `1.0.199`, ledger `SV-0072`,
> schema `26` UNCHANGED, `verilog_2005` ONLY):** the **seventeenth and last** IEEE-1800-only
> punctuation member, `'` (`tick`), is now gated out of the strict profile, closing the class
> `.13e.10` opened. IEEE 1800 A.6.7.1 spells assignment patterns and A.8.4 spells casts with `'`;
> IEEE 1364-2005 has neither construct.
>
> **What no longer parses under `verilog_2005`:** `a = '{1,2};` and the assignment-pattern /
> cast family reached through it. ⭐ **`1'b0` and `8'hff` still parse under all three profiles** —
> the `'` occurrences in IEEE 1364-2005 are inside sized/based number literals, which the `number`
> terminal lexes and which never reach this rule. That is the standing control, and it is pinned.
>
> **`sv_2017` / `sv_2023` are UNMOVED**: 10 of 10 typed ASTs byte-identical over 5 inputs × 2 SV
> profiles, and the chained cast `int'(2)'(3)` still ACCEPTs on the **generated** parser — the
> observation `ENGINE-UNIVERSAL-SERVICES.50`(c) owed, which the interpreter could not settle because
> TOOLBOX 1.5b marks its verdicts untrustworthy exactly when `left_recursion_unhandled > 0`, the
> state under test.
>
> ⛔ **IT TOOK THREE ENGINE REPAIRS TO MAKE ONE TAG LANDABLE**, and they are why it was deferred:
> `.50`(b) — tagging `constant_cast` makes its `@profiles` **cross** `constant_primary_sv_2017`'s,
> the route intersection `["sv_2017"]` is a set no rule on the route declares, and the eliminator
> **refused the whole `casting_type` candidate** (`left_recursion_unhandled` **0 → 15**, a WARNING,
> so `--lint-grammar` still exited 0); it now SYNTHESIZES that intersection onto the clone. `.55` —
> the pass's own `casting_type_lr_suffix` / `_lr_guard{0,1}_suffix` are an ordered choice over the
> ROUTES and inherited `@profiles` from the BASE rule, which declares none, leaving them profile
> **orphans whose lint-derived fix nobody could apply** (there is no source line to tag); the wrapper
> now takes the UNION of its branches' gates. `.50`(d) — the counter is now RATCHETED.
> ⭐ **The companion set is 7 SOURCE rules, not the 11 the lint names**: four are eliminator
> temporaries that resolve once the engine derives their gates, so the edit is **8 tags**, not the
> 24-28 `(c4)` priced.
>
> **`verilog_2005` cert baseline moved and is adjudicated BY NAME**: `1131/356/762/13` →
> `1119/373/732/14`, and the arithmetic closes exactly. **total −12** = the 8 gated source rules plus
> the 4 eliminator temporaries, all 12 of which were in `witness`. **proof +17** = 17 survivors moved
> `witness → proof`, i.e. to the STRONGER certificate — the `casting_type` LR seeds/guards/base plus
> `array_pattern_key`, `assignment_pattern_key`, `assignment_pattern_expression_type`,
> `structure_pattern_key`, `integer_type`, `simple_type`. **UNKNOWN +1** is `member_identifier`, and
> it is the fix working rather than a parse change: its `verilog_2005` witness was carried by
> `'{a:1}` — the illegal assignment pattern this release removes — measured as `committed=1` under
> `sv_2017` and ACCEPT-before / reject-after under `verilog_2005`. The rule stays satisfiable under
> the profile through `select` / `constant_select` / `nonrange_select`.

> **Current-state note (2026-08-25, `SV-CORPUS-GRAD.13e.10`(c4) — GRAMMAR-level, release `1.0.198`,
> ledger `SV-0071`, schema `26` UNCHANGED, `verilog_2005` ONLY):** the strict IEEE 1364-2005 profile
> still accepted **sixteen IEEE-1800-only OPERATORS**, and the class was invisible to every census
> that preceded it (`PGEN-SV-CORPUS-GRAD-0305`). `1.0.197`'s instrument crosses profile-filtered
> reachability against IEEE 1364-2005 **Annex B** — a normative list of *keywords* — so a construct
> spelled entirely in **punctuation** can never become a candidate. `.13e.10`(a)+(b) closed that gap
> with a derived population of **19**; (c) adjudicated all 19 two-sided, confirming **17** and
> correctly excluding `++` / `--`.
>
> **What no longer parses under `verilog_2005`:** the twelve compound assignments
> `a += 1;` `-=` `*=` `/=` `%=` `&=` `|=` `^=` `<<=` `>>=` `<<<=` `>>>=` (A.6.2 gives `=` and `<=`
> and nothing else) · `if (a ==? b)` / `if (a !=? b)` (A.8.6 defines no wildcard equality) ·
> `if (a <-> b)` (the equivalence operator is IEEE 1800 only) · `s u(.*);` (A.4.1
> `named_port_connection` has no implicit form). ⭐ The LRM evidence is re-derivable: **13 of the 16
> occur ZERO times** in the whole in-repo IEEE 1364-2005 corpus; `+=` / `-=` occur only inside the
> clause-14 specify-path polarity operators `+=>` / `-=>`, and `.*` only as a clause-13 library-map
> filename glob.
>
> ⛔⛔ **THE SEVENTEENTH IS NOT FIXED AND A CONSUMER MUST KNOW IT.** The assignment-pattern / cast
> apostrophe — `a = '{1,2};` and `T'(expr)` — **still parses under `verilog_2005`**. Gating it
> requires tagging `constant_cast`, which puts two **crossing** dialect gates on the indirect-LR
> route PGEN's eliminator rewrites; the eliminator then refuses that route and 15 left-recursive
> cycles fall back to the runtime guard, which `--lint-grammar` reports only as a *warning* while the
> parser silently loses derivations under `sv_2017` / `sv_2023`. Taking that trade to close one
> `verilog_2005` over-acceptance would be a bad bargain, so it is **deferred, not unwatched**: the
> engine repair is `ENGINE-UNIVERSAL-SERVICES.50`, and
> `accepts_invalid_v2005_punct_tick.sv` stays pinned `accepts_invalid` in the adjudication ratchet
> until it lands. **Treat `'{…}` as SystemVerilog-only when running the strict profile.**
>
> ⭐ **`sv_2017` and `sv_2023` are untouched.** Each gate is a whole-rule `@profiles` tag on a
> TERMINAL: no alternative is added, no branch index moves, and every reference site keeps its return
> annotation — so the typed AST is unchanged and a consumer of `1.0.197` needs to do nothing.

> **Current-state note (2026-08-24, `SV-CORPUS-GRAD.13e.7` — GRAMMAR-level, release `1.0.197`,
> ledger `SV-0070`, schema `26` UNCHANGED, `verilog_2005` ONLY):** the strict IEEE 1364-2005 profile
> still accepted **thirteen IEEE-1800 surfaces**, and the population was **measured over a closed
> set rather than listed** (`PGEN-SV-CORPUS-GRAD-0293`). A new instrument
> (`stimuli/sv/v2005_keyword_faithfulness_census.py`) crosses profile-filtered reachability against
> IEEE 1364-2005 Annex B — a closed, normative keyword list — and adjudicates every candidate on the
> shipped probe. It read **14 confirmed over-acceptances of 24 candidates** before the fix and
> **1** after; the survivor, `class_qualifier`, is owned by `SV-CORPUS-GRAD.13c.2m` and its remedy
> is deletion in *every* profile, not a `verilog_2005` gate.
>
> **What no longer parses under `verilog_2005`:** `chandle c;` · `enum { A } e;` · `string s;` ·
> `virtual interface I vi;` (each keyword occurs **0×** in Annex A; `string` is the A.8.8 `:1218`
> **literal**, never a type) · `const integer i = 1;` · `var integer i;` · `static integer i;` ·
> `automatic integer i;` (A.2.1.3 `:152` carries no qualifier and no lifetime) ·
> `module automatic m;` / `module static m;` (A.1.2 `:29` has no lifetime in either alternative) ·
> `extern module m;` · `default disable iff x;` · `fork join_any` / `fork join_none` (A.6.3
> `par_block:529` — **`join` is the only terminator**) · `generate rand integer x;` ·
> `a.sum with (item)` · `{ << { a } }`.
>
> ⭐ **`lifetime` itself is NOT gated, and that is the load-bearing distinction.** IEEE 1364-2005
> A.2.6 `:254` and A.2.7 `:275` give `function` and `task` a legitimate `[ automatic ]`. The gate
> therefore sits on the `( lifetime )?` **optional** at the declaration and module-header sites —
> the `SV-0069` idiom — and `control_v2005_function_automatic.sv` / `control_v2005_task_automatic.sv`
> are pinned to fail if that is ever broken.
>
> ⭐ **The streaming concatenation was found OUTSIDE the census**, because `<<` and `>>` are
> operators and no keyword-level lens can see a construct made only of punctuation. It is gated as a
> whole rule; the class it belongs to is not yet enumerated.
>
> **Consumer impact:** **none on `sv_2017` / `sv_2023`.** Each gate is a bare pass-through alias and
> the reference site keeps its own return annotation, so no branch index moves and the typed AST is
> byte-identical — `ast_shape_contract_gate` **18 passed / 0 failed**, AST-dump schema unchanged at
> `26`. `sv_cert_recognized_union_gate` moves `total` 1388 → 1401 and `union_witness` 1380 → 1393,
> both **+13 = exactly the rules added**, with `union_unknown` still **0**,
> `union_residual_rules` still `[]` and `fully_certified_via_union` still `true`. On
> `verilog_2005`, `verilog_2005_conformance_gate` is **GREEN at 303 file×profile checks / 0
> mismatches** (from 255) — the 48 new checks being 16 reject cases × 3 profiles, each
> `verilog_2005=reject` + `sv_2017=accept` + `sv_2023=accept`.
>
> ⚠️ **The `verilog_2005` cert baseline moved and every movement is adjudicated by name**:
> `1148/334/799/15` → `1147/356/769/22`, both failure counts still 0 and all three seeds agreeing.
> `total −1` is `streaming_concatenation` leaving the profile — the only pre-existing rule to move.
> `UNKNOWN +7` is a **witness-generation** artifact, not a parse change, and the parser says so:
> `--dump-rule-outcome-counts-json` reports `data_declaration_sv_2017` COMMITTED **6×** on the legal
> A.2.1.3 declaration set, `event_expression` **1×** on `always @(posedge clk)`, and `lifetime`
> **1×** on both `function automatic` and `task automatic`. Each of those rules used to be witnessed
> through an optional the generator could render; with that optional's content gated it can no
> longer produce a sample, while the parse is unchanged. Tracked as `SV-CORPUS-GRAD.13e.9`.
>
> ⚠️ Parse cost `entries` **+0.45 %**, ACCEPTED under the coded `profile_split_respelling`
> invariant, which the ratchet **re-derived** on the measurement: *0 pre-existing rule counters
> rose; 13 introduced rules gained +2,772,652 entries.* That is the standing structural cost of the
> `@profiles` named-gate idiom, which `PGEN-SV-CORPUS-GRAD-0289` ruled is bought deliberately
> because a named gate is visible to four instruments and an inline guard to none.

> **Current-state note (2026-08-24, `SV-CORPUS-GRAD.13e.4` — GRAMMAR-level, release `1.0.196`,
> ledger `SV-0069`, schema `26` UNCHANGED, `verilog_2005` ONLY):** an **integer or time declaration
> could carry a `signed` / `unsigned` under the strict IEEE 1364-2005 profile**
> (`PGEN-SV-CORPUS-GRAD-0288`). A.2.1.3 spells them `integer_declaration ::= integer
> list_of_variable_identifiers ;` and `time_declaration ::= time list_of_variable_identifiers ;`,
> A.2.8's block forms are the same shape, and neither `signing` nor `integer_atom_type` appears
> **anywhere** in that Annex — signing on an integer atom is IEEE 1800 A.2.2.1 only. Yet
> `integer unsigned u;`, `integer signed i;`, `time signed t;` and `time unsigned t;` all parsed, at
> module, block, `function` and `task` scope. PGEN carried the **IEEE 1800** shape,
> `integer_atom_type ( signing )?`, on every profile and at **two** grammar sites.
>
> ⭐ **The gate goes on the OPTIONAL, not on the alternative** — and that is the difference from
> `SV-0068`, which the same-shaped fix would have got wrong. `integer_atom_type` **must** stay
> reachable under `verilog_2005`: that admission is exactly what makes plain `integer i;` parse. So
> instead of splitting the `data_type` alternative into `_sv_only` / `_v2005` variants, one new rule
> `integer_atom_signing_sv_only := signing`, gated `@profiles: ["sv_2017", "sv_2023"]`, is referenced
> **inside** the optional at both sites. No `data_type` alternative is added, no branch index moves,
> and `signing: $2` keeps its slot — it is simply always absent under `verilog_2005`.
>
> **Consumer impact:** none on `sv_2017`/`sv_2023` — the accept set is unchanged and the typed AST is
> **byte-identical**, proven on 11 inputs (module, block, `function`, `task` and `enum_base_type`
> paths) before any code was generated. On `verilog_2005`, four spellings that previously parsed are
> now correctly rejected at every scope; everything IEEE 1364-2005 does derive — `integer i;`,
> `time t;`, `integer j = 1;`, `reg signed [7:0] r;`, `real`/`realtime`, `event` — still parses. AST
> schema is **unchanged at `26`**.
>
> ⚠️ **Residual, stated rather than implied:** `enum integer signed { A } e;` still parses under
> `verilog_2005`, because `enum_base_type` carries its own `( signing )?` and the whole `enum`
> construct is a *separate* over-acceptance on this profile — gating the signing inside it would fix
> a leaf of a branch that has to be removed whole. That, and eight more `data_type` /
> `data_declaration_sv_2017` constructs measured accepting under `verilog_2005` (`string`, `chandle`,
> `virtual interface`, `const`, `var`, `automatic`, `static`, a second packed dimension on `reg`), are
> owned by `SV-CORPUS-GRAD.13e.7`.

> **Current-state note (2026-08-24, `SV-CORPUS-GRAD.13e.5` — GRAMMAR-level, release `1.0.195`,
> ledger `SV-0068`, schema `26` UNCHANGED, `verilog_2005` ONLY):** a **task or function port item
> could start with NO DIRECTION under the strict IEEE 1364-2005 profile**
> (`PGEN-SV-CORPUS-GRAD-0287`). A.2.7 spells `task_port_item` as one of exactly three
> `tf_*_declaration`s and each of those BEGINS with its `input` / `output` / `inout` keyword, so
> `task t(a, b);` and `task t(integer a, b);` have no derivation — yet both parsed. PGEN carried
> the **IEEE 1800** shape, in which the direction is optional and inherited, on every profile.
> **Fixed by splitting `tf_port_item`/`tf_port_list` into profile-gated variants**: the
> `sv_2017`/`sv_2023` body is a byte-for-byte re-spelling of what shipped, and only the
> `verilog_2005` path narrows.
>
> ⛔ **The obvious fix is wrong, and it was MEASURED wrong before it shipped.** Requiring a
> direction on *every* item rejects `task t(input integer a, b);`, which A.2.7 **does** derive —
> `tf_input_declaration ::= input task_port_type list_of_port_identifiers` takes a *list* of names,
> so one `input` covers both. Because PGEN renders a port list the IEEE 1800 way (one item per
> comma), a continuation name arrives as its own directionless item. The shipped rule therefore
> admits a directionless item **only when it is a bare port identifier** — no data type, no `var` —
> and requires the **first** item of the list to carry a direction. Both halves are pinned as
> reproducers, the refuting input among them.
>
> **Consumer impact:** none on `sv_2017`/`sv_2023` (accept set and AST byte-identical; the two rules
> added there are a re-spelling). On `verilog_2005`, three constructs that previously parsed are now
> correctly rejected. AST schema is **unchanged at `26`** — both variants emit the identical
> five-field `tf_port_item` return, and `ast_shape_contract_gate` is 18/18 with drift 0.

> **Current-state note (2026-08-20, `SV-CORPUS-GRAD.13c.2y` — GRAMMAR-level, release `1.0.193`,
> ledger `SV-0066`, SCHEMA `25` → `26`):** an **IEEE 1800 KEYWORD WAS BOUND TO AN OPERATOR TOKEN OF
> THE SAME NAME** (`PGEN-SV-CORPUS-GRAD-0261`). `grammars/systemverilog.ebnf` defines
> `implies := trivia "->"` — a token rule named after a reserved keyword — and LRM extraction bound
> A.2.10's property operator `implies` to it instead of minting a `kw_<name>_<hash>` terminal as it
> did for every other property operator. Measured on both SV profiles: `a implies b` was
> **REJECTED** in every shape, while `a -> b`, a bare `-> b`, and `a |= b` in property position were
> **ACCEPTED** — and Annex A's `property_expr` production defines none of those three.
>
> ⭐ **The fix is a precedence-table question, not a token swap.** IEEE 1800-2017 **Table 16-3**
> (§16.12) puts `implies` in ONE right-associative group with `until` / `s_until` / `until_with` /
> `s_until_with` — a group this grammar already has, because `SV-0011` built the Table 16-3 cascade
> `prop_until > prop_iff > prop_or > prop_and > prop_primary`. So `kw_implies_470cec58` is minted
> (profile-gated SV-only) and joins `prop_until_*` as a fifth operator, and all three defect branches
> are deleted from `prop_primary_*`.
>
> ⛔ **The over-acceptance half had been named in prose and left standing for four weeks.**
> `SV-0039`'s own recorded root cause (2026-07-23) reads *"the LRM-markdown→EBNF extractor split the
> `|`-prefixed operators on `|` … leaving the mangled `implies` (`->`) / `or_assign` (`|=`) remnants
> in `prop_primary`"*. It added the correct `|->` / `|=>` branches and removed neither remnant, and
> it did not notice that the third site — A.2.10's genuine `property_expr implies property_expr` —
> was compiled against the same arrow. A defect written into a fix's prose is not tracked work.
>
> ⭐⭐ **AND THE FIX PAYS FOR ITSELF IN ENGINE COST, WHICH WAS NOT THE POINT AND IS WORTH SAYING.**
> `| property_expr implies property_expr` was the **only** left-recursive alternative in
> `prop_primary_*`, so removing it dissolved the `property_expr` indirect-left-recursion knot
> entirely: `--report-indirect-lr-plan` goes `indirect_eliminated_base_rules 3 → 2`,
> `indirect_clone_rules 24 → 12`, `indirect_guard_chains 3 → 2` at `surviving_cycle_rules=0`; the
> post-elimination census falls **1611 → 1516** — ⛔ **not a deletion**: 96 of those are build-time
> temporaries the eliminator mints and no longer needs, and the SOURCE grammar goes **1484 → 1485**
> (+1 token, zero rules removed). The parser's declared
> LR-family names fall **127 → 31**; and the generated artifact sheds **8.41 MB (−6.0 %)**.
>
> **What consumers gain and lose.** GAIN: `property_expr implies property_expr` parses on `sv_2017`
> and `sv_2023`, at Table 16-3 precedence and associativity. LOSE: `prop_primary_sv_2017`/`_sv_2023`
> no longer emit `{kind:"implies_unary", body}`, `{kind:"sequence_or_assign", lhs, rhs}` or
> `{kind:"implies_binary", lhs, rhs}` — all three carried syntax that is not SystemVerilog and can no
> longer be produced by any conforming input. A consumer walking property operators reads
> `{kind:"implies", lhs, rhs}` from `prop_until_<profile>`, beside its four Table 16-3 siblings.
>
> ⛔ **`implies` IS RESERVED, AND THAT WAS CHECKED RATHER THAN ASSUMED** — IEEE 1800-2023 **Annex B**
> (`docs/systemverilog/2023/txt/section-Annex_B-normative-keywords.txt:105`) lists it between
> `implements` and `import`, which is also exactly where the new token sits in the grammar. `SV-0065`
> was bitten by the opposite case one release earlier (`randomize` is **not** reserved, so a rule
> matching it bare captured a legal identifier), which is why this was verified first.

> **Current-state note (2026-08-20, `SV-CORPUS-GRAD.13c.2v` — GRAMMAR-level, release `1.0.192`,
> ledger `SV-0065`, SCHEMA UNCHANGED at `25`):** a **SCOPE RANDOMIZE carrying a constraint block was
> unreachable from every EXPRESSION** (`PGEN-SV-CORPUS-GRAD-0256`). IEEE 1800-2017/2023 A.8.4 spells
> `primary`'s call alternative `function_subroutine_call`, and A.8.2 expands that to
> `subroutine_call ::= tf_call | system_tf_call | method_call | [ std :: ] randomize_call`. PGEN
> renders it as `call_primary` — the postfix-chain rule the left-recursion lift authored — and none
> of its eleven alternatives is `randomize_call`. That alternative lived only on the sibling wrapper
> `function_subroutine_call`, which no expression path reaches.
>
> ⭐ **The asymmetry is what made the diagnosis certain rather than plausible.** The STATEMENT path
> (`subroutine_call`) and the CONSTANT path (`constant_function_call`) both list the qualified
> `randomize_call` explicitly, so `initial randomize(a, b) with { a < b; };` parsed as a statement
> while the identical text failed as an expression. ⛔ And `std::randomize(a, b)` *without* the
> `with` clause parsed all along — through `tf_call`, whose identifier matches `randomize` because
> IEEE 1800 Annex B does **not** reserve it (it reserves `rand`, `randc`, `randcase`,
> `randsequence`). `tf_call` has no `with` tail, so the construct became unparseable at exactly the
> moment a constraint block appeared.
>
> **What consumers gain.** `if (std::randomize(a, b) with { a < b; })`, `v = randomize(a, b) with
> { … };`, the parenthesised identifier list `randomize(a) with (a) { … }`, and the three host
> contexts real constrained-random code uses — negated inside an `if`, discarded through
> `void'(…)`, and as an immediate assertion's expression — all go REJECT → ACCEPT on `sv_2017` and
> `sv_2023`.
>
> ⛔ **`verilog_2005` is UNMOVED and that is enforced, not hoped.** `primary_sv_2017` is
> verilog_2005-admitted, so the fix is lifted into a profile-gated `scope_randomize_sv_only` rather
> than added inline — IEEE 1364-2005 has no `randomize` and no `std` package. Measured: the
> `verilog_2005` adjudication manifest is **byte-identical end to end**, 0 of 2 459 rows moved.
>
> ⛔⛔ **POSITION IS PART OF THE FIX, and the first spelling was wrong in a way no verdict could
> see.** Every part of `randomize_call` is optional, so the rule matches a **bare `randomize`**.
> Placed mid-list it captured `randomize` used as an ordinary user identifier — legal under Annex B.
> The verdict never moved; only the AST did. The alternative is therefore `primary`'s **LAST**,
> which is also the cheaper spelling: a last alternative is reached only when every other has
> failed. The guard is a pinned control (`control_randomize_as_user_identifier.sv`), and it reads
> `scope_randomize=0 / hierarchical=4` on the shipped grammar — the broken spelling read `1 / 3`.
>
> **Schema stays `25`, measured over the whole pinned manifest rather than argued:**
> `widen=6 narrow=0 shape=0` across `repro_checks=180`. The six widenings are the three defect rows
> on two profiles each; **no previously-emitted AST shape is replaced**, which is the additive
> `SV-0052` class. The new `{kind: "scope_randomize"}` is a NEW node a consumer may now encounter in
> expression position — additive vocabulary, not a migration. ⚠️ A consumer exhaustively matching
> `primary` kinds must add an arm for it.
>
> **Corpus proof:** SV lane pass **9 787 → 9 793**, and the delta is computed over the MANIFEST
> rather than the pass/fail counts because those disagree: **seven rows left the bar and only six
> crossed `fail → pass`**. The seventh, `verilator/test_regress/t/t_randomize_within_func.v`, still
> fails and reclassified `unexplained_rejects_valid → explained_svpp_macro_use` — its parse now runs
> *past* the scope randomize and dies in a macro window. All seven are `randomize`-themed, two of
> them IEEE 1800's own §18.12 clause-keyed examples, and **no unrelated row moved**. Axis-2 bar
> **282 → 275**; `unexplained_rejects_valid` 261 → 254; accepts-invalid unchanged at 21; the
> ADJUDICATED/ROUTED/NO-VERDICT/DARK split entirely unmoved.
>
> ⭐⭐ **Its cost is the first rise this repository has accepted for a reason that is NOT an
> arithmetic identity.** `entries +1,917,021` (+0.46 %), `memo_hits +1,012,779` (+0.55 %),
> `committed +0` — a ratio of 1.893 that fits none of `PARSE-COST-RATCHET`'s three total-based
> invariants. It landed under the fourth, `contained_in_introduced_subgraph`, re-derived on the
> landing run: **4 rules rose, 0 fell, 0 escaped the 475-rule sub-graph reachable from
> `scope_randomize_sv_only`**, so the whole rise is failed speculation confined to the construct
> that caused it and not one accepted derivation got dearer. ⛔ The LRM-faithful spelling —
> `function_subroutine_call` in `primary`, which is what A.8.4 literally says — was built and
> measured, and REFUSED on cost: it wraps every existing call primary in an extra AST level, a
> schema bump across the whole consumer surface to buy one construct.

> **Current-state note (2026-08-19, `SV-CORPUS-GRAD.13c.2s` — GRAMMAR-level, release `1.0.191`,
> ledger `SV-0063` + `SV-0064`, SCHEMA UNCHANGED at `25`):** a **greedy `( X )*` standing in front of
> an OPTIONAL `X`-shaped tail** made four constructs unparseable (`PGEN-SV-CORPUS-GRAD-0242`). It is
> the same starving-star mechanism as `SV-0060`, at a different severity: there the starved element
> was MANDATORY so the rule matched nothing, here it is OPTIONAL so each rule matched everything
> *except* the form the tail exists for.
>
> **`SV-0063` — a system task/function call could never reach its CLOCKING EVENT argument.**
> IEEE 1800-2017 A.8.2 writes `system_tf_identifier ( expression { , [ expression ] }
> [ , [ clocking_event ] ] )`. PGEN spelled the middle group as a bare `( comma ( expression )? )*`,
> whose body is nullable after the comma — so on `$rose(a, @(posedge clk))` the star ate the comma,
> its inner optional matched EMPTY, and the clocking-event group then needed a comma and saw `@`.
> ⇒ every `$rose`/`$past`/`$fell`/`$stable` call carrying the sampled-value clocking argument of
> §16.9.3 REJECTED.
>
> **`SV-0064` — a `let`, `property` or `sequence` instance could never take a NAMED argument after a
> positional one.** A.2.10 gives all three `[ X_actual_arg ] { , [ X_actual_arg ] }
> { , . identifier ( [ X_actual_arg ] ) }` — ordered arguments, then named ones — and all three
> carried the same starving star, so the comma introducing the first named argument was eaten.
> ⛔ **This one was MASKED, which is why it needed measuring rather than reading.** The general
> `list_of_arguments` route rescues a mixed list *while every positional argument is also a plain
> expression*, so `sq(a, .q(b))` parsed all along — through the general production rather than the
> sequence-specific one. The moment a positional argument is something only the family rule can
> parse, both routes fail: `sq(a ##1 b, .q(d))` and `pr(a ##1 b, .q(d))` REJECTED, while the
> positional-only control `sq(a ##1 b, d)` parsed on both sides.
>
> ⚠️ **CONSUMER ACTION — additive, `sv_2017` + `sv_2023`.** Three constructs flip REJECT→ACCEPT and
> nothing flips the other way. **Schema stays `25`, measured rather than argued:** every one of the
> **155** pinned reproducer checks was replayed against the pre-fix and post-fix grammars on both
> axes, and the result is `widen=2 narrow=0 shape=0` — no previously-emitted tree moves, so this is
> the additive `SV-0052`/`SV-0057` class rather than the replacing `SV-0053` one.
> ⭐ **Deliberately still REJECTED, and pinned so it stays that way:** `$rose(a, @(posedge clk), b)`
> — A.8.2 puts the clocking event LAST — and `$rose(a, @)`, a bare `@` that is neither an expression
> nor a `clocking_event`. A guard that merely let the star *skip* `@` would have admitted both.
> ⛔ **The `let` site is repaired with NO demonstrated source-level flip, stated so it is not
> inferred**: `let_actual_arg := expression`, so the general route always rescues a let argument
> list. It is fixed anyway because the defect is identical and leaving one member of a swept class
> unfixed is what `SV-0053`'s entry was written to stop — and measured, it moves no verdict and no
> tree on any pinned reproducer.

> **Current-state note (2026-08-19, `SV-CORPUS-GRAD.13c.2l` — CONTRACT-level, releases `1.0.184`
> through `1.0.190`, ledger `SV-0054`–`SV-0062`, schema `21` → `25`):** ⛔ **THIS CONTRACT WAS SEVEN
> GRAMMAR REVISIONS STALE, AND FOUR OF THEM REPLACED AN AST SHAPE A CONSUMER WAS ALREADY READING.**
> It last described `1.0.183` (2026-08-12). Between then and 2026-08-19 `grammars/systemverilog.ebnf`
> moved in **nine** commits; **seven** changed what the code generator consumes. The seven are
> numbered individually below, newest first, each with its own ledger row — deliberately, because
> collapsing them into one release would have left five of the seven owning no release number at all.
>
> ⭐ **HOW THE POPULATION WAS DERIVED, and why it is not a ninth hand-kept list.** Two instruments
> that share no parent agree 9/9 on the partition:
> 1. a **semantic digest** — sha256 of the EBNF frontend's own `raw_ast` envelope, which is what the
>    generator consumes and from which comments are absent by construction — moves for exactly seven
>    of the nine commits; and
> 2. a **behavioural sweep** — every pinned reproducer in `stimuli/sv/adjudication_repros/`, on every
>    profile it declares, replayed against all ten grammar revisions through the grammar-AST
>    interpreter — moves a verdict or a typed AST for exactly the same seven
>    (`docs/tasks/artifacts/sv_corpus_grad/contract_accept_set_ledger/`).
>
> The first is blind to behaviour and the second is blind to grammar text, so their agreement is
> evidence rather than a restatement. The two `NEUTRAL` commits (`-0234`, `-0238`) rewrote comment
> blocks only: byte-identical digests, zero witnesses moved.
>
> ⛔ **AND THE HABIT IS NOW A GATE, because the habit is what failed.** Nothing in the repository
> compared the grammar's last-modified commit with this contract's, so the drift was invisible for
> seven days — and the leaf that opened to repair it tracked the population by hand and went stale
> twice more inside the same week, routing `-0237` as *"the FIFTH change"* when it was the seventh
> commit and the fifth to move a verdict. Every revision of the SV grammar now owes a row in
> [`PGEN_SV_GRAMMAR_REVISION_REGISTER.tsv`](PGEN_SV_GRAMMAR_REVISION_REGISTER.tsv) — a `RELEASE` with
> a section here and a ledger row, or a `NEUTRAL` whose digest must equal its predecessor's — held by
> the `SV-CONTRACT-CURRENCY` doctrine (`scripts/check_sv_contract_currency.sh`), which runs in
> `scripts/check_doctrines.sh` on every commit.
>
> ⚠️ **THE HONEST BOUND, stated rather than implied.** The behavioural sweep's input population is
> the pinned manifest, not the language. It reported `-0220` as moving nothing — and `-0220` widened
> the accept set, because no row pinned its witness. That is a gap in the manifest, not a property of
> the change; the witness is pinned in this same commit, and a `0` from that instrument means *"no
> PINNED witness moved"*, never *"nothing changed"*. The digest instrument has no such bound, which
> is exactly why both are run.

> **Current-state note (2026-08-19, `SV-CORPUS-GRAD.13c.2k` + `.13c.2t` — GRAMMAR-level, release
> `1.0.190`, ledger `SV-0061` + `SV-0062`, schema `24` → `25`):** two defects landed together
> (`PGEN-SV-CORPUS-GRAD-0237`), one strictly-LESS-permissive and one strictly-MORE.
>
> **`SV-0061` — a RESERVED KEYWORD parsed in every NON-FINAL component of a hierarchical path.**
> IEEE 1800-2017 A.9.3 writes `hierarchical_identifier ::= { identifier [ [ constant_expression ] ] . }
> identifier` where every `identifier` is subject to §5.6.2, and PGEN guarded only the tail. So
> `ral.module[0].g()`, `ral.module.g()` and `module.g()` all parsed, while the method name, the final
> component and a declared name correctly refused the same token — **the leak was POSITIONAL, not a
> missing keyword table**. The same hole admitted a reserved word as an instance name, an attribute
> name and a generate label. The exclusion now lives inside `identifier` itself.
>
> ⚠️ **CONSUMER ACTION — this is the direction that BREAKS a consumer, and it is the first such
> direction this contract has had to publish for SystemVerilog.** Text that parsed under `1.0.183`
> now REJECTS: any hierarchical path with a reserved word in a non-final component, any instance
> named with a reserved word, any attribute or generate label so named. Sixteen pinned witnesses move
> ACCEPT→REJECT across the three profiles, and **six third-party corpus files that parsed on
> 2026-08-18 do not parse today** — every one of them correctly, adjudicated against the tracked LRMs
> file by file. ⭐ Two of the six are DIALECT PAIRS rather than plain rejections: `(* type=1 *)` and
> `begin:byte` REJECT under `sv_2017`/`sv_2023` and **ACCEPT under `verilog_2005`**, because `type`
> and `byte` are in IEEE 1800's Annex B and in neither case in IEEE 1364-2005's. Both profiles
> wrongly accepted before, so the fix is what makes the three-profile distinction real here.
>
> **`SV-0062` — `assignment_pattern_key` was narrower than the standard's own worked example.**
> Annex A gives `assignment_pattern_key ::= simple_type | default`, and `simple_type` cannot derive
> `string`; §10.9.2 prints `s2 = '{int:1, default:0, string:""};` as a worked example
> (`docs/systemverilog/2017/txt/section-10-assignment-statements.txt:939`). The rule now derives the
> UNION of the two normative surfaces, so the example parses through the TYPE-key route instead of
> riding `member_identifier`'s acceptance of a reserved word — which is what `SV-0061` was about to
> take away.
>
> ⚠️ **CONSUMER ACTION — a REPLACED shape, which is WHY THE SCHEMA MOVES.** An assignment-pattern key
> that names a type no longer arrives as `{kind:"member", body:"int"}`; it arrives as
> `{kind:"pattern", body:{kind:"type", body:{kind:"integer", body:{kind:"atom", body:{kind:"int"}}}}}`,
> and the `default` key moves from `{kind:"member", body:"default"}` to
> `{kind:"pattern", body:{kind:"default"}}`. A consumer reading the key as a bare string must read the
> typed node. This is the `SV-0053` class — a real previously-emitted shape is replaced — not the
> additive `SV-0052` class.
>
> ⭐ **COST, since a strictness fix is normally assumed to cost:** the shipped spelling measures
> `entries` **−1.07 %**, `memo_hits` **−2.41 %** and `committed` **−5.11 %** against the parser it
> replaces. All three binding counters FALL.

> **Current-state note (2026-08-19, `SV-CORPUS-GRAD.13c.2q` + `.13c.2r` — GRAMMAR-level, release
> `1.0.189`, ledger `SV-0059` + `SV-0060`, schema `23` → `24`):** ⛔⛔ **THE VERDICT NEVER MOVED, AND
> THAT IS THE ENTRY'S POINT.** Two defects (`PGEN-SV-CORPUS-GRAD-0233`) made every enable-gate,
> pass-enable-switch and multi-output gate instantiation parse as **a UDP instantiation whose type
> name is a reserved keyword**. The text was accepted before and is accepted now; what changed is the
> production it travels, and no pass/fail oracle in this repository could see that.
>
> **`SV-0059` — eight gate/switch keywords lost their trailing DIGIT in extraction.** All three
> tracked LRMs write `enable_gatetype ::= bufif0 | bufif1 | notif0 | notif1` and
> `pass_en_switchtype ::= tranif0 | tranif1 | rtranif1 | rtranif0`; the extraction collapsed each
> quad to its alphabetic STEM, leaving `/bufif\b/`, `/notif\b/`, `/tranif\b/`, `/rtranif\b/` —
> **unmatchable by construction**, because `\b` demands a word boundary and `0` is a word character.
> The contrast is what makes it a targeted failure rather than a general one: `n_input_gatetype`,
> `mos_switchtype`, `cmos_switchtype` and `pass_switchtype` are all complete and all worked. The
> collapse happened exactly where the alternatives differ only by a trailing digit.
>
> **`SV-0060` — a greedy `( X )*` immediately before a MANDATORY `X`-shaped tail.**
> `n_output_gate_instance` was spelled `… output_terminal ( comma output_terminal )* comma
> input_terminal rparen`; `output_terminal := net_lvalue` and `input_terminal := expression` overlap,
> and the engine does not backtrack into a committed repetition, so the star ate the mandatory tail
> whenever `net_lvalue` could match a prefix of the final terminal — i.e. for the COMMON form. Its
> sibling `n_input_gate_instance` was unaffected because its star sits at the END. Fixed with one
> lookahead, `( comma output_terminal &comma )*`.
>
> ⚠️ **CONSUMER ACTION — a REPLACED node type, on all three profiles. This is the largest
> consumer-visible change in the batch and it carries no accept-set signal at all.** `bufif0 g(o,i,e);`,
> `tranif0 g(o,i,e);`, `buf g(o,o2,i);` and `not g(o,i);` used to arrive as `udp_instantiation` with
> the terminals in an undifferentiated `instances[].inputs[]` list. They now arrive as
> `gate_instantiation`, whose `body` carries `gatetype: {kind:"bufif0"}` and whose instances carry
> **named, role-separated** `output`, `input` and `enable` fields. A consumer that matched
> `udp_instantiation` to find gates must match `gate_instantiation`; one that read positional
> terminals must read the named ones. Twelve pinned witnesses move on this axis and **zero** move a
> verdict.

> **Current-state note (2026-08-18, `SV-CORPUS-GRAD.13c.2o` — GRAMMAR-level, release `1.0.188`,
> ledger `SV-0058`, SCHEMA UNCHANGED at `23`):** PGEN could not parse IEEE 1800-2023 covergroup
> inheritance unless you typed the LRM's FOOTNOTE NUMBER into your source
> (`PGEN-SV-CORPUS-GRAD-0232`). A.2.11 of the 2023 edition prints the alternative as
> `| covergroup extends covergroup_identifier ;29`, where `29` is a footnote marker whose text reads
> *"The extends specification of covergroup is allowed only within a class."* The extraction emitted
> the marker as a token — `kw_n_29_7719a1c7 := trivia /29\b/` — and placed it in the sequence with
> **no `?`**. ⇒ under `sv_2023` the only spelling of covergroup inheritance PGEN accepted was the one
> no Annex A production derives.
>
> ⚠️ **CONSUMER ACTION — `sv_2023` only, both directions.** `covergroup extends base;` inside a class
> flips REJECT→ACCEPT; `covergroup extends base;29` flips ACCEPT→REJECT. ⭐ `sv_2017` REJECTS both,
> before and after, and that was CHECKED rather than assumed: `covergroup extends` appears in
> 1800-2023's Annex A and **nowhere** in the 1800-2017 source. `verilog_2005` has no covergroups.
> **Schema stays `23`:** the accepted form was 100 % unparseable before, so no witnessed shape moves;
> the footnote form is REMOVED rather than replaced, since that text no longer parses at all. The
> positional references renumbered `6→5` / `8→7` with every AST field name and source unchanged, and
> zero pinned witnesses moved on the shape axis.

> **Current-state note (2026-08-18, `SV-CORPUS-GRAD.13c.2j` — GRAMMAR-level, release `1.0.187`,
> ledger `SV-0057`, SCHEMA UNCHANGED at `23`):** a CLASS-SCOPED name could not be the receiver of a
> method call (`PGEN-SV-CORPUS-GRAD-0229`). IEEE 1800 A.8.4 writes the scope prefix once —
> `[ class_qualifier | package_scope ]` — and `grammars/systemverilog.ebnf` renders it **four times**.
> The canonical rendering `primary_hier_scope_prefix` gained a `class_scope` branch for a measured
> defect; the three inline copies never did, so `p::base::m.g()` REJECTED while both halves
> (`p::base::m` as a value, and `h.g()` through a handle) parsed alone.
>
> ⚠️ **CONSUMER ACTION — additive, `sv_2017` + `sv_2023`.** `p::base::m.g()` and
> `p::base::h.arr[0].g()` flip REJECT→ACCEPT; four pinned witnesses move, all in that direction, and
> **zero** move a shape. **Schema stays `23`:** the alternative added is one the parser could not
> previously reach, so no previously-emitted shape is replaced — the `SV-0052` class, stated verbatim
> in that entry's note. ⛔ **What this does NOT do, so it is not inferred:** the four renderings are
> still four. Detection of their drift is now automated
> (`docs/tasks/artifacts/sv_corpus_grad/scope_prefix_rendering_drift/`), and unifying them is an
> AST-shape change tracked as `SV-CORPUS-GRAD.13c.2n`, deliberately NOT taken before the SV release.

> **Current-state note (2026-08-18, `SV-CORPUS-GRAD.13c.2c` — GRAMMAR-level, release `1.0.186`,
> ledger `SV-0056`, schema `22` → `23`):** a negative lookahead that **could never pass** made the
> hierarchical method-call receiver collapse to its FIRST component
> (`PGEN-SV-CORPUS-GRAD-0227`). `split_hierarchical_callable_receiver`'s member loop was guarded by
> `!callable_method_call_body`, meant to stop the receiver before the method name — but IEEE 1800
> A.8.2 writes `array_manipulation_call ::= array_method_name { attribute_instance }
> [ ( list_of_arguments ) ] [ with ( expression ) ]` with the parens **optional**, so a bare member
> name already IS a `callable_method_call_body` and the guard fired on every identifier. ⇒ the loop
> ran **zero iterations for the life of the parser**, and multi-component receivers survived only
> through `method_call`'s `( dot method_call_body )*` chain, which carries no bit-select. That is
> exactly why `a.b.g()` parsed and `a.b[0].g()` did not.
>
> ⚠️ **CONSUMER ACTION — all three profiles, and the shape change reaches inputs that already
> parsed.** `ral.arr[0].g()` flips REJECT→ACCEPT, and so does `top.u1[0].t;` — an IEEE 1364-2005
> §12.4 indexed hierarchical task enable that had been rejecting in **plain Verilog**, with nothing
> pointing at it. Ten pinned witnesses move a verdict. **WHY THE SCHEMA MOVES:** for a hierarchical
> call that parsed before, the components no longer travel `method_call`'s chain. Measured on
> `top.u1.t;`, all three profiles: the receiver's `path` was `[]` with `u1` and `t` appearing inside
> `chain: [[{kind:"dot"}, {kind:"built_in", body:{kind:"array_manipulation", …}}], …]`; it is now
> `chain: []` with `path: [[{body:"top"}, {bits:[]}, {kind:"dot"}, []]]` and the callable name `t`.
> A consumer walking `chain` to recover a hierarchical name must walk `path`.

> **Current-state note (2026-08-17, `SV-CORPUS-GRAD.13c.2f` slice 4 — GRAMMAR-level, release
> `1.0.185`, ledger `SV-0055`, SCHEMA UNCHANGED at `22`):** the two `PATHPULSE$` tokens of IEEE 1800
> A.7.5 were **transliterated rather than transcribed** (`PGEN-SV-CORPUS-GRAD-0221`). The extraction
> turned the literal `$` into the word `_dollar` and, for the path-specific form, flattened two
> NONTERMINAL references into the token text as well, leaving
> `/PATHPULSE_dollar_specify_input_terminal_descriptor_dollar_specify_output_terminal_descriptor\b/`.
> Both alternatives of `pulse_control_specparam` could therefore fire only on source containing those
> literal characters, which no SystemVerilog source does. The rule NAMES are faithful
> transliterations of the LRM spelling and are kept; it is the matched TEXT that was wrong.
>
> ⚠️ **CONSUMER ACTION — both directions, all three profiles.** IEEE 1800-2023 §30.7.1's own example
> and four neighbouring spellings flip REJECT→ACCEPT (fifteen pinned witnesses, five reproducers ×
> three profiles). ⛔ **And a spelling NARROWS**: `specparam PATHPULSE_dollar = (1, 2);` was accepted
> on all three profiles and now REJECTS on all three — correctly, since `PATHPULSE_dollar` is an
> ordinary identifier and `(1, 2)` is not a `constant_mintypmax_expression`. It parsed only because
> the transliterated word was a magic token. **Schema stays `22`:** the accepted forms were 100 %
> unparseable before, and the narrowed form is REMOVED rather than replaced; zero pinned witnesses
> move on the shape axis.
> ⛔ One regex per token, deliberately, rather than composing `PATHPULSE$` + descriptor + `$` +
> descriptor from grammar elements: `trivia` skips layout before every element, so the composed form
> would also accept `PATHPULSE$ clk $ q`, and §A.9.3 makes `PATHPULSE$clk$q` ONE simple identifier.
> A contiguous regex cannot match the illegal spacing by construction.

> **Current-state note (2026-08-17, `SV-CORPUS-GRAD.13c.2d` — GRAMMAR-level, release `1.0.184`,
> ledger `SV-0054`, schema `21` → `22`):** an LRM **TYPO** was transcribed as a literal keyword
> (`PGEN-SV-CORPUS-GRAD-0220`). IEEE 1800-2017 A.2.11 writes `function_declaraton` — corrected to
> `function_declaration` in 1800-2023 — and the extraction emitted
> `kw_function_declaraton_06b7ed29 := trivia /function_declaraton\b/`. An alternative that can only
> fire on source literally containing those characters is unreachable, so §19.6.1's own example — a
> `cross` body declaring a function and using it in a `bins` — was REJECTED under `sv_2017` while
> `sv_2023` accepted it.
>
> ⭐ **The `;` had to move with it, which is why this is not a one-word edit.** `cross_body_sv_2017`
> read `lbrace ( cross_body_item semi )* rbrace`, demanding a `;` after EVERY item, so merely
> re-pointing the reference would then have required `endfunction ;`, which §19.6.1 does not write.
> The repair mirrors the `sv_2023` pair, which already carried the `;` on the
> `bins_selection_or_option` alternative rather than on the loop.
>
> ⚠️ **CONSUMER ACTION — `sv_2017` ONLY, and it is a REPLACED shape, which is WHY THE SCHEMA MOVES.**
> Each element of `cross_body_sv_2017`'s `items` array was a two-element pair
> `[ <cross_body_item envelope>, {kind:"semi"} ]`; it is now the `cross_body_item` envelope itself and
> the `;` token is gone. A consumer reading `items[i][0]` must read `items[i]`. Seven pinned witnesses
> move on this axis. **Measured per profile rather than reasoned:** under `sv_2023` an
> already-parsing cross body's AST is byte-identical across the fix and §19.6.1's example already
> parsed; under `verilog_2005` covergroups do not exist and both spellings reject on both sides. ⚠️
> The accepts-invalid half of `.13c.2a` is PRESERVED by this shape rather than re-opened:
> `cross_body_item` is non-nullable and a bare `;` matches neither alternative, so
> `option.weight = 2;;` still fails.

> **Current-state note (2026-08-12, `SV-CORPUS-GRAD.13c.2e` — GRAMMAR-level, release `1.0.183`,
> ledger `SV-0053`, schema `20` → `21`):** IEEE 1800-2017 A.2.11 writes
> `select_condition ::= binsof ( bins_expression ) [ intersect { covergroup_range_list } ]` with
> **literal** braces. Annex A spells repetition with the same two characters, and the grammar had
> taken the literal pair for the metasyntax one — `( kw_intersect covergroup_range_list* )?`. This
> restores the braces. It is the **same defect as `SV-0049`**, one clause away, which survived
> because that fix addressed the instance in front of it rather than the class; a census of every
> LRM `{ X_list }` group across both the 2017 and 2023 editions now runs as an instrument and puts
> the remaining count at **0**.
>
> **Two consumer-visible effects, in opposite directions.** (1) ACCEPTANCE WIDENS: every ranged
> intersect list — `binsof(a) intersect { 5, [1:3] }`, the standard's own §19.6.2 example — flips
> REJECT→ACCEPT, because a range is not an expression and the old concatenation route could not
> derive one. (2) ACCEPTANCE NARROWS, correctly: the brace-less `binsof(a) intersect 5`, which
> A.2.11 has no production for, now REJECTS. A third effect is a repair rather than a flip — the
> un-parenthesized `binsof(a) intersect { 1 } && binsof(b)` previously parsed with the whole
> remainder swallowed into the range list (ONE `condition` node, ZERO `and` nodes); it now yields a
> real `{kind:"and"}`, which is what `1.0.182`'s revived `&&` alternative was for.
>
> ⚠️ **CONSUMER ACTION — a REPLACED shape, so this one is not additive.** The `intersect` slot goes
> from the 2-element `[[trivia,"intersect"], [ <one concat-valued item> ]]` to the 4-element
> `[[trivia,"intersect"], {kind:"lbrace"}, [ <item>, … ], {kind:"rbrace"}]`. A consumer that indexed
> `intersect[1]` to reach the list must read `intersect[2]`; one that read a single item whose value
> was a concatenation must now read the individual `covergroup_value_range`s, where a range appears
> as `{kind:"range"}` — a kind that could not previously occur there. The `intersect`-absent case is
> unchanged (`[]`). **WHY THE SCHEMA MOVES**, unlike `SV-0052` directly below: `intersect {5, 8}`
> PARSED before, so a real previously-emitted shape is replaced — the test `SV-0049`'s schema-`20`
> note states verbatim.
>
> **Current-state note (2026-08-11, `GRAMMAR-WELLFORMED.A2.5` — ENGINE-level, release `1.0.182`,
> ledger `SV-0052`, schema STAYS `20`):** PGEN's left-recursion elimination now handles the **direct**
> shape — `A := A op B | …`, the self-reference written inline in the choice, which is how every
> standard's Annex A writes an operator production. It previously matched only the indirect wrapper
> shape, so a directly left-recursive alternative reached codegen intact and the runtime cycle guard
> **rejected** it at the seed position. The rule still parsed its operands, so the construct looked
> supported until something had to follow the first operand.
>
> **What flips REJECT→ACCEPT (four alternatives, ZERO grammar bytes changed):** `select_expression`'s
> `&&`, `||` and `with ( … ) [ matches … ]` (IEEE 1800-2017 A.2.11) and `block_event_expression`'s
> `or` (A.6.11, reached from `coverage_event`'s `@@( … )` form). Each now returns the AST its
> annotation declares, left-nested: `{kind:"and"|"or", lhs, rhs}`, `{kind:"with_matches", lhs,
> expression, matches}`.
>
> ⚠️ **CONSUMER ACTION — additive, but not zero.** A consumer exhaustively matching
> `select_expression` kinds must add `and` / `or` / `with_matches` arms, and one matching
> `block_event_expression` must add `or`. **WHY THE SCHEMA DOES NOT MOVE:** all four alternatives were
> **100 % unparseable** before, so no previously-emitted shape is replaced — this is the `SV-0044`
> class, and the rule is stated verbatim in the schema-`19` note for `SV-0048`: *"an additive
> vocabulary reachability change, not a schema change."* Verified, not argued: `ast_shape_contract_gate`
> green with its 18 locked samples unchanged, and the two `select_expression` inputs that parsed before
> still parse to the same `{kind:"condition"}` root.
>
> ⛔ **What this does NOT fix, stated so it is not inferred:** `binsof(a) intersect { 1 } && binsof(b)`
> *without* parentheses still parses as a single `condition` with the `&&` inert — not because the
> continuation is dead, but because `intersect`'s LITERAL braces are still transcribed as EBNF
> repetition, so the seed swallows the remainder (`SV-CORPUS-GRAD.13c.2a.1`, open). Parenthesize the
> left operand to exercise the continuation. Also unchanged: precedence. LR elimination fixes the
> *recursion*; a language's operator precedence is language-specific and stays in the grammar.

> **Current-state note (2026-07-06, MEMO-STORE-SOUNDNESS.2 — engine-level, NO release bump):** the
> shared parse engine's packrat memo is now **taint-gated with store-write-epoch validation** — a
> memoized outcome whose rule body consulted the semantic store is replayable only while the store
> is unchanged, closing a platform-level staleness class (a store-dependent cached failure/success
> could previously replay after the store changed). Every locked SV surface is verified
> byte-identical through this change (certificate-coverage all profiles at seeds 0/7/42, the
> verilog_2005 conformance matrix 240/0, the AST shape contract 18/18, the external corpus 14/14
> verdicts), so the contract and release version are unchanged. Operational note for big-file
> consumers: corpus wall-clock measured ≈ +15% on the uvm_pkg case (soundness-driven cache
> validation); tracked optimization headroom is `docs/tasks/MEMO-STORE-SOUNDNESS.md` leaf `.3`.


> **Current-state note (2026-08-08, `SV-CORPUS-GRAD.3.12` — engine-level, release `1.0.177`, ledger
> `SV-0047`):** the same packrat memo was also **recursion-blind**. A parse outcome produced under a
> cycle-guard rejection depends on which rules were on the live PARSE STACK, which the memo key
> `(rule_id, position)` does not carry — so such a FAILURE could be replayed from a stack where the
> guard would never have fired, refusing legal input. Measured on SV: `$clog2(P)'(P)` (an IEEE
> 1800-2017 A.8.4-legal size cast) rejected, while the SAME parser parsed the SAME bytes under
> `--entry-rule cast`. A recursion-tainted failure is no longer cached. ⚠️ Unlike the store fix above
> this one **does change verdicts**, in the permissive direction only — corpus pass 9 694 → 9 712
> with **0 pass→fail** over 16 336 files and 0 over the 2 459-file `verilog_2005` lane — hence the
> release bump. Schema stays `19` (no emitted kind added, renamed or removed). ⭐ A cached SUCCESS is
> deliberately still replayed: a guard limits the SEARCH, not the LANGUAGE, and on cyclic rules that
> replay is what makes indirect left recursion parse at all (`docs/tasks/SV-CORPUS-GRAD.md` `.11c`).

> **Current-state note (2026-08-09, `SV-CORPUS-GRAD.3.14b` — GRAMMAR-level, release `1.0.178`,
> ledger `SV-0048`):** a compiler directive was tolerated **only at file top level**. The
> `compiler_directive` rule was an alternative of `source_text_item` and of nothing else, so the
> same directive one line lower — inside a module or class body — REJECTED. Legally-placed
> directives now parse at module-body and class-body item positions, via a **name WHITELIST**
> derived from IEEE 1800-2017 clause 22 and IEEE 1364-2005 clause 19 read in full: tolerated are
> `` `celldefine `` `` `endcelldefine `` `` `undef `` `` `timescale `` `` `pragma `` `` `line ``
> (plus `` `undefineall ``, SV profiles only — it is absent from 1364-2005). ⛔ **Deliberately
> still REJECTED in-scope, and this is the contract-relevant half:** the six names whose clause
> forbids them inside a design element (`` `resetall `` §22.3/§19.6, `` `default_nettype ``
> §22.8/§19.2, `` `unconnected_drive ``/`` `nounconnected_drive `` §22.9/§19.9,
> `` `begin_keywords ``/`` `end_keywords `` §22.14/§19.11); the seven text-hiding directives
> (`` `include ``, `` `define ``, `` `ifdef `` `` `ifndef `` `` `else `` `` `elsif `` `` `endif ``),
> whose files are preprocessing-dependent rather than honest parser input; and unexpanded MACRO
> uses including `` `__FILE__ ``/`` `__LINE__ `` (§22.13 text macros, not directives) — so an
> unexpanded `` `MY_MACRO(x) `` at an item position still correctly fails rather than being
> silently swallowed. Strictly more permissive on legal input: corpus pass 9 712 → 9 720 with
> **0 pass→fail** over 16 336 files, and 2 180 → 2 181 with **0 pass→fail** over the 2 459-file
> `verilog_2005` lane; accepts-invalid unchanged in both lanes (21 / 14). ⚠️ **What a consumer
> sees:** `non_port_module_item` and `class_item` can now yield
> `{"kind": "compiler_directive", "body": "<the directive line>"}` — a kind that previously
> appeared only under `source_text`. A consumer exhaustively matching module-item or class-item
> kinds must add that arm. Schema stays `19`: the node is byte-identical to the existing
> top-level one, so no emitted kind is added, renamed or removed. Honest bound: tolerance is at
> ITEM positions (between items), not between two tokens of one statement, and the hosts are
> module and class bodies only (`docs/tasks/SV-CORPUS-GRAD.md` `.3.14d`).

> **Current-state note (2026-08-09, `SV-CORPUS-GRAD.3.19` — GRAMMAR-level, release `1.0.180`,
> ledger `SV-0050`, SCHEMA UNCHANGED at `20`):** the config `use` clause could not consume a `#`,
> so **no parameter override written the way IEEE 1800's own clause-33 examples write it would
> parse**. ⭐ The standard contradicts itself here, identically in 2017 and 2023: Annex A /
> Syntax 33-4 gives `use_clause` three alternatives with **no `#` anywhere**, while clause 33.4.3
> prints `instance top use #(.WIDTH(32));` **seven times per revision**. PGEN had transcribed
> Annex A correctly and completely — this is NOT the dropped-delimiter class of `SV-0044`/`SV-0049`,
> where the LRM was right and PGEN mis-read it. The rule now carries the **union of the two
> normative surfaces**: `use [lib.]cell [:config]`, `use .P(1), …`, `use [lib.]cell .P(1), …` **and**
> `use #( .P(1), … )` including the empty `use #()`. ⛔ **Deliberately still REJECTED:** positional
> overrides such as `use #(32)` — 33.4.3 states *"Configurations may not use positional parameter
> notation to override parameters"*, so the new arm takes `named_parameter_assignment`, never
> `list_of_parameter_assignments`. ⚠️ **What a consumer sees:** `config_rule_statement` rows of
> `kind: "inst_use"` / `"cell_use"` can now carry a `body` containing `{kind:"hash"}`,
> `{kind:"lparen"}`, the named assignments and `{kind:"rparen"}`. **Schema stays `20`** and that is
> MEASURED, not assumed — the construct was 100 % unparseable before, so no witnessed shape can
> move, and the ASTs of all 10 already-parsing repro cases are byte-identical across the fix.
> Strictly more permissive on legal input: corpus pass 9 726 → 9 734 with **0 pass→fail** over
> 16 336 files, and the 2 459-file `verilog_2005` lane is **byte-inert**; accepts-invalid unchanged
> in both lanes (21 / 14). ⚠️ **Known residual, tracked and fixed next
> (`docs/tasks/SV-CORPUS-GRAD.md` `.3.20`):** `use_clause` carries no `@profiles` gate, so
> `verilog_2005` accepts this spelling and the four pre-existing named-override alternatives, none
> of which exist in IEEE 1364-2005 — pre-existing for four forms, extended to five here, with a
> measured blast radius of **0 corpus rows**.

> **Current-state note (2026-08-09, `SV-CORPUS-GRAD.3.20` — GRAMMAR-level, release `1.0.181`,
> ledger `SV-0051`, SCHEMA UNCHANGED at `20`):** the residual flagged one paragraph above is now
> **CLOSED**. Parameter override in a config `use` clause is a SystemVerilog addition — IEEE
> 1364-2005 declares exactly **one** `use_clause` production, `use [library_identifier.]
> cell_identifier[:config]` (Annex A A.1.5, restated in 13.3), and the string `use #(` occurs
> **zero** times in the 2005 text — but `use_clause` carried no `@profiles` gate, so all five
> override spellings were accepted under `--profile verilog_2005`. The four override alternatives
> now live in a `@profiles: ["sv_2017", "sv_2023"]` sibling `use_clause_param_override_sv_only`
> that `use_clause` references first (the `GRAMMAR-WELLFORMED.G.4.8` specific-before-general
> ordering is preserved by the reference position, not relaxed).
> ⚠️ **What a consumer sees — SV consumers see NOTHING.** This is a `verilog_2005`-only
> tightening. Under `sv_2017` / `sv_2023` the emitted AST is **byte-identical**: a bare
> rule-reference alternative carrying no return annotation adds no wrapper node, measured as
> `cmp`-identical ASTs on **15/15** accepting repro cases and confirmed independently by the
> `use_clause_hash_named_override` shape lock staying `aligned` with an unchanged observed
> `content_kind`. Under `verilog_2005`, nine previously-accepted spellings (`use .W(8)`,
> `use .W(8), .D(16)`, `use adder .W(8)`, `use rtlLib.adder .W(8)` and the five `use #( … )` forms)
> now REJECT; the three legal 1364-2005 spellings and every neighbouring surface are unmoved.
> **Schema stays `20`**, measured on the byte-identical ASTs above rather than argued.
> **Corpus proof:** both lanes **byte-inert** — 16 336 sv_2017 files and 2 459 v2005 files, pass
> 9 734 / 2 181 unchanged, **zero** per-file transitions of any kind, and both adjudication
> manifests `cmp`-clean, i.e. **0 rows changed adjudication class anywhere**. Predicted before the
> run and then confirmed: **0 of the 2 459 v2005-lane files contain the token `config` at all**, and
> `use_clause` is reachable only through `config_rule_statement` inside a `config_declaration`.
> ⚠️ `defined_rule_count` **1477 → 1478**, so `sv_cert_recognized_union_gate` was re-baselined
> **in the same commit** (total/witness +1, all-positive: proof, canonical UNKNOWN 11 and union
> UNKNOWN 0 unchanged), while `verilog_2005_conformance_gate` needed **no** re-baseline — its cert
> census is 1122/329/779/14, unchanged, because the new rule is gated out of that profile.

## Contract Identity
- Contract version:
  - `1.0.253`
- Parser release version:
  - `1.0.253`
  - history: `1.0.252`; `1.0.251`; `1.0.250`; `1.0.249`; `1.0.248`; `1.0.247`; `1.0.246`; `1.0.245`; `1.0.244`; `1.0.243`; `1.0.242`; `1.0.241`; `1.0.240`; `1.0.239`; `1.0.238`; `1.0.237`; `1.0.236`; `1.0.235`; `1.0.234`; `1.0.233`; `1.0.232`; `1.0.231`; `1.0.230`; `1.0.229`; `1.0.228`; `1.0.227`; `1.0.226`; `1.0.225`; `1.0.224`; `1.0.223`; `1.0.222`; `1.0.221`; `1.0.220`; `1.0.219`; `1.0.218`; `1.0.217`; `1.0.216`; `1.0.215`; `1.0.214`; `1.0.213`; `1.0.212`; `1.0.211`; `1.0.210`; `1.0.209`; `1.0.208`; `1.0.207`; `1.0.206`; `1.0.205`; `1.0.204`; `1.0.203`; `1.0.202`; `1.0.201`; `1.0.200`; `1.0.199`; `1.0.198`; `1.0.197`; `1.0.196`; `1.0.195`; `1.0.194`; `1.0.193`; `1.0.192`; `1.0.184`-`1.0.190` were assigned together by `SV-CORPUS-GRAD.13c.2l` (2026-08-19) after this
    document was found seven grammar revisions stale. They are numbered INDIVIDUALLY, newest-first in the
    Current-state notes above, because collapsing them would have left five of the seven owning no release
    number at all and their ledger rows pointing at a release that never described them.
- SV grammar identity (what this contract describes, and what the `SV-CONTRACT-CURRENCY` doctrine checks):
  - `e880d4d69928f48d2241c644b461285015741317a849009b39c92cd4da9779d6` (SV grammar semantic digest — sha256 of the
    EBNF frontend's own `raw_ast` envelope for `grammars/systemverilog.ebnf`, i.e. what the code generator
    consumes, from which comments are absent by construction. Every revision of that grammar carries a row in
    [`PGEN_SV_GRAMMAR_REVISION_REGISTER.tsv`](PGEN_SV_GRAMMAR_REVISION_REGISTER.tsv); `scripts/check_sv_contract_currency.sh`
    re-derives this digest from the producer and refuses a tree where the grammar has moved and this document has not.)
  - ⚠️ **The digest moved without a release, and that is a CHECKED claim, not a footnote.** The
    previous digest `470d4998…` shipped as `1.0.197`; the current one is the `SV-CORPUS-GRAD.13e.9`
    `@sample` repair (`PGEN-SV-CORPUS-GRAD-0296`, 2026-08-25), registered as the register's first
    **`GENERATOR-ONLY`** row. `@sample` steers STIMULI GENERATION and nothing else, so the accept
    set and the AST cannot move — and that is not argued, it is measured: the parser generated from
    this grammar is **byte-identical** to the one generated from `470d4998…` and to the shipped
    `generated/systemverilog_parser.rs`, all three `sha256 8bc4746aeb0363e92b13a59d6a3e473f6992a73b`
    `a90ac0d75435303b6ecb23cc`. The doctrine's tier E re-hashes that parser on every run, so this
    paragraph cannot quietly become false. **No parser release, no ledger row, schema unchanged at
    `26`** — a consumer of `1.0.197` needs to do nothing.
- Embedding API contract baseline:
  - `1.3.2` (backward-compatible fix completing `1.3.1`, `ENGINE-UNIVERSAL-SERVICES.43` 2026-08-21: crossing the 4096-frame recursion ceiling now returns its clean `E_PARSE_FAILURE` in **bounded time**. `1.3.1` guaranteed the host's STACK and left the parse unbounded; measured on the shipped SV parser, 315 nested parens were accepted in 0.16 s and 320 returned no result in 30 s, because the ceiling — the one guard verdict that names no blocking frame — tainted the packrat failure cache globally and one trip disabled failure memoisation for the rest of the parse. The ceiling now files its failures under a DEPTH STAMP (monotone: a failure at depth `D` holds at every depth `>= D`), which restores memoisation without touching the frame-scoped taint `SV-CORPUS-GRAD.3.12` added. Engine-level and grammar-agnostic — VHDL showed the identical cliff in a byte-identical parser. See the Stack-Robustness Contract in `rust/docs/EMBEDDING_API_CONTRACT.md`. Parser release `1.0.193` → `1.0.194` because the generated artifact IS regenerated; AST-dump schema unchanged at `26` and no accept-set change.)
  - history: `1.3.1` (backward-compatible stack-robustness fix, `SV-CORPUS-GRAD.8c.3` 2026-07-22: every SV/VHDL embedding parse runs on a dedicated 256 MiB-stack thread, so over-deep recursion returns a clean `E_PARSE_FAILURE` diagnostic — the engine's 4096-frame recursion ceiling — instead of aborting the HOST process with a stack-overflow SIGABRT; measured pre-fix, a ~400-deep parenthesized expression (≈4 KB of text) killed a release embedder at the default 8 MB main stack. See the Stack-Robustness Contract in `rust/docs/EMBEDDING_API_CONTRACT.md`. No parser release/schema bump — the generated parser artifact is unchanged.)
  - history: `1.3.0` (backward-compatible addition of the `verilog_2005` profile; see `rust/docs/EMBEDDING_API_CONTRACT.md` — the previously stated `1.2.0` here was a stale lockstep gap closed by `VERILOG-2005-PROFILE.4.3`)
- SystemVerilog AST-dump schema version:
  - `34` (UNCHANGED by **`1.0.246`** (`ENGINE-UNIVERSAL-SERVICES.111` + `SV-CORPUS-GRAD.33`, `PGEN-ENGINE-UNIVERSAL-SERVICES-0163`, 2026-09-24, ledger `SV-0131`): the wildcard escapes render the known arms' existing shapes at new positions and the deleted qualifier arm's `qualifier: null` keeps its node; UNCHANGED by **`1.0.245`** (`SV-CORPUS-GRAD.32`, `PGEN-SV-CORPUS-GRAD-0414`, 2026-09-23, ledger `SV-0130`): `block_data_type`'s permissive arm `provisional_unscoped_block_class_type` deleted — it passed its child through unannotated, the same `{head, params, scope_chain}` object its typedef-excluded sibling renders, and the corpus liveness census plus the identity sweeps read no accepted tree passing through it; no kind, field or shape leaves) and BUMPED by **`1.0.244`** (`SV-CORPUS-GRAD.13c.2m`, `PGEN-SV-CORPUS-GRAD-0413`, 2026-09-23, ledger `SV-0129`): three kinds that only the LRM's own production text could produce leave the schema — `"null_class"` (`method_call_receiver_sv_2017` / `_sv_2023`), `"null_class_assign"` (`primary_sv_2017` / `_sv_2023`), `"class_qualifier"` (`primary_hier_scope_prefix`) — the extractor's weld of Annex A's nonterminal `class_qualifier` and the footnote superscripts `43` / `48`; a branch removed is a bump by the book's rule although no SystemVerilog text ever produced them, so every previously accepted tree is byte-identical; `"local_qualifier"` gains a second producer, `ps_type_identifier_scope_sv_2017` / `_sv_2023` rendering the LRM's `local ::` before a type name; `primary_hier_scope_prefix` is SV-gated)
  - `33` (UNCHANGED by **`1.0.243`** (`SV-CORPUS-GRAD.31`, `PGEN-SV-CORPUS-GRAD-0412`, 2026-09-23): a passthrough twin and one call-prefix arm rendering the `{body: {head, params, scope_chain}}` / `{head, params, scope_chain}` objects `class_scope` and `class_scoped_call_prefix` already render (measured byte-identical) — no node, kind or field new to the schema; every previously accepted tree byte-identical, a class scope whose parameterized class is unresolved under an open wildcard import gains that node where it had none) and by **`1.0.242`** (`SV-CORPUS-GRAD.30`, `PGEN-SV-CORPUS-GRAD-0411`, 2026-09-23): one sv-only `block_data_type` arm rendering the `{head, params, scope_chain}` node `known_unscoped_block_class_type` already renders (measured byte-identical) — no node, kind or field new to the schema; every previously accepted tree byte-identical, a block-local parameterized class under an open wildcard import gains that node where it had none, and a declared typedef with a parameter list under an open import loses the `class_type` tree the `.28` escape had wrongly given it) and by **`1.0.241`** (`SV-CORPUS-GRAD.29`, `PGEN-SV-CORPUS-GRAD-0410`, 2026-09-22): one sv-only prefix arm rendering the `{kind: "instance", handle}` node `instance_or_class_scope` / `variable_lvalue_scope` already render — no node, kind or field new to the schema; every previously accepted tree byte-identical, a `this.member[select]` rvalue gains a `"hierarchical"` primary tree under `sv_2017` where it had none) and by **`1.0.240`** (`SV-CORPUS-GRAD.28`, `PGEN-SV-CORPUS-GRAD-0409`, 2026-09-22): one wildcard escape arm rendering exactly the known-class arm's shape — no node, kind or field touched; every previously accepted tree byte-identical) and by **`1.0.239`** (`SV-CORPUS-GRAD.27`, `PGEN-SV-CORPUS-GRAD-0408`, 2026-09-17): two wildcard escapes rendering exactly the known rules' shapes — no node, kind or field touched; every previously accepted tree byte-identical) and by **`1.0.238`** (`SV-CORPUS-GRAD.26`, `PGEN-SV-CORPUS-GRAD-0407`, 2026-09-17): `primary` gains `{kind: "type_comparison", body: {left, op, right}}` on the SV profiles — an ADDITIVE kind, no existing node, kind or field touched; every previously accepted tree byte-identical) and by **`1.0.237`** (`SV-CORPUS-GRAD.3.69` + `ENGINE-UNIVERSAL-SERVICES.108`, `PGEN-SV-CORPUS-GRAD-0405`, 2026-09-17): a store-gated default helper rendering what the inline optional rendered, a zero-width reset — no node, kind or field touched; every accepted tree byte-identical) and by **`1.0.236`** (`SV-CORPUS-GRAD.3.68`, `PGEN-SV-CORPUS-GRAD-0404`, 2026-09-17): a `verilog_2005` net-declaration twin with the sv twin's six fields — no node, kind or field touched; every accepted tree byte-identical) and by **`1.0.235`** (`SV-CORPUS-GRAD.3.67`, `PGEN-SV-CORPUS-GRAD-0403`, 2026-09-16): a terminal re-spelled in its own slot, the `number` carrier untouched — no node, kind or field touched; every accepted tree byte-identical on every profile) and by **`1.0.234`** (`SV-CORPUS-GRAD.3.66`, `PGEN-SV-CORPUS-GRAD-0402`, 2026-09-16): profile twins with the same template fields at six declaration sites — no node, kind or field touched; every accepted tree byte-identical on every profile); UNCHANGED by **`1.0.233`** (`SV-CORPUS-GRAD.3.65`, `PGEN-SV-CORPUS-GRAD-0401`, 2026-09-16): profile twins with the same template fields — no node, kind or field touched; every accepted tree byte-identical on every profile); UNCHANGED by **`1.0.232`** (`SV-CORPUS-GRAD.3.64`, `PGEN-SV-CORPUS-GRAD-0400`, 2026-09-16): deletions only — four unreachable terminals removed, no node, kind or field touched; every accepted tree byte-identical); UNCHANGED by **`1.0.231`** (`SV-CORPUS-GRAD.3.63`, `PGEN-SV-CORPUS-GRAD-0399`, 2026-09-15): additive — `delayed_data` / `delayed_reference` emit their published `with_expr` kind for the first time, `edge_control_specifier.descriptors` is the list of descriptor nodes, `sv_dollar_width_timing_check` gains the `no_threshold` wrapper on `verilog_2005` only; no kind renamed, no field moved); **`1.0.230`** (`SV-CORPUS-GRAD.3.61`, `PGEN-SV-CORPUS-GRAD-0393`, 2026-09-10), ledger `SV-0104` / `SV-0105` / `SV-0106`: A LOOKAHEAD IS A CHILD — `type_assignment_sv_2017` / `_sv_2023` and the `_with_default` twins read `{name: $2, value: $3}` behind their leading `!kw_type` guard (`1.0.79`–`1.0.229` bound the guard as `name` (`[]`) and the identifier as `value`, so every `parameter type T = int` carried NO default type); `list_of_type_assignments` / `_with_default` read `[$1, $2::3*]` (`1.0.210`–`1.0.229` sliced the guard's comma in place of every assignment after the first); `let_` / `property_` / `sequence_list_of_arguments` `mixed` read `ordered_tail: [$2::3*]` (`1.0.191`–`1.0.229` took the `!dot` guard, every further positional `[]`); `mixed_string_parameter_port_list` reads `type_tail: [$3::3*]`, `decl_tail: [$7::2*]` (raw guard-carrying envelopes before). **Migration:** a `type_assignment` reader takes the identifier from `name` and the optional `[{kind: "assign"}, <data_type>]` envelope from `value`; a type-list reader sees every assignment; an instance's `ordered_tail` carries its further positionals. Measured two-sided: the movers are 196 accepted files on `sv_2017` and 204 on `sv_2023`, every one a `.3.61` shape and nothing else (`rust/target/s360/classify_movers.py`, every mover's two trees diffed node by node: 193 / 201 files carry a `type_assignment` whose `name` and `value` are now bound — 822 / 838 nodes — 9 / 9 carry a type list whose second and later assignments replace a `comma` node — 26 / 26 elements — and 3 / 3 carry a property or sequence instance's `ordered_tail` — 19 / 19 arguments; 0 unclassified), the other 9 651 / 9 668 accepted trees byte-identical, `verilog_2005` untouched (2 313 identical); the corpus AST identity sweeps read `ast_moved=196 verdict_moved=0` on `sv_2017`, `204 / 0` on `sv_2023` and `0 / 0` on `verilog_2005` — every mover a `.3.61` shape (type assignments, type lists, ordered tails), the other 9 651 / 9 668 / 2 313 accepted trees byte-identical. The same release's `.3.60` widening (`SV-0103`) adds five rules and touches no accepted tree's shape.)
  - `32` (UNCHANGED by **`1.0.229`** — `SV-CORPUS-GRAD.3.59`, `PGEN-SV-CORPUS-GRAD-0392`, 2026-09-10, ledger `SV-0102`: `hierarchical_index_operand` gains `{kind: "dollar"}` (a `$` inside a chain's select) and the two `sv_2023` range rules gain `{kind: "relative_tolerance"}` (`[A +%- B]`) — ADDITIVE kinds; no node the earlier releases emitted changes shape)
  - `32` (UNCHANGED by **`1.0.228`** — `SV-CORPUS-GRAD.3.58`, `PGEN-SV-CORPUS-GRAD-0391`, 2026-09-09, ledger `SV-0101`: `event_control_sv_2017` gains the three-token `@ ( * )` alternative as `{kind: "wildcard_paren"}` — an ADDITIVE kind the `sv_2023` node already emitted; no node the earlier releases emitted changes shape)
  - `32` (UNCHANGED by **`1.0.227`** — `SV-CORPUS-GRAD.3.56`, `PGEN-SV-CORPUS-GRAD-0386`, 2026-09-09, ledger `SV-0100`: an opaque directive line continues over backslash-newline (§22.5.1) and a based literal's tokens may be separated by any white space or comment (§5.7.1 / §5.3), and a time literal’s magnitude is `unsigned_number`/`real_number` per A.8.7 (a based-literal over-acceptance the widening exposed, dropped; 0 corpus rows, value node byte-identical) — a `{kind: "opaque", text}` directive node now carries a multi-line macro's whole text (the three accepted corpus files carrying the shape are the only AST movers, pre-registered: friscv `rtl/friscv_h.sv`, sv-tests `tests/generic/preproc/preproc_test_8.sv`, uvm-core `src/deprecated/macros/uvm_sequence_defines.svh`), a based literal's `{kind: "integral", body}` keeps the source text verbatim; no kind added, no template changed; the corpus AST identity sweep reads `ast_moved=3` on each SV lane (those files) and 0 on `verilog_2005`; the verdict movers: 76 corpus files `fail → pass` on each SV lane (macro-heavy headers and number-whitespace tests across opentitan, black-parrot, friscv, sv2v, Surelog, verilator) and 3 on `verilog_2005` (sv2v `default_nettype.v`, `pattern_revised.v`, `typename_deep.v`), 0 the other way; the three AST movers are the already-accepted define files friscv `friscv_h.sv`, sv-tests `preproc_test_8.sv`, uvm-core `uvm_sequence_defines.svh`; beside it twelve bar rows re-keyed must_reject by production)
  - `32` (UNCHANGED by **`1.0.226`** — `SV-CORPUS-GRAD.3.55`, `PGEN-SV-CORPUS-GRAD-0385`, 2026-09-08, ledger `SV-0099`: the time literal's unit is glued to its number (IEEE 1800-2017 Annex A footnote 44) through the new rule-level `@glued` directive; the node is the `{value: {kind, body}, unit: {kind}}` time literal the grammar has always carried — no kind added, no template changed; the corpus AST identity sweep reads `ast_moved=0` on every lane; the verdict movers are the spaced shapes the literal used to swallow (ispras `16.08_04.sv` and `16.09.11_01.sv`, Surelog `FSMBsp13/top.v`, iverilog `pr2785294.v` — `fail → pass` on `sv_2017` and `sv_2023`, 0 the other way; `verdict_moved=0` on `verilog_2005`); beside it six bar rows re-keyed must_reject by production)
  - `32` (UNCHANGED by **`1.0.225`** — `SV-CORPUS-GRAD.3.54`, `PGEN-SV-CORPUS-GRAD-0384`, 2026-09-08, ledger `SV-0098`: a DECLARED package is a scope even while a typedef of the same name is visible — `non_typedef_package_scope`'s typedef exclusion made branch-local and a second alternative admitting the scope by its `package_name` fact under `ordered`; the node is the `{kind: "package", name}` scope the rule has always carried, in the same slots — no kind added, no template changed; the corpus AST identity sweep reads `ast_moved=0` and `verdict_moved=0` on every lane; the one lane file carrying the shape advances to a macro use and stays a non-accept; beside it six bar rows re-keyed must_reject by production)
  - `32` (UNCHANGED by **`1.0.224`** — `SV-CORPUS-GRAD.3.53`, `PGEN-SV-CORPUS-GRAD-0383`, 2026-09-08, ledger `SV-0097`: the parenthesised operator assignment is admitted as a chain OPERAND on the SV profiles and gated out of `verilog_2005`; the top-level form keeps its `{kind: \"paren_op_assign\", body}` node at the base of the expression and every chain step's triple carries the node it did — no template changed, no kind added, the kind only enters slots it never occupied; the corpus AST identity sweep reads `ast_moved=0` on every lane; the verdict movers exactly the pre-registered rows: ispras `11.04.02_01.sv`, sv-tests `11.3.6--two_assign_in_expr.sv` and `-sim`, `fail → pass` on `sv_2017` and `sv_2023`, `verdict_moved=0` on `verilog_2005`)
  - `32` (UNCHANGED by **`1.0.223`** — `SV-CORPUS-GRAD.3.49`, `PGEN-SV-CORPUS-GRAD-0380`, 2026-09-08, ledger `SV-0096`: the unscoped checker name's `has_fact(checker_name) post` store gate is deleted, so a checker instantiated before its declaration or declared in another file parses; the node is the `{kind: "checker_instantiation", body: {name, instance, connections}}` branch documented since schema 26 — no kind added, no template changed; the corpus AST identity sweep reads `ast_moved=0` on every lane; the verdict movers exactly the pre-registered rows: Surelog `tests/CheckerInst/dut.sv`, ispras `ieee-1800-2012/14/14.14_05.sv`, iverilog `ivltests/program5b.v` — `fail → pass` on `sv_2017` and `sv_2023`, 3 / 3 / 0 movers, 0 `pass → fail` anywhere)
  - `32` (UNCHANGED by **`1.0.222`** — `SV-CORPUS-GRAD.3.50`, `PGEN-SV-CORPUS-GRAD-0379`, 2026-09-08, ledger `SV-0095`: the IEEE 1800 unpacked SIZE form `x [N]` is gated to the SV profiles by the `_sv_only` helper `unpacked_dimension_size_sv_only` behind a pass-through dispatcher alternative (`-> $1`), so the size form keeps its `{kind: "expression", body}` node and the range form its `{kind: "range", body}` node on `sv_2017` / `sv_2023` — no template changed, no kind added; the corpus AST identity sweep reads `ast_moved=0` on every lane; `verilog_2005` narrows by the size form alone, 0 corpus rows moved)
  - `32` (UNCHANGED by **`1.0.221`** — `SV-CORPUS-GRAD.3.47`, `PGEN-SV-CORPUS-GRAD-0377`, 2026-09-08, ledger `SV-0094`: the string literal is split by edition; every double-quoted twin carries the shipped `{kind: "double_quoted", body}` template with the body the source text verbatim, so every string the parser accepted before is byte-identical (the corpus AST identity sweep `ast_moved=0` on every lane); the `sv_2023` twin's `{kind: "triple_quoted", body}` appears only on IEEE 1800-2023 §5.9 text no earlier release accepted — an addition, not a replacement.)
  - `32` (**`1.0.220`** (`SV-CORPUS-GRAD.3.46`, `PGEN-SV-CORPUS-GRAD-0376`, 2026-09-07), ledger `SV-0093`: `block_data_type` alternative 1 — the block-scoped integer VECTOR, the `body` of `{kind: "block_data", body}` — carries `data_type`'s own `{kind: "integer_vector", base, signing, dims}` where `1.0.96`–`1.0.219` emitted the same fields without `kind` (SV-Slice-96), the one integer-vector shape in the grammar without one after `1.0.219` typed its four multi-element siblings. A REPLACEMENT of the node's key set: a consumer that recognised the vector form by the absence of `kind` reads `kind == "integer_vector"`; a `data_type` reader reads the body unchanged; every other alternative is byte-identical. Pre-registered by the `.3.45` census (712 nodes in 259 accepting files on `sv_2017` and `sv_2023`, 130 in 73 on the 1364-2005 lane) and measured two-sided by the corpus AST identity sweep on the final bytes — every predicted file moved, no other accepted tree moved a byte, no verdict moved. The same release narrows the accept set by one LRM-illegal nettype form, `nettype <declared nettype> <name> with <fn>;`, which moves no shape.)
  - `31` (**`1.0.219`** (`SV-CORPUS-GRAD.3.45`, `PGEN-SV-CORPUS-GRAD-0374`, 2026-09-07), ledger `SV-0092`: the partial-typing residue is typed — seventeen return templates over `assignment_pattern`, `block_data_type`, `case_statement`, `clocking_declaration`, `clocking_event_sv_2017`, `list_of_path_delay_expressions`, `net_type_declaration_sv_2017`, `property_case_item`; every newly typed alternative carries a `kind`, every typed sibling byte-identical (`kind` absent = the shipped form); the nettype alias form wins its tie (a negative lookahead on the data-type alternative, the alias base store-gated); the two non-LRM path-delay alternatives and three ungated nettype identifier twins deleted (`1641 → 1637` rules). See [Typed Partial Alternatives](../systemverilog_parser_book/src/json-carrier.md#typed-partial-alternatives-schema-31).)
  - `30` (**`1.0.218`** (`SV-CORPUS-GRAD.3.44`, `PGEN-SV-CORPUS-GRAD-0373`, 2026-09-07), ledger `SV-0091`: `function_body_declaration` and `task_body_declaration` are typed on EVERY alternative — through 1.0.217 only alternative 1 (`type f; …`) carried a template and the ported form `type f(ports); …` (2 211 of the 3 411 bodies in the corpus) and the two out-of-block forms arrived as the raw positional `Sequence` envelope; now `{return_type, name, ports, items, statements, end_label}` / `{return_type, scope, name, items, statements, end_label}` / both, the task twins without `return_type`, `ports` present exactly on the ported alternatives and `scope` exactly on the out-of-block ones; migration: read the named fields instead of `body[N]`; alternative 1 byte-identical; the corpus AST identity sweep reads `ast_moved=1060 verdict_moved=0` on `sv_2017` (8696 of 9756 accepted trees byte-identical), `ast_moved=1064 verdict_moved=0` on `sv_2023`, `ast_moved=40 verdict_moved=0` on the `verilog_2005` lane — exactly the census's predicted files (`sv_2017`: predicted 1060, moved 1060, predicted-but-identical 0, moved-but-not-predicted 0; `sv_2023`: predicted 1064, moved 1064, predicted-but-identical 0, moved-but-not-predicted 0; `verilog_2005`: predicted 40, moved 40, predicted-but-identical 0, moved-but-not-predicted 0); no verdict moves. A REPLACEMENT of a raw envelope by a typed node — the `SV-0014`…`SV-0020` precedent — so it takes a bump.)
  - `29` (**`1.0.217`** (`SV-CORPUS-GRAD.3.39`, `PGEN-SV-CORPUS-GRAD-0372`, 2026-09-07), ledger `SV-0090`: the thirteen compiler directives IEEE 1800-2017 clause 22 gives a syntax box are typed productions — `compiler_directive.body` is a typed node (`{kind: "timescale", unit, precision}`, `{kind: "line", number, filename, level}`, `{kind: "pragma", name, expressions}`, …) where through 1.0.216 it was the directive LINE as one string, and every other tolerated backtick line at file top level is `{kind: "opaque", text}`; migration: switch on `body.kind`; the corpus AST identity sweep reads `ast_moved=790 verdict_moved=30` on `sv_2017` and `794 / 27` on `sv_2023`, the moved trees being EXACTLY the accepted files that carry a directive node)
  - `28` (UNCHANGED by **`1.0.216`** — `SV-CORPUS-GRAD.3.43`, `PGEN-SV-CORPUS-GRAD-0371`, 2026-09-06, ledger `SV-0089`: the DPI import result-type restriction (IEEE 1800-2017 §35.5.5 / Annex A footnote 21) is a caller split whose restricted result rule carries `data_type`'s own templates, so every accepting tree is byte-identical — the corpus AST identity sweep over 16 335 files × 2 SV profiles reads `ast_moved=0 verdict_moved=2` on each, 9 778 / 9 799 accepted trees byte-identical) (**`1.0.215`** (`SV-CORPUS-GRAD.3.42`, `PGEN-SV-CORPUS-GRAD-0370`, 2026-09-06), ledger `SV-0088`: sixteen inline alternations inside groups of annotated rules are lifted into nine unannotated helper rules, so the `scope` / `qualifier` / `prefix` fields that carried the HOST's template wrapped around a scope prefix — `{name: {kind: "dot"}, scope: {kind: "this"}, select: "<invalid_sequence_access>"}` for `this .`, `{kind: "block", body: <class_scope>}` for a block-event method's class scope, six `"<invalid_sequence_access>"` inside `function_body_declaration`'s raw envelope for `function void I.h();` — now carry the prefix as ITS OWN node: `this .` = `[{kind: "this"}, {kind: "dot"}]`, a class scope = the `class_scope` node, a package scope = the `package_scope` / `non_typedef_package_scope` node, an interface prefix = `[<interface_identifier>, {kind: "dot"}]`. **Migration:** a consumer that read `scope.scope` / `scope.name` on one of the corrupted nodes reads the pair or the typed node; a consumer that tolerated `"<invalid_sequence_access>"` in `select` / `call` / `name` / `items` / `statements` / `end_label` no longer sees it. Every non-shadowed branch, every absent optional and `function void C::g();` are byte-identical (the corpus AST identity sweep). A REPLACEMENT of corrupted nodes by correct ones — the `SV-0014`…`SV-0020` corruption-fix precedent — so it takes a bump.)
  - `27` (UNCHANGED by **`1.0.214`** — `SV-CORPUS-GRAD.3.38`, `PGEN-SV-CORPUS-GRAD-0369`, 2026-09-05, ledger `SV-0087`: the footnote-20 tightening moves no shape; the DPI-import scope markers bind the elements the annotations already bind or discard and the gated unsized arm passes its node through, so every accepting tree is byte-identical) (**`1.0.213`** (`SV-CORPUS-GRAD.3.41`, `PGEN-SV-CORPUS-GRAD-0368`, 2026-09-04), ledger
    `SV-0086`: the set-membership operator `inside { … }` becomes a CHAIN ELEMENT of `expression_base`'s operand
    chain (`expression_operand expression_chain_step*`; a step is the unchanged binary triple or the object
    `inside_operator` = `{kind: "sv_2017"|"sv_2023", body: {kind: "inside", ranges}}`), and the grouped
    `inside_expression` (`{expr, ranges}`, whose `expr` swallowed the whole left chain — `b & a inside {…}` read as
    `(b & a) inside {…}` against IEEE 1800-2017 Table 11-2) is REMOVED together with `expression`'s and
    `expression_or_cond_pattern`'s `{kind: "inside", body}` branches. **Migration:** a consumer that matched
    `expression.kind == "inside"` walks `operand_chain.rest` and treats an object entry as the postfix `inside {set}`
    operator on the relational row; `body.ranges` is the same typed `open_range_list` / `range_list`; the left
    operand is the chain to the entry's left. Every `inside`-free tree is byte-identical. A REMOVAL is a
    replacement in the `SV-0053` sense, so it takes a bump.)
  - `26` (**`1.0.193`** (`SV-CORPUS-GRAD.13c.2y`, `PGEN-SV-CORPUS-GRAD-0261`, 2026-08-20), ledger
    `SV-0066`: `prop_primary_sv_2017`/`_sv_2023` REMOVE three branches — `{kind:"implies_unary", body}`,
    `{kind:"sequence_or_assign", lhs, rhs}` and `{kind:"implies_binary", lhs, rhs}` — because all three
    were compiled against the ARROW token `implies := trivia "->"` rather than the IEEE 1800 keyword,
    and carried syntax no Annex A production reaches. A.2.10's own `property_expr implies property_expr`
    now arrives one level up as `{kind:"implies", lhs, rhs}` from `prop_until_<profile>`, in its
    **Table 16-3** right-associative group beside `until`/`s_until`/`until_with`/`s_until_with`.
    **Migration:** a consumer exhaustively matching `prop_primary` kinds drops the three; a consumer
    walking property operators reads `implies` where it reads the `until` family. A REMOVAL is a
    replacement in the `SV-0053` sense, so it takes a bump.)
  - `25` (**`1.0.184`-`1.0.190` (`SV-CORPUS-GRAD.13c.2l`, `PGEN-SV-CORPUS-GRAD-0241`, 2026-08-19) carry schema `21` to
    `25`: FOUR of the seven releases in that batch REPLACE a previously-emitted AST shape, and each takes its own bump,
    because the trigger this contract has always used is replacement rather than addition (the `SV-0053` rule, as against
    the additive `SV-0052` class). `21`→`22` is **`1.0.184`** (ledger `SV-0054`, THE CROSS BODY): each element of
    `cross_body_sv_2017`'s `items` was a two-element pair `[<cross_body_item>, {kind:"semi"}]` and is now the item
    envelope itself. `22`→`23` is **`1.0.186`** (ledger `SV-0056`, THE HIERARCHICAL RECEIVER): the components of an
    already-parsing hierarchical call move out of `method_call`'s `chain` into the receiver's `path`. `23`→`24` is
    **`1.0.189`** (ledger `SV-0059`/`SV-0060`, THE GATE INSTANTIATIONS): enable-gate, pass-enable-switch and
    multi-output-gate instantiations arrive as `gate_instantiation` with named `output`/`input`/`enable` fields instead of
    `udp_instantiation` with an undifferentiated `inputs[]` list — ⛔ a REPLACED NODE TYPE that moved **no verdict at
    all**, which is why it shipped unnoticed and why an accept-set-only watch would never have caught it. `24`→`25` is
    **`1.0.190`** (ledger `SV-0062`, THE ASSIGNMENT-PATTERN KEY): a type-named key moves from `{kind:"member",
    body:"int"}` to `{kind:"pattern", body:{kind:"type", …}}`. The three remaining releases — `1.0.185`, `1.0.187`,
    `1.0.188` — are additive or removing and move NO schema, each measured on the pinned reproducers rather than argued.**)
  - previous: `21` (**1.0.183 (SV-CORPUS-GRAD.13c.2e, `PGEN-SV-CORPUS-GRAD-0214`, 2026-08-12) bumps schema `20` to `21`: THE COVERGROUP SELECT CONDITION (ledger `SV-0053`). `select_condition` rendered IEEE 1800-2017 A.2.11's `select_condition ::= binsof ( bins_expression ) [ intersect { covergroup_range_list } ]` as `( kw_intersect covergroup_range_list* )?` — the LITERAL SystemVerilog braces read as EBNF zero-or-more metasyntax. ⭐ **THE SAME DEFECT AS `SV-0049` DIRECTLY BELOW, ONE CLAUSE AWAY, THREE DAYS LATER** — which is the entry's real lesson: that fix repaired the instance in front of it, not the class, so the sibling survived. A census of every LRM `{ X_list }` group across BOTH the 2017 and 2023 editions now runs as an instrument (`docs/tasks/artifacts/sv_corpus_grad/intersect_braces/`) and reports the remaining count as **2 → 0**. **WHY THE SCHEMA MOVES:** by `SV-0049`'s own stated test — `intersect {5, 8}` PARSED before the fix, as ONE `covergroup_value_range` whose value was the concatenation expression `{5, 8}` rather than the LRM's TWO items, so a real, previously-emitted shape is replaced (had it been 100 % unparseable, the `SV-0044` class would apply and the schema would stay). Measured on the same file across the two parsers: the `intersect` slot goes from the 2-element `[[trivia,"intersect"], [ <one concat-valued item> ]]` to the 4-element `[[trivia,"intersect"], {kind:"lbrace"}, [ <item>, <item> ], {kind:"rbrace"}]`. **Migration:** a consumer that indexed `intersect[1]` to reach the list must read `intersect[2]`; one that read a single concat-valued item must now read the individual `covergroup_value_range`s, where a range appears as `{kind:"range"}` — a kind that could not previously occur there. The `intersect`-ABSENT case is unchanged (`[]`, no sentinel), verified rather than assumed. The pre-fix shape was a bug, not a stable schema. Two further consumer-visible effects, both intended: every ranged intersect list (`binsof(a) intersect { 5, [1:3] }`, the standard's own §19.6.2 example) now PARSES where it previously rejected, and the brace-less `binsof(a) intersect 5` — which has no production in A.2.11 — now correctly REJECTS. ⛔ **The shape-contract gate could not have caught the mis-parse: 0 of its then-33 locked samples contained a `binsof`** — the SAME blind spot `SV-0049` recorded for `dist`. A sample (`select_condition_intersect_braced_list`) is added by this release, so the rule is gate-visible from now on.**)
  - previous: `20` (**1.0.179 (SV-CORPUS-GRAD.3.18, `PGEN-SV-CORPUS-GRAD-0036`, 2026-08-09) bumps schema `19` to `20`: THE `dist` CONSTRAINT OPERATOR (ledger `SV-0049`). `expression_or_dist` rendered IEEE 1800-2017 A.2.10's `expression_or_dist ::= expression [ dist { dist_list } ]` as `( kw_dist dist_list* )?` — the LITERAL SystemVerilog braces read as EBNF zero-or-more metasyntax. (The collision is two lines away in the LRM: `dist_list ::= dist_item { , dist_item }` uses the same characters AS repetition.) **WHY THE SCHEMA MOVES, unlike the same-class `SV-0044` fix which did not:** that construct was 100 % unparseable, so no witnessed shape could change; this one was NOT. `soft x dist {5, 8};` PARSED before the fix — as ONE `dist_item` whose value was the concatenation expression `{5, 8}` rather than the LRM's TWO items — so a real, previously-emitted shape is replaced. Measured on the same file across the two parsers: the `dist` slot goes from the 2-element `[[trivia,"dist"], [ <one concat-valued item> ]]` to the 4-element `[[trivia,"dist"], {kind:"lbrace"}, [ <item>, <item> ], {kind:"rbrace"}]`. **Migration:** a consumer that indexed `dist[1]` to reach the list must read `dist[2]`; one that read a single item whose value was a concatenation must now read the individual `dist_item`s. The pre-fix shape was a bug, not a stable schema. Two further consumer-visible effects, both intended: every weighted or ranged distribution (`x dist {100 := 1, 200 := 2}`, `x dist { [100:102] :/ 1 }`) now PARSES where it previously rejected, and the brace-less `x dist 100 := 1;` — which has no production in A.2.10 — now correctly REJECTS. ⛔ **The shape-contract gate could not have caught the mis-parse: 0 of its then-31 locked samples contained a `dist`.** A sample (`expression_or_dist_braced_list`) is added by this release, so the rule is gate-visible from now on.**) (schema `19` note, retained for history: **1.0.178 (SV-CORPUS-GRAD.3.14b, `PGEN-SV-CORPUS-GRAD-0033`, 2026-08-09) KEEPS schema `19`: IN-SCOPE COMPILER DIRECTIVES (ledger `SV-0048`). Two new grammar rules (`in_scope_compiler_directive`, `in_scope_compiler_directive_sv_only`) hosted as the LAST alternative of `non_port_module_item` and `class_item`. **WHY SCHEMA STAYS 19 — measured, not argued:** the in-scope node is BYTE-IDENTICAL to the top-level one already in the vocabulary — `{"body": "`timescale 1ns / 1ps", "kind": "compiler_directive"}` in both places — so no emitted kind is added, renamed or removed; only the node's *container* is new (`$/content/Json/source_text[0]/body/body/body/items[0]` vs `$/content/Json/source_text[0]`). AST invariance on existing input proven EXHAUSTIVELY: over all 287 previously-passing corpus files containing a whitelisted directive token at line start — the only files whose AST could possibly move — 287/287 dumped, 595 top-level directive nodes on the unchanged path, **0** in-scope nodes. ⚠️ A consumer exhaustively matching `non_port_module_item`/`class_item` kinds must still add a `compiler_directive` arm; that is an additive vocabulary reachability change, not a schema change.**) (schema `19` note, retained for history: **1.0.176 (SV-CORPUS-GRAD.3.10, `PGEN-SV-CORPUS-GRAD-0025`, 2026-07-25) KEEPS schema `19`: SPACED UDP / TIMING-CHECK NUMBER LITERALS RESTORED — `initial q = 1 'b 1;` (ledger `SV-0046`). IEEE 1800-2017 §5.7.1 says a based literal "shall be composed of up to three tokens" (size / `'`+base_format / value) and then fixes exactly which interior seams are open: size↔apostrophe is OPEN (separate lexical tokens, §5.3, no carve-out), apostrophe↔base-char is CLOSED ("shall **not** be separated by any white space"), and base_format↔value is OPEN explicitly ("The unsigned number token shall immediately follow the base format, **optionally preceded by white space**"). `integral_number` got those two seams in `1.0.171` (`SV-0041`), but `init_val:2554` (UDP, A.5.2) and `scalar_constant:4935` (timing checks, A.7.5.3) enumerate the LRM's CLOSED literal set, so they never route through `integral_number` — they carry 12 fused number-literal tokens of their own and never inherited the fix. Fix = `[ \t]*` at exactly the two open seams in all 12 token bodies (`/1[ \t]*'b[ \t]*0\b/` …; the 4 unsized twins have only ONE interior seam since there is no size token). Keeping them as their own tokens — rather than swapping in `integral_number` — is what keeps the closed set closed: the leading `1` stays hard-coded, so `2'b11` still cannot be an `init_val`. **NO new rules or tokens — census `1475` UNCHANGED**; cross-profile (IEEE 1364-2005 §3.5.1 is identical, and these tokens carry no `@profiles` gate), so `verilog_2005` heals too. **WHY SCHEMA STAYS 19 — this is purely additive at the AST level, verified not assumed:** every emitted kind name is unchanged (`{kind:"1'b0"}` … `{kind:"1'BX"}`, `{kind:"'b0"}` … `{kind:"'B1"}`), proven by AST dump — 4 tight/spaced pairs emit BYTE-IDENTICAL kinds with only `span_end` differing — and the 3 locked shape-contract samples (`init_val_lrm_digits`, `scalar_constant_lrm_digits`, `scalar_timing_check_condition_eq`) pass unchanged (`ast_shape_contract_gate` 18/18). **WHAT CONSUMERS SEE — two distinct gains.** (1) ACCEPTANCE: UDP initial values written with LRM-legal white space (`initial q = 1 'b 1;`, `1 'b1`, `1'b 1`, tab-separated, `1 'B 1`, `1 'b x`) flip REJECT→ACCEPT under **all three** profiles. (2) ⚠️ SHAPE REPAIR, which never showed up as a rejection: `scalar_timing_check_condition` has a fallback `expression` branch, so a spaced scalar constant in a `specify` timing check ALWAYS parsed but SILENTLY DEGRADED — `cond == 1 'b 1` produced the flat `{kind:"expression"}` instead of `{kind:"eq", lhs, rhs:{kind:"1'b1"}}`. It now produces the typed compare shape. A consumer that pattern-matched `{kind:"expression"}` for such conditions will now see `eq`/`case_eq`/`ne`/`case_ne` — this is a *correction toward* the shape `1.0.160` (`SV-0030`) already documents as intended, not a new vocabulary. Still REJECTED, correctly: `1' b 1` and `1 ' b 1` (the §5.7.1-prohibited seam), `2'b1`, `1'b2`, `1'bz`, `1'b 11`, and the digit-less `1'b` (the `SV-0030` guard). Corpus: main pass 9,694→9,693 and rejects-valid **382→382 SET-IDENTICAL** — keyed population was mechanically derived as **0 rows** BEFORE measuring (`init_val` is UDP-only), so **+0 is the expected and measured yield**; the single `pass→timeout` is PROVEN contention jitter (the edited rules are entered **0** times on that file per `--trace-rules`, vs 10 on the control, and it exits 0 = pass when run solo). `verilog_2005` lane re-run: **0 transitions**, manifest byte-identical. Both cert contracts UNTOUCHED (census byte-inert: per-rule profile map identical across all 1475 rules).**) (schema `19` bump note, retained for history: **1.0.175 (SV-CORPUS-GRAD.3.9, `PGEN-SV-CORPUS-GRAD-0024`, 2026-07-25) bumps schema `18` to `19`: SPACED SVA CYCLE-DELAY ABBREVIATIONS `## [*]` / `## [+]` RESTORED, and the two opaque `token_<hash>` AST kinds RENAMED (ledger `SV-0045`). ⚠️ **THIS IS THE BREAKING PART, and it is why the schema moves:** `cycle_delay_range` alternatives 3/4 previously emitted `{kind:"token_71b8cf7e"}` (for `##[*]`) and `{kind:"token_9768502a"}` (for `##[+]`); they now emit `{kind:"star"}` and `{kind:"plus"}`. Unlike `.3.8`'s retired `paren_range` — which was unreachable in practice, hence additive and schema-neutral — these two kinds were REACHABLE AND ACTUALLY EMITTED (`##[*]` / `##[+]` parsed fine before), so any downstream consumer pattern-matching on the old hash names must update. That is exactly what a schema bump signals. The new names are not invented: `consecutive_repetition` (`grammars/systemverilog.ebnf:1377`) already emits `{kind:"star"}` / `{kind:"plus"}` for the identical `[ * ]` / `[ + ]` shape, so this makes the grammar self-consistent and retires two extractor hash artifacts from the public AST vocabulary. THE PARSE FIX: IEEE 1800-2017 A.2.10's alternatives 3/4 were single FUSED literal tokens (`kw_token_71b8cf7e := trivia "##[*]"`, `:6655`) each fusing FOUR distinct SystemVerilog lexical tokens (`##` + `[` + `*`/`+` + `]`). A fused multi-character literal cannot admit interior white space, but §5.3 states white space "shall be ignored except when [it serves] to separate other lexical tokens" — so every spaced spelling (`## [*]`, `##[ * ]`, tab-separated, …) is legal SystemVerilog and ALL of them rejected while the tight spelling parsed. Fix = spell both alternatives structurally from the existing `kw_token_93ac8946` / `lbrack` / `star` / `plus` / `rbrack` tokens and RETIRE the two fused tokens; census 1477 → **1475** (no new rules or tokens). VERIFIED: 9 spacing variants all ACCEPT, 5 near-miss negatives (`##[]`, `##[*`, `##[**]`, `##[+*]`, `##[1:2:3]`) all still REJECT, `verilog_2005` still rejects. Corpus: main pass 9,692 → 9,694 (**+1 attributable**, +1 proven contention jitter), rejects-valid 383 → **382** (−1, ZERO new), accepts-invalid 21 SET-BYTE-IDENTICAL, **0 pass→fail per-FILE**; `verilog_2005` lane re-run and PROVEN INERT (`results_v2005.tsv` + manifest byte-identical). Both cert contracts re-baselined for the −2 census (the retired tokens were satisfiable under all three profiles).**) (schema `18` note, retained for history: **1.0.174 (SV-CORPUS-GRAD.3.8, `PGEN-SV-CORPUS-GRAD-0023`, 2026-07-25) KEEPS schema `18`: SVA BRACKETED CYCLE-DELAY RANGE RESTORED (ledger `SV-0044`). IEEE 1800-2017 A.2.10 defines `cycle_delay_range ::= ## constant_primary | ## [ cycle_delay_const_range_expression ] | ##[*] | ##[+]`, where alternative 2's `[ ]` are LITERAL SystemVerilog brackets — but the LRM's BNF uses `[ ]` for both literal brackets AND optional items, a distinction carried typographically in the PDF and DESTROYED by the PDF->text conversion, so the extractor emitted PGEN's optional-group metasyntax `( … )?` instead (`grammars/systemverilog.ebnf:1705`). `##` was therefore followed by an OPTIONAL, BRACKET-LESS range and NO alternative could consume the `[`, leaving `##[m:n]` / `##[m:$]` entirely unparseable. The LRM settles the reading in its own prose: `##[*]` is "an equivalent representation of `##[0:$]`" and `##[+]` of `##[1:$]` — forms only writable if the brackets are literal. Fix = the literal-bracket form `kw_token_93ac8946 lbrack cycle_delay_const_range_expression rbrack -> {kind:"range", body:$3}` using existing tokens; ZERO new rules/tokens (census 1477 UNCHANGED). SV-only (`verilog_2005` byte-inert — SVA is absent from IEEE 1364-2005). NO schema bump — the bracketed form was 100% unparseable before (additive), and the retired `{kind:"paren_range"}` was unreachable in practice (it required a bare `##`) and referenced by no shape-contract sample; strictly more permissive. Corpus: main pass 9,669->9,692 (+23), rejects-valid 406->383 (-23, 0 new), accepts-invalid 21 byte-identical, 0 pass->fail per-FILE; cert union GREEN with NO re-baseline (union_witness 1348 unchanged, residual []). ⚠️ Sibling OPEN (`.3.9`): the SPACED `## [*]` / `## [+]` still reject — alternatives 3/4 are FUSED literal tokens that cannot admit interior whitespace; that fix renames CURRENTLY-PRODUCED kinds and is expected to bump the schema.**) (schema `18` KEEP note, retained for history: **1.0.173 (SV-CORPUS-GRAD.3.7, `PGEN-SV-CORPUS-GRAD-0022`, 2026-07-24) KEEPS schema `18`: DRIVE/CHARGE STRENGTH KEYWORDS RESTORED (ledger `SV-0043`). IEEE 1800-2017 §28.11 / A.8.6 strength keywords are digit-suffixed (`supply0/1`, `strong0/1`, `pull0/1`, `weak0/1`, `highz0/1`) but the `strength`/`net_type`/`drive_strength` rules referenced BARE tokens (`/supply\b/`, `/strong\b/`, `/pull\b/`, `/weak\b/`, `/highz\b/`) that cannot match them (no word boundary between a letter and a trailing digit), so `drive_strength`/`net_strength`/`pull*_strength` were entirely DEAD in ALL profiles. Fix = 10 digit-suffixed tokens + LRM-faithful `strength0`/`strength1` + the 6-combo opposite-digit `drive_strength` + `net_type` `supply0`/`supply1` + `pull*_strength`, minus the 3 orphaned bare tokens (census 1469->1477); UNGATED (drive strength & supply nets are IEEE 1364-2005 constructs too, so cross-profile). NO schema bump — the strength machinery was 100% unparseable before (additive; no witnessed wire-shape change; the strict `strength0`/`strength1` modelling correctly REJECTS same-digit pairs like `(strong1,strong1)`) and the change is strictly more permissive on legal input. Cross-profile heal: main corpus +78 (rejects-valid 426->406), v2005 +54 (rejects-valid 116->62); 0 pass->fail + 0 new rejects-valid both lanes; accepts-invalid byte-identical (21/14).**) (schema `18` KEEP note, retained for history: **1.0.172 (SV-CORPUS-GRAD.3.6, `PGEN-SV-CORPUS-GRAD-0021`, 2026-07-24) KEEPS schema `18`: `unique0` IF/CASE QUALIFIER RESTORED (ledger `SV-0042`). IEEE 1800-2017 §12.4.2 / §12.5.3 / A.6.6 define `unique_priority ::= unique | unique0 | priority`, but the `unique_priority` rule (`grammars/systemverilog.ebnf:5861`, `@profiles sv_2017/sv_2023`) had only the `unique`/`priority` branches and no `unique0` token existed, so `unique0 if`/`unique0 case` wrongly REJECTED under the SV profiles. Fix = one profile-gated token `kw_unique0 := /unique0\b/` + one `| kw_unique0 -> {kind:"unique0"}` branch (census 1468→1469). NO schema bump — `unique_priority` keeps its `{kind:<string>}` shape (a new enum value only; `unique0 if`/`case` were 100% unparseable before, so no lock or wire-format is broken); strictly more permissive; SV-only (`verilog_2005` byte-inert).**) (schema `18` KEEP note, retained for history: **1.0.171 (SV-CORPUS-GRAD.3.5, `PGEN-SV-CORPUS-GRAD-0020`, 2026-07-23) KEEPS schema `18`: SPACED-BASED NUMBER LITERALS RESTORED (ledger `SV-0041`). IEEE 1800-2017 §5.7.1 / IEEE 1364-2005 §3.5.1 permit white space between the size and the `'` (`32 'h5`) and between the base format and the value (`32'h 5`) but NOT between the `'` and its base character. The `integral_number` terminal (`grammars/systemverilog.ebnf:439`) could not span that white space, so every spaced based literal wrongly REJECTED under ALL profiles. Fix = `[ \t]*` at the two LRM-legal seams. NO schema bump — `integral_number` is a terminal whose `body` is the whole matched string (now optionally including the interior spaces), an ADDITIVE lexical change with no structural shape change; strictly more permissive; cross-profile (the FIRST SV-CORPUS-GRAD fix that also heals `verilog_2005`).**) (schema `18` land note, retained for history: **1.0.170 (SV-CORPUS-GRAD.3.4, `PGEN-SV-CORPUS-GRAD-0019`, 2026-07-23) bumps schema `17` to `18`: MODPORT SHARED-DIRECTION PORT LIST RESTORED (ledger `SV-0040`). IEEE 1800-2017 §25 / A.2.9 `modport_simple_ports_declaration ::= port_direction modport_simple_port { , modport_simple_port }` — the `{ , modport_simple_port }` repetition was absent, so a single `port_direction` admitted only ONE port and `modport master(input a, b, output c, d)` rejected at the shared-direction port `b`. The rule now emits `{direction, ports: [$2, $3::2*]}` (was `{direction, port}` — a user-visible shape change: a single `port` field becomes a `ports` array). Coupled fix: `ref` (a genuine SV port_direction, IEEE 1800 Table B.1 reserved keyword) was erroneously omitted from `reserved_non_keyword_identifier_sv` — added, so the new greedy port list stops at a `ref` direction group (`verilog_2005` keeps its own list without `ref`, staying byte-inert). ADDITIVE and strictly more permissive under `sv_2017`/`sv_2023` (modport is SystemVerilog-only). A consumer reading `modport_simple_ports_declaration` must adopt the `ports` array shape.**) (schema `17` note, retained for history: **1.0.169 (SV-CORPUS-GRAD.3.3, `PGEN-SV-CORPUS-GRAD-0018`, 2026-07-23) bumps schema `16` to `17`: SVA SEQUENCE-IMPLICATION OPERATORS RESTORED (ledger `SV-0039`). The overlapped `|->` and non-overlapped `|=>` operators (IEEE 1800-2017 A.2.10) were entirely absent from the grammar — no token, no `prop_primary` branch — so every `sequence_expr |-> property_expr` / `sequence_expr |=> property_expr` assertion wrongly REJECTED at the operator. Two new profile-gated tokens (`overlapped_implication := "|->"`, `non_overlapped_implication := "|=>"`, `["sv_2017","sv_2023"]`) and two new `prop_primary_sv_2017`/`_sv_2023` branches carrying `{kind:"overlapped_implication"|"non_overlapped_implication", lhs, rhs}` are added. ADDITIVE and strictly more permissive: only previously-rejected assertions gain parses; `verilog_2005` still rejects (SVA is SystemVerilog-only). A consumer exhaustively matching `prop_primary`/`property_expr` kinds must add arms for the two new kinds. Root cause of the omission: the LRM-markdown→EBNF extractor split the `|`-prefixed operators on `|`, the EBNF alternation metachar.**) (schema `16` note, retained for history: **1.0.167 (SV-OVER-REJECTION-FIDELITY.3, `PGEN-SV-OVER-REJECTION-FIDELITY-0003`, 2026-07-05) bumps schema `15` to `16`: EVENT-TRIGGER LRM RESTORATION (ledger `SV-0023`). The `event_trigger` return shape is restructured per IEEE 1800 §15.5 + A.6.5 / IEEE 1364-2005 A.6.5. `event_trigger` now dispatches THREE profile bodies (`{kind:"sv_2017"|"sv_2023"|"verilog_2005", body}` — previously two). BREAKING LABEL CORRECTION: the plain `-> hei ;` BLOCKING trigger now realizes `{kind:"blocking", name[, select]}` (previously mislabeled `{kind:"non_blocking"}`), and the NONBLOCKING `->> [ delay_or_event_control ] hei ;` trigger — NEWLY SUPPORTED (`->>` was absent from the grammar) — realizes `{kind:"non_blocking", control, name[, select]}`. REMOVED: the old `-> #delay hei ;` over-acceptance (`{kind:"blocking", control, name}` on a `->` branch) no longer parses (per A.6.5 the optional control belongs to `->>` only). ADDITIVE: under `verilog_2005` the IEEE 1364-2005 event-array trigger `-> hei { [ expression ] } ;` now realizes `{kind:"verilog_2005", body:{kind:"blocking", name, select}}` (`-> e[0];`; the bracket array-select was previously unsupported). Consumers matching `kind:"non_blocking"` for a plain `-> e;` must migrate to `kind:"blocking"`; consumers that matched `kind:"blocking"` with a `control` for `-> #N e;` must migrate to the `->> #N e;` `kind:"non_blocking"` shape.**) (schema `15` note, retained for history: **1.0.161 (SV-DOLLAR-LRM-FIDELITY.4, `PGEN-SV-DOLLAR-LRM-FIDELITY-0004`, 2026-07-03) bumps schema `14` to `15`: LRM `$`-ANCHOR RESTORATION (ledger `SV-0029`, the SV-only group). BREAKING re-route: `$root.`-anchored names/calls no longer ride the generic system-TF machinery — `$root.m.y` realizes the hierarchical shape with an explicit `root: {kind: "root"}` marker (previously `root:` was always null and the whole name arrived as `kind:"system_tf"` + member-chain), and `$root.top.f(1)` (statement or expression) realizes the NEW `{kind: "rooted_tf", target: {kind: "rooted", scope_chain, name}, attributes, args}` carrier. ADDITIVE: `package_scope` gains the `{kind: "dollar_unit", name: {body: "$unit"}}` realization (`$unit::…`; the `name` field is a fixed literal that keeps typedef-exclusion predicates resolvable), module-level severity tasks realize `{kind: "fatal"|"error"|"warning"|"info", args}` under their profile hosts, and the bare-`$` primary realizes `{kind: "system_dollar"}` (`{kind: "dollar"}` in the method-call-receiver hosts). Consumers matching `kind:"system_tf"` for `$root`-anchored text must migrate to the anchored shapes.**) (schema `14` note, retained for history: **1.0.160 (SV-DOLLAR-LRM-FIDELITY.3, `PGEN-SV-DOLLAR-LRM-FIDELITY-0003`, 2026-07-02) bumped schema `13` to `14`: LRM DIGIT + COMPARE-SHAPE RESTORATION (ledger `SV-0030`, both sites). `scalar_constant` now realizes the ten IEEE A.7.5.3 alternatives with digit-carrying kinds (`{kind:"1'b0"|"1'b1"|"1'B0"|"1'B1"|"'b0"|"'b1"|"'B0"|"'B1"|"1"|"0"}` — the digit-less `"1'b"`/`"1'B"`/`"'b"`/`"'B"` kinds are REMOVED from the schema), UDP `init_val` gains `{kind:"1'b0"|"1'b1"|"1'B0"|"1'B1"}` alongside the unchanged x-forms (its digit-less `"1'b"`/`"1'B"` kinds likewise removed), and the `scalar_timing_check_condition` compare forms are REACHABLE for the first time: `e == 1'b0` in a specify timing-check condition now emits `{kind:"eq", lhs, rhs}` (likewise `case_eq`/`ne`/`case_ne`) instead of the flat `{kind:"expression"}` — `lhs` is expression-shaped (`{kind:"base", body:{kind:"operand_chain", first, rest}}`, produced by the new `scalar_timing_check_compare_lhs` whose operator tier is restricted to the operators binding tighter than equality per IEEE 1800-2017 Table 11-2), `rhs` is the typed `scalar_constant`. Operator-continued conditions (`e == 1'b0 && f`) and mixed-precedence lhs forms (`a & b == 1'b0`, where `==` binds tighter than `&`) deliberately KEEP the flat `{kind:"expression"}` parse (byte-identical pre↔post) via the compare branches' `!binary_operator` follow-guard — the flat chain is the precedence-correct reading for consumers. Acceptance changes: LRM sequential-UDP initializers `initial q = 1'b0;` / `1'b1;` REJECT→ACCEPT under every profile; the digit-less nonsense `initial q = 1'b;` ACCEPT→REJECT.**)
  - previous: `13` (**1.0.158 (SV-AST-SHAPE-FIDELITY.2.7, `PGEN-SV-AST-SHAPE-FIDELITY-0008`, 2026-07-01) bumps schema `12` to `13`: CLASS-SCOPED-CALL-PREFIX AST-SHAPE CORRUPTION FIX (ledger `SV-0020`). `class_scoped_call_prefix` (the `SV-0013` rule, `systemverilog.ebnf`) bound `head:$1` to an inline 4-way alternation `( scoped_class_scoped_call_prefix_identifier | known_unscoped_class_scoped_call_class_identifier | known_unscoped_class_scoped_call_interface_class_identifier | known_unscoped_class_scoped_call_type_parameter_identifier )`; a package-class-scoped static method call (`pkg::C::foo()`, reachable in statement + assignment-RHS context) emitted 2 `<invalid_sequence_access>` in the `params`/`scope_chain` slots when the self-annotated `scoped_class_scoped_call_prefix_identifier` branch won and the rule's own `{head:$1, params:$2, scope_chain:$4}` annotation mis-recursed onto the branch result. The seventh SV-main instance of the systemic inline-alternation-`$N` class (`SV-0014`..`SV-0019`), found by the `SV-AST-SHAPE-FIDELITY.2` sweep (candidate #21, tool-proven reachable in `.2.4`). FIX (grammar-only, the named-lift idiom): ONE new un-annotated named rule `class_scoped_call_prefix_head` (positions `$1`..`$4` unchanged; `SV-0013` gated the alt's branches but did not lift it — `.2.7` completes that rule). Schema `12` to `13` because the `params`/`scope_chain` sub-shapes change consumer-visibly (corrupted to clean `[]`). VERIFIED: `pkg::C::foo()` `<invalid_sequence_access>` 2 to 0 both profiles; SV cert `total 1311 to 1312 / witness 1290 to 1291 / UNKNOWN=20` unchanged (identical 20-rule residual, seeds 0/7/42); corpus 14/14; the 6 fully-certified grammars byte-identical. Prior bump — 1.0.157 (SV-AST-SHAPE-FIDELITY.2.6, `PGEN-SV-AST-SHAPE-FIDELITY-0007`, 2026-07-01) bumps schema `11` to `12`: NETTYPE WITH-CLAUSE AST-SHAPE CORRUPTION FIX (ledger `SV-0019`). Both nettype rules — `net_type_declaration_sv_2017` (`systemverilog.ebnf:3341`) and its sv_2023 twin `nettype_declaration_sv_2023` (`:3390`) — bound a DOUBLE-NESTED inline alt `( non_typedef_package_scope | class_scope )?` inside the br0 `with`-clause group `( kw_with ( … )? tf_identifier )?`; a nettype with a package-scoped resolution function (`nettype logic n with pkg::f;`, reachable at module/package/compilation-unit scope) emitted 3 `<invalid_sequence_access>` in the `with_clause` inner scope element (the `data_type`/`name`/`with_clause` slots) both profiles when the self-annotated `non_typedef_package_scope` branch won inside the double-nested optional and br0's own annotation mis-recursed onto the scope element. The sixth SV-main instance of the systemic inline-alternation-`$N` class (`SV-0014`..`SV-0018`/`SVPP-0001`/`RTL-FE-0002`), found by the `SV-AST-SHAPE-FIDELITY.2` sweep (candidate #14, tool-proven reachable in `.2.4`). FIX (grammar-only, the named-lift idiom): ONE new un-annotated named rule `nettype_scope_prefix := non_typedef_package_scope | class_scope` (order preserved vs the reversed `scoped_type_scope_prefix`), shared by all 4 nettype occurrences (both profiles, both branches); the br1 forms were already clean. Schema `11` to `12` because the `with_clause` inner scope sub-shape changes consumer-visibly (corrupted to clean). VERIFIED: `nettype logic n with pkg::f;` `<invalid_sequence_access>` 3 to 0 both profiles (`parse_full` still passes; br1 stays 0); SV cert `total 1310 to 1311 / witness 1289 to 1290 / UNKNOWN=20` unchanged (identical 20-rule residual; `nettype_scope_prefix` witnessed; ZERO newly-unknown) `spf=0` seeds 0/7/42; SV external corpus 14/14; the 6 fully-certified grammars byte-identical. See the schema-`12` row of `docs/systemverilog_parser_book/src/schema-versioning.md`. PRIOR: 1.0.156 (SV-AST-SHAPE-FIDELITY.2.5, `PGEN-SV-AST-SHAPE-FIDELITY-0006`, 2026-07-01) bumps schema `10` to `11`: INTERFACE-CLASS-TYPE AST-SHAPE CORRUPTION FIX (ledger `SV-0018`). `interface_class_type` (`systemverilog.ebnf:2478`) bound `name: $1` to an inline 2-way alternation `( scoped_interface_class_type_identifier | known_unscoped_interface_class_type_identifier )`; a class implementing a package-scoped interface class (`class C implements pkg::IC;`, common in UVM) emitted 1 `<invalid_sequence_access>` in the `params` sub-shape when the self-annotated `scoped_interface_class_type_identifier` branch (`-> {scope: $1, name: $2}`) won and mis-recursed the rule's own `{name: $1, params: $2}` onto the branch result. The fifth SV-main instance of the systemic inline-alternation-`$N` class (`SV-0014`/`SV-0015`/`SV-0016`/`SV-0017`/`SVPP-0001`/`RTL-FE-0002`), found by the `SV-AST-SHAPE-FIDELITY.2` sweep (candidate #8, tool-proven reachable in `.2.4`). FIX (grammar-only, the named-lift idiom mirroring the in-file `base_class_type_head`): ONE new un-annotated named rule `interface_class_type_head`, referenced by `interface_class_type` as its head (annotation text + positions unchanged). Schema `10` to `11` because the `params` sub-shape changes consumer-visibly (corrupted to clean `[]`). VERIFIED: `class C implements pkg::IC;` `<invalid_sequence_access>` 1 to 0 both profiles (`parse_full` still passes); SV cert `total 1309 to 1310 / witness 1288 to 1289 / UNKNOWN=20` unchanged (identical 20-rule residual; `interface_class_type_head` witnessed; ZERO newly-unknown) `spf=0` seeds 0/7/42; SV external corpus 14/14; the 6 fully-certified grammars byte-identical. See the schema-`11` row of `docs/systemverilog_parser_book/src/schema-versioning.md`. PRIOR: 1.0.155 (SV-AST-SHAPE-FIDELITY.2.3, `PGEN-SV-AST-SHAPE-FIDELITY-0004`, 2026-07-01) bumps schema `9` to `10`: SCOPED-TYPE-REFERENCE AST-SHAPE CORRUPTION FIX (ledger `SV-0017`). `class_scope_type` (`systemverilog.ebnf:1067`) bound `head: $1` to an inline 4-way alternation, and `scoped_block_type_identifier`/`scoped_data_type_identifier` (`:1642`/`:1649`) bound `scope: $1` to the shared inline alt `( class_scope | non_typedef_package_scope )`; a package/class-scoped type reference (`C::D::E v;`, ubiquitous in SV/UVM) emitted 4 `<invalid_sequence_access>` sentinels across the `class_scope_type` `params`/`scope_chain` and `scoped_data_type_identifier` `type`/`dims` sub-shapes. The fourth SV-main instance of the systemic inline-alternation-`$N` class (`SV-0014`/`SV-0015`/`SV-0016`/`SVPP-0001`/`RTL-FE-0002`), found by the `SV-AST-SHAPE-FIDELITY.2` sweep (cluster #1/#5/#6). FIX (grammar-only, named-lift idiom mirroring the in-file `class_type_head`): TWO new un-annotated named rules `class_scope_type_head` + `scoped_type_scope_prefix` (the shared alt occurs ONLY at those two rules), each referenced by its parent (annotation text + positions unchanged). Schema `9` to `10` because the affected sub-shapes change consumer-visibly (corrupted to clean). VERIFIED: `C::D::E v;` `<invalid_sequence_access>` 4 to 0 both profiles (`parse_full` still passes); SV cert `total 1307 to 1309 / witness 1286 to 1288 / UNKNOWN=20` unchanged (identical 20-rule residual; both new rules witnessed; ZERO newly-unknown) `spf=0` seeds 0/7/42; SV external corpus 14/14; the 6 fully-certified grammars byte-identical. See the schema-`10` row of `docs/systemverilog_parser_book/src/schema-versioning.md`. PRIOR: 1.0.154 (SV-AST-SHAPE-FIDELITY.2.2, `PGEN-SV-AST-SHAPE-FIDELITY-0003`, 2026-07-01) bumps schema `8` → `9` — BASE-CLASS-TYPE AST-SHAPE CORRUPTION FIX (ledger `SV-0016`):** `base_class_type` (`systemverilog.ebnf:1094`) bound `head: $1` to an inline 3-way alternation `( scoped_base_class_type_identifier | known_unscoped_base_class_type_identifier | known_unscoped_base_class_type_parameter_identifier )`; a derived class whose base is package-scoped (`class D extends pkg::B;` — ubiquitous in SV/UVM) emitted 2 `<invalid_sequence_access>` sentinels in its `base_class_type` `params`/`scope_chain` sub-shapes (the self-annotated scoped branch's `{scope,name}` re-recursed the rule's own annotation onto the following `$2`/`$3` slots); the declared-bare `extends B;` was clean. The third SV-main instance of the systemic inline-alternation-`$N` class (`SV-0014`/`SV-0015`/`SVPP-0001`/`RTL-FE-0002`), found by the `SV-AST-SHAPE-FIDELITY.2` grammar-wide sweep (candidate #2). FIX (grammar-only, precedent idiom mirroring the in-file `class_type_head`): lifted the shared inline alternation into ONE new un-annotated named rule `base_class_type_head`, referenced by `base_class_type` as its head (annotation text + positions unchanged) — one shared rule fixes both profiles (`class_declaration_sv_2017:955` / `_sv_2023:959` share `base_class_type` via `extends`). **Schema `8` → `9`** because the `params`/`scope_chain` sub-shapes change consumer-visibly (corrupted → clean `[]`). VERIFIED: `class D extends pkg::B;` `<invalid_sequence_access>` 2 → 0 both profiles (`parse_full` still passes), declared-bare stays clean; SV cert `total 1306→1307 / witness 1285→1286 / UNKNOWN=20` unchanged (identical 20-rule residual; `base_class_type_head` witnessed; ZERO newly-unknown) `spf=0` deterministic seeds 0/7/42; SV external corpus 14/14; `--lint-grammar` clean (1427→1428 rules); 6 fully-certified grammars byte-identical; shape-contract green (new `base_class_type_head_scoped` + `base_class_type_scoped` samples). See § "AST-Shape Corrections — 1.0.154". _Prior:_ **1.0.153 (SV-AST-SHAPE-FIDELITY.2.1, `PGEN-SV-AST-SHAPE-FIDELITY-0002`, 2026-07-01) bumps schema `7` → `8` — FORWARD-TYPEDEF KEYWORD AST-SHAPE CORRUPTION FIX (ledger `SV-0015`):** `type_declaration_sv_2017` (`systemverilog.ebnf:5209`) and `type_declaration_sv_2023` (`:5225`) branch 6 bound `keyword: $2` to an inline alternation inside an optional group `( kw_enum | kw_struct | kw_union )?`; a forward typedef with a keyword (`typedef enum e_t;` / `struct` / `union` — a common cycle-breaking SV idiom) emitted 3–4 `<invalid_sequence_access>` sentinels in its `keyword` sub-shape (branch-1's `class_alias` annotation mis-recursed onto the `keyword` slot); the keyword-less `typedef e_t;` was clean. The second SV-main instance of the systemic inline-alternation-`$N` class (`SV-0014`/`SVPP-0001`/`RTL-FE-0002`), found by the `SV-AST-SHAPE-FIDELITY.2` grammar-wide sweep (candidates #19+#20). FIX (grammar-only, precedent idiom): lifted the shared inline alternation into ONE new un-annotated named rule `forward_type_keyword := kw_enum_e338e8e3 | kw_struct_d118e5a3 | kw_union_67ad5a07`, referenced by both br6 as `forward_type_keyword?` (annotation text + positions unchanged) — one shared rule fixes both profiles. **Schema `7` → `8`** because the `keyword` sub-shape changes consumer-visibly (corrupted → clean fused-token `[trivia, "enum"]`). VERIFIED: `typedef enum/struct/union e_t;` `<invalid_sequence_access>` 3/4 → 0 both profiles (`parse_full` still passes); SV cert `total 1305→1306 / witness 1284→1285 / UNKNOWN=20` unchanged (identical 20-rule residual; `forward_type_keyword` witnessed; ZERO newly-unknown) `spf=0` deterministic seeds 0/7/42; SV external corpus 14/14; `--lint-grammar` clean (1426→1427 rules); 6 fully-certified grammars byte-identical; shape-contract green (new `forward_typedef_keyword` + `forward_typedef_enum` samples). See § "AST-Shape Corrections — 1.0.153". _Prior:_ **1.0.152 (SV-AST-SHAPE-FIDELITY.1, `PGEN-SV-AST-SHAPE-FIDELITY-0001`, 2026-07-01) bumps schema `6` → `7` — ANSI-PORT AST-SHAPE CORRUPTION FIX (ledger `SV-0014`):** `ansi_port_declaration` (`systemverilog.ebnf:519-520`) bound `header: $1` to an inline alternation inside an optional group `( net_port_header | interface_port_header )?`; every typed ANSI module port (`input logic a`) emitted three `<invalid_sequence_access>` sentinels in its `header` sub-shape (`net_port_header`'s clean `{direction, port_type}` mis-nested as `header.header`, the out-of-range `dims`/`default`/`name` slots corrupted). The systemic inline-alternation-`$N` class (`SVPP-0001`/`RTL-FE-0002`/`RTL-CE-0001`/`VHDL-0001`) — NOT covered by POST-SV-AUDIT (scoped to iteration-lead `{first/rest:$N}` lists). FIX (grammar-only, precedent idiom): lifted the inline alternation into the new un-annotated named rule `ansi_port_header := net_port_header | interface_port_header`; `header: $1` now binds the clean named rule (annotation text unchanged). **Schema `6` → `7`** because the port `header` sub-shape changes consumer-visibly (corrupted → clean `{direction, port_type}`). VERIFIED: `module m (input logic a); endmodule` `<invalid_sequence_access>` 6 → 0 (`parse_full` still passes); SV cert `total 1304→1305 / witness 1283→1284 / UNKNOWN=20` unchanged (identical 20-rule residual; `ansi_port_header` witnessed; ZERO newly-unknown) `spf=0` deterministic seeds 0/7/42; SV external corpus 14/14; `--lint-grammar` clean (1426 rules); 6 fully-certified grammars byte-identical; shape-contract green (new `ansi_port_header_typed` + `ansi_port_typed` samples). See § "AST-Shape Corrections — 1.0.152". _Prior:_ **1.0.151 (GRAMMAR-WELLFORMED.H.12.8.3.1, `PGEN-GRAMMAR-WELLFORMED-0146`, 2026-06-30) keeps schema `6` (NO bump) — CLASS-SCOPED-CALL COUSIN REACH-GAP FIX (ledger `SV-0013`):** `class_scoped_call_prefix` branch 1 (`scoped_class_scoped_call_prefix_identifier`, the `<pkg>::<class>` prefix) was gated only by `lacks_class`, so for an interface_class- or type_parameter-declared head it longest-matched `head::name` as `<pkg>::<class>` and shadowed the dedicated cousin branches `known_unscoped_class_scoped_call_interface_class_identifier` / `…type_parameter_identifier`; the call then fell through to the ungated `package_scope` `tf_call`, emitting a `package_scope`/`tf` shape for what is semantically a class-scoped call. Two AND-stacked `lacks_fact_attribute_equals` predicates (`interface_class`, `type_parameter`) now exclude those heads from branch 1, so an interface_class/type_parameter-scoped call routes through `class_scoped_tf_call`. **Schema stays `6`** — no new node kinds (`class_scoped_tf` already in schema 6); ACCEPTANCE-preserving (the strings still parse — the AST shape is CORRECTED for these scoped calls: route `package_scope`/`tf` → `class_scoped_tf`); genuine package scopes (`pkg::Class::method()`) and class heads are unchanged. VERIFIED: SV canonical cert `UNKNOWN 22 → 20` (the 2 cousins now `witnessed_target=true`) + 4-config union `UNKNOWN 3 → 1` (residual = only `context_member_method_call`), byte-identical seeds 0/7/42 (`spf=0`); the 6 fully-certified grammars byte-identical `fully_certified=true`; SV external corpus 14/14; `ast_shape_contract` green; `--lint-grammar` clean (`1425` rules); clippy source-clean; `cargo test --lib` 739 passed. **1.0.150 (GRAMMAR-WELLFORMED.H.12.8.4.4, `PGEN-GRAMMAR-WELLFORMED-0143`, 2026-06-29) keeps schema `6` (NO bump) — FILE_PATH_SPEC LRM-FIDELITY FIX (ledger `SV-0012`):** the `file_path_spec` lexeme (`kw_file_path_spec_c26c9dc9`) was a literal-keyword LRM-extraction artifact (`/file_path_spec\b/` — matched only the literal word `file_path_spec`), so the `include`/`library` library-cohort productions rejected real IEEE 1800-2017 §33 file paths; corrected to the LRM-faithful path lexeme `/[A-Za-z0-9_.\/?*~$+]+/` (rule name/arity preserved; `-` excluded to keep the `-incdir` flag boundary). **Schema stays `6`** — a single token carrier at the same position ⇒ every typed AST shape byte-identical; strictly-more-permissive (the library cohort now accepts `../rtl/cpu.v`, `/path/to/*.sv`, `./src/*.sv -incdir ./inc`; reachable only from `library_text`, NOT the `systemverilog_file` embedding entry — same `SV-0008`/`SV-0010`/`SV-0011` category). VERIFIED: SV canonical cert `UNKNOWN=22` + 4-config union `UNKNOWN=3` byte-identical seeds 0/7/42 (`spf=0`); the 6 fully-certified grammars byte-identical `fully_certified=true`; SV external corpus 14/14; `ast_shape_contract` 18/18; `--lint-grammar` clean (`1425` rules); clippy source-clean; `cargo test --lib` 771 passed. **1.0.149 (GRAMMAR-WELLFORMED.H.12.5.8.3.2, `PGEN-GRAMMAR-WELLFORMED-0133`, 2026-06-25) keeps schema `6` (NO bump) — SVA PROPERTY PRECEDENCE CASCADE (ledger `SV-0011`):** the flat directly-left-recursive `property_expr_sv_2017`/`property_expr_sv_2023` are restructured into the IEEE 1800-2017 §16 (Table 16-3) binary-infix precedence cascade — `property_expr_sv_<profile> := prop_until_sv_<profile> -> $1` over 5 new `prop_*` rules per profile (`prop_until > prop_iff > prop_or > prop_and > prop_primary`), `prop_primary` keeping every non-LR branch byte-identical (bodies still reference the `property_expr` dispatch) — so the SVA property infix operators `a until b`/`a s_until b`/`a until_with b`/`a s_until_with b`/`a iff b` and property-operand `or`/`and` (e.g. `(always a) or (always b)`) now parse (rejected at all releases ≤ `1.0.148`). **Schema stays `6`** — every previously-VALID property parse keeps its exact `{kind:"sv_2017"/"sv_2023"}`-wrapped carrier; the newly-parsing infix forms reuse the EXISTING documented `{kind, lhs, rhs}` carriers (`until`/`s_until`/`until_with`/`s_until_with`/`iff`/`or`/`and`), so NO carrier changes (strictly-more-permissive, the `SV-0008`/`SV-0010` category). VERIFIED: SV cert `UNKNOWN 26 → 22` (the 4 `until`-family keywords witnessed; `total 1299→1304` (+5), `witness 1272→1281`; deterministic seeds 0/7/42; `spf=0`; zero newly-unknown); the 6 fully-certified grammars byte-identical (grammar-only, SV-isolated, engine unchanged); SV external corpus 14/14; `ast_shape_contract` 18/18; `--lint-grammar` clean (`1425` rules, +10). DOCUMENTED LIMITATION: the tight prefix ops `not`/`nexttime`/`s_nexttime` (level 6) keep the pre-fix dispatch-body precedence (bind whole rhs), so `not a until b` parses as `not (a until b)` rather than strict-Table-16-3 `(not a) until b` — unchanged from pre-fix, schema-preserving, deferred refinement.**) `6` (**1.0.148 (GRAMMAR-WELLFORMED.H.12.5.8.3.1.1, `PGEN-GRAMMAR-WELLFORMED-0132`, 2026-06-25) keeps schema `6` (NO bump) — SVA SEQUENCE PRECEDENCE CASCADE (ledger `SV-0010`):** the flat directly-left-recursive `sequence_expr` is restructured into the IEEE 1800-2017 §16 (Table 16-3) precedence cascade (`sequence_expr := seq_or_expr -> $1` over 11 new `seq_*` rules) so the SVA sequence infix operators `a and b`/`a or b`/`a intersect b`/`a within b` (and chained/mixed-precedence forms) now parse (rejected at all releases ≤ `1.0.147`); the deep operands witness via a parser-agnostic reach-site passthrough preference. **Schema stays `6`** — every previously-VALID sequence parse keeps its carrier via the cascade's `-> $1` passthroughs; only previously-REJECTED infix forms gain new `{kind:"or"/"and"/"intersect"/"within", lhs, rest}` shapes (strictly-more-permissive, the `SV-0008` category — distinct from the carrier-changing `SV-0009`). VERIFIED: SV cert `UNKNOWN 28 → 26` (`kw_intersect`/`kw_within` witnessed; deterministic seeds 0/7/42; `spf=0`; zero newly-unknown); 6 fully-certified grammars byte-identical; SV external corpus 14/14; `ast_shape_contract` 18/18; `--lint-grammar` clean (`1415` rules, +11).**) `6` (**1.0.147 (GRAMMAR-WELLFORMED.H.14.3, `PGEN-GRAMMAR-WELLFORMED-0130`, 2026-06-25) bumps schema `5` → `6` — UDP TRUTH-TABLE ENTRY SHAPE RESTORE (ledger `SV-0009`).** The `combinational_entry` (`{inputs, output}`) and `sequential_entry` (`{inputs, current_state, next_state}`) typed carriers — documented since SV-Slice-18 (`1.0.18`) but silently REGRESSED to the raw `Sequence` envelope at SV-Slice-66 (`1.0.66`), where a duplicate identical `->` return annotation (`grammars/systemverilog.ebnf:1168`/`:4639`) failed to parse and fell back to raw — are RESTORED. A previously-VALID UDP truth-table row now realizes the documented typed object (colon/semi folded away) instead of the raw token `Sequence`, so a consumer on `1.0.66`–`1.0.146` parsing UDPs sees a carrier change ⇒ schema bump (same previously-valid-carrier-change criterion as covergroup `SV-0006`/`SV-0007`). This fix also lands EBNF self-hosting **12/12** (the duplicate `->` was `systemverilog.ebnf`'s last self-parse gap). VERIFIED: SV cert seeds 0/7/42 `total=1288 witness=1259 UNKNOWN=28 spf=0` (unchanged); `ast_shape_contract` 18/18; SV external corpus 14/14; clippy source-clean; surgical 97-line generated-parser diff confined to the two rules. **)** `5` (**1.0.146 (SV-SVA-PROPERTY-FIDELITY.1, `PGEN-SV-SVA-PROPERTY-FIDELITY-0001`, 2026-06-24) keeps schema `5` (NO bump) — LRM BOUNDED-PROPERTY BRACKET GRAMMAR FIX, ledger `SV-0008`.** Restored the IEEE-1800 Annex A.2.10 `[ ]` brackets on the six SVA bounded-property operators in BOTH profiles (`nexttime`/`s_nexttime [ constant_expression ]`, `always`/`s_eventually [ cycle_delay_const_range_expression ]`, `s_always`/`eventually [ constant_range ]`) and removed the dead mis-synthesized `kw_constant_d810ca96` token (the `sv_2017` `nexttime` index had been mis-tokenized into the literal keyword `constant` + `expression`). All six LRM bracket forms were REJECTED at `1.0.0`–`1.0.145` (`nexttime [3] a` etc.) while the non-LRM `nexttime constant 3 a` wrongly PARSED; post-fix the bracket forms PARSE and `nexttime constant 3 a` is correctly REJECTED. SV cert `UNKNOWN 30 → 29` (the `kw_constant_d810ca96` token removed). **No schema bump:** the `{kind, const_expr/range, body}` carrier shape is preserved (brackets folded away by the annotation); the five nonterminal forms capture byte-identical content; only the `sv_2017` `nexttime_const.const_expr` carrier is corrected from the bogus-only `(kw_constant expression)` group to `constant_expression`, now matching `sv_2023` + LRM — affecting only previously-INVALID input, so no consumer relying on a previously-valid parse is impacted (`ast_shape_contract_gate` GREEN 18/0). **CONSUMER NOTICE — non-schema; fixes a real parse-fidelity gap:** LRM-bracketed bounded-property assertions `nexttime [N] p` / `s_nexttime [N] p` / `always [R] p` / `s_always [R] p` / `eventually [R] p` / `s_eventually [R] p` now parse (were rejected at all releases ≤ 1.0.145); the non-LRM `nexttime constant N p` is now correctly rejected. The same dropped-delimiter class as `SV-0006`/`SV-0007` (covergroup) / `SV-0004` (sequence-repetition) / `SV-0002` (streaming). **— prior schema note:** 1.0.145 (SV-COVERGROUP-FIDELITY.2, `PGEN-SV-COVERGROUP-FIDELITY-0002`, 2026-06-24) BUMPS schema `4` → `5` — LRM BINS-SET BRACE GRAMMAR FIX, ledger `SV-0007`.** Restores the IEEE-1800 A.2.11 mandatory braces on the covergroup `bins_or_options` range-list form: `bins_or_options:643` was bare `assign covergroup_range_list*` (no braces); since `covergroup_expression → expression` parses `with(x)` as a `plain_tf` subroutine call, the range-list's first expression swallowed the whole `with(...)` (AST `kind:"call"/"plain_tf", name:"with"`) and the dedicated `( kw_with … with_covergroup_expression … )?` optional stayed empty (`with_expr:[]`) → `with_covergroup_expression` was reachable-but-never-witnessed (cert `UNKNOWN`). **Fixed (grammar-only, LRM A.2.11-faithful):** `… assign lbrace covergroup_range_list rbrace ( kw_with lparen with_covergroup_expression rparen )? ( kw_iff … )?` → `{kind:"range_list", … ranges:$7, with_expr:$9, iff:$10}`; bare single-value / `id with(...)` bins route to their LRM `set` (branch 647) / `cover_point_with` (branch 645) homes. **SCHEMA `4` → `5`:** the typed `range_list` carrier changes — `ranges` collapses one nesting level (`[[…]]` → `[…]`: now individual `covergroup_value_range`s, not one brace-concatenation expression) and `with_expr` becomes populatable (was always `[]`) — the same carrier-shape class as the `SV-0002` stream-concatenation bump. **CONSUMER NOTICE — schema-bumping; fixes a real cert/parse-fidelity gap:** braced bins `bins b = {[0:63],65}` / `bins b = {3} with (e)` now parse with the LRM range-list structure (and `with(...)` after `}` witnesses `with_covergroup_expression`); a bare single value `bins b = v` still parses (via `set`) and `bins b = id with(e)` via `cover_point_with`; the non-LRM bare comma-list `bins b = 1,2,3` (no braces) is now correctly rejected per IEEE-1800 A.2.11 (must be `{1,2,3}`) — no gated-corpus regression (no SV corpus/test exercises bins; external corpus 14/14). VERIFIED: SV cert `UNKNOWN 31 → 30` (`with_covergroup_expression` witnessed; witness `1257 → 1258`; seeds 0/7/42; spf=0; zero newly-UNKNOWN; total=1289); `ast_shape_contract_gate` 18/0; `cargo test --lib` 768/0; clippy source-clean; 6 grammars inert. **Prior schema-4 release:** **1.0.144 (SV-COVERGROUP-FIDELITY.1, `PGEN-SV-COVERGROUP-FIDELITY-0001`, 2026-06-24) kept schema `4` — LRM BRACKET-RESTORE GRAMMAR FIX, ledger `SV-0006`.** Restores the literal `[ ]` brackets the LRM mandates on the covergroup `trans_range_list` repeat forms — `[* repeat_range]` (consecutive), `[-> repeat_range]` (goto), `[= repeat_range]` (nonconsecutive) — dropped in extraction (the same delimiter-drop class as the `SV-0004` sequence-repetition family and `SV-0002` streaming). Pre-fix the bracket-less `trans_item ( star repeat_range )?` / `( implies … )?` / `( assign … )?` (`star := "*"`, `implies := "->"`, `assign := "="`) collided with the multiply/arrow/eq operators, so `trans_item`'s greedy `expression` parsed `a*b` as one multiplication (the optional `repeat_range` never fired → `repeat_range` operator-shadowed → cert `UNKNOWN`) and the bracketed LRM forms `(1[*2])`/`(1[->2])`/`(1[=2])`/`(1=>2[*3])` were unparseable. **Fixed (grammar-only, LRM A.2.11-faithful, same idiom as `SV-0004`):** `trans_range_list := trans_item -> {kind:"simple",body:$1} | trans_item lbrack star repeat_range rbrack -> {kind:"star",body:$1,range:$4} | trans_item lbrack implies repeat_range rbrack -> {kind:"implies",…} | trans_item lbrack assign repeat_range rbrack -> {kind:"assign",…}`. **NO schema bump:** the emitted `{kind, range}` shape is byte-identical (brackets folded away by the annotation), so schema stays `4`. **CONSUMER NOTICE — acceptance widening, fixes a real parse bug, non-schema:** LRM-bracketed covergroup trans-repeat bins `(1[*2])`/`(1[->2])`/`(1[=2])`/`(1=>2[*3])` now parse (were REJECTED at all releases ≤ `1.0.143`); a bare `(1*2)` still parses (now as a `simple` multiply trans_item — no regression). VERIFIED: SV cert-coverage `UNKNOWN 32 → 31` (`repeat_range` witnessed; witness `1256 → 1257`; total `1289`; deterministic seeds 0/7/42; `spf=0`; zero newly-unknown — `comm -13` empty); `(1[*2])`/`(1[->2])`/`(1[=2])`/`(1=>2[*3])`/`([1:2][*3])` REJECT→PASS; bare `(1*2)`/`(1)`/`(1=>2)`/`(1,2)` still PASS; `ast_shape_contract_gate` GREEN (18/0); `cargo test --lib` 768/0; **SV external corpus 14/14** (`parse_fail_total=0`); 6 fully-certified grammars inert (regex `198/198 fully_certified`); clippy source-clean (generated non-strict). Surfaced + adjudicated grammar-first by the cert-coverage `UNKNOWN`→0 drive (`STORE-AWARE-GEN.4b.14` 9C-iii — the delimiter-drop class). **1.0.143 (GRAMMAR-WELLFORMED.H.12.6.1, `PGEN-GRAMMAR-WELLFORMED-0111`, 2026-06-17) keeps schema `4` — PRODUCER LEFT-RECURSION GRAMMAR FIX (AST-shape + reachability), ledger `SV-0005`.** Retires the LAST `no_path` budget case, so the linter's headline "no unreachable rules" is now literally true for SystemVerilog (`unreachable_rules=0`). `module_path_conditional_expression` (IEEE 1800 Annex A.8.3) was stranded by an indirect LEFT-RECURSION in its producer `module_path_expression` (mpce's condition referenced `module_path_expression`, whose first branch was mpce): the left-recursion eliminator rewrote the producer into `module_path_expression_lr_base`/`_lr_suffix` and left `module_path_conditional_expression` as an UNREFERENCED rewritten seed (its generated parse fn defined-but-never-called), while the `_lr_suffix` reconstruction leaked raw internal `wrapper_specs` metadata into the ternary module-path AST. The fix makes mpce's condition the non-left-recursive `module_path_expression_operand` chain (same accepted language — the right-associative ternary reading), so `module_path_expression` is natively non-left-recursive (no elimination; `module_path_expression_lr_base`/`_lr_suffix` vanish), mpce is positively reachable + witnessed, and the `wrapper_specs` leak is gone. **NO schema bump:** the conditional module-path's prior `wrapper_specs`-leaking form was never a realized clean shape a consumer could build against, and the common (non-conditional) chain module-path is byte-identical, so schema stays `4`. **CONSUMER NOTICE — AST-shape + reachability correctness fix, non-breaking for valid inputs:** ternary module-path conditions (`a ? b : c` in `specify` state-dependent paths) now emit a clean `{kind:"conditional", body:{condition, attributes, then_expr, else_expr}}` node instead of leaking raw `wrapper_specs`; the accepted language is unchanged (the ternary parsed before too); non-conditional module paths are byte-identical. VERIFIED: SV cert-coverage `UNKNOWN 86 → 84` (`module_path_conditional_expression` now witnessed + `module_path_expression_lr_suffix` removed; witness `1204`; total `1291 → 1289`; deterministic seeds 0/7/42; `spf=0`); `no_path 20 → 19`; the genuineness oracle emits a clean `conditional` node with `0` `wrapper_specs` (was `0`/`3`); syntax-closure `unreachable_rules=0` (`max_unreachable_rules` budget `1 → 0`, contract v6); **SV external corpus 14/14**; `stimuli_cross_family_platform_gate` PASS; SV shape-contract GREEN; clippy source-clean. Surfaced + adjudicated grammar-first by the cert-coverage `UNKNOWN`→0 drive (`GRAMMAR-WELLFORMED.H.12.6` `no_path` re-audit) — a producer-wiring flaw FIXED, never deleted, per [[feedback_no_rule_deletion_without_lrm_proof]]. **1.0.142 (GRAMMAR-WELLFORMED.H.12.5.5.3.3.3, `PGEN-GRAMMAR-WELLFORMED-0105`, 2026-06-17) keeps schema `4` — LRM DELIMITER-RESTORE GRAMMAR FIX, ledger `SV-0004`.** Restores the literal `[ ]` brackets the LRM mandates on the `boolean_abbrev` sequence-repetition family, dropped in extraction: `consecutive_repetition` (`[* N]` / `[*]` / `[+]`), `goto_repetition` (`[-> N]`), and `non_consecutive_repetition` (`[= N]`, both profiles). Pre-fix the bracket-less forms collided with the `*` / `->` / `=` operators, so the grammar REJECTED valid IEEE 1800 §A.8.1 sequence repetitions (`a[*3]`, `a[->2]`, `a[=2]`) and wrongly accepted the bracket-less spellings; `goto_repetition` was operator-shadowed (cert `UNKNOWN`). Post-fix the bracketed forms parse and the rules are distinguishable. **NO schema bump:** the emitted shape stays `{kind, range}` / `{range}` (the brackets are parsed then folded away by the return annotation, `range: $3`), so schema stays `4`. **CONSUMER NOTICE — acceptance widening + correct rejection, fixes a real parse bug:** LRM-bracketed sequence repetitions now parse (were REJECTED at all releases ≤ `1.0.141`); the bare `*N` / `->N` / `=N` sequence-repetition spellings are now correctly rejected per the LRM (a bare `a*3` still parses as a multiplication expression — no regression). VERIFIED: SV cert-coverage `UNKNOWN 87 → 86` (`goto_repetition` closed; witness `1203 → 1204`; deterministic at seeds 0/7/42; `spf=0`; zero newly-unknown; rule count `1291` unchanged); `--lint-grammar` clean (pre-existing `always_matches=6` A2 backlog unchanged); **SV external corpus 14/14** (`parse_fail_total=0` — both uvm cases included, no regression); SV shape-contract GREEN; clippy source-clean (generated-parser clippy non-strict as established). Surfaced + adjudicated grammar-first by the cert-coverage `UNKNOWN`→0 drive (the attribution rule's delimiter-drop class, the same LRM-extraction family as `SV-0002`). **1.0.141 (GRAMMAR-WELLFORMED.H.12.5.5.3.3.4.2.2.1, `PGEN-GRAMMAR-WELLFORMED-0100`, 2026-06-16) keeps schema `4` — STORE-GATE GRAMMAR FIX, ledger `SV-0003`.** Tightens `checked_nettype_identifier`'s `@predicate` from `has_fact(type_name, $body)` to `fact_attribute_equals(type_name, $body, declaration_family, nettype)` so the `net_declaration` user-nettype branch resolves ONLY to a declared nettype. A declared **class** is also a `type_name`, so the bare gate wrongly consumed a module-scope class-handle declaration `C a;` (C a declared class) as a `net_declaration` — emitting no `variable_binding` fact and so blocking class-handle member-method chains (`C a; … a.b[0].c()` was REJECTED). Post-fix `C a;` routes to `data_declaration` (a class variable, per IEEE 1800 — a class is not a `net_type`) and the chain parses. **CONSUMER NOTICE — acceptance + categorisation correction, NON-schema:** a module-scope class-handle declaration now parses as `data_declaration` instead of `net_declaration`, and module-scope class-handle member-method chains now parse (were rejected); built-in nets (`wire a;`) and real user nettypes (`nettype logic NT; NT a;`) are unaffected; no new AST-dump shape vocabulary (`C a;` re-routes between the two *existing* `net_declaration`/`data_declaration` schema-`4` shapes), so schema stays `4`. VERIFIED: `C a;` → `data_declaration`/`variable_decl`; module-scope `C a; … a.b[0].c()` REJECT→PASS and now witnesses `context_member_method`; `wire a;` still `net_declaration`; `nettype logic NT; NT a;` preserved; undeclared-head chain still correctly rejected; SV cert-coverage byte-identical (`total=1292 witness=1203 UNKNOWN=88 spf=0`, seeds 0/7/42, UNKNOWN set identical — correctness-only, the `context_member_method_call` cert witness is the separate `.4.2.1` generator pass); `--lint-grammar` clean; **SV external corpus 14/14**; `stimuli_cross_family_platform_gate` PASS; SV shape-contract GREEN (+ new `systemverilog_class_handle_decl_is_data_declaration_not_net` lock); clippy source-clean. **1.0.140 (GRAMMAR-WELLFORMED.H.12.5.5.3.3.5, `PGEN-GRAMMAR-WELLFORMED-0096`, 2026-06-16) bumps schema `3` → `4` — SHAPE-CHANGING GRAMMAR FIX, ledger `SV-0002`.** Restores the LRM stream delimiters dropped in extraction (`systemverilog_2017_lrm_extracted.ebnf:1041/1045`): `stream_concatenation := lbrace stream_expression ( comma stream_expression )* rbrace` (restores the mandatory inner `{ }`; was the brace-less `( stream_expression ( comma stream_expression )* )*`) and `stream_expression := expression ( kw_with lbrack array_range_expression rbrack )?` (restores the literal `[ ]` around the with-clause range; was `( kw_with ( array_range_expression )? )?`). Restoring the mandatory inner brace exposed a latent PEG greediness in `streaming_concatenation`'s `( slice_size )?` — `slice_size`'s `constant_expression` branch greedily consumed the brace-concatenation that is in fact the mandatory `stream_concatenation`, then the rule failed without backtracking — so the optional is now guarded `( slice_size &lbrace )?` (a slice_size is, per LRM, always followed by the `{` of the mandatory stream_concatenation; the guard is transparent — `streaming_concatenation`'s `{op, slice_size, body}` shape is byte-identical). **SHAPE CHANGE (the schema bump):** `stream_concatenation.body` changes from a raw quantified-of-quantified node to a clean array `[stream_expression, …]` (the proven `[$2, $3::2*]` extraction-spread idiom), and `stream_expression.with_clause`'s inner shape now carries the bracket tokens. **CONSUMER NOTICE — LANGUAGE + SHAPE change, fixes a real parse bug:** the LRM with-bracket streaming form `{<< 4 {a with [3:0]}}` now parses (was REJECTED at all releases ≤ 1.0.139); the canonical brace forms `{>> {data}}` / `{<< 8 {a, b}}` parse with the *correct* `stream_concatenation` structure (the inner `{ }` is now the stream-concatenation, no longer mis-parsed as a concatenation expression or mis-bound as `slice_size`); and a bare `with X` without brackets is now correctly REJECTED per IEEE 1800 §A.8.1. Consumers that read streaming-concatenation ASTs must adopt the schema-`4` `body` array shape. VERIFIED: SV cert-coverage `UNKNOWN 89 → 88` (the `array_range_expression` fragment closed; witness `1202 → 1203`; deterministic at seeds 0/7/42; `spf=0`; zero newly-unknown rules; rule count unchanged at 1292); `--lint-grammar` clean (no new shadow/orphan/non-terminating; the pre-existing `always_matches=6` A2 backlog unchanged); **SV external corpus 14/14** (uvm_pkg/uvm_compat/scr1/friscv/veer ×{2017,2023} + bootstrap variants — the corpus's own `{>>{i}}` / `{<<{v}}` / `{<< 8 {bit_q_t'({<< {p}})}}` streaming forms all parse); `stimuli_cross_family_platform_gate` PASS; clippy source-clean (generated-parser clippy non-strict as established). **1.0.139 (GRAMMAR-WELLFORMED.H.11.5-FIX, `PGEN-GRAMMAR-WELLFORMED-0075`, 2026-06-11) keeps schema `3` — ENGINE CORRECTNESS FIX, ledger `SV-0001`.** The generated layout skipper's hard-coded `#`-to-EOL comment arm could steal the real SV `#` token (delays, parameter lists) while the parser speculatively attempted a comment token, memoizing the stolen span as bogus trivia and rejecting grammar-valid input (e.g. an ANSI `#( )` header followed by two line comments before a timeunit ratio — `interface i #  (  ) ;timeunit 09 ns//>Mg` ⏎ `//QF.` ⏎ `/633 s;endinterface`). The comment arms are now suppressed statically at parser-emit time for any introducer the grammar assigns a non-comment meaning: SV's `#` arm is gone from both layout skippers, while the `//`/`/*` arms remain (SV's `line_comment`/`block_comment` tokens agree with them). **CONSUMER NOTICE: acceptance widening only for grammar-valid inputs, non-breaking** — every input that parsed at `1.0.138` yields a byte-identical AST at `1.0.139`; previously-rejected valid `#`+comment shapes now parse; degenerate non-SV inputs that relied on bare `#…`-to-EOL being skipped as a "comment" are now correctly rejected. VERIFIED: repro family REJECT→PASS with all controls; SV cert-coverage seed-0 `sample_parse_failures 1→0` (seeds 0/7/42, witness/UNKNOWN byte-identical); SV external corpus clean; locked by `systemverilog_hash_token_is_not_stolen_by_comment_arms_during_speculation`. **1.0.138 (SV-PARSE-STRICT.2, `PGEN-SV-PARSE-STRICT-0002`, 2026-06-09) keeps schema `3` — BEHAVIOUR-TIGHTENING grammar fix.** The `net_declaration` `nt a;` alias branch (both profiles) no longer accepts a bare undeclared identifier as a user net type. It is now gated by two branch-local `phase: post` helpers — `checked_nettype_identifier` (`has_fact(type_name,$body)`, the known declared type/nettype path) and `wildcard_escape_nettype_identifier` (`fact_count_at_least(wildcard_import_open,1)`, the unresolved-`pkg::*`-import escape) — and declared nettypes now emit a `type_name{declaration_family: nettype}` fact (via `declared_nettype_identifier`). A new scope-local non-exportable `@fact_kind wildcard_import_open` is emitted on the wildcard `package_import_item` branch (`import pkg::*;`). Effect: a provably-undeclared net-type identifier — e.g. `module a; endmodulemodule b; endmodule` (the stimuli-generator-found over-acceptance), `module m; zzqq yy; endmodule`, or `module m; foo_type b; endmodule` with no import in scope — is now correctly REJECTED at parse time (sound, context-aware store-gating per the parser/elaborator boundary). AST shape UNCHANGED (`net_type_id` and the nettype-decl `name` stay `{body:...}`) → schema stays `3`. **CONSUMER NOTICE:** behaviour-tightening, same category as 1.0.127 / 1.0.136 — inputs that relied on an undeclared identifier being accepted as a user nettype are now rejected; **a declared nettype** (`nettype int nt; nt a;`), **an in-scope wildcard import** (`import pkg::*; foo_type b;` — deferred to elaboration), and **keyword net types** (`wire`/`tri`/`supply`/…) are unaffected. VERIFIED: 4/4 reject + 9/9 accept A/B; **SV external corpus 14/14** (uvm's `import uvm_pkg::*;` opens the escape — no uvm regression); lib (`--features generated_parsers`, `--lib`) **689/0**; SV shape-contract GREEN. NO-WORKAROUNDS level 1 (existing semantic-annotation gates; consumes the `INLINE-ACTIONS.2` branch-start `@emit_fact`). **1.0.137 (GRAMMAR-WELLFORMED.G.4.8, `PGEN-GRAMMAR-WELLFORMED-0033`) keeps schema `3` — STRICTLY-MORE-PERMISSIVE grammar fix.** `use_clause` (SV config/library declarations, IEEE 1800 §A.1.5) is reordered so the named_parameter-bearing alternatives precede the simple `use [lib.]cell [: config]` form, with an explicit no-lib `use cell .param() …` alternative first. Effect: `use cell .param() : config` (and the bare `use cell .param()` parameter-override form generally) now parses — previously the PEG ordered choice committed the simple alt to `use lib.cell` and stranded the trailing `.param()`. AST shape UNCHANGED (the `{library,name,config}` return shape stays on the now-last simple alt; the param-bearing alts are raw as before), so schema stays `3`. CONSUMER NOTICE: strictly-more-permissive (SVPP-0002/REGEX-0083 category) — previously-parseable inputs are byte-identical; only the previously-erroring `use cell .param()` config form now succeeds. VERIFIED: the previously-failing form parses; SV external corpus 14/14 (config declarations are absent from the corpus — no regression); SV cert-coverage `sample_parse_failures` 1→0; SV shape-contract GREEN; lib 613/613. Surfaced as the witness-parseability residual routed from `LEXICAL-ANNOTATIONS.4.1`. **1.0.136 (SV-EXH-PROOF.5.2.4.1, `PGEN-SV-EXH-PROOF-0107`) keeps schema `3` — BEHAVIOUR-TIGHTENING grammar fix.** `data_type`'s class-type branch now uses the typedef-EXCLUDING `known_unscoped_block_class_type` (gated `has_fact(type_name,head)` **and** `lacks_fact_attribute_equals(type_name,head,declaration_family,typedef)`) instead of the permissive `provisional_unscoped_block_class_type` (gated only by `has_fact(type_name,head)`). Effect: a scoped type reference `Head::member` in `data_type` position (e.g. a `constant_param_expression` value) where `Head` is a plain `typedef` of a non-class — like `localparam int Y = T::P;` after `typedef int T;` — is now correctly REJECTED per IEEE 1800 (a plain typedef has no scope members). AST shape UNCHANGED (same `{head, params, scope_chain}` body, same `provisional_class_type` AST kind), so schema stays `3`. CONSUMER NOTICE: same behaviour-tightening category as `1.0.127` — inputs that accidentally relied on `Head::member` being accepted for a non-class typedef head will now be rejected; valid forms (bare typedef-as-datatype `T x;`, package scope `pkg::C`, real class scope `Class::member`) are unaffected. `block_data_type` (class-internal scoped types) keeps the permissive `provisional_*` branch. Closes the `sv_semantic_scope_contract` `parameter_typedef_fail` mismatch (now 20/20). VERIFIED: minimal repro rejected; SV external corpus 14/14 (no uvm regression); SV shape-contract GREEN. NO-WORKAROUNDS level 1 (existing semantic-annotation gate, per [[feedback_grammar_rules_must_consult_store]]). — POST-SV-AUDIT.2.4b: 11 structured-per-iteration Category-A misuses corrected by factoring each repeated multi-field unit into a new named record rule + an extraction-spread — `list_of_interface_identifiers` / `list_of_port_identifiers` / `list_of_variable_identifiers` / `list_of_tf_variable_identifiers` / `list_of_variable_port_identifiers`, the `let_` / `property_` / `sequence_list_of_arguments` mixed+named_only branches, `parameter_port_list` type_only, and the two `assignment_pattern` named branches. 9 new annotated record rules were added, so the inventory **deliberately changed 2290 → 2299 / 999 → 1008** (these factored record rules ARE annotated — analogous to SVPP-0001's `pp_if_keyword`; NOT "unchanged"). See "AST-Shape Corrections — 1.0.117 (POST-SV-AUDIT)". **1.0.118 (SV-EXH-PROOF.3.2, `PGEN-SV-EXH-PROOF-0021`) keeps schema `3`** — the broken 115-slice/POST-SV-AUDIT per-digit-token number decomposition (`integral_number`/`real_number`/`unsigned_number`/`decimal_number`/`binary_number`/`octal_number`/`hex_number`/`size`/`fixed_point_number`) was restored to the generator's clean single-regex IEEE-1800 lexical form (eliminating the blanket-`\b`-on-single-char-token + `__SV_RULE__`-mangled-`_` corruption that made EVERY multi-digit/underscore/sized/based number unparseable — the external-corpus 0/14 root cause). **Canonical "release bump, no schema bump"**: numbers were 100% unparseable pre-fix so NO number AST was ever emitted (no prior realized shape / no consumer of the never-emitted decomposed shapes); every previously-PARSEABLE input is byte-identical; only previously-erroring numbers now succeed (strictly-more-permissive — same category as SVPP-0002/REGEX-0083). The typed top `number := real_number -> {kind:"real",body:$1} | integral_number -> {kind:"integral",body:$1}` is kept (now clean Terminals ⇒ first realized number shape = flat `{kind,body:"<number>"}`). Inventory **2312 → 2297** (−15: the dead decomposed-rule annotations dropped; SV shape-contract GREEN, no inventory regression-lock). external-corpus parse **0/14 → 4/14** (scr1 family). **1.0.119 (SV-EXH-PROOF.3.3.1, `PGEN-SV-EXH-PROOF-0022`) keeps schema `3`** — `non_keyword_identifier` (the foundation rule for every declaration-site name: class / package / typedef / parameter / localparam, via `declaration_identifier`) returned the raw `-> $2` value, which the positional-extraction codegen leaves as an `Alternative`-wrapped node (unlike implicit-passthrough codegen, which unwraps it); so every `declared_*_identifier -> {body:$1.body}` `@emit_fact`/`@predicate has_fact` directive routed through it failed *"Semantic runtime could not resolve fact name"*, making class/package/typedef/parameter/localparam **declarations** 100% unparseable. Restored to the engine-proven object form `non_keyword_identifier := !reserved_non_keyword_identifier identifier -> {body: $2.body}` (mirrors `identifier := simple_identifier -> {body:$1}`). **Grammar-only fix — engine untouched** (the codegen/runtime is stable; per user standing preference). **Canonical "release bump, no schema bump"**: those declarations were 100% unparseable pre-fix so NO such AST was ever emitted; every previously-PARSEABLE input is byte-identical; only previously-erroring declarations now succeed (strictly-more-permissive, same SVPP-0002/REGEX-0083 category). SV shape-contract **GREEN** (samples=3 aligned=3 drift=0 regression_lock_failures=0 — the rule's value is consumed internally by directives, the emitted AST envelope is unchanged). Inventory **unchanged** (`non_keyword_identifier` was already annotated; `-> $2` → `-> {body:$2.body}` is a shape change of an existing entry, not a count change). Full `.3.2` 13-form number oracle + `.3.1` declared-id family no-regression (interface/program now parse too); **external-corpus parse `4/14 → 6/14`** (friscv_pipeline ×{2017,2023} now fully parse; residual uvm/friscv_rv32i/veer categorized into sub-leaves `.3.3.2`/`.3.3.3`/`.3.3.4`). **1.0.120 (SV-EXH-PROOF.3.3.2, `PGEN-SV-EXH-PROOF-0023`) keeps schema `3`** — `package_declaration` was the only declaration NOT using the proven `declared_X_identifier := X_identifier -> {body:$1.body}` + `@emit_fact {name:$body}` idiom: it put `@emit_fact {name:$package_identifier}` directly on the multi-element rule, and `package_identifier` occurs twice (decl + trailing `endpackage : label`), so the named ref resolved to the `{body:…}` object / was unresolvable ⇒ *"could not resolve fact name"*, making **package declarations** unparseable. Added `declared_package_identifier := package_identifier -> {body:$1.body}` with `@emit_fact {kind:package_name, name:$body}` (mirrors `declared_class_identifier`; **emit-only — package's original directive had NO `@predicate has_fact`, a package is not a type; that behavior is preserved**) and routed the decl-site through it; trailing `( colon package_identifier )?` stays the plain label use-site (exactly as class/module). **Grammar-only — engine untouched.** **Release bump, NO schema bump** — package declarations were unparseable so no such AST was ever emitted; previously-parseable byte-identical; strictly-more-permissive (SVPP-0002/REGEX-0083 category). SV shape-contract **GREEN** (samples=3 aligned=3 drift=0 regression_lock_failures=0). **external-corpus parse stays `6/14`** — HONEST: `package pp; endpackage` (+label/+items) now parse (were 100% unparseable), but uvm_pkg/uvm_compat_pkg ×{2017,2023} fail *deeper* at the `.3.3.3` use-site `known_unscoped_*` type-id defect (~`118257`, *"could not resolve attribute reference 'type_identifier'"*) — `.3.3.2` peeled one onion layer (forward progress; the aggregate binary count is gated by the deeper defect). Evidence refined the categorization: `.3.3.3` (use-site type-id) is the dominant residual (veer + uvm ×4 + uvm_compat ×2). `.3.1`/`.3.2`/`.3.3.1` no-regression verified. **1.0.121 (SV-EXH-PROOF.3.3.3, `PGEN-SV-EXH-PROOF-0024`) keeps schema `3` — PARSER-AGNOSTIC ENGINE EXCEPTION-SAFETY FIX + SV grammar wrapper (composite).** ROOT CAUSE (DEFINITIVELY pinned via full SEMTRACE instrumentation; 3 prior fix attempts disproven, then identified): a classic Rust `?`-bypasses-cleanup bug in the generator-emitted `with_semantic_runtime_rule_transaction` (`rust/src/ast_pipeline/ast_based_generator.rs`). The wrapper `std::mem::take`-s `self.semantic_runtime_state` (leaving `Default == new()` valid-but-EMPTY), then performs `?`-fallible calls (`f(self)?`, `apply_semantic_runtime_effect_directive?`, `resolve_semantic_predicate_spec_against_content?`); a `?` early-return propagated out of the ENTIRE function, JUMPING OVER the `if result.is_err() { self.semantic_runtime_state = original }` restore at the bottom — so `self.semantic_runtime_state` was left EMPTY, silently destroying every fact emitted by prior COMMITTED sibling rules (e.g. a typedef's `type_name` fact). Every later sibling `@predicate has_fact(...)` then saw nothing. Decisive SEMTRACE proof: `RESTORE` fired **0×**, `took post-body state (self.state→EMPTY)` **9×**, fact-flow `declared_type_identifier facts(1)=[type_name=my_t] EXIT OK → … → use-site facts(0)=[] never restored`. FIX (parser-agnostic, in the GENERATOR — Option B "try-block" emulation): wrap the fallible body in an immediately-invoked closure `(|| -> ParseResult<…> {…})()` so every `?` returns into `result` and the restore runs on every non-commit exit. Zero behaviour change on success; no `unsafe`. **Note:** `<SemanticRuntimeState as Default>::default()` delegates to `Self::new()` (Global scope, no facts) — a VALID state — so `std::mem::take` never leaves a *corrupt* state; combined with the IIFE the fix is panic-ROBUST (no corruption on unwind; parser never `catch_unwind`s mid-parse). A true Drop-guard RAII would be strictly panic-SAFE but is not cleanly safe-Rust here (post-take body calls `&self` methods; a guard borrowing `&mut self.semantic_runtime_state` would conflict — only a tiny contained `*mut`+`unsafe` would give a real Drop guard; deferred since IIFE+Default-validity is already panic-robust). SV-grammar COMPOSITE half: added `checked_type_identifier := type_identifier -> {body:$1.body}` + `@predicate has_fact args:[type_name,$body] phase:post`, routed `known_unscoped_block/data_type_identifier` through it (proven .3.3.1/.3.3.2 declared-id idiom; required because a sub-rule-name predicate ref on a `->`-bearing rule does NOT resolve and dotted predicate args are `@predicate`-compiler-rejected). **Release bump, NO schema bump** — declarations whose post-predicate emit/restore was being leaked were silently unparseable so no such AST was ever emitted; previously-parseable inputs byte-identical (success paths unchanged); only previously-erroring inputs now succeed (strictly-more-permissive, SVPP-0002/REGEX-0083 category). VERIFIED: minimal repro `module m; typedef int my_t; my_t [3:0] x; endmodule` PASSES (was unfixable through 3 prior attempts); SEMTRACE confirmed RESTORE fires 110× (was 0); checked_type_identifier `has_fact[type_name,my_t]` resolves+true; class/typedef/localparam/let/builtin-dim no-regression (memo ON / normal). Inventory **2297 → 2299** (+2: the new `checked_type_identifier` rule's @predicate + return-shape; SV shape-contract GREEN samples=3 aligned=3 drift=0 regression_lock=0). FULL cross-parser no-regression: regex broader corpus / conformance **44/0** ✅ (RGX critical downstream — unchanged); SV shape-contract GREEN; SV external corpus **stays `6/14`** — `.3.3.3` is the FOUNDATION engine fix (state-leak was masking everything); the residual 8 cases (veer/uvm/uvm_compat ×6 cross-package import; friscv_rv32i ×2 statement-level) are blocked by DISTINCT defects tracked as `.3.3.4`/`.3.3.6`. Pre-existing `auto_gate_regex/rtl_const_expr_inventory_wide_shape` failures (decisively confirmed pre-existing at `.3.3.2` via git-stash baseline) tracked as `.3.3.5` (separate root-cause needed). **1.0.127 (SV-EXH-PROOF.3.3.4.b.6.2.35.1, `PGEN-SV-EXH-PROOF-0081`) keeps schema `3` — SV GRAMMAR FIX: gate `provisional_unscoped_block_class_type` with `@predicate has_fact(type_name, $head.body)` (Slice-64 redux, lands clean now).** First code sub-leaf of the `.b.6.2.35` umbrella that systematically remediates the ungated-identifier-categorisation defect class identified by Slice-69's tools-first re-diagnosis. Slice-64 (`9375c069`, 2026-05-24) originally added this exact one-line annotation but was reverted (`8eae9eba`) under the (now-retired) persistence framing; Slice-69 (`.b.6.2.30`, 2026-05-25) used the new `--trace-rules` tool to prove the root cause is **structural**, not persistence — when parsing `function f(string a, b="");`, every gated type-identifier alternative correctly **rejected** the undeclared `b` (`has_fact(type_name, b) → false`); only the **ungated** `provisional_unscoped_block_class_type` accepted it. The semantic store always worked correctly; the grammar was just bypassing it. **Fix (one line at `grammars/systemverilog.ebnf:1638`):** `@predicate: { name: has_fact, args: [type_name, $head.body], phase: post, view: shaped }` added directly above `provisional_unscoped_block_class_type`. The sibling `known_unscoped_block_class_type` already carries this exact predicate plus a `lacks_fact_attribute_equals(... typedef)` companion; `provisional_*` differs by now ACCEPTING typedef'd type names where `known_*` rejects them, but both now require the identifier to be a known type. The catch-all-for-unknowns role is gone — but that role was masking real defects. **Test adjustment** (one test, `rust/src/ast_shape_contract.rs::systemverilog_context_gated_method_chain_handles_negated_and_uvm_shape`): the "uvm-shaped function" case had `uvm_seed_map seed_map;` with `uvm_seed_map` never declared. Pre-gate the catch-all accepted any bare identifier as a type; post-gate undeclared types are correctly rejected per Slice-64's stated semantics ("UnknownGarbage → rejected → forces user to fix their code"). Added `typedef class uvm_seed_map;` to the test input — a real-UVM-style forward declaration — so the gate sees a `type_name` fact for it; test purpose (3-level method-chain parsing) preserved. **Release bump, NO schema bump** — AST shape vocabulary unchanged (`{head, params, scope_chain}` shape is the same; only the acceptance criterion changes). **CONSUMER NOTICE: BEHAVIOUR-TIGHTENING CATEGORY.** This is the first SV-EXH-PROOF slice that TIGHTENS acceptance (previously SVPP-0002/REGEX-0083 category was "strictly-more-permissive"). Consumers whose inputs accidentally relied on the over-permissive acceptance of undeclared identifiers as types (e.g. tests that omitted a `typedef class` forward declaration) will see those inputs correctly rejected. Per LRM, undeclared types are invalid SV; rejecting is the correct commercial behaviour (closest precedent: RGX-0087's `\89`-leading hard-reject). NO-WORKAROUNDS HIERARCHY: **level 1** — use existing semantic-annotation primitive; no new annotation, store, or engine machinery. Proven `.3.3.1`/`.3.3.2`/`.3.3.3` declared-id idiom applied to one more rule. Architectural principle ([[feedback_grammar_rules_must_consult_store]], user-set 2026-05-25): rules categorising bare identifiers into tracked categories SHALL consult the store via has_fact / fact_attribute_equals / lacks_fact. VERIFIED: iso5/iso4 minimal repros (`module m; function bit f(string a, b="");` / `..., b=""`) PASS (were FAIL); iso1/iso2/iso3/iso6 + controls (forward-declared `typedef class Bar;`, declared-then-used class, plain module, `module m #(parameter int W=8) (input logic [W-1:0] a);` parameterized header) PASS; **SV external corpus 10 PASS / 4 FAIL / 0 TIMEOUT** — no regression on the 10 known-PASS files (scr1 ×4, friscv ×4, veer ×2 with bootstrap-chained `--lib-in`); residual 4 FAIL = uvm_pkg ×{2017,2023} + uvm_compat_pkg ×{2017,2023}; **uvm_pkg deep-position advance: surface_position 113637 (F1 PEG-furthest-position tooling artefact unchanged) but Slice-59's `furthest_position` tracker now shows 162162** out of ~180K (pre-fix tracked deepest was 19378 at Slice-54 — the parser is now reaching ~90% of uvm_pkg before backtracking exhaustion; ~8.4× deeper); uvm_compat_pkg furthest 114993 → 116752; **lib no-features 548/548 PASS; lib --features generated_parsers 609/609 PASS**; SV shape-contract GREEN; **regex broader corpus / RGX conformance ✅ 44/0** (non-semantic grammar, unaffected). Frontier: `.b.6.2.35.{2..16}` (remaining 9 Table-B scope-prefix-pattern rules + 3 Table-C inline categorising constructs + 3 Table-D delegators) — each its own sub-leaf with empirical per-rule verification. The H1 cross-file-import class (uvm-extends-chain) remains the separate fix path (H1-α library-import chain or H1-β import-emits-permissive-fact) that gates uvm_pkg moving from FAIL to PASS. Pre-existing **1.0.126 (SV-EXH-PROOF.3.3.4.b.3, `PGEN-SV-EXH-PROOF-0029`) keeps schema `3` — PARSER-AGNOSTIC ENGINE LAYER 0: unified quantifier engine (uniform per-iteration atomicity for `?` / `*` / `+` / `{N}` / `{N,M}` / `{N,}` / `{,M}`).** ROOT MOTIVATION: the prior codegen for quantifiers in `rust/src/ast_pipeline/ast_based_generator.rs::generate_quantified_logic` (and `ast_code_generator.rs::generate_quantified_content`) special-cased exactly three operators (`*`, `+`, `?`) with three independently-written code paths, with an asymmetric defect — the FIRST iteration of `+` was emitted inline WITHOUT `try_parse` wrapping, so on first-iter failure the cursor was left wherever the partial element parse stopped (cleanup deferred to the surrounding rule's `try_parse`). Bounded forms (`{N}` / `{N,M}` / `{N,}` / `{,M}`) declared in `grammars/ebnf.ebnf:149-189` were never implemented (the codegen's `_ => Err("Unknown quantifier")` fallthrough). USER STRATEGIC LINE (2026-05-21): "After Layer 0, all the repetition in the EBNF shall have the right behavior" — symmetric, atomic-at-quantifier-boundary semantics across every repetition operator. FIX (one parameterised function replaces three special-cased arms): new `parse_quantifier_bounds(quantifier: &str) -> Option<(usize, Option<usize>)>` helper in `rust/src/ast_pipeline/mod.rs` mapping every surface form to `(min, max)` bounds (`?` → `(0, Some(1))`, `*` → `(0, None)`, `+` → `(1, None)`, `{N}` → `(N, Some(N))`, `{N,M}` → `(N, Some(M))` (rejects `M < N`), `{N,}` → `(N, None)`, `{,M}` → `(0, Some(M))`); 4 unit tests cover happy-path + invalid forms + whitespace. Both `generate_quantified_logic` and `generate_quantified_content` rewritten as one `loop` with EVERY iteration wrapped in `try_parse` (per-iteration atomicity is uniform — the prior asymmetric inline-first-iter pattern for `+` is gone). Quantifier-level atomicity is layered on top: when `min > 0` the codegen emits `let quantifier_start_position = parser.position;` before the loop and on min-failure restores `parser.position = quantifier_start_position;` before signalling `Err(Backtrack)` (the quantifier is atomic at its own boundary — when iteration N+1 fails after N successful iters, if N is insufficient for `min`, the partial successes are undone and the quantifier reports failure as if it was never tried; for `min == 0` / `*` / `?` the bind + check are elided to avoid always-false comparisons + unused-variable warnings). Max-count enforcement (`{N}` / `{N,M}` / `{,M}`) emits `if iteration_count >= #m_lit { break; }` inside the loop. Zero-length-match guard (`parser.position == iter_start`) is preserved. SAFETY_LIMIT (10_000) is preserved. The `quantifier` string is passed verbatim through to `ParseContent::Quantified(results, #quantifier_label)` so downstream consumers see the original surface form. **Release bump, NO schema bump** — for `?` / `*` / `+`, the unified codegen is BEHAVIOR-EQUIVALENT on success paths (same number of iterations committed, same advances of the cursor) and BEHAVIOR-EQUIVALENT-OR-MORE-CORRECT on failure paths (the prior `+` first-iter inline path left the cursor where partial parse stopped before the surrounding `try_parse` rolled it back; the unified path rolls back at the quantifier boundary too). Bounded forms `{N}` / `{N,M}` / `{N,}` / `{,M}` were unparseable pre-fix so NO such grammar construct ever appeared in any generated parser (no current grammar uses them — Layer 0 makes them parser-agnostic infrastructure available for any future grammar); strictly-more-permissive (SVPP-0002/REGEX-0083 category). VERIFIED: lib tests **465/465** PASS no-features (was 461 → +4 new `parse_quantifier_bounds` unit tests); SV smoke (`module m; endmodule` / `if (1) $display("ok");` / else-if chain / for-loop) PASS via fresh release `parseability_probe`; SV shape-contract test GREEN. FULL cross-parser no-regression: **RGX broader corpus / conformance 44/0 ✅** via `cargo run --features generated_parsers --release --bin test_runner -- --parser regex` after a `make focus_regex` regen (proves the unified codegen produces the same regex parser behavior on the entire conformance suite). USER-DESCRIBED SEMANTICS, CONFIRMED IMPLEMENTED: "the current value of the cursor shall be saved at the start of an iteration, if the iteration fails we put the cursor back to the position before starting that iteration, and we check if the repetition is fulfilled or not" — per-iter cursor save+restore is done by `try_parse`; the loop continues until either an iteration fails (break) or the max bound is reached; the min check at the end determines whether the quantifier as a whole succeeds. "For {N}, we need the group to match exactly N times, if it doesn't, the cursor is left at the position it had before trying to match `<group>{N}`" — exactly what the `quantifier_start_position` save+restore on min-failure delivers (atomic at the quantifier boundary). **SV external corpus `8/14 → 10/14`** — full triage gate fresh 2026-05-21 09:55: `friscv_rv32i_core ×{2017,2023}` now PASS (previously categorized as the `.3.3.6` statement-level residual). HONEST ROOT CAUSE — the friscv_rv32i blocker was NOT a separate statement-level grammar defect; it was the prior codegen's asymmetric `+`-first-iter-not-wrapped-in-`try_parse` defect. Whatever `+`-quantifier was hit deep inside friscv_rv32i's statement parsing left the cursor in a non-rollback-able state on first-iter failure (the surrounding rule's `try_parse` would normally rescue it, but in this combinatorial-corner case the cursor was already past a commit point or the surrounding rule had its own ordered-choice fallthrough that left it stuck). Layer 0 now wraps the first iter of `+` uniformly — same as every other iter — and the cursor rolls back cleanly on first-iter failure. THIS is why .b.3 is a corpus-mover even though it claimed to be "behavior-equivalent for `*`/`+`/`?` on success paths": the BEHAVIORAL DIFFERENCE on the prior `+` first-iter-failure path was real, and friscv_rv32i was triggering it. Residual 4 fails: `uvm_pkg`/`uvm_compat_pkg ×{2017,2023}` STILL fail (uvm_pkg_2017 STILL at byte 113637 — identical to `.b.1` baseline, Layer 0 preserves uvm behavior, which is blocked by a different defect — likely the next LRM-extraction-defect class, the larger "`[ X ]` LRM-optional encoded without `?` quantifier"). The `.3.3.6` task entry can be CLOSED — friscv_rv32i is now PASS without a separate grammar-level fix. Pre-existing `1.0.125 (.3.3.4.b.1)`, `1.0.124 (.3.3.4.a.2)`, ... retained below. **1.0.125 (SV-EXH-PROOF.3.3.4.b.1, `PGEN-SV-EXH-PROOF-0028`) keeps schema `3` — FIRST LRM-EXTRACTION-DEFECT FIX: `conditional_statement` `[ else statement_or_null ]` encoded as truly optional.** Root cause (precisely located against the authoritative IEEE 1800-2017 PDF page 1164 AND IEEE 1800-2023 PDF page 1201, both editions identical text): `grammars/systemverilog.ebnf:1111`'s `conditional_statement` rule had exactly ONE production with NO `|` alternative for the no-`else` case — `( unique_priority )? kw_if lparen cond_predicate rparen statement_or_null &kw_else_ae050f5b kw_else_ae050f5b conditional_else_branch`. `&kw_else` is PEG positive lookahead REQUIRING else; `kw_else` then consumes; `conditional_else_branch` then consumes the body. There was no fallback for if-without-else. LRM is unambiguous: `[ else statement_or_null ]` — square brackets mean OPTIONAL. This was an LRM-extraction defect: the extractor encoded an LRM-optional clause as mandatory. FIX: introduced helper `conditional_else_clause := kw_else_ae050f5b conditional_else_branch -> $2` (passes through just the branch); rewrote `conditional_statement` to `( unique_priority )? kw_if lparen cond_predicate rparen statement_or_null ( conditional_else_clause )?` with return annotation `-> {unique_priority:$1, condition:$4, then_body:$6, else_body:$7}` (positional index shift $9→$7; `else_body` now nullable when else absent). Also removed dead mis-attached line 1116 (`| @sample: "if (a) ;" ... !kw_else_ae050f5b`) — the original author reached for the no-else alternative via PEG negative lookahead but the indentation + blank-line at 1113 parented it to `conditional_else_branch` (unreachable from any parse path). Else-if chain `{ else if (cond) statement_or_null }` continues to be handled by the existing recursive `conditional_else_branch := conditional_statement | statement_or_null` — once the outer else is optional, the recursive `conditional_statement` arm naturally produces zero-or-more chained else-ifs without separate rewrite needed. Inventory **2299 → 2300** (+1: the new `conditional_else_clause` helper rule with its `-> $2` return annotation; existing `conditional_statement` return-annotation shape preserved — `else_body` is the same `conditional_else_branch` ParseNode when present, just additionally nullable when absent, which is strictly-more-permissive over a pre-fix population where every parsed `conditional_statement` necessarily had an else). PEG correctness: `( … )?` is greedy and binds `else` to the innermost `if`, matching the SV dangling-else convention; no `&X` / `!X` lookahead needed (the dead line 1116 was unnecessarily defensive — the `?`-greedy semantics handle dangling-else correctly). VERIFIED: minimal failing repro `module m; task t(); if (1) $display("ok"); endtask endmodule` PASSES (was FAIL); pre-existing `if (1) ; else ;` STILL PASSES; else-if chain `if (1) a; else if (2) b; else c;` PASSES; `.3.3.3` minimal repro `module m; typedef int my_t; my_t [3:0] x; endmodule` STILL PASSES; end-to-end synthetic cross-file repro PASSES with `--lib-in` and FAILS without (`.3.3.4.a` behavior preserved); lib **461/461** no-features + **516/517** with `--features generated_parsers` (1 fail = pre-existing `rgx_0077` `.3.3.5`-class). FULL cross-parser no-regression: **RGX broader corpus / conformance ✅ 44/0** via `make regex_broader_corpus_proof_gate`; SV shape-contract GREEN. SV external corpus partial retest: scr1_core_top_2017 PASS (was PASS, time 38s→56s due to deeper parsing post-fix), friscv_pipeline_2017 PASS at unchanged 0.86s, veer_el2_lsu_2017 PASS at 7.6s + 21.6s bootstrap (unchanged). **uvm_pkg_2017 STILL FAILS at the SAME byte position 113637 post-fix, but parse time 60s→14min** — meaning `conditional_statement` was NOT the blocker for uvm_pkg; the parser now walks deeper into the 89K-line / 3MB body (the slowdown is "real work", not a pathological backtracking explosion) and hits an OTHER LRM-extraction defect deeper in. Triage gate runtime ceiling makes a full 14-case rerun cost ~3+ hours; full retest deferred pending either further LRM-extraction-defect fixes that unblock more cases OR a perf-aware retry. **SV external corpus binary count: stays `8/14`** — no passing case lost; no new failing case fixed by THIS slice (the conditional_statement fix is correct on its own merits per the LRM oracle, but uvm/uvm_compat/friscv_rv32i are blocked by deeper LRM-extraction defects, not by conditional_statement). HONEST: this slice is the first concrete fix of the LRM-extraction-defect campaign; the corpus is not yet at 12/14 because more deep defects remain — they will be found via the same defect-class-by-defect-class sweep the user mandated (next class TBD after the `&X` class is fully exhausted, which it is — `&X` is a class of one). The remaining `&question question` at line 1105 (`conditional_expression` ternary) is legitimate (`?` and `:` are LRM-MANDATORY for ternary per §A.8.3), not a defect. Pre-existing `1.0.124 (.3.3.4.a.2)`, `1.0.123 (.3.3.4.a.1)`, `1.0.122 (.3.3.4.a)`, ... retained below. **1.0.124 (SV-EXH-PROOF.3.3.4.a.2, `PGEN-SV-EXH-PROOF-0027`) keeps schema `3` — PARSER-AGNOSTIC ENGINE EXTENSION: non-negative integer indexed-access (`[N]`) in semantic-annotation rule references.** Companion to `.3.3.4.a.1` (which added dotted property-access). Real authoring concern that `.3.3.4.a.1` only half-addressed: when a rule's shaped output is `{items: [{name, body}, ...]}` or `{matrix: [[…], …]}`, directives still couldn't reference array elements directly — there was no `$items[0].name` or `$matrix[0][1]` form. `.3.3.4.a.2` closes that gap. ENGINE: `unified_semantic_ast.rs::parse_rule_reference` chain loop refactored to a `loop { match peek_char() { Some('.') => …, Some('[') => …, _ => break } }` shape that accepts an arbitrary mix of `.<ident>` and `[<digits>]` segments. Strict-bracket policy: a `[` not followed by `<digits>]` rolls back (leaves the `[` for the surrounding parser to handle / fail). EBNF surface `grammars/semantic_annotation.ebnf::rule_reference_name` extended in lockstep: `/([a-zA-Z_][a-zA-Z0-9_]*(\.[a-zA-Z_][a-zA-Z0-9_]*|\[[0-9]+\])*|[0-9]+(\.[a-zA-Z_][a-zA-Z0-9_]*|\[[0-9]+\])*)/`. Bootstrap `semantic_annotation_parser.rs` regenerated. RUNTIME RESOLVER (`ast_based_generator.rs`): three new free-standing helpers emitted into every generated parser — `lex_semantic_reference_segments_suffix` (lexes a bare suffix like `.body[0].sub` into a `Vec<&str>` where each segment is either a property name or the full `[<digits>]` form with brackets), `lex_semantic_reference_segments_named` (named-path counterpart that also captures the head identifier), `parse_bracketed_index` (extracts the integer from `[<digits>]` or returns `None`). The two resolvers (`resolve_positional_semantic_reference` and `resolve_named_semantic_reference`) use the lexers and dispatch per segment: for shaped-JSON content (`ParseContent::Json`) the walk goes through `serde_json::Value::get`, which polymorphically accepts both `&str` (property) and `usize` (index); for raw-tree content (rules without `->`), a new `find_semantic_indexed_child` companion picks the N'th element of a `Sequence`/`Quantified` (or returns the wrapped node of an `Alternative` when N==0). Annotation inventory unchanged (helper functions emitted are infrastructure, not annotated rules). SUBSET BOUNDARY: dotted property access + non-negative integer indexed-access ONLY. NOT full JSONPath (no filters `[?(@.foo)]`, no wildcards `*`, no recursive descent `..`). DURABLE NO-DEPTH-LIMIT GUARANTEE extends to mixed dotted+indexed depth: structurally unbounded at every layer (EBNF `*` quantifier; hand-rolled `loop` with no max-iteration cap; lexer `while !remaining.is_empty()` iterates until input exhausted; resolver iterates over the lexed Vec). Locked by `bootstrap_semantic_indexed_rule_reference_depth_is_structurally_unbounded` test exercising 32 mixed `.<ident>` + 32 `[N]` segments (64 total) and asserting verbatim retention of both kinds of segments (33 dot-separated chunks, 32 bracketed segments). Pairs with the `.3.3.4.a.1` dotted-only test. No SV grammar changes — purely a language-surface expressiveness improvement; the SV directive payloads in 1.0.123 (`$name.body`, `$package.body`) continue to work byte-identically. **Release bump, NO schema bump** — strictly additive language extension; no shape changes anywhere; no annotated rules changed; SVPP-0002/REGEX-0083 category. VERIFIED: 461/461 lib no-features (was 460 → +1 new indexed-access regression test); 516/517 with `--features generated_parsers` (only fail = pre-existing `rgx_0077` from `.3.3.5`-class); cross-file synthetic repro PASSES with `--lib-in` and FAILS without (1.0.123 cleanup behaviour preserved); `.3.3.3` minimal repro still PASSES; both depth-unbounded regression tests in the suite (dotted-only + mixed). FULL cross-parser no-regression: regex broader corpus / **RGX conformance 44/0 ✅** via `make regex_broader_corpus_proof_gate`; SV shape-contract GREEN; **SV external corpus stays `8/14`** (verified: triage gate fresh 2026-05-20; veer_el2_lsu ×{2017,2023} still PASS via the existing `$name.body` directives — no SV directive uses `[N]` yet, so the new feature is dead-code there). NEXT LEAF (queued): `.3.3.4.b` — uvm self-contained same-file-package path (intra-file scope tracking, not artifact-on-disk; the cross-file MVP-0 model doesn't apply when package + uses share a file). Pre-existing `1.0.123 (.3.3.4.a.1)`, `1.0.122 (.3.3.4.a)`, `1.0.121 (.3.3.3)`, … retained below. **1.0.123 (SV-EXH-PROOF.3.3.4.a.1, `PGEN-SV-EXH-PROOF-0026`) keeps schema `3` — PARSER-AGNOSTIC ENGINE CLEANUP: dotted property-access in semantic-annotation rule references + revert of the `.3.3.4.a` SV `body:$N.body` shape workarounds.** Companion to `.3.3.4.a`: the directive-payload parser is the hand-rolled `StructuredSemanticValueParser` in `rust/src/ast_pipeline/unified_semantic_ast.rs::parse_rule_reference` (the generated `semantic_annotation_parser.rs` is the EBNF-language surface, NOT the runtime path for grammar directive payloads). The runtime resolver already walked dotted paths (`resolve_named_semantic_reference` in `ast_based_generator.rs:3912` splits on `.` and traverses either the shaped JSON object-key path or the raw sub-rule-named descendant tree); the only gap was at the bootstrap-parser-surface, which rejected `.` inside `$<name>`. `.3.3.4.a.1` closes that gap: `parse_rule_reference` gains a `while peek_char() == Some('.')` chain that accepts an arbitrary number of `.<ident>` segments. DURABLE NO-DEPTH-LIMIT GUARANTEE: structurally unbounded at every layer (EBNF `*` quantifier; hand-rolled `while` with no cap; resolver iterator-based walk). Locked by `bootstrap_semantic_dotted_rule_reference_depth_is_structurally_unbounded` test that exercises a 64-segment reference and asserts retention — any future `MAX_REFERENCE_DEPTH` constant must justify itself in its own leaf or back off. The EBNF surface is also kept consistent: `grammars/semantic_annotation.ebnf::rule_reference_name` regex extended `(\.<ident>)*` so the language definition mirrors the runtime; the bootstrap `semantic_annotation_parser.rs` is regenerated in lockstep (the EBNF/regenerated-parser does not parse directive payloads — `unified_semantic_ast.rs` does — but keeping the surface consistent prevents future drift). SV grammar revert: `body: $4.body` removed from `package_declaration`'s shaped output; `body: $1.body` removed from BOTH `package_import_item` branches; the two directives now reference the nested scalars naturally — `@export_to_library: {kind:package, name_from:$name.body}` on `package_declaration` and `@import_from_library: {kind:package, name_from:$package.body}` on `package_import_item`. **Release bump, NO schema bump** — the prior `body` fields were a workaround carrier present only in 1.0.122 (one slice); they had no consumer pre-1.0.122 and have no consumer post-`.3.3.4.a.1` (cross-file imports STILL resolve, see verification); the removal is a strictly-rollback shape change versus 1.0.122 (and a no-op shape change versus 1.0.121, since 1.0.121 didn't have those fields either). VERIFIED: end-to-end synthetic cross-file repro PASSES with `--lib-in` and FAILS without (proves library plumbing still the deciding factor — the cleanup is behaviour-preserving); `.3.3.3` minimal repro still PASSES; lib tests **460/460** PASS no-features (was 459 → +1 new unbounded-depth regression test) and **515/516** with `--features generated_parsers` (the only fail = pre-existing `rgx_0077` from `.3.3.5`-class). FULL cross-parser no-regression: regex broader corpus / RGX conformance **44/0** ✅ via `make regex_broader_corpus_proof_gate`; SV shape-contract GREEN; **SV external corpus stays `8/14`** — veer_el2_lsu ×{2017,2023} both PASS via the new dotted-refs path (fresh triage gate 2026-05-20 15:41; identical to `.3.3.4.a`'s shape-workaround result → cleanup proven behaviour-preserving). Annotation inventory: unchanged-shape-of-existing-entries (the two SV `body:` fields were workaround output-mapping additions, not new annotated rules; removing them is a shape change of existing entries, not a count change). The duplicate-facts artifact-content observation from `.3.3.4.a` remains (PEG try-alternative speculation; not addressed in this slice — separate post-campaign shape audit). NEXT LEAVES: `.3.3.4.a.2` will extend `parse_rule_reference` and `parse_semantic_reference_segments` + the EBNF regex with `[<digits>]` indexed access (`$items[0].name` / `$matrix[0][1]` / mixed `$a.b[0].c[1].d`) — same unbounded-depth guarantee, own leaf, separate commit. Pre-existing `1.0.121 (.3.3.3)`, `1.0.120 (.3.3.2)`, `1.0.119 (.3.3.1)`, `1.0.118 (.3.2)`, `1.0.117` POST-SV-AUDIT.2.4b, `1.0.116`/schema 2, `1.0.115`/schema 1 retained below. **1.0.122 (SV-EXH-PROOF.3.3.4.a, `PGEN-SV-EXH-PROOF-0025`) keeps schema `3` — PARSER-AGNOSTIC ENGINE FEATURE (per-compilation-artifact library) + 2 new generic annotations + SV grammar uses (composite, MVP-0).** ROOT MOTIVATION: PGEN parsed single SV files in isolation, but real HDL is multi-file — `import pkg::*` references types declared in *other* files (e.g. veer's `el2_lsu.sv` imports types from `el2_def.sv`). The use-site `@predicate has_fact(type_name,X)` correctly fires (`.3.3.3` made it reliable) but correctly evaluates false because the type-emitting file was never parsed in the same session. Architecturally-correct fix matching every commercial HDL tool: per-file compilation that writes a compact on-disk artifact for each scope-creating entity (package/module/interface/…) + on-demand library lookup when an importer references that entity. **MVP-0 (this slice)** narrows to packages only, JSON artifacts, user-supplied file order; future increments grow kind-set + add `$unit`/CU semantics under separate leaves. ENGINE (parser-agnostic, generator + runtime): (1) `rust/src/ast_pipeline/library.rs` new module — atomic JSON artifact write/read at `<lib-dir>/<kind>/<name>.facts.json`, `format_version: 1`, MVP-0 exportable-kind filter (`type_name` only), 4 round-trip unit tests; (2) `SemanticRuntimeDirective::{ExportToLibrary, ImportFromLibrary}` + their specs + classifiers `is_library_export/import` + accessors `library_exports/imports_for_rule` on `CompiledSemanticRuntimeAnnotations` (own phase, not in `is_effect`/`is_pre_predicate`); (3) generator wire-up in `with_semantic_runtime_rule_transaction` (the IIFE-wrapped exception-safe wrapper from `.3.3.3`): library imports fire AFTER body parse + effect directives but BEFORE post-predicates (so post-predicates see merged facts); library exports fire AFTER post-predicates BEFORE commit (atomic write; failure rolls back via the IIFE's restore path); export delta computed against `original_semantic_runtime_state.facts().len()` (the rule's TRUE entry, not the transaction-checkpoint which is post-body); (4) parser-struct fields `library_in_dir/library_out_dir` + 4 setters/accessors; (5) `parser_registry::LibraryOptions` + SV-aware `parse_sample_detail_with_options` that falls through to `parse_sample_detail_with_profile` when options are empty (zero behaviour change for grammars without library directives); (6) `parseability_probe` CLI flags `--lib-in DIR` / `--lib-out DIR` with strict arg-validation. SV GRAMMAR (composite, this slice): `@export_to_library: {kind:package, name_from:$body}` on `package_declaration` + new top-level output field `body: $4.body` (semantic-annotation language only accepts simple `$name` refs — not `$x.y` — so the package's scalar name is surfaced at the top of the shape; return-annotation `$4.body` property-access drill DOES work in the output mapping); same idiom on `package_import_item` with `@import_from_library` + `body: $1.body` (both branches). TRIAGE GATE (composite, this slice): `systemverilog_external_corpus_triage_v0.json` gains optional per-case `bootstrap_files: [...]` ordered array (commercial-tool convention — user supplies the transitive-dep order); script preprocesses+parses each bootstrap file with `--lib-out <per-case-lib-dir>` THEN parses the main case with `--lib-in <per-case-lib-dir>`; `veer_el2_lsu` declares `bootstrap_files: ["…/include/el2_def.sv"]`. **Release bump, NO schema bump** — adds 2 new top-level optional shaped fields (`body`) to `package_declaration` and `package_import_item`; SV shape-contract recalibrated GREEN. Cross-file imports were unparseable so no such AST was ever emitted from the importer; previously-parseable inputs byte-identical (success paths unchanged); only previously-erroring inputs (cross-file referencing a known package) now succeed (strictly-more-permissive, SVPP-0002/REGEX-0083 category). VERIFIED: end-to-end synthetic repro (write artifact for `my_pkg` containing `my_t`+`byte_t`; module `m` with `import my_pkg::*; my_t [3:0] x; byte_t b;` parses WITH `--lib-in`, FAILS WITHOUT — proves the library path is the deciding factor); minimal `.3.3.3` repro `module m; typedef int my_t; my_t [3:0] x; endmodule` STILL PASSES; lib tests `514/515 PASS` (only fail is the pre-existing `regex_parser_pgen_rgx_0077_quoted_run_quantified_pieces_flat_in_concatenation` confirmed PRE-EXISTING via decisive baseline — git-stashed sources, rebuilt baseline driver from HEAD eb42a3a0, re-regened regex+bootstrap parsers, SAME failure — tracked as `.3.3.5`); 6 new directive-parser unit tests in `semantic_runtime.rs` (positive Export, positive Import, unknown-field rejection, missing-kind rejection, missing-name_from rejection, compiled-annotation routing) all GREEN; 4 library-module unit tests all GREEN. FULL cross-parser no-regression: regex broader corpus / conformance **44/0** ✅ (RGX critical downstream — unchanged via `make regex_broader_corpus_proof_gate`); SV shape-contract GREEN; **SV external corpus `6/14 → 8/14`** — veer_el2_lsu ×{2017,2023} now PASS via the bootstrap-files+library-artifact path (THE corpus-mover for veer, exactly as projected). Residual 6 fails: uvm_pkg/uvm_compat_pkg ×{2017,2023} fail at `.3.3.3` use-site `known_unscoped_*` deeper still — they are SELF-CONTAINED files (no separate `import` declarations the way veer does); friscv_rv32i_core ×{2017,2023} statement-level (`.3.3.6`). The `1.0.116`/schema `2`, `1.0.115`/schema `1`, the AST-Shape-Corrections-1.0.116, the DOC-ENVELOPE / earlier slice history are retained below.)
- Current grammar family label:
  - `systemverilog`
- Current stable host profiles:
  - `sv_2017`
  - `sv_2023`
  - `verilog_2005` (strict IEEE 1364-2005 subset dialect profile on the same grammar — see § "Dialect Profile — `verilog_2005`")
- Current live status:
  - Tracked in `docs/book/src/roadmap-and-live-status.md`

## Current Trust Statement
- The PGEN `systemverilog` parser is **closure-grade for the current Nexsim-facing scope** when consumed through the stable `pgen::embedding_api` host surface.
- Closure is established via the family status / contract / telemetry gates listed under "Validation / Release Gates" below.
- The current sign-off bar is Nexsim-facing SystemVerilog parsing, not an open-ended promise for every imaginable SystemVerilog dialect or tool ecosystem.
- The grammar covers IEEE 1800-2017 (`sv_2017` profile) and the IEEE 1800-2023 delta (`sv_2023` profile). Both profiles share `grammars/systemverilog.ebnf` as the single source of truth.
- The grammar additionally carries a strict **`verilog_2005`** (IEEE 1364-2005) subset dialect profile on that same single source of truth (`VERILOG-2005-PROFILE`, 2026-07-02): plain Verilog-2005 parses, the SystemVerilog-only surface rejects, and keyword reservation follows IEEE 1364-2005 Annex B both ways. Its machine-checkable conformance surface is `make -C rust SHELL=/bin/bash verilog_2005_conformance_gate`; its trust posture is **Mostly Done** (curated-corpus proof; the all-profile over-REJECTION defects `SV-0021`/`SV-0022`/`SV-0023` are now ALL FIXED (`SV-0021` at `1.0.163`, `SV-0022` at `1.0.166`, `SV-0023` — the event-trigger `->>`/delay/array complex — at `1.0.167`, schema `15`→`16`); the all-profile over-ACCEPTANCE leaks `SV-0024`/`SV-0028` are now BOTH FIXED at `1.0.165` (`SV-LRM-SHAPE-FIDELITY.1`/`.2`, 2026-07-04 — `SV-0028` re-adjudicated `verilog_2005`-only, a stray `;` being a legal IEEE 1800 empty package item); `SV-0029` and `SV-0030` are FIXED — `SV-0029` in full at `1.0.161` (the 12 timing checks at `1.0.159`, the SV-only `$root`/`$unit`/severity/bare-`$` group at `1.0.161` with 4 more corpus locks), `SV-0030` at `1.0.160` with 4 corpus locks; the profile-boundary leaks `SV-0025`/`SV-0026`/`SV-0027`/`SV-0031`/`SV-0032`/`SV-0033`/`SV-0034` are FIXED with corpus reject-locks — `SV-0026`, the widest, closed at `VERILOG-2005-PROFILE.6.3`, 2026-07-02; `SV-0032` (the `void` surface) at `.6.8`, `SV-0033` (the dynamic-array `[]` unsized dimension) at `.6.9`, and `SV-0034` (the SV-only associative-array/queue dimensions) at `.6.10`, all 2026-07-03; and `SV-0031` (the `::` scope-resolution surface — the LAST `verilog_2005`-only leak) at `.6.11`, 2026-07-04, by whole-rule `@profiles` gates on the `::` token root plus its 26 linter-derived orphan-cascade consumers; a distinct ALL-PROFILE leak `SV-0035` — a reserved *type* keyword (`integer`) parsing as a bare primary expression under every profile — was found tools-first while fixing `SV-0034` and is now FIXED at `1.0.162` (`SV-KEYWORD-PRIMARY-FIDELITY.2`); its moved-leak re-probe surfaced the new `SV-0036` (net/gate/structural SV keywords leaking as primaries under `sv_2017`/`sv_2023` only — a reserved-list-completeness residual) — now FIXED at `1.0.164` (`SV-KEYWORD-PRIMARY-FIDELITY.3.2`, with its `SV-0037` implicit-ANSI-port prerequisite at `1.0.163`), closing the reserved-keyword-as-primary family) — see § "Dialect Profile — `verilog_2005`" for the full write-up and the honest boundary.
- **Accept-set note (GRAMMAR-WELLFORMED.H.12.5.3, 2026-06-15 — no release/schema bump; stays `1.0.139` / schema `3`):** the `source_text_item` typed union no longer carries the `comment_only_source_region` kind (its `comment_only_source_region` rule, and the now-orphaned `white_space` rule, were removed from the grammar). Both were *engine-shadowed-dead* — the generated layout skipper consumes whitespace and comments as leading trivia before those rules can match, so neither ever appeared in any accepted parse (a comment-only file parses to an empty `source_text`). This is an **accept-identical correction** narrowing the declared `source_text_item` union from eight kinds to the seven that can actually occur (`description`, `local_parameter_declaration`, `parameter_declaration`, `package_import_declaration`, `timeunits_declaration`, `compiler_directive`, `semi`); every input that parsed before yields a byte-identical AST. The historical `comment_only_source_region` mentions in the release-highlight sections below are retained as dated records of the earlier grammar.
- **Recognized certificate-coverage basis (GRAMMAR-WELLFORMED.H.12.8.5.2, `PGEN-GRAMMAR-WELLFORMED-0147`, 2026-06-30 — internal trust figure; NO release/schema bump; no consumer-visible parser change):** SystemVerilog's honest internal "how close to fully certified" figure is the sound multi-config certificate-coverage **union**. As of `VERILOG-2005-PROFILE.6.7` (2026-07-05, the per-profile `proof` promotion) the canonical single-config (`sv_2017`) accounting is `proof=10 witness=1321 UNKNOWN=12` (the 8 `sv_2017`-profile-entry-unreachable SystemVerilog-only rules are now per-profile `proof`, down from `UNKNOWN=20`) and the recognized 4-config union is `proof=10 witness=1332 UNKNOWN=1`, the residual being exactly the one remaining store-gated reach-gap `context_member_method_call` (unchanged — the union invariant is preserved because proof gathering is canonical-only, so `union_proof == canonical_proof`, and the 8 rules are proof-under-`sv_2017` + witness-under-`sv_2023`). *(Count pins re-baselined 2026-07-02, `VERILOG-2005-PROFILE.5`: the `SV-AST-SHAPE-FIDELITY` + `verilog_2005`-profile named-lift campaigns added 20 accounted rules — canonical `total=1324 proof=2 witness=1302`, union `witness=1321`; then `+2` at `.6.2` for the `delay_value_sv_only`/`primary_literal_sv_only` lifts — `1326/1304`, union `1323`; then `+2` at `.6.3` for the `description_unit_item_sv_only`/`source_text_item_unit_sv_only` lifts — canonical now `total=1328 proof=2 witness=1306`, union `witness=1325` — with the semantic invariants unchanged throughout: canonical `UNKNOWN=20`, union `UNKNOWN=1`, the same single residual rule, `sample_parse_failures=0`, deterministic seeds 0/7/42.)* That figure is now locked by a re-runnable deterministic gate — `make -C rust SHELL=/bin/bash sv_cert_recognized_union_gate` (tracked contract `rust/test_data/grammar_quality/systemverilog_recognized_cert_union_contract.json`), asserting the canonical/union numbers + the residual rule set + determinism across seeds 0/7/42 + `sample_parse_failures=0`. **⭐ 2026-07-22 (`STRUCTURED-WITNESS-SYNTH.3`/`.4`, `PGEN-STRUCTURED-WITNESS-SYNTH-0004`/`-0005` — generator-only; NO release/schema bump; no consumer-visible parser change): the union `1 → 0` flip has HAPPENED.** The last residual (`context_member_method_call`) is witnessed by the structured-witness composition pass (`generate_structured_witnesses` — the declare-then-use prelude + head-leaf name pin + target-own structure forcing composed into one reach plan, the real parser the sole judge); the gate is GREEN at the re-baselined contract: canonical `proof=10 witness=1322 UNKNOWN=11` (the 11 = the entry-relative library/include/parseable cohort, each union-covered under its own entry), union `proof=10 witness=1333 UNKNOWN=0`, residual `[]`, deterministic seeds 0/7/42, `spf=0` — the contract's `done_rule` fires and **SystemVerilog is recognized `fully_certified` on the sound multi-config union basis** (delivering `GRAMMAR-WELLFORMED.H.12.8.3.2`). This remains an internal trust statement about certificate coverage — the parser's consumer-facing support boundary and the family status-gate axis (`sv_parser_family_status_gate`, still `Mostly Done` on its own `focused_replay_target_debt_zero` criterion) are unchanged.

## Companion Documentation — SystemVerilog Parser Integration mdBook
- The systemverilog-parser integration mdBook lives at `docs/systemverilog_parser_book/` and is the **canonical AST reference** for downstream consumers (Nexsim in particular).
- The book documents: build recipe, public API, the AST envelope, every annotated/un-annotated rule shape (as the annotation campaign progresses), per-feature worked examples, schema versioning, glossary, and a release-by-release index.
- Build it with `make systemverilog_parser_book_gate` (uses `mdbook build docs/systemverilog_parser_book`).
- Where the book and this contract disagree, **the contract wins** for compliance — but please report the disagreement as a documentation bug.

## Front End — the preprocess→parse pipeline (`SVPP-EXPANSION.7`, 2026-09-09; `.8` 2026-09-14; no release / schema bump — the grammar and the generated parser are unchanged at `1.0.230` / schema `33`)

The parser this contract describes reads SystemVerilog **after** the compiler directives of IEEE 1800-2017
clause 22 have been executed. Raw text with `` `include ``, `` `define `` and `` `ifdef `` in it is
grammar-development scaffolding (the raw corpus lane); the **release reading** of any file is the
preprocess→parse chain below, and every corpus number this contract's companion book publishes for that
chain is derived by that chain (`docs/systemverilog_parser_book/src/preprocess-parse-pipeline.md`; the
platform book's verdict-coverage anchor carries the raw tuple and the tuple under the pipeline side by side).

- **`ENGINE-UNIVERSAL-SERVICES.113` (2026-09-29; no release / schema bump — no verdict and no tree moved)**: the
  preprocessor is the SystemVerilog family's own component (`rust/src/families/systemverilog/preprocessor.rs`,
  its §22.14 tables beside it), published to the engine as one neutral preprocessing stage
  (`rust/src/preprocessing_stage.rs`); the CLI spellings below are the neutral ones — `--preprocess <family>`,
  `--include-dir`, `--symbol` (alias `--define`; the concept is every preprocessor's, the word one family's — director, 2026-09-29), `--unit-file`, `--profile`, `--stage-opt KEY=VALUE` — and the pre-`.113`
  spellings (`--preprocess-systemverilog`, every `--sv-…` flag) are accepted through release `1.0.247`, each
  rewritten with one deprecation line on stderr, and removed after it.
- **The single command** (the probe, `rust/src/bin/parseability_probe.rs`):
  `parseability_probe --parse systemverilog FILE --profile sv_2017|sv_2023 --preprocess [--include-dir DIR]… [--symbol NAME[=VALUE]]… [--unit-file FILE]… [--stage-opt KEY=VALUE]… [--lib-in DIR] [--lib-out DIR]`.
  Exit `0` = accepted, `1` = the parser rejected the expanded text (the message carries `furthest_position=N`
  and `[origin=FILE:LINE:COL]`), `3` = the preprocessor refused (`preprocess_failed …`, never `1` — a corpus
  lane keys on exactly that difference). The expansion alone: `ast_pipeline INPUT.sv --preprocess systemverilog
  [--output OUT.sv] [--source-map-json MAP] [--event-log-json LOG] [--diagnostics-json DIAG]` with the same
  context flags. Both are embedding-API-free CLIs; the runtime module is
  `rust/src/families/systemverilog/preprocessor.rs`, reached through `pgen::preprocessing_stage::stage_for("systemverilog")`.
- **The context is the consumer's, and it is four things**: the include search directories
  (`--include-dir`, in order), the predefined macros (`--symbol`, in order, exactly as `` `define NAME
  TEXT `` before the first line), the compilation unit's earlier files (`--unit-file`, in order — §22.5.1: a
  macro is unit-wide once defined; their macro tables are folded into one, their text is not parsed), and the
  library (`--lib-in`, below). The corpus lane DERIVES all four from the corpus itself
  (`stimuli/sv/pipeline_context.py` → `stimuli/sv/pipeline_context{,_files,_defines,_units}.tsv`, and
  `stimuli/sv/pipeline_library.py`); a downstream consumer supplies its own from its build declaration. The
  lane never guesses a context: an ambiguous include, a computed define, a `-f` list, a glob, a package two
  files declare differently are REPORTED and left unapplied — and, since `SVPP-EXPANSION.4d` (2026-10-06), an
  include candidate is any file of the tree whatever its suffix (§22.4 constrains neither), a tie between copies
  of one text resolves per file, and a directory that would resolve an ambiguous name by the side door is never
  emitted (a suite directory in that position refuses the derivation; a per-file one is withheld for its includer).
- **What the expansion stage guarantees** (the `SVPP-EXPANSION.5` / `.6b.2` / `.6b.5` rules, each with its
  clause and a corpus reproducer in `docs/tasks/SVPP-EXPANSION.md`): object and function-like macros with
  actuals and defaults (§22.5.1), the `` `` `` paste and `` `" `` / `` `\`" `` string forms — the paste with
  the ORDER §22.5.1's *"allowing identifiers to be constructed"* implies (`SVPP-EXPANSION.5`(i), 2026-09-15): an
  operand that is a defined macro usage is expanded before the join, an undefined usage joined by a paste is
  looked up as the WHOLE constructed name in place, and a constructed function-like usage collects its
  arguments unexpanded (Verilator's and sv2v's readings; Icarus differs and is recorded) — and the `` `" `` region
  rendered as ONE string literal in EXPANSION text on both the object- and the function-macro path, with its embedded
  macro usages expanded and `` `\`" `` rendered `\"` (`SVPP-EXPANSION.5`(j), 2026-09-15 — the clause's three things;
  a region in a SOURCE line is left as written, the form being defined for macro text), and `()` read as one empty
  actual when the macro declares formals (`.5`(j′)), and a plain `"…"` string in macro text OPAQUE — no formal
  substituted, no paste joined — as §22.5.1 `:370-372` and its own `H(x) "Hello, x"` read it (`.5`(k), 2026-09-15; the
  two tool readings are `--stage-opt plain_string_substitution=paste_delimited` (OpenTitan's marked-formal idiom) and `all`
  (Icarus's), never the default) —, a `$ident` never
  substituted (A.9.3), a missing or extra actual REFUSED (`E_SVPP_MISSING_MACRO_ARGUMENT`,
  `E_SVPP_EXTRA_MACRO_ARGUMENT`; §22.5.1 *"shall be an error"*), direct or indirect recursion REFUSED
  (`E_SVPP_RECURSIVE_MACRO`, §22.5.1 — by PROVENANCE: a usage the caller wrote starts its own expansion, a
  usage a macro body produced is checked against the stack, so OpenTitan's nested `` `DV_SPINWAIT `` expands
  and `` `define A(x) `B(`A(x)) `` is refused), `//` comments stripped from macro text (§22.5.1; block comments
  kept) and, since `SVPP-EXPANSION.5`(l) (2026-10-02), from an ACTUAL argument when it is bound (its text had been
  substituted onto the expansion's single line, commenting out the call's tail; block comments and a `//` inside a
  string literal stay), an `` `include `` FOLLOWED BY A MACRO expanded first and the result read as Syntax 22-1's
  `"filename"` / `<filename>` (§22.5.1: *"The `` `include `` directive can be followed by a macro, instead of a string
  literal"* — `SVPP-EXPANSION.5`(m), 2026-10-02; through `1.0.248` the form was refused as a tool extension, a ruling
  retracted; a macro whose text is not one of the two forms is still refused, naming both clauses), a well-formed `` `line `` consumed (§22.12; a malformed one passes through) — its parameters expanded first when they are macro usages (1800-2023 §22.2: *"Macro expansion shall occur within a compiler directive"*; `SVPP-EXPANSION.5`(n), 2026-10-05) —, a comment between the formals of a `` `define `` refused naming §22.5.1 (*"separated by commas and optionally white space"*; `.5`(o) — the verdict is not new, the message is), three readings selected by the EDITION of the profile (`SVPP-EXPANSION.5`(p), 2026-10-05; `sv_2023` / `sv_2023_tolerant` read IEEE 1800-2023, every other profile and the unprofiled reading read 1800-2017) — under 2023 a block comment in macro text is not substituted, a block comment carries a `` `define `` across its newlines, and a macro usage as the argument of `` `ifdef `` / `` `ifndef `` / `` `elsif `` / `` `undef `` is expanded first (§22.5.1, §22.2: *"Macro expansion shall occur within a compiler directive"*); under 2017 the comment is substituted, a `` `define `` whose text ends inside a block comment is refused naming the clause, and the argument is read as written —, include nesting bounded by DEPTH (64 levels; §22.4 *"shall be finite … at least 15"*) rather than by a file's reappearance on the include stack, so a guarded self-include is read and an unguarded loop is refused naming the clause (`SVPP-EXPANSION.5`(q), 2026-10-06), an ESCAPED identifier accepted as a macro name in definition and in usage, read without its backslash (§22.5.1 / §5.6.1; `.5`(q)), `` `pragma protect
  begin_protected `` … `` end_protected `` consumed as an envelope (§34; unterminated →
  `E_SVPP_UNTERMINATED_PROTECTED_ENVELOPE`), block comments spanning lines and following an `` `include ``, the
  fifteen `` `SV_COV_* `` coverage-control macros predefined (§40.3.1), conditional groups with the
  `--stage-opt conditional_symbol_policy=…` (`E_SVPP_UNDEFINED_CONDITIONAL_SYMBOL` under the strict policy) and macro
  redefinition under `--stage-opt macro_redefine_policy=…` (`E_SVPP_MACRO_REDEFINED` when disallowed); since
  `SVPP-EXPANSION.5`(s)–(v) (2026-10-06): macro text that leaves a plain STRING LITERAL open at its end is refused
  naming §22.5.1 (*"shall not be split across … String literals"*, the standard's own `first_half` example), a
  COMPILER DIRECTIVE cannot be redefined as a macro name and IS a defined name to `` `ifdef `` / `` `ifndef `` /
  `` `elsif `` (§22.5.1 *"All compiler directives shall be considered predefined macro names"*; the set is the
  edition's own — 1364-2005's nineteen under a Verilog profile, plus `undefineall`, `__FILE__`, `__LINE__` under
  1800), a backslash-newline INSIDE a string literal of the macro text omits both characters under the 2023 reading
  (1800-2023 §22.5.1) and becomes a newline under 2017, and a formal's default argument may hold `(`, `)` or `,`
  inside a string literal; and since `SVPP-EXPANSION.6f` (2026-10-06) the stage's POST-CONDITION: no backtick
  survives expansion outside a comment or string literal unless it is a directive the parser reads — an undefined
  name (§22.5.1: a usage means substitution; "neither a defined macro nor a compiler directive"), a function-like
  macro used without its argument list or with an unclosed one (§22.5.1 *"the parentheses are always
  required"*, Syntax 22-3), a `` `" `` / `` `\`" `` / `` `` `` outside macro text, and an `` `end_keywords ``
  closing no region are refused with their clause — and since `SVPP-EXPANSION.6h` (2026-10-08) so is a backtick
  before white space, a line end, the end of the text, a digit or punctuation (§22.2, Syntax 22-3: the character
  has meaning only before a name), while an escaped identifier (§5.6.1) is one token the scan steps over, a
  backtick inside it included, and an escaped usage `` `\name `` is looked up by its name; `--stage-opt
  undefined_macro_policy=keep` restores the
  passthrough for a consumer that resolves the rest itself (the SV stimuli quality gate replays its GENERATED
  raw-lane text under it by contract, `SVPP-EXPANSION.6g`; no corpus lane does). Every event is
  logged (`Define Undef Include Ifdef Ifndef Elsif Else Endif PassthroughLine SkippedLine MacroExpand
  LineDirective ProtectedEnvelope KeywordsRegion`).
- **`` `begin_keywords `` regions are APPLIED, and the profile reaches this stage** (`SVPP-EXPANSION.8`,
  IEEE 1800-2017 §22.14). A region selects which identifiers are reserved; the generated parser reserves one
  set per PROFILE and cannot switch mid-file, so the expansion renders the difference as text — every token
  the parse profile reserves and the region's `version_specifier` does not is re-emitted as an ESCAPED
  identifier (`reg [63:0] logic;` → `reg [63:0] \logic ;`), which §5.6.2 makes not a keyword and §5.6.1 makes
  the same identifier as the unescaped spelling. ⛔ **A consumer walking the AST sees that escaped spelling**
  at a rendered site and the source map points back at the bytes the author wrote. The profile is
  `--profile` on both commands (`ast_pipeline --preprocess systemverilog --profile P`; `--sv-keyword-profile` accepted through `1.0.247`);
  omitted, the parser's unprofiled reading is used, which reserves IEEE 1800-2017 Table B.1. Regions NEST
  and are compilation-UNIT state — they carry across `--unit-file` boundaries, as the clause requires.
  The per-version tables are the standard's own (Table 22-1 … 22-6 and §22.14.4's `"1364-2001-noconfig"`
  exclusions), carried in `rust/src/families/systemverilog/keyword_versions.rs` (the family's own component since `.113`), derived from the in-repo LRM text and held
  equal to it on every commit by the `SV-KEYWORD-VERSIONS` doctrine. Refusals and reports:
  `E_SVPP_UNKNOWN_KEYWORDS_VERSION` (§22.14: *"It shall be an error if an implementation does not recognize
  the version_specifier"*), `W_SVPP_KEYWORDS_VERSION_ABOVE_PROFILE` (the region reserves MORE than the
  profile — text can unreserve a keyword, it cannot reserve a non-keyword, so the region is applied only in
  the direction text can express; ⭐ **that direction cannot produce an over-acceptance, and it is GATED**:
  a specifier is admitted by a profile exactly when the profile's own edition lists it, and an edition lists
  only versions at or below itself, so `admitted ⇒ subset ⇒ never above` — the `SV-KEYWORD-VERSIONS`
  doctrine's `admitted_specifiers_are_subsets` arm re-derives that implication from the LRM tables on every
  commit, so a future edition that broke it would fail the gate rather than turn this warning live), `W_SVPP_KEYWORDS_VERSION_NOT_IN_EDITION` (the profile's own edition does
  not define that specifier; the parser then refuses the directive), `W_SVPP_UNMATCHED_END_KEYWORDS`,
  `W_SVPP_MALFORMED_KEYWORDS_DIRECTIVE`. Event kind `KeywordsRegion` records each directive, the words the
  region unreserves and the tokens it rendered.
- **The source-map contract** (`origin_of_position`): the preprocessor appends one source-map chunk per
  emitted line; a rejection's `[origin=FILE:LINE:COL]` is the chunk holding the rejection's byte offset in the
  EXPANDED text — an `` `include ``d line names the included file, a macro expansion names the line of the
  macro's USE (the expansion has no line of its own), the column is the chunk's first column plus the
  characters (not bytes) from the chunk's start, a position at or past the end of the text lands on the last
  chunk's end, and an unmappable position says `origin=unknown` rather than nothing. ⚠️ **The origin is the
  last MAPPED byte at or before the rejection, not necessarily the offending token**: when the parser's furthest
  position lies past a legal expansion (a `` `FOO `` that expands to a string literal, followed on the next
  line by an underivable item), the origin names the expansion's line and the construct that failed is the
  furthest TOKEN — a reader buckets on the token, not on the mapped line (`SVPP-EXPANSION.6b.4`).
- **The library**: `--lib-out DIR` writes `DIR/package/<name>.facts.json` for every package a PASSING parse
  declares (`@export_to_library` on `package_declaration`; `{format_version, kind: "package", name, facts: [{kind:
  "type_name", name, attributes: [{key: "declaration_family", value: typedef|class|…}]}, …]}`); a rejected
  parse exports NOTHING. `--lib-in DIR` reads `DIR/package/<name>.facts.json` when the text imports the package
  (`@import_from_library` on `package_import_item`, both `import X::*` and `import X::name`), and the parser's
  fact-gated productions (`known_unscoped_data_type_identifier`, `known_unscoped_base_class_type_identifier`,
  …) admit the imported names. A design is therefore parsed package-first, in dependency order, each package
  with `--lib-in` of its predecessors' exports and `--lib-out` of itself (`stimuli/sv/pipeline_library.py` does
  exactly this for the corpus's design suites; the same tool is the reference for a consumer's own flow).
  Neither flag may be given twice; the directory must exist.
- **Trust boundary of this section**: the pipeline's numbers are the SV parser book's, re-derived by
  `PIPELINE-LANE-CURRENCY` on every commit; what the pipeline still refuses is published there by construct
  and by origin, never hidden — including the UVM library's dependence on the `.5`(d) ruling (`const local`,
  `uvm_transaction.svh:443`), which gates every UVM-based unit.

## Dialect Profile — `verilog_2005` (strict IEEE 1364-2005 subset; `VERILOG-2005-PROFILE`, 2026-07-02; no release/schema bump — stays `1.0.158` / schema `13`)

### What it is
- A strict IEEE 1364-2005 (Verilog) parsing profile carried by the **same** `grammars/systemverilog.ebnf` single source of truth — realized purely by `@profiles: [...]` semantic-annotation gating, with **no separate parser, no fork, and no out-of-band validator**.
- Select it exactly like the existing profiles:
  - embedding API: profile string `verilog_2005` (aliases `1364-2005`, `ieee1364-2005`, `ieee_1364_2005` — grammar-declared), resolved as `GrammarFamily::from_str("systemverilog")?.profile("verilog_2005")` since embedding API `2.0.0` (the `GrammarProfile::Verilog2005` variant of `1.3.0`–`1.3.6` is gone); available since embedding API `1.3.0`,
  - CLI: `parseability_probe --parse systemverilog file.v --profile verilog_2005`.
- Because IEEE 1364-2005 is (nearly) a strict syntactic subset of IEEE 1800, the profile **rides the `sv_2017` baseline** of shared core constructs; its job is *enforcement*: reject the SystemVerilog-only surface that would otherwise silently parse.

### What a downstream consumer may rely on
- **Subset enforcement, proven both ways** by the repo-standard gate `make -C rust SHELL=/bin/bash verilog_2005_conformance_gate` (script `rust/scripts/verilog_2005_conformance_gate.sh`; tracked contract `rust/test_data/grammar_quality/verilog_2005_conformance_contract_v0.json`; curated corpus `rust/test_data/grammar_quality/verilog_2005_conformance/` — 12 accept + 42 reject files):
  - every accept-corpus file (modules, generate/genvar, parameterized counters, gate instantiation, tasks/functions, event triggers, core loop forms, `real`/`realtime`, keyword-as-identifier cases such as `reg bit;` and `wire logic;`) **parses** under `verilog_2005`;
  - every reject-corpus file (classes, packages/imports, interfaces/modports, programs, assertions, properties, covergroups, `typedef`, `int`-family declarations, `++`, `return`-with-argument, `final`, `timeunit`, `unique case`, `alias`, `randcase`, `always_comb/latch/ff`, `do…while`, `foreach`, `bit`/`logic`/`shortreal` as types, `interconnect`, `ref`/`const ref` directions, `wait fork;`, `wait_order(…) else …`, the `-> #5 e;` delay-trigger form) **rejects** under `verilog_2005` while still **parsing** under `sv_2017`/`sv_2023` (the leak-free profile boundary) — the sole exception is `bind_dir.sv`, which rejects under every profile per the pre-existing ledgered `SV-0022`;
  - the full per-file × per-profile expectation matrix (162 checks) is pinned in the contract JSON with spec-derived notes on every non-obvious row (extended `.6.2` with the 4 `SV-0025`/`SV-0027` reject-locks: `wire #1step w;`, `wire #10ns w;`, `assign w = 10ns;`, `assign w = '0;`; extended `.6.3` with the 4 `SV-0026` top-level reject-locks: `wire w;`, `reg r;`, `localparam p = 1;`, `parameter p = 1;`);
  - **keyword reservation is profile-faithful both ways**: the full IEEE 1364-2005 Annex B keyword set is reserved under `verilog_2005` (so `initial assert (1);` parses as a *task enable*, never as a bogus module instantiation), while the ~48 SV-only reserved words (`logic`, `bit`, `int`, `class`, `typedef`, `do`, …) parse as ordinary Verilog-2005 identifiers;
  - profile-alias normalization is spot-locked in the same gate (`ieee1364-2005` accept probe, `1364-2005` reject probe).
- **Wellformedness coherence, mechanized**: the gate asserts `ast_pipeline grammars/systemverilog.ebnf --lint-grammar` exits `0` with **0 `verilog_2005` profile-orphans** (a rule active under the profile whose every production references a profile-absent rule would be an unsatisfiable defect — the build-to-coherence invariant from `VERILOG-2005-PROFILE.3`/`.4.1` cannot silently regress).
- **AST shape**: an input accepted under `verilog_2005` produces the same schema-`13` typed AST it would produce under `sv_2017` (the profile admits/gates rules; it does not reshape them). The `sv_2017`/`sv_2023` languages and ASTs are proven **byte-invariant** to the whole profile campaign (24/24 before→after AST dumps; release/schema unchanged).
- **Profiled certificate-coverage baseline (internal trust figure, pinned in the same gate)**: `--report-certificate-coverage --grammar-profile verilog_2005 --entry-rule systemverilog_file --count 40` (with the declared entry universe wired via the gate's `union_configs` — `sv_multi_entry_root`/`library_text`/`systemverilog_parseable_file`) reports `total=1117 proof=327 witness=773 UNKNOWN=17 (sample_parse_failures=0, proof_reverify_failures=0)`, deterministic across seeds 0/7/42. Read this honestly: as of `VERILOG-2005-PROFILE.6.7` (2026-07-05, the per-profile `proof` PROMOTION) the cert now **PROVES** the profile-excluded surface rather than leaving it UNKNOWN — the run promotes the 298 profile-entry-unreachable SV-only rules (stranded by the pruning of every context that could reach them) + the 25 store-unproducible use-sites (their mandatory store-gate needs a fact-kind no profile-live rule can emit) into per-profile `proof` certificates (`ProfileEntryUnreachable` / `ProfileUnproducibleGate`, each independently re-derived), so `proof` rose `4→327` and `UNKNOWN` dropped `340→17`. That 17 is the **genuine remainder** — the entry-relative library/include/parseable cohort (never branded dead, since the analysis quantifies over the declared entry universe) + `context_member_method_call` + 5 in-profile witnessable ratchet targets — a regression lock on the honest residual, not a mixed dead-rule pile. (Historical pin evolution: `826/310`→`817/319` at `.6.2`, `817/319`→`809/327` at `.6.3`, `809/327`→`1147/819/326` at `SV-DOLLAR-LRM-FIDELITY.3`, `1147/4/816/327` at `.4`, `1144/4/813/327` after the `.6.9`/`.6.10` dimension gates, `1117/4/773/340` after the `.6.11` `::` gate, then `1117/327/773/17` at `.6.7`.) Ratcheting the genuine remainder is tracked follow-up work (`VERILOG-2005-PROFILE.6`), and any drift in either direction fails the gate.

### Honest boundary (ledgered, each with its own fix leaf)
- `SV-0025` (found 2026-07-02 by the `VERILOG-2005-PROFILE.6.1` profiled-cert residual adjudication) — **FIXED same-day (`.6.2`)**: the SystemVerilog-only `delay_value` alternatives (`1step`, time literals) were un-gated, so `wire #1step w;` and `wire #10ns w;` wrongly accepted under `verilog_2005` (both legal IEEE 1800, SV profiles unaffected). Now branch-lifted into the gated `delay_value_sv_only`; 2 corpus reject-locks pin the rejects; AST byte-invariant on the SV profiles.
- `SV-0027` (found while pinning the `SV-0025` fix locus) — **FIXED same-day (`.6.2`)**: `primary_literal`'s SV-only `time_literal`/`unbased_unsized_literal` alternatives were un-gated, so `assign w = 10ns;` and `assign w = '0;` wrongly accepted under `verilog_2005`. Now lifted into the gated `primary_literal_sv_only`; 2 corpus reject-locks.
- `SV-0026` (same adjudication) — **FIXED (`.6.3`, 2026-07-02)**: SystemVerilog's `$unit` compilation-unit surface was active under `verilog_2005`, so bare top-level declarations (`wire w;`, `reg r;`, `localparam p = 1;`, `parameter p = 1;` outside any module) wrongly accepted (IEEE 1364-2005 A.1.2 allows only module/UDP/config at top level; the forms are legal IEEE 1800 `$unit` items, so the SV profiles are unaffected). BOTH carriers now branch-lifted behind `["sv_2017","sv_2023"]` gates (`description_unit_item_sv_only`, `source_text_item_unit_sv_only`); 4 corpus reject-locks pin the rejects; SV-profile parses byte-invariant. The first landing attempt was reverted on a witness-routing collateral, root-caused and fixed engine-side (`.6.3.1`/`.6.3.2` — the armed name-prelude integrity fix) before this replay landed cleanly with the recognized-union invariant intact.
- `SV-0032` (found 2026-07-03 by the `VERILOG-2005-PROFILE.6.6` machine residual-classification, probed tools-first) — **FIXED (`.6.8`, 2026-07-03)**: the SystemVerilog-only `void` surface was active under `verilog_2005` — the `void` function return type (`function void f;`) and the `void'(…)` cast (`initial void'(f());`) both wrongly accepted (`void` is categorically absent from IEEE 1364-2005 §10.4 / Annex A; both legal IEEE 1800, so the SV profiles are unaffected). Two ungated `kw_void_e9cede9b` surfaces (`data_type_or_void`'s return-slot + `subroutine_call_statement`'s cast) now branch-lifted behind `["sv_2017","sv_2023"]` gates (`data_type_or_void_sv_only`, `void_cast_statement_sv_only`); 2 corpus reject-locks; SV-profile parses byte-invariant. The re-pin: `verilog_2005` cert `1147/4/816/327` byte-identical (`kw_void_e9cede9b` reclassifies UNKNOWN-genuine → NO-reach, 295→296); canonical/union `1341`→`1343` (both lifts witnessed under `sv_2017`, UNKNOWN invariants unchanged); conformance matrix 192→198 (64→66 cases). No release bump (SV profiles AST-byte-invariant).
- `SV-0033` (found 2026-07-03 by the `VERILOG-2005-PROFILE.6.6` machine residual-classification, probed tools-first) — **FIXED (`.6.9`, 2026-07-03)**: the SystemVerilog-only dynamic-array `[]` (unsized) dimension was active under `verilog_2005` — `reg q [];` wrongly accepted (IEEE 1364-2005 §4.9 / Annex A array dimensions are always ranged `[msb:lsb]`; the bare `[]` unsized form is IEEE 1800 §7.5 only; legal IEEE 1800, so the SV profiles are unaffected). Fixed by a single whole-rule `@profiles: ["sv_2017","sv_2023"]` gate on the entirely-SV-only `unsized_dimension` rule (`:5731`, the `uniqueness_constraint` idiom) — ZERO new rules, all three carriers (`packed_dimension`, `variable_dimension`, and the `variable_decl_assignment` dynamic-array branch) pruned at once, `sv_2017`/`sv_2023` AST byte-identical; 2 corpus locks (`reject/dynamic_array_unsized.sv` + no-over-gate control `accept/array_ranged_dim.v`). The re-pin: `verilog_2005` cert `1147/4/816/327` → `1146/4/815/327` (`unsized_dimension` LEAVES the profile universe — total −1 — and drops its one leak-earned false witness — witness −1; `UNKNOWN` 327 / NO-reach 296 unchanged); canonical/union byte-identical (`1343/2/1321/UNKNOWN=20`, union `1343/2/1340/UNKNOWN=1` — no new rule, SV tree untouched); conformance matrix 198→204 (66→68 cases). No release bump (SV profiles AST-byte-invariant).
- `SV-0034` (found tools-first 2026-07-03 while fixing `SV-0033`) — **FIXED (`.6.10`, 2026-07-03)**: the SV-only associative-array (`[*]` / `[data_type]`, IEEE 1800 §7.8) and queue (`[$]`, §7.10) dimension alternatives of `variable_dimension` were active under `verilog_2005` (`reg q [*];` / `reg q [$];` wrongly accepted). Fixed by two whole-rule `@profiles: ["sv_2017","sv_2023"]` gates on the entirely-SV-only single-carrier rules `associative_dimension` (`:627`) / `queue_dimension` (`:4587`) — ZERO new rules; `variable_dimension` stays v2005-reachable via its ranged `unpacked_dimension` alt; SV profiles AST-byte-invariant. Re-pin: `verilog_2005` cert `1146/4/815/327` → `1144/4/813/327` (both rules leave the profile universe, dropping 2 leak-earned false witnesses; `UNKNOWN` 327 / NO-reach 296 unchanged); union byte-identical (`1343/2/1321/UNKNOWN=20`); conformance matrix 204→210 (68→70 cases). No release bump.
- `SV-0031` (found 2026-07-03 by the `VERILOG-2005-PROFILE.6.6` machine residual-classification, probed tools-first) — **FIXED (`.6.11`, 2026-07-04)**: the SystemVerilog-only `::` scope-resolution surface was active under `verilog_2005` — `initial p::f();`, `p::C::f()`, expression `wire w = p::X;`, and `import p::*;` all wrongly accepted (IEEE 1364-2005 has no `::` token anywhere in Annex A; legal IEEE 1800, so the SV profiles are unaffected). This was the LAST `verilog_2005`-only leak. Fixed by whole-rule `@profiles: ["sv_2017","sv_2023"]` gates on **27 rules**: the `::` token root `scope_resolution` (`:6601`, the single `::` producer) plus the 26 `::`-only consumer rules the multi-profile `--lint-grammar` orphan-coherence check DERIVED as newly-unsatisfiable once the token was gated (`package_scope`/`package_import_item`/`package_export_declaration`/`non_typedef_package_scope`, `class_scope`/`class_scoped_call_prefix`/`class_scoped_tf_call`, and the 15 `scoped_*` scope-prefix identifiers) — the well-formedness contract deriving the complete `::`-surface gate, reaching the 0-`verilog_2005`-orphan fixpoint in one wave; ZERO new rules, `sv_2017`/`sv_2023` AST byte-identical. 3 corpus reject-locks (`pkg_scope_call.sv`/`pkg_class_scope_call.sv`/`pkg_scope_expr.sv`). Re-pin: `verilog_2005` cert `1144/4/813/327` → `1117/4/773/340` deterministic seeds 0/7/42 (all 27 leave the profile universe and the token de-witnesses the `::`-reachable SV-only surface it falsely witnessed through the leak — witness 813→773, UNKNOWN 327→340, NO-reach 296→297; every de-witnessed rule is SV-only `::` surface, ZERO core-Verilog-2005 constructs); union byte-identical (`1343/2/1321/UNKNOWN=20`); conformance matrix 210→219 (70→73 cases). No release bump (SV profiles AST-byte-invariant). A distinct **ALL-PROFILE** leak `SV-0035` — a reserved type keyword (`integer`) parsing as a bare primary expression under EVERY profile (`localparam p = integer;` accepts under `sv_2017`/`sv_2023`/`verilog_2005`) — was surfaced here and has since been FIXED at `1.0.162` (`SV-KEYWORD-PRIMARY-FIDELITY.2`); the two all-profile over-acceptance leaks noted alongside it, `SV-0024`/`SV-0028`, are likewise now FIXED at `1.0.165` (`SV-LRM-SHAPE-FIDELITY.1`/`.2`).
- `SV-0024` — un-braced multi-identifier non-ANSI port expressions (`module m (a b);`) previously **accepted under every profile** (an LRM-extraction meta-notation defect: `port_expression`'s multi-reference alternative was extracted as un-braced `*`-repetition). **FIXED at `1.0.165` (`SV-LRM-SHAPE-FIDELITY.1`, 2026-07-04)**: restored the LRM shape `lbrace port_reference ( comma port_reference )* rbrace`, so `module m (a b);` now correctly **REJECTS under `verilog_2005`** (its only reading is the un-braced `port_expression`) and the braced concatenation port `module m ({a, b});` is now correctly **accepted under every profile** (the LRM's valid multi-reference form). Under `sv_2017`/`sv_2023` `module m (a b);` still ACCEPTS as a legitimate ANSI interface port (`a`=interface type, `b`=port name — valid IEEE 1800 syntax, an elaboration-time not-a-declared-interface check). The conformance corpus now locks both the bare-form reject (`reject/port_bare_multi_id.sv`) and the braced-concat accept (`accept/port_concat.v`). (The sibling `SV-0028` — a stray top-level `;` — was likewise re-adjudicated `verilog_2005`-only and FIXED in the same tree; see the next bullet. With both closed, the `SV-LRM-SHAPE-FIDELITY` tree is COMPLETE; no all-profile over-ACCEPTANCE leak remains — the strict-subset claim's remaining bound is the all-profile over-REJECTION defect `SV-0023` (`SV-0021`/`SV-0022` since FIXED — see their bullets below).)
- `SV-0028` — a stray top-level `;` previously accepted under `verilog_2005`. **FIXED at `1.0.165` (`SV-LRM-SHAPE-FIDELITY.2`, 2026-07-04, grammar-only; NO release bump — `verilog_2005`-only, SV profiles byte-invariant, AST-identical, the `SV-0026`/`SV-0031`..`SV-0034` precedent)**. Tools-first re-adjudication (AST dumps + IEEE LRM markdown) corrected an earlier "all-profile" framing: under `verilog_2005` a stray `;` is invalid (IEEE 1364-2005 A.1.2 `source_text ::= { description }`, `description ::= module_declaration | udp_declaration | config_declaration` — no null/empty top-level item), and it accepted only via `source_text_item`'s LRM-less direct `| semi -> {kind:"semi"}` alternative (`grammars/systemverilog.ebnf:231`); under `sv_2017`/`sv_2023` a stray `;` is the **legitimate LRM empty package-or-generate item** (`description ::= … | { attribute_instance } package_item`, `package_or_generate_item_declaration ::= … | ;` — IEEE 1800-2017 §26 / 2023 Annex A) and correctly stays ACCEPT (the PEG-first `description` path, which shadows `:231` under the SV profiles). Fixed by removing the LRM-less `:231` branch (ZERO rule census change — a branch, not a rule; census `1465`). `;` / `;;` / `module m; endmodule\n;` now REJECT under `verilog_2005`, unchanged ACCEPT under `sv_2017`/`sv_2023` (empty-item AST byte-shape-identical). Conformance corpus lock `reject/stray_top_level_semi.sv` (v2005 reject / SV accept). Cert byte-identical (`verilog_2005` `1117/4/773/340`, canonical `1343/2/1321/20`, union `1`; seeds 0/7/42) — the stray-`;` path never earned a v2005 witness.
- `SV-0029` (found 2026-07-02 by the `VERILOG-2005-PROFILE.6.4` re-adjudication) — **all-profile LRM-fidelity defect family, `Released` (fixed in full at `1.0.161`), fix tree `SV-DOLLAR-LRM-FIDELITY`**: 19 keyword-like `$` tokens matched the mangled literal text `sv_dollar_*` instead of the LRM `$*` spellings. **Wave 1 FIXED at `1.0.159`** (`.2`, 2026-07-02): the 12 specify timing-check tokens (core IEEE 1364-2005 §15 / 1800-2017 §31 surface), locked by 2 conformance-corpus rows. **Wave 3 FIXED at `1.0.161`** (`.4`, 2026-07-03, schema `14`→`15`): `$unit::name`, module-level severity tasks, `$root.`-anchored names/calls (now carrying the explicit anchor in the AST instead of the generic system-TF mis-route), and the bare-`$` primary — all ACCEPT under the SV profiles and REJECT under strict `verilog_2005`, locked by 4 more corpus rows + 6 shape samples. The mangled spellings are ordinary identifier text and keep their identifier readings (the pre-fix `sv_dollar_fatal;` acceptance was the implicit-variable-declaration reading — never a severity-host leak; dump-proven in `.4`).
- `SV-0030` (same adjudication; second site found by `SV-DOLLAR-LRM-FIDELITY.1`) — **FIXED at `1.0.160` (`SV-DOLLAR-LRM-FIDELITY.3`, 2026-07-02, schema `13`→`14`)**: `scalar_constant` AND the UDP `init_val` rule had lost their trailing digits vs the LRM (digit-less `1'b`/`1'B` prefixes instead of the LRM's `1'b0`-style alternatives), and the `scalar_timing_check_condition` eq/case_eq/ne/case_ne branches were doubly dead (PEG-shadowed by the bare-`expression` first branch AND starved by their own greedy full-`expression` lhs). Now: sequential-UDP initializers `initial q = 1'b0;` / `1'b1;` ACCEPT under every profile (the digit-less nonsense `initial q = 1'b;` REJECTS), and specify compare conditions `e == 1'b0` emit the typed `{kind:"eq", lhs, rhs}` shapes (see the schema-`14` note). Locked by 4 conformance-corpus rows (2 accept + 2 reject) and 4 shape-contract samples.
- `SV-0023` — the event-trigger `->>`/delay-form LRM-fidelity complex. **FIXED at `1.0.167` (`SV-OVER-REJECTION-FIDELITY.3`, 2026-07-05, schema `15`→`16`):** added the `->>` nonblocking-trigger token, split the event-trigger block into three profile-faithful rules, moved the optional `[ delay_or_event_control ]` onto the `->>` (nonblocking) branch, corrected the swapped `blocking`/`non_blocking` `kind` labels (IEEE 1800 §15.5.3/15.5.4), and gave the `verilog_2005` branch a bracket array-select. Now `->> e;` / `->> #5 e;` ACCEPT under `sv_2017`/`sv_2023` (REJECT under `verilog_2005` — SV-only); `-> #5 e;` correctly REJECTS under every profile; the IEEE 1364-2005 event-array trigger `-> e[0];` ACCEPTS under `verilog_2005`. Regression-locked in the conformance corpus (`reject/event_trigger_nonblocking.sv`, `reject/event_trigger_nonblocking_delay.sv`, `accept/event_trigger_array.v`, re-adjudicated `reject/event_trigger_delay.sv`). **With `SV-0021`/`SV-0022`/`SV-0023` all closed, no all-profile over-REJECTION defect remains bounding the strict-subset claim.**
- `SV-0022` — `bind` double-`semi`: a spec-valid single-`;` `bind … c1 ();` previously **rejected under every profile** (over-rejection). **FIXED at `1.0.166` (`SV-OVER-REJECTION-FIDELITY.2`, 2026-07-04, grammar-only)**: `bind_directive` (`grammars/systemverilog.ebnf:668`/`:670`) required a trailing `semi` after `bind_instantiation`, but each `bind_instantiation` alternative (`module_instantiation`/`program_instantiation`/`interface_instantiation`/`checker_instantiation`) already consumes its own `;` — the IEEE 1800-2017 §23 BNF's `bind … bind_instantiation ;` is a redundant double-`;` that every commercial tool resolves as a single `;`. Removing the trailing `semi` (uncaptured last token → NO schema change) makes `bind … c1 ();` ACCEPT under `sv_2017`/`sv_2023` and correctly REJECT under `verilog_2005` (`bind` is SV-only). Regression-locked by `reject/bind_dir.sv` (v2005 reject / SV accept). No-regression gates byte-identical (canonical `1343/2/1321/20`, v2005 `1117/4/773/340`, conformance 231/0, union `1`, ast_shape 1/1, external corpus 14/14).
- `SV-0021` — mixed implicit-type→typed ANSI ports (`module m (input a, output reg b);`) previously **rejected under every profile** (a pre-existing all-profile over-rejection: the ungated `net_type_identifier` in `net_port_type_sv_2017`/`_sv_2023` ate the implicit port's bare name under the global `longest_match` policy, corrupting the first port into a `nonansi` shape so a following explicitly-typed ANSI port had no valid continuation). **FIXED at `1.0.163` as a consequence of `SV-0037`** (`SV-KEYWORD-PRIMARY-FIDELITY.3.1` — routed both `net_port_type` alts to the store-gated `checked_nettype_identifier`), then **adjudicated + regression-locked at `SV-OVER-REJECTION-FIDELITY.1` (2026-07-04)**: the mixed implicit-then-typed ANSI header now parses under every profile, locked by `accept/ansi_implicit_then_typed_port.v` (accept under all 3 profiles; conformance matrix 231/0). No release bump (fixed at `1.0.163`; the lock leaf makes no parser change).

### Support boundary
- Bug reports against the `verilog_2005` profile follow the standard protocol (`docs/contracts/PGEN_PARSER_ISSUE_REPORTING_PROTOCOL.md`); classify "SV-only construct wrongly accepted under `verilog_2005`" as a **profile-leak** defect and include the construct's IEEE 1364-2005 Annex A / IEEE 1800 Annex A adjudication in the report.
- The profile shares the family's release/versioning stream (release `1.0.166`, schema `15`); a future profile-affecting grammar change bumps the family release exactly like any other SV change and must keep `verilog_2005_conformance_gate` green (or re-baseline its contract in the same commit with justification).

## Release 1.0.169 / Contract 1.0.169 Highlights — SV-CORPUS-GRAD.3.3: SVA implication operators `|->`/`|=>` restored (IEEE 1800-2017 A.2.10; ledger `SV-0039`); schema 16 → 17

The two core SVA sequence-implication operators — overlapped `|->` and
non-overlapped `|=>` (IEEE 1800-2017 A.2.10) — were **entirely absent** from
`grammars/systemverilog.ebnf`: no token, no `prop_primary` branch, never
present (`git log -S` empty). Every `sequence_expr |-> property_expr` /
`sequence_expr |=> property_expr` concurrent assertion — the most fundamental
SVA construct — wrongly REJECTED at the operator, while an implication-free
`assert property (@(posedge clk) p3)` parsed. Root cause of the omission: the
LRM-markdown→EBNF extractor split the `|`-prefixed operators on `|` (the EBNF
alternation metachar), leaving the mangled `implies` (`->`) / `or_assign`
(`|=`) remnants in `prop_primary`. Orthogonal to the H.12.5.8 infix-operator
fix, whose matrix tested only the *word* operators (`until`/`or`/`and`/
`intersect`/`within`/`##`).

**Fix (grammar-only):** two new profile-gated tokens
(`overlapped_implication := "|->"`, `non_overlapped_implication := "|=>"`,
gated `["sv_2017","sv_2023"]` like `nonblocking_implies`/`SV-0023` — SVA is
SystemVerilog-only, absent from IEEE 1364-2005) and the two A.2.10 branches
`sequence_expr <op> property_expr` mirrored into `prop_primary_sv_2017` and
`prop_primary_sv_2023` (census `1466` → `1468`, +2 token rules).

**What consumers see:** `sequence_expr |-> property_expr` and
`sequence_expr |=> property_expr` flip **REJECT → ACCEPT** under `sv_2017`/
`sv_2023` at every property-expression position (implication with a signal,
named-property, or delayed-sequence RHS), emitting
`{kind:"overlapped_implication", lhs, rhs}` /
`{kind:"non_overlapped_implication", lhs, rhs}` in `prop_primary`;
`verilog_2005` still REJECTS (SVA is SV-only, byte-inert). **Why schema bumps
16 → 17:** two new `prop_primary` branches carrying new return annotations are
added — a user-visible shape change; consumers exhaustively matching
`prop_primary`/`property_expr` kinds must add arms for the two new kinds.

**Surfaced corpus-first** by the external-corpus graduation campaign
(`SV-CORPUS-GRAD.3.2`: the refreshed rejects-valid family map — SVA
implication/property is the #1 construct family at 101 rows) — the SECOND
defect (after `SV-0038`) the campaign caught and closed end-to-end. **Full
external corpus 16,336: pass 9,361 → 9,433 (+72, timeout 9 → 9 unchanged),
ZERO per-suite regressions** (ispras +34, verilator +21, Surelog +6, opentitan
+4, sv-tests +4, iverilog +3 — all SVA-assertion files); adjudication baseline
rejects-valid 543 → 481, accepts-invalid 21 unchanged (no over-acceptance),
unexplained 564 → 502. Proof (all seeds
0/7/42): `sv_cert_recognized_union_gate` GREEN on an evidence-grounded
re-baseline (the 2 new tokens POSITIVELY WITNESSED — total 1343→1345, union
witness 1337→1339, UNKNOWN 11/0 residual `[]`, still `fully_certified_via_union`);
`verilog_2005_conformance_gate` GREEN byte-inert (cert `1115/328/773/14`,
matrix 240/0, orphans 0); `sv_stimuli_quality_gate` PASS
(`closed_loop_replay_targets_total` 120 → 126 — the 4 new branches + 2 tokens
are new closed-loop generation targets, feeding `SV-REPLAY-DEBT`); ast_shape
18/0; external-corpus triage 14/14. Full matrix in ledger row `SV-0039`.

## Release 1.0.168 / Contract 1.0.168 Highlights — SV-CORPUS-GRAD.3.1: `interface class` restored under `sv_2017` (IEEE 1800-2017 §8.26; ledger `SV-0038`); schema 16 unchanged

- **What changed (grammar-only, strictly more permissive):** the three
  `interface_class_declaration` consumer wirings existed only in the
  `*_sv_2023` variant rules; the `_sv_2017` twins omitted the branch, so
  `interface class Foo; … endclass` — an IEEE 1800-**2017** §8.26 construct —
  had NO derivation path under the `sv_2017` profile (rejected at the `class`
  keyword). Release `1.0.168` mirrors the branch into
  `anonymous_program_item_sv_2017`, `class_item_sv_2017`, and
  `package_or_generate_item_declaration_sv_2017`. Zero new rules (census
  `1466`).
- **What consumers see:** `interface class` declarations (plain,
  parameterized, `extends`-chained, `pure virtual` prototypes) flip
  **REJECT → ACCEPT under `sv_2017`** at compilation-unit scope, inside
  classes, and in anonymous programs. `sv_2023` behavior is byte-identical
  (it always parsed these). `verilog_2005` still rejects (SV-only construct).
  **Schema stays `16`** — the `interface_class_declaration` carrier and the
  `{kind:"interface_class"}` item shapes already existed via the `sv_2023`
  wiring; only previously-rejected input gains parses.
- **Surfaced by:** the external-corpus graduation campaign
  (`SV-CORPUS-GRAD.2`/`.3.0` adjudication + stuck-point clustering — the F1
  cluster, ~47 rejects-valid rows). First defect caught and closed end-to-end
  by that pipeline: full external corpus pass 3,049 → 3,091 of 5,128 (+42,
  zero per-suite regressions); adjudication baseline 321 → 279 unexplained.
- **Proof battery (all green):** `sv_stimuli_quality_gate` PASS
  (`closed_loop_replay_targets_total` 120 unchanged);
  `sv_cert_recognized_union_gate` GREEN on an evidence-grounded re-baseline —
  4 unreachability proofs convert to genuine witnesses (canonical
  `1343/6/1326/11`, union witness `1337`, UNKNOWN unchanged `11`/`0`,
  residual `[]`, deterministic seeds 0/7/42, spf=0);
  `verilog_2005_conformance_gate` GREEN byte-inert (orphans 0, matrix 240/0,
  cert `1115/328/773/14` byte-identical); `ast_shape_contract_gate` PASS;
  `sv_external_corpus_triage_gate` PASS; `--lint-grammar` clean (1,466 rules,
  all error classes 0). Full detail: ledger row `SV-0038`.

## Release 1.0.167 / Contract 1.0.167 Highlights — SV-OVER-REJECTION-FIDELITY.3: event-trigger `->>`/delay/array LRM complex restored (ledger `SV-0023` closed); schema 15 → 16

**What consumers see:** the IEEE 1800 §15.5 / A.6.5 event-trigger family is now LRM-faithful across profiles.
- The NONBLOCKING trigger `->> e;` (and `->> #5 e;` with a delay/event control) — previously **REJECTED** (the `->>` operator was absent from the grammar) — now **ACCEPTS** under `sv_2017`/`sv_2023` (REJECTS under `verilog_2005`; `->>` is SystemVerilog-only) and realizes `{kind:"non_blocking", control, name[, select]}`.
- The plain BLOCKING trigger `-> e;` still ACCEPTS everywhere but its `kind` label is **CORRECTED** from `non_blocking` to `blocking` (IEEE 1800 §15.5.3 — `->` is the blocking named-event trigger). **Consumers matching `kind:"non_blocking"` for a plain `-> e;` MUST migrate to `kind:"blocking"`.**
- `-> #5 e;` (a delay on the `->` blocking form) — previously **ACCEPTED** — now correctly **REJECTS** under every profile (per A.6.5 the optional `[ delay_or_event_control ]` belongs to `->>` only). Consumers that matched `kind:"blocking"` with a `control` field must migrate to the `->> #5 e;` `kind:"non_blocking"` shape.
- The IEEE 1364-2005 event-array trigger `-> e[0];` — previously **REJECTED** under `verilog_2005` — now **ACCEPTS** (A.6.5 `-> hei { [ expression ] } ;`). It also accepts under `sv_2023` (Annex A adds a trailing `nonrange_select`) and REJECTS under `sv_2017` (the strict 2017 BNF has no select).
- `event_trigger` now dispatches THREE profile bodies (`{kind:"sv_2017"|"sv_2023"|"verilog_2005", body}`).

**Root cause & fix (grammar-only + schema/release bump):** the event-trigger block (`grammars/systemverilog.ebnf:2152-2168`) had no `->>` token, mis-attached the optional control to a second `->` branch (`event_trigger_control_sv_only`), SWAPPED the `blocking`/`non_blocking` labels, and shared the `sv_2017` rule with `verilog_2005` (no array-select). Fix: added `nonblocking_implies := trivia "->>"` (gated `["sv_2017","sv_2023"]`); split into three profile-faithful rules — `event_trigger_sv_2017` (2017 A.6.5), `event_trigger_sv_2023` (2023 Annex A, trailing `nonrange_select`), `event_trigger_verilog_2005` (1364-2005 A.6.5, trailing `bit_select`); moved the optional control onto the `->>` (nonblocking) branch; corrected the labels; removed `event_trigger_control_sv_only`. Census `1465`→`1466`. **Why schema bumps 15→16:** the event-trigger return annotations are restructured and branches are added/removed (a user-visible shape change per the schema-versioning policy).

**Proof (all seeds 0/7/42):** `sv_cert_recognized_union_gate` GREEN (canonical `1343/2/1321/20` byte-identical, union UNKNOWN=1 residual `context_member_method_call`, witness 1340), `verilog_2005_conformance_gate` GREEN (orphans 0, matrix `240/0` with 3 new event-trigger locks + the `event_trigger_delay.sv` re-adjudication, cert `1117/4/773/340` byte-identical), `ast_shape_contract` 18/18, external corpus 14/14, the six fully-certified grammars byte-identical (SV-only regen), clippy source strict-clean (grammar-only). Full matrix in ledger row `SV-0023`.

## Release 1.0.166 / Contract 1.0.166 Highlights — SV-OVER-REJECTION-FIDELITY.2: spec-valid single-`;` `bind` no longer over-rejects (ledger `SV-0022` closed); schema 15 unchanged

**What consumers see:** a spec-valid single-semicolon `bind` directive — e.g. `bind m my_checker c1 ();` at top level or `module m; bind m my_checker c1 (); endmodule` at module scope — previously **REJECTED** under `sv_2017`/`sv_2023` and now correctly **ACCEPTS**. This is an over-rejection fix (the parser rejected known-valid IEEE 1800 input); no shape changes and no previously-accepting input changes. Under `verilog_2005` `bind` correctly stays REJECT (`bind` is SystemVerilog-only). **Why schema stays 15:** the fix removed a trailing `semi` token that was the last, *uncaptured* position of `bind_directive`, so every return-annotation `$`-ref (`instantiation: $4` / `$3`) and the emitted `{kind:"scoped"|"single", …}` shape are byte-identical.

**Root cause & fix (grammar-only):** `bind_directive` (`grammars/systemverilog.ebnf:668`/`:670`) required a trailing `semi` AFTER `bind_instantiation`, but every `bind_instantiation` alternative (`module_instantiation`/`program_instantiation`/`interface_instantiation`/`checker_instantiation`) already consumes its own `;`. The IEEE 1800-2017 §23 published BNF (`bind_directive ::= bind … bind_instantiation ;` with `module_instantiation ::= … ;`) has a REDUNDANT double-`;` that every commercial simulator resolves as a single `;`; the grammar over-specified by keeping both. Removing the trailing `semi` makes the single-`;` form parse. The double-`;` form (`c1 ();;`) still accepts under the SV profiles — now as a legitimate `bind … ();` + trailing empty package-or-generate item (`package_or_generate_item_declaration ::= … | ;`, cf. `SV-0028`), not the old defect.

**Proof (all seeds 0/7/42):** `sv_cert_recognized_union_gate` GREEN (canonical `1343/2/1321/20` byte-identical, union UNKNOWN=1 residual `context_member_method_call`, witness 1340), `verilog_2005_conformance_gate` GREEN (orphans 0, matrix `231/0` with `reject/bind_dir.sv` flipped `reject`→`accept` for `sv_2017`/`sv_2023`, cert `1117/4/773/340` byte-identical), `ast_shape_contract` 1/1, external corpus 14/14, the six fully-certified grammars byte-identical, clippy source strict-clean (grammar-only). Full matrix in ledger row `SV-0022`.

## Release 1.0.165 / Contract 1.0.165 Highlights — SV-LRM-SHAPE-FIDELITY.1: un-braced multi-identifier port lists no longer over-accept; braced concatenation ports now accepted (ledger `SV-0024` closed); schema 15 unchanged

- **What changed for consumers:** the multi-reference form of a port expression now requires the IEEE-mandated concatenation braces (`port_expression ::= port_reference | { port_reference {, port_reference} }`).
  - **accept→reject (`verilog_2005`):** a whitespace-separated multi-identifier port with no comma and no braces — `module m (a b);` — previously accepted; it now correctly **REJECTS** under `verilog_2005` (where the only reading is the non-ANSI `port_expression`).
  - **reject→accept (all profiles):** the braced concatenation port — `module m ({a, b});` — is the LRM's valid multi-reference form and was previously (wrongly) rejected because the grammar's list alternative was written un-braced; it now correctly **ACCEPTS** under every profile, emitting `port_expression {kind:"list", body:[{name,select}, …]}`.
  - **unchanged (`sv_2017`/`sv_2023`):** `module m (a b);` still ACCEPTS under the SV profiles as a legitimate ANSI interface port (`a` = interface type, `b` = port name; the "`a` is not a declared interface" check is elaboration, not parse). Single ports, comma-separated ports, named `.p(a)` ports, and ANSI declaration ports are all unchanged.
- **Why schema stays 15:** grammar-only — `port_expression`'s second alternative (`grammars/systemverilog.ebnf:4092`) changed from the un-braced `( port_reference ( comma port_reference )* )*` to `lbrace port_reference ( comma port_reference )* rbrace -> {kind:"list", body:[$2, $3::2*]}` (the established `assignment_pattern` concatenation-list idiom). ZERO new rules (census `1465` unchanged); the `{kind:"list"}` body is still typed-JSON — the braced-concat shape is newly *reachable*, not a new envelope shape.
- **Proof (all seeds 0/7/42):** `sv_cert_recognized_union_gate` GREEN (canonical `1343/2/1321/UNKNOWN=20` byte-identical, union UNKNOWN=1 residual `context_member_method_call`); `verilog_2005_conformance_gate` GREEN (orphans 0, matrix 225/0 with 2 new locks `reject/port_bare_multi_id.sv` + `accept/port_concat.v`, cert `1117/4/773/340` byte-identical — `port_expression` still witnessed under v2005 via braces); `ast_shape_contract_gate` 18/18; `sv_external_corpus_triage_gate` 14/14 parse_pass / 0 fail; `--lint-grammar` 1465 rules / profile_orphans=0 / always_matches_shadowing=8; the six fully-certified grammars byte-identical (json `9/0`, regex `198/0`); `clippy_on_rust_change` source strict-clean (grammar-only). Full matrix in ledger row `SV-0024`.

## Release 1.0.164 / Contract 1.0.164 Highlights — SV-KEYWORD-PRIMARY-FIDELITY.3.2: net/gate/structural reserved keywords no longer parse as bare expression primaries (ledger `SV-0036` closed); schema 15 unchanged

- **What changed for consumers (accept→reject, `sv_2017`/`sv_2023`):** 91 net-type / gate-primitive / structural / config Verilog keywords that SystemVerilog also reserves (`wire`, `and`, `always`, `assign`, `initial`, `posedge`, `negedge`, `nand`, `nor`, `xor`, `buf`, `supply0`, `tri`, `genvar`, `specparam`, …) previously matched as a bare expression **primary** (e.g. `assign w = wire;`, `initial x = and;`) — they now correctly **REJECT** under `sv_2017` / `sv_2023`. IEEE 1800 Annex B reserves these words; they can never be an operand. `verilog_2005` was already correct (its reserved list is the complete IEEE 1364-2005 Annex B).
- **What is unchanged:** keyword-**prefixed** identifiers (`wire_en`, `input_data`, `and_gate`, `always_on`) still parse as ordinary identifiers — the reserved regex is `\b`-anchored. Legal designs are unaffected (external corpus 14/14, 0 fail).
- **Why schema stays 15:** the fix is grammar-only for parser behavior — the `_v2005 \ _sv` reserved-word delta was appended to `reserved_non_keyword_identifier_sv` (`grammars/systemverilog.ebnf:386`; the `_sv` list grows 81→172 words so `_sv` ⊇ `_v2005`). No new rule (census `1465` unchanged), no AST shape change (`{body:X}` preserved; only reserved keywords flip acceptance).
- **Closed-loop (generator) prerequisite:** landing the reservation exposed a latent stimuli-generator gap — its RTL-FE-CLOSURE.6 keyword-exclusion guard recognized only the fixed-literal `!kw_A…!kw_Z` identifier shape, not SystemVerilog's `!reserved_non_keyword_identifier identifier` (regex whole-word-list) shape, so the closed loop could emit a reserved word (`import or::…`) the parser now rejects (`sample_parse_failures` 0→1). The guard was extended (`rust/src/ast_pipeline/stimuli_generator.rs`) to resolve a whole-word regex alternation (incl. via an `Or` of profile-gated sub-rules); the existing RNG-neutral `_`-prefix repair (`or`→`_or`) restores `sample_parse_failures=0`. Provably inert for the six fully-certified grammars (measured byte-identical). This is a generation-side quality fix; it does not change any parser output shape.
- **Proof (all seeds 0/7/42):** canonical cert `1343/2/1321/UNKNOWN=20` (spf=0, byte-identical); `sv_cert_recognized_union_gate` GREEN (canonical UNKNOWN=20, union UNKNOWN=1, residual `context_member_method_call`); `verilog_2005_conformance_gate` GREEN (orphans 0, matrix 219/0, cert `1117/4/773/340`); `ast_shape_contract_gate` 18/18; `sv_external_corpus_triage_gate` 14/14 parse_pass / 0 fail; `--lint-grammar` 1465 rules / profile_orphans=0; the six fully-certified grammars byte-identical; `clippy_on_rust_change` source strict-clean. Full matrix in ledger row `SV-0036`.

## Release 1.0.163 / Contract 1.0.163 Highlights — SV-KEYWORD-PRIMARY-FIDELITY.3.1: implicit-type ANSI ports parse correctly (ledger `SV-0037` closed); schema 15 unchanged

- **All-profile AST-fidelity fix (`SV-KEYWORD-PRIMARY-FIDELITY.3.1`, `PGEN-SV-KEYWORD-PRIMARY-FIDELITY-0004`, 2026-07-04, grammar-only).** An implicit-type ANSI port such as `module m(input a);` (and `module m(input [7:0] a);`) was **mis-parsed** as a non-ANSI port list (`{kind:"nonansi"}`, treating the direction word as a port identifier) instead of the correct ANSI port (`{kind:"ansi"}`, direction=`input`, implicit type, name=`a`). Latent on all shipped profiles.
- **Root cause (`SV-0037`):** `net_port_type_sv_2017` alt 1 (`grammars/systemverilog.ebnf:3474`) and `net_port_type_sv_2023` alt 1 (`:3479`) used the *ungated* `net_type_identifier`/`nettype_identifier` (bare `declaration_identifier`). Under the grammar's global default `branch_policy=longest_match`, that bare identifier out-consumed alt 0's empty implicit-type match and greedily ate the port NAME as a net-type, so the ANSI port had no name and the parser fell back to `nonansi`.
- **Fix:** both alt-1 carriers routed to the existing store-gated `checked_nettype_identifier` (`:3537`, `fact_attribute_equals(type_name, …, declaration_family, nettype)`), so only a DECLARED nettype can match there; an undeclared identifier no longer masquerades as a net-type, the implicit-type path wins, and the name binds. ZERO new rules (census `1465` unchanged).
- **Acceptance change (all profiles):** `module m(input a);` / `module m(input [7:0] a);` now realize `{kind:"ansi"}` (correct) instead of `{kind:"nonansi"}` (a mis-parse) under `sv_2017`/`sv_2023`/`verilog_2005`. Consumers keying off the module `kind` for such modules should expect the corrected `ansi` shape. Declared-nettype ports (`nettype logic n; module m(n a);`) are byte-identical (`{kind:"typed"}` via `known_unscoped_data_type`).
- **Schema:** unchanged at `15` — both `ansi` and `nonansi` shapes already exist; this is a mis-parse *correction*, not a new shape.
- **Proof:** canonical cert `1343/2/1321/UNKNOWN=20` (seeds 0/7/42, byte-identical); `ast_shape_contract_gate` 18/18; `sv_cert_recognized_union_gate` GREEN (canonical=20, union=1, residual `context_member_method_call`); `verilog_2005_conformance_gate` GREEN (orphans 0, matrix 219/0, cert `1117/4/773/340`); `sv_external_corpus_triage_gate` 14/14; `--lint-grammar` 1465 rules / profile_orphans=0 / always_matches_shadowing=8 (byte-identical to baseline); json `9/0` + regex `198/0` fully_certified.
- **Unblocks `SV-0036`/`.3.2`:** reserving the direction keywords (`input`/`output`/`inout`) to close `SV-0036` had exposed this latent bug; with `SV-0037` fixed, `.3.2` can safely reserve the remaining SV keyword set.

## Release 1.0.162 / Contract 1.0.162 Highlights — SV-KEYWORD-PRIMARY-FIDELITY.2: reserved *type* keywords no longer parse as bare expression primaries (ledger `SV-0035` closed); schema 15 unchanged

- **All-profile behavior fix (`SV-KEYWORD-PRIMARY-FIDELITY.2`, `PGEN-SV-KEYWORD-PRIMARY-FIDELITY-0002`, 2026-07-04, grammar-only).** A reserved *type* keyword (`integer`/`real`/`time`/`reg`/`logic`/`int`/`bit`/`byte`/`shortint`/`longint`/`shortreal`/`string`/`chandle`/`event`/…) could match as a bare **expression primary** (`assign w = integer;`, `initial x = integer;`, `localparam p = integer + 1;`) under every profile. IEEE 1364-2005 / IEEE 1800 Annex B reserve these words, so they may only appear as a data type, never as a `primary`/operand.
- **Fix:** the three expression-primary bare-`identifier` carriers — `specparam_identifier` (`grammars/systemverilog.ebnf:5083`) and `genvar_identifier` (`:2375`) in `constant_primary_sv_2017`/`_sv_2023`, and the trailing name of `hierarchical_identifier` (`:2412`) in `primary_sv_2017` — now route through the keyword-excluding `non_keyword_identifier` (`:359`). ZERO new rules (census `1465` unchanged).
- **Acceptance change (SV profiles):** `assign w = <reserved-type-kw>;` and the arithmetic/operand forms flip **ACCEPT → REJECT** under `sv_2017`/`sv_2023` for the whole reserved-type family; `verilog_2005` likewise for its reserved family. The residual `localparam p = integer;` still ACCEPTs — that is the **LRM-faithful** `constant_param_expression ::= … | data_type | $` route (`:1380`), not a keyword-as-primary — as does `reg q [integer];` under the SV profiles (the legitimate `associative_dimension ::= [ data_type ]`).
- **Schema:** unchanged at `15` — shape-preserving by construction (`identifier` and `non_keyword_identifier` both emit `{body:X}`; only reserved keywords change acceptance, never a legal name's AST).
- **Proof:** canonical cert `1343/2/1321/UNKNOWN=20` + `verilog_2005` cert `1117/4/773/340` (seeds 0/7/42, byte-identical); `verilog_2005_conformance_gate` GREEN (orphans 0, matrix 219/0); `sv_cert_recognized_union_gate` GREEN (canonical UNKNOWN=20, union UNKNOWN=1, residual `context_member_method_call`); `ast_shape_contract_gate` 18/18; `sv_external_corpus_triage_gate` 14/14 parse_pass / 0 fail; `clippy_on_rust_change` source strict-clean; json `9/0` + regex `198/0` fully_certified.
- **Known residual (`SV-0036`, OPEN, owned by `SV-KEYWORD-PRIMARY-FIDELITY.3`):** 35 net-type/gate/structural SV keywords (`wire`/`and`/`always`/…) that are reserved in SV but absent from `reserved_non_keyword_identifier_sv` still leak as primaries under `sv_2017`/`sv_2023` (a reserved-list-completeness defect, distinct mechanism). `verilog_2005` is unaffected (its reserved list is the complete Annex B).

## Release 1.0.161 / Contract 1.0.161 Highlights — SV-DOLLAR-LRM-FIDELITY.4 wave 3: the SV-only `$` anchors restored (`$root`/`$unit`/severity/bare-`$`; ledger `SV-0029` closed) (schema 14 → 15)

- **What changed (grammar + shape-test dispatch, `PGEN-SV-DOLLAR-LRM-FIDELITY-0004`, 2026-07-03):** the remaining 7 mangled keyword-token literals now carry their IEEE spellings (`/\$root\b/`, `/\$unit\b/`, `/\$fatal\b/`, `/\$error\b/`, `/\$warning\b/`, `/\$info\b/`, and the bare-`$` `/\$/`). Because the generic `system_tf_identifier` regex also matches `$root`/`$unit` as whole tokens (and a succeeding PEG alternative commits), `system_tf_call`'s optional-parens alternative gained two inline negative-lookahead steal guards (`!( $root . )`, `!( $unit :: )`) — `$root(…)`, bare `$unit`, and every ordinary `$name` stay on the generic route byte-identically. Three profile-gated `_sv_only` lifts scope the new surface to the SystemVerilog profiles (`hierarchical_root_prefix_sv_only`, `package_scope_dollar_unit_sv_only`, `primary_dollar_sv_only`), and a dedicated `rooted_tf_call_sv_only` carrier (wired into `subroutine_call` and `call_primary`, guarded by `!( identifier )` so no identifier-headed call can re-route) carries `$root.`-anchored task/function calls.
- **Consumer impact:** IEEE 1800 `$unit::name` scopes, module-level elaboration severity tasks (`$fatal;` … with args forms), `$root.`-anchored hierarchical names and calls, and the bare-`$` primary now **accept** under `sv_2017`/`sv_2023` and **reject** under strict `verilog_2005` (all four are SystemVerilog-only surfaces). `$root.`-anchored input that previously parsed now carries the anchor explicitly (see Schema) — consumers that pattern-matched the old generic `kind:"system_tf"` reading of `$root…` must migrate. The mangled `sv_dollar_*` spellings are ordinary identifiers (they were never language keywords) and parse exactly as before, byte-identically.
- **Schema:** `14` → `15` — breaking re-route for `$root` anchors (`root: {kind:"root"}` marker on hierarchical names; the new `{kind:"rooted_tf", target, attributes, args}` call carrier) + additive `{kind:"dollar_unit", name:{body:"$unit"}}` / severity `{kind:"fatal"|"error"|"warning"|"info", args}` / bare-`$` `{kind:"system_dollar"}` realizations. Full detail in the schema-identity note above and the SV parser book's schema-versioning chapter.
- **Proof surface:** the 28-probe accept/reject matrix (every LRM spelling ACC under both SV profiles / REJ under `verilog_2005`; 24/24 mangled + control ASTs byte-identical, cmp-proven); canonical cert `1341/2/1319/UNKNOWN=20` `spf=0` deterministic at seeds 0/7/42 with the 20-rule residual set-identical pre↔post (total +4 = exactly the 4 new rules, all witnessed); `verilog_2005` cert `1147/4/816/327` deterministic (sole residual add = the bare-`$` token, NO-reach-by-design 294→295; `$root`/`$unit` tokens upgrade witness→PROOF); `verilog_2005_conformance_gate` GREEN — 192 checks / 0 mismatches (64 cases; 4 new SV-0029 wave-3 reject-locks); union gate GREEN re-pinned (`1341/2/1319/20`, union `1338/1`); AST-shape contract green (31 samples; 6 new); external corpus: the 10 non-uvm rows pass (the 4 uvm rows remain the documented 24 GB-host mem-cap posture); clippy source strict-clean; ledger row `SV-0029` → `Released`.

## Release 1.0.160 / Contract 1.0.160 Highlights — SV-DOLLAR-LRM-FIDELITY.3 wave 2: the SV-0030 LRM digits + timing-check compare shapes restored (schema 13 → 14)

- **What changed (grammar-only, both defect sites):** (1) `scalar_constant` now carries the ten IEEE 1364-2005 / 1800-2017 A.7.5.3 alternatives (`1'b0 | 1'b1 | 1'B0 | 1'B1 | 'b0 | 'b1 | 'B0 | 'B1 | 1 | 0`) instead of 6 digit-less prefix branches; (2) UDP `init_val` carries the ten IEEE A.5.1 alternatives — **LRM sequential-UDP initializers `initial q = 1'b0;` / `1'b1;` now ACCEPT under every profile** (they REJECTED outright at `1.0.0`–`1.0.159`) and the digit-less nonsense `initial q = 1'b;` now REJECTS; (3) the `scalar_timing_check_condition` compare branches (`expression ==/===/!=/!== scalar_constant`) are reachable for the first time — `e == 1'b0` in a specify timing-check condition emits `{kind:"eq", lhs, rhs}` instead of a flat `{kind:"expression"}`.
- **Consumer impact:** anyone parsing gate-level netlists with sequential UDPs should adopt `1.0.160` (the initializer rejection was a hard parse failure). Consumers of timing-check conditions gain the LRM compare shapes; note the deliberate precedence boundary — operator-continued conditions (`e == 1'b0 && f`) and mixed-precedence lhs forms (`a & b == 1'b0`, where `==` binds tighter than `&` per IEEE 1800-2017 Table 11-2) keep the flat `{kind:"expression"}` parse byte-identically, because the flat operand chain is the precedence-correct reading for those inputs.
- **Schema:** `13` → `14` (digit-carrying `scalar_constant`/`init_val` kinds — the digit-less kinds are removed; the compare shapes `{kind:"eq"|"case_eq"|"ne"|"case_ne", lhs, rhs}` are now realizable). Full detail in Contract Identity § "SystemVerilog AST-dump schema version".
- **Proof surface:** canonical cert `1337/2/1315/UNKNOWN=20` `spf=0` deterministic at seeds 0/7/42 with the 20-rule residual set-identical pre↔post (total +9 = exactly the census delta of the fix); `verilog_2005` cert `1147/2/819/326` with `scalar_constant` itself leaving the UNKNOWN residual (set-diff-proven sole change); `verilog_2005_conformance_gate` GREEN — 180 checks / 0 mismatches (60 cases; 4 new SV-0030 locks); AST-shape contract 18/18 (4 new samples); external corpus green; ledger row `SV-0030` → `Released`.

## Release 1.0.159 / Contract 1.0.159 Highlights — SV-DOLLAR-LRM-FIDELITY.2 wave 1: the 12 specify timing-check `$` spellings restored (ledger `SV-0029`, partial); schema 13 unchanged

- **What changed (grammar-only, `PGEN-SV-DOLLAR-LRM-FIDELITY-0002`, 2026-07-02):** the 12 specify timing-check keyword tokens (`$setup`, `$hold`, `$setuphold`, `$recovery`, `$recrem`, `$removal`, `$skew`, `$timeskew`, `$fullskew`, `$period`, `$width`, `$nochange`) previously matched the mangled literal text `sv_dollar_setup` … `sv_dollar_nochange` (an LRM-extraction name-canonicalization leak, ledger `SV-0029`). They now match the IEEE `$` spellings.
- **Consumer impact:** LRM specify timing checks — core surface in BOTH IEEE 1364-2005 §15 and IEEE 1800-2017 §31 — now **accept** under every profile (`sv_2017`, `sv_2023`, `verilog_2005`); the mangled non-language spellings now correctly **reject** everywhere. Anyone parsing gate-level netlists with timing checks should adopt `1.0.159`. No AST schema change (the timing-check typed kinds — `{kind:"setup", …}` etc. — are unchanged; schema stays `13`).
- **(Resolved since:** the SV-only `$` group below landed at `1.0.161`, `SV-DOLLAR-LRM-FIDELITY.4` — see the `1.0.161` highlights; the row is `Released`.**)** At `1.0.159` the not-yet-fixed remainder was: the SV-only `$` tokens — `$root`, `$unit`, the module-level elaboration severity tasks (`$fatal`/`$error`/`$warning`/`$info`), and the bare-`$` primary — still carry mangled literals (leaf `SV-DOLLAR-LRM-FIDELITY.4`), and `SV-0030` (`scalar_constant`/`init_val` digit loss) was the next wave — since FIXED at `1.0.160` (leaf `.3`).
- **Regression locks:** conformance corpus `accept/specify_timing_checks.v` (all 12 LRM spellings, accept ×3 profiles) + `reject/specify_timing_check_mangled.v` (mangled spelling, reject ×3) — matrix now 168 checks; cert pins unchanged and re-verified (canonical `1328/2/1306/20`, union `1325/1`, `verilog_2005` `1138/2/809/327`, seeds 0/7/42).

## Release 1.0.152 / Contract 1.0.152 Highlights — SV-AST-SHAPE-FIDELITY.1: ANSI-port `<invalid_sequence_access>` corruption fixed (ledger `SV-0014`); schema 6 → 7

- Grammar fix (`grammars/systemverilog.ebnf`, AST-shape corruption): `ansi_port_declaration` (`:519-520`) bound `header: $1` to an inline alternation inside an optional group `( net_port_header | interface_port_header )?`. That inline-alt-`$N` pattern corrupts the positional model (`ast_return_transform.rs:193/438`), so **every typed ANSI module port** (`input logic a`, `output logic b`, …) emitted three `<invalid_sequence_access>` sentinels in its `header` sub-shape (`net_port_header`'s real `{direction, port_type}` mis-nested as `header.header`). This is the SV-main instance of the systemic inline-alternation-`$N` class (`SVPP-0001` / `RTL-FE-0002` / `RTL-CE-0001` / `VHDL-0001`); it was **not** covered by POST-SV-AUDIT (that audit swept only iteration-lead `{first/rest:$N}` list rules).
- Fix (the proven `pp_if_keyword` / `event_separator` named-lift idiom): lifted the inline alternation into the new **un-annotated** named rule `ansi_port_header := net_port_header | interface_port_header`; `ansi_port_declaration`'s bare `header: $1` now binds the clean named rule (annotation text unchanged). Parse behavior is byte-identical (same alternation, same order) — only the corrupted AST shape is repaired.
- Consumer impact: `port.header` for a typed ANSI port changes from the corrupted sentinel-bearing object to the clean `net_port_header` `{direction, port_type}` (or `interface_port_header` `{kind:"named", name, modport}`). This is a consumer-visible AST structure change → **schema `6` → `7`**.
- See § "AST-Shape Corrections — 1.0.152" for the before/after and the shape-contract lock.

## AST-Shape Corrections — 1.0.158 (SV-AST-SHAPE-FIDELITY.2.7) — `class_scoped_call_prefix` head inline-alternation-`$1` corruption → clean `params`/`scope_chain` (reachable, consumer-visible; bug-ledger `SV-0020`); schema 12 → 13

- Affected rule: `class_scoped_call_prefix` (`grammars/systemverilog.ebnf`, the `SV-0013` rule), `-> {head: $1, params: $2, scope_chain: $4}`. It bound `head: $1` to an inline 4-way alternation `( scoped_class_scoped_call_prefix_identifier | known_unscoped_class_scoped_call_class_identifier | known_unscoped_class_scoped_call_interface_class_identifier | known_unscoped_class_scoped_call_type_parameter_identifier )`. When the self-annotated `scoped_class_scoped_call_prefix_identifier` branch (`-> {scope: $1, name: $2}`) won, the rule's own `{head:$1, params:$2, scope_chain:$4}` annotation mis-recursed onto that branch result (`rust/src/ast_pipeline/ast_return_transform.rs:193`/`:438`), corrupting `params`/`scope_chain`. `SV-0013` gated the branches *inside* the alt but did not lift it — this completes that rule. The `{head:$1, params:$2, scope_chain:$4}` twin of `class_scope_type` (fixed in `.2.3`).
- BEFORE (`1.0.0`–`1.0.157`), `pkg::C::foo()` (a package-class-scoped static method call, in a statement or an assignment RHS), at the `class_scoped_call_prefix` node: **2** `<invalid_sequence_access>` (`{"head": {…scoped id…}, "params": "<invalid_sequence_access>", "scope_chain": "<invalid_sequence_access>"}`). Both profiles.
- AFTER (`1.0.158`), the same input yields a clean `{"head": {"name": {"body": "C"}, "scope": {"body": {"kind": "package", "name": {"body": "pkg"}}}}, "params": [], "scope_chain": []}` — no `<invalid_sequence_access>` anywhere, in both profiles.
- Fix (the `SV-0014`..`SV-0019` named-lift idiom): ONE new un-annotated named rule `class_scoped_call_prefix_head := scoped_class_scoped_call_prefix_identifier | known_unscoped_class_scoped_call_class_identifier | known_unscoped_class_scoped_call_interface_class_identifier | known_unscoped_class_scoped_call_type_parameter_identifier`; `class_scoped_call_prefix`'s bare `head: $1` now binds it (annotation text + positions `$1`..`$4` unchanged) — mirroring the in-file `class_scope_type_head`.
- Locked by two new `systemverilog_v1.json` shape-contract samples via the non-root `rule_under_test` dispatch: `class_scoped_call_prefix_head_scoped` (`class_scoped_call_prefix_head`, `pkg::C`, keys `["scope","name"]`, corruption-site + compile-time revert guard) + `class_scoped_call_prefix_scoped` (`class_scoped_call_prefix`, `pkg::C::`, keys `["head","params","scope_chain"]`, root lock with the now-clean `params`/`scope_chain`).
- This is the seventh reachable, consumer-visible member of the inline-alternation-`$N` class (after `SV-0014`..`SV-0019`) → bug-ledger `SV-0020`, and the LAST of the three `SV-AST-SHAPE-FIDELITY.2.4`-classified REACHABLE-CORRUPT candidates. `SV-AST-SHAPE-FIDELITY.2` sweep: 9 of 21 candidates now fixed; the remaining candidates are the tool-proven LATENT (PEG-shadowed) / BENIGN / STALE set.
- Schema `12 → 13` because the `params`/`scope_chain` sub-shapes change consumer-visibly (corrupted → clean `[]`), same rationale as `SV-0014`..`SV-0019`.
- VERIFIED: `<invalid_sequence_access>` 2 → 0 both profiles (`parse_full` still passes; statement + assignment-RHS forms); SV cert `total 1311→1312 / witness 1290→1291 / UNKNOWN=20` unchanged (identical 20-rule residual — 19 `no_path` + `context_member_method_call`; `class_scoped_call_prefix_head` witnessed; ZERO newly-unknown) `spf=0 proof_reverify_failures=0`, deterministic seeds 0/7/42; SV external corpus 14/14; `--lint-grammar ordered_choice_shadowing=0 non_terminating=0 unreachable=0 profile_orphans=0` (1432→1433 rules; `always_matches=8` unchanged); the 6 fully-certified grammars byte-identical (SV-only regen; no codegen touched); `cargo test --lib --features generated_parsers` 739/0/21ignored (`systemverilog_ast_shape_contract` PASS, 2 new samples) + `make -C rust SHELL=/bin/bash systemverilog_parser_book_gate`.

## AST-Shape Corrections — 1.0.157 (SV-AST-SHAPE-FIDELITY.2.6) — nettype `with`-clause double-nested inline-alternation-`$N` corruption → clean `with_clause` scope (reachable, consumer-visible; bug-ledger `SV-0019`)

- Affected rules (two, both nettype declarations): `net_type_declaration_sv_2017` (`grammars/systemverilog.ebnf:3341`), `-> {data_type: $2, name: $3, with_clause: $4}`, and its sv_2023 twin `nettype_declaration_sv_2023` (`:3390`), `-> {kind: "data_type", data_type: $2, name: $3, with_clause: $4}`. Both br0 bound a **double-nested** inline alt `( non_typedef_package_scope | class_scope )?` inside the `with`-clause group `( kw_with ( … )? tf_identifier )?`. When the self-annotated `non_typedef_package_scope` branch won, the rule's own annotation mis-recursed onto that scope element (`rust/src/ast_pipeline/ast_return_transform.rs:193`/`:438`).
- BEFORE (`1.0.0`–`1.0.156`), `nettype logic n with pkg::f;`, at the `with_clause` inner scope element: **3** `<invalid_sequence_access>` (`{"data_type": "<invalid_sequence_access>", "kind": "data_type", "name": "<invalid_sequence_access>", "with_clause": "<invalid_sequence_access>"}` glued in place of the scope). Both profiles. The br1 forms (scope alt a direct element, single-nested) were already clean.
- AFTER (`1.0.157`), the same input yields a clean `with_clause`: `[[[], "with"], {"body": {"kind": "package", "name": {"body": "pkg"}}}, {"body": "f"}]` — no `<invalid_sequence_access>` anywhere, in both profiles.
- Fix (the `SV-0014`..`SV-0018` named-lift idiom): ONE new un-annotated named rule `nettype_scope_prefix := non_typedef_package_scope | class_scope` (order preserved — distinct from the reversed `scoped_type_scope_prefix`), referenced at all 4 nettype occurrences (both profiles, both branches).
- Locked by two new `systemverilog_v1.json` shape-contract samples via the non-root `rule_under_test` dispatch: `nettype_scope_prefix_pkg` (`nettype_scope_prefix`, `pkg::`, keys `["body"]`, corruption-site + compile-time revert guard) + `net_type_declaration_with_clause` (`net_type_declaration_sv_2017`, `nettype logic n with pkg::f;`, keys `["data_type","name","with_clause"]`, root lock with the now-clean inner scope).
- This is the sixth reachable, consumer-visible member of the inline-alternation-`$N` class (after `SV-0014`..`SV-0018`) → bug-ledger `SV-0019`. `SV-AST-SHAPE-FIDELITY.2` sweep: 8 of 21 candidates now fixed; 1 REACHABLE-CORRUPT fix remaining (`.2.7` #21).
- VERIFIED: `<invalid_sequence_access>` 3 → 0 both profiles (br1 stays 0; `parse_full` still passes); SV cert `total 1310→1311 / witness 1289→1290 / UNKNOWN=20` unchanged (identical 20-rule residual; `nettype_scope_prefix` witnessed; ZERO newly-unknown) `spf=0`, deterministic seeds 0/7/42; SV external corpus 14/14; `--lint-grammar ordered_choice_shadowing=0` (1431→1432 rules; `always_matches=8` unchanged); the 6 fully-certified grammars byte-identical (SV-only regen); `cargo test --lib --features generated_parsers` 739/0/21ignored (`systemverilog_ast_shape_contract` PASS, 2 new samples) + `make -C rust SHELL=/bin/bash systemverilog_parser_book_gate`.

## AST-Shape Corrections — 1.0.156 (SV-AST-SHAPE-FIDELITY.2.5) — `interface_class_type` inline-alternation-`$N` corruption → clean `params` (reachable, consumer-visible; bug-ledger `SV-0018`)

- Affected rule: `interface_class_type` (`grammars/systemverilog.ebnf:2478`), `-> {name: $1, params: $2}` — bound `name: $1` to an inline 2-way alternation `( scoped_interface_class_type_identifier | known_unscoped_interface_class_type_identifier )`. When the self-annotated `scoped_interface_class_type_identifier` branch (`-> {scope: $1, name: $2}`) won, the rule's own annotation mis-recursed onto the branch result and glued a corrupt `params` slot onto the inner `name` (`rust/src/ast_pipeline/ast_return_transform.rs:193`/`:438`).
- BEFORE (`1.0.0`–`1.0.155`), `class C implements pkg::IC;`, at the class's `implements` interface-class-type node: **1** `<invalid_sequence_access>` — `{"name": {"name": {"body": "IC"}, "scope": {…pkg…}, "params": "<invalid_sequence_access>"}, "params": []}`. Both profiles.
- AFTER (`1.0.156`), the same input yields a fully clean node: `{"name": {"name": {"body": "IC"}, "scope": {…pkg…}}, "params": []}` — no `<invalid_sequence_access>` anywhere, in both profiles.
- Fix (the `SV-0014`/`SV-0015`/`SV-0016`/`SV-0017` named-lift idiom, mirroring the in-file `base_class_type_head`): ONE new un-annotated named rule `interface_class_type_head := scoped_interface_class_type_identifier | known_unscoped_interface_class_type_identifier`, referenced by `interface_class_type` as its head (annotation text + positions unchanged).
- Locked by two new `systemverilog_v1.json` shape-contract samples via the non-root `rule_under_test` dispatch: `interface_class_type_head_scoped` (`interface_class_type_head`, `pkg::IC`, keys `["scope","name"]`, corruption-site + compile-time revert guard) + `interface_class_type_scoped` (`interface_class_type`, `pkg::IC`, keys `["name","params"]`, with `params` now the clean `[]`).
- This is the fifth reachable, consumer-visible member of the inline-alternation-`$N` class (after `SV-0014`/`SV-0015`/`SV-0016`/`SV-0017`) → bug-ledger `SV-0018`. `SV-AST-SHAPE-FIDELITY.2` sweep: 6 of 21 candidates now fixed (#2, #19, #20, #1, #5, #6 prior; #8 here); the rest are tool-classified (7 LATENT PEG-shadowed, 4 BENIGN, 1 STALE) with 2 real fixes remaining (`.2.6` #14, `.2.7` #21).
- VERIFIED: `<invalid_sequence_access>` 1 → 0 both profiles (`parse_full` still passes); SV cert `total 1309→1310 / witness 1288→1289 / UNKNOWN=20` unchanged (identical 20-rule residual; `interface_class_type_head` witnessed; ZERO newly-unknown) `spf=0`, deterministic seeds 0/7/42; SV external corpus 14/14; `--lint-grammar ordered_choice_shadowing=0` (1430→1431 rules; `always_matches=8` unchanged); the 6 fully-certified grammars byte-identical (SV-only regen); `cargo test --lib --features generated_parsers` 739/0/21ignored (`systemverilog_ast_shape_contract` PASS, 2 new samples) + `make -C rust SHELL=/bin/bash systemverilog_parser_book_gate`.

## AST-Shape Corrections — 1.0.155 (SV-AST-SHAPE-FIDELITY.2.3) — scoped-type-reference cluster (`class_scope_type` + `scoped_block_type_identifier`/`scoped_data_type_identifier`) inline-alternation-`$N` corruption → clean `params`/`scope_chain` + `type`/`dims` (reachable, consumer-visible; bug-ledger `SV-0017`); schema 9 → 10

- Affected rules (two, both reached by the ubiquitous package/class-scoped type reference `C::D::E v;`):
  - `class_scope_type` (`grammars/systemverilog.ebnf:1067`), `-> {head: $1, params: $2, scope_chain: $3}` — bound `head: $1` to an inline 4-way alternation `( scoped_class_scope_identifier | known_unscoped_class_scope_class_identifier | known_unscoped_class_scope_interface_class_identifier | known_unscoped_class_scope_type_parameter_identifier )`. The un-lifted twin of `base_class_type` (`SV-0016`) / `class_type`.
  - `scoped_block_type_identifier` (`:1642`) and the byte-identical `scoped_data_type_identifier` (`:1649`), both `-> {scope: $1, type: $2, dims: $3}` — bound `scope: $1` to the shared inline alt `( class_scope | non_typedef_package_scope )`.
- BEFORE (`1.0.0`–`1.0.154`), `module m; C::D::E v; endmodule`, at the variable's `data_type` (kind `scoped_data_type`) node: **4** `<invalid_sequence_access>` sentinels — the outer `scoped_data_type_identifier` mis-recursed its own annotation onto the self-annotated `class_scope` branch (`type`/`dims` corrupt), and the nested `class_scope_type` head corrupted `params`/`scope_chain` (when the self-annotated `scoped_class_scope_identifier` branch won). Both profiles.
- AFTER (`1.0.155`), the same input yields a fully clean node: `{"scope": {"body": {"head": {"name": …, "scope": …}, "params": [], "scope_chain": []}}, "type": {"body": "E"}, "dims": []}` — no `<invalid_sequence_access>` anywhere, in both profiles.
- Fix (the `SV-0014`/`SV-0015`/`SV-0016` named-lift idiom, mirroring the in-file `class_type_head`): TWO new un-annotated named rules — `class_scope_type_head` (lifts the `class_scope_type` alt) and `scoped_type_scope_prefix` (lifts the shared `( class_scope | non_typedef_package_scope )` alt, which occurs ONLY at `scoped_block_type_identifier`/`scoped_data_type_identifier`) — each referenced by its parent; the bare `$1`s now bind the clean named rules (annotation text + positions unchanged). One shared prefix fixes both byte-identical `scoped_*_type_identifier` rules.
- Locked by four new `systemverilog_v1.json` shape-contract samples via the non-root `rule_under_test` dispatch: `class_scope_type_head_scoped` (`class_scope_type_head`, `pkg::B`, keys `["scope","name"]`) + `class_scope_type_scoped` (`class_scope_type`, `C::D`, keys `["head","params","scope_chain"]`) + `scoped_type_scope_prefix_class` (`scoped_type_scope_prefix`, `C::D::`, keys `["body"]`) + `scoped_data_type_identifier_scoped` (`scoped_data_type_identifier`, `C::D::E`, keys `["scope","type","dims"]`). The two corruption-site samples double as compile-time revert guards (reverting a named-lift fails its `parse_*()` to compile). `cargo test --lib --features generated_parsers systemverilog_ast_shape_contract` green.
- This is the fourth reachable, consumer-visible member of the inline-alternation-`$N` class (after `SV-0014`/`SV-0015`/`SV-0016`) → bug-ledger `SV-0017`. `SV-AST-SHAPE-FIDELITY.2` sweep: 5 of 21 candidates now fixed (#2, #19, #20 prior; #1, #5, #6 here) — the rest verified tools-first and fixed one cluster per leaf.

## AST-Shape Corrections — 1.0.154 (SV-AST-SHAPE-FIDELITY.2.2) — `base_class_type` head inline-alternation-`$1` corruption → clean `params`/`scope_chain` (reachable, consumer-visible; bug-ledger `SV-0016`); schema 8 → 9

- Affected rule: `base_class_type` (`grammars/systemverilog.ebnf:1094`), `-> {head: $1, params: $2, scope_chain: $3}`, reached via the `extends` clause of both `class_declaration_sv_2017` (`:955`) and `class_declaration_sv_2023` (`:959`).
- BEFORE (`1.0.0`–`1.0.153`), `class D extends pkg::B; endclass`, at the `extends` base's `base_class_type` node:
  `{"head": {"name": {"body": "B"}, "scope": {…pkg…}}, "params": "<invalid_sequence_access>", "scope_chain": "<invalid_sequence_access>"}` — when the self-annotated `scoped_base_class_type_identifier` branch (`-> {scope:$1,name:$2}`) won, the rule's own `{head:$1, params:$2, scope_chain:$3}` annotation mis-recursed so the following `$2`/`$3` slots resolved out-of-range (2 sentinels; both profiles). The declared-bare `class B; class D extends B;` was clean (the bare branch carries no per-branch annotation).
- AFTER (`1.0.154`), the same input at the `base_class_type` node:
  `{"head": {"name": {"body": "B"}, "scope": {…pkg…}}, "params": [], "scope_chain": []}` — `params`/`scope_chain` now the correct empty arrays; no `<invalid_sequence_access>` anywhere, in both profiles. The declared-bare form stays `{head:{body:"B"}, params:[], scope_chain:[]}`.
- Fix (the `SV-0014`/`SV-0015` named-lift idiom, mirroring the already-correct in-file sibling `class_type_head`): the inline `( scoped_base_class_type_identifier | known_unscoped_base_class_type_identifier | known_unscoped_base_class_type_parameter_identifier )` alternation is lifted into ONE new un-annotated named rule `base_class_type_head`, referenced by `base_class_type` as its head; the bare `head: $1` now binds the clean named rule (annotation text + positions unchanged) — one shared rule fixes both profiles.
- Locked by two new `systemverilog_v1.json` shape-contract samples via the non-root `rule_under_test` dispatch: `base_class_type_head_scoped` (`rule_under_test: base_class_type_head`, input `pkg::B`, keys `["scope","name"]` — the corruption-site + compile-time revert guard: reverting the named-lift fails `parse_base_class_type_head()` to compile) and `base_class_type_scoped` (`rule_under_test: base_class_type`, input `pkg::B`, keys `["head","params","scope_chain"]`). `cargo test --lib --features generated_parsers systemverilog_ast_shape_contract` green.
- This is the third reachable, consumer-visible member of the inline-alternation-`$N` class (after `SV-0014`/`SV-0015`) → bug-ledger `SV-0016`. The `SV-AST-SHAPE-FIDELITY.2` sweep enumerated 21 candidates; the rest are being verified tools-first and fixed one cluster per leaf.

## AST-Shape Corrections — 1.0.153 (SV-AST-SHAPE-FIDELITY.2.1) — `type_declaration` forward-typedef inline-alternation-`$2` corruption → clean `keyword` (reachable, consumer-visible; bug-ledger `SV-0015`); schema 7 → 8

- Affected rules: `type_declaration_sv_2017` branch 6 (`grammars/systemverilog.ebnf:5209`) and `type_declaration_sv_2023` branch 6 (`:5225`), `-> {kind: "forward", keyword: $2, name: $3}`.
- BEFORE (`1.0.0`–`1.0.152`), `module m; typedef enum e_t; endmodule` (also `struct`/`union`), at the forward node's `keyword`:
  `{"kind": "class_alias", "class_type": "<invalid_sequence_access>", "dims": "<invalid_sequence_access>", "name": "<invalid_sequence_access>"}` — branch-1's `class_alias` annotation mis-recursed onto the `keyword` slot (3–4 sentinels; the keyword-less `typedef e_t;` was clean because the optional group is absent).
- AFTER (`1.0.153`), the same input at the forward node's `keyword`:
  `[[], "enum"]` — the clean fused-token `[trivia, "enum"]` shape the grammar uses for a direct keyword field; no `<invalid_sequence_access>` anywhere, in both profiles. The outer node keeps its correct `{kind:"forward", keyword, name}` with `name:{body:"e_t"}`.
- Fix (the `SV-0014` named-lift idiom): the shared inline `( kw_enum | kw_struct | kw_union )` alternation is lifted into ONE new un-annotated named rule `forward_type_keyword := kw_enum_e338e8e3 | kw_struct_d118e5a3 | kw_union_67ad5a07`, referenced by both br6 as `forward_type_keyword?`; the bare `keyword: $2` now binds the clean named rule (annotation text + positions unchanged) — one shared rule fixes both profiles.
- Locked by two new `systemverilog_v1.json` shape-contract samples via the non-root `rule_under_test` dispatch: `forward_typedef_keyword` (`rule_under_test: forward_type_keyword`, input `enum`, content_kind `sequence` — the corruption-site + compile-time revert guard: reverting the named-lift fails `parse_forward_type_keyword()` to compile) and `forward_typedef_enum` (`rule_under_test: type_declaration_sv_2017`, input `typedef enum e_t;`, keys `["kind","keyword","name"]`, `{"kind":"forward"}`). `cargo test --lib --features generated_parsers systemverilog_ast_shape_contract` green (18/0).
- This is the second reachable, consumer-visible member of the inline-alternation-`$N` class (after `SV-0014`) → bug-ledger `SV-0015`. The `SV-AST-SHAPE-FIDELITY.2` sweep enumerated 21 candidates; the rest are being verified tools-first and fixed one cluster per leaf.

## AST-Shape Corrections — 1.0.152 (SV-AST-SHAPE-FIDELITY) — `ansi_port_declaration` inline-alternation-`$1` corruption → clean `header` (reachable, consumer-visible; bug-ledger `SV-0014`); schema 6 → 7

- Affected rule: `ansi_port_declaration` branch 1 (`grammars/systemverilog.ebnf:519-520`), `-> {kind: "net_or_interface", header: $1, name: $2, dims: $3, default: $4}`.
- BEFORE (`1.0.0`–`1.0.151`), `module m (input logic a); endmodule`, at `ports[…].header`:
  `{"kind": "net_or_interface", "header": {"direction": {"kind": "input"}, "port_type": {…}}, "default": "<invalid_sequence_access>", "dims": "<invalid_sequence_access>", "name": "<invalid_sequence_access>"}`
- AFTER (`1.0.152`), the same input at `ports[…].header`:
  `{"direction": {"kind": "input"}, "port_type": {…}}` — the clean `net_port_header` shape; the outer port object keeps its correct `{kind:"net_or_interface", header, name, dims, default}` with `dims:[]`, `default:[]`, `name:{body:"a"}`.
- Locked by two new `systemverilog_v1.json` shape-contract samples parsed via the non-root `rule_under_test` dispatch: `ansi_port_header_typed` (`rule_under_test: ansi_port_header`, input `input logic`, keys `["direction","port_type"]` — the corruption-site lock) and `ansi_port_typed` (`rule_under_test: ansi_port_declaration`, input `input logic a`, keys `["kind","header","name","dims","default"]`, `{"kind":"net_or_interface"}`). `cargo test --lib --features generated_parsers systemverilog_ast_shape_contract` green.
- This is the reachable, consumer-visible member of the class → bug-ledger `SV-0014`. (The related latent `named_dot`-branch annotation-drop of SV-Slice-112, on the same leading-optional-group shape, is a separate non-corruption matter tracked in the manifest calibration history.)

## Release 1.0.140 / Contract 1.0.140 Highlights — GRAMMAR-WELLFORMED.H.12.5.5.3.3.5: stream LRM delimiter-drop fixed (ledger `SV-0002`); schema 3 → 4

- Grammar fix (`grammars/systemverilog.ebnf`, LRM-grounded): restores the literal delimiters dropped from the streaming-concatenation family during LRM extraction, matching IEEE 1800 §A.8.1 (`systemverilog_2017_lrm_extracted.ebnf:1041/1045`):
  - `stream_concatenation := lbrace stream_expression ( comma stream_expression )* rbrace -> {body: [$2, $3::2*]}` — restores the mandatory inner `{ }`; `body` is now a clean array of stream-expressions (was the brace-less `( stream_expression ( comma stream_expression )* )*` with a raw quantified-of-quantified `body`).
  - `stream_expression := expression ( kw_with lbrack array_range_expression rbrack )?` — restores the literal `[ ]` around the with-clause range (was `( kw_with ( array_range_expression )? )?`, which could never accept the bracketed LRM form and left `array_range_expression` unreachable/unwitnessable).
  - `streaming_concatenation := lbrace stream_operator ( slice_size &lbrace )? stream_concatenation rbrace` — the optional `slice_size` is now guarded by an `&lbrace` positive lookahead. Restoring the mandatory inner brace exposed a latent PEG greediness (`slice_size`'s `constant_expression` branch consumed the brace-concatenation that is in fact the mandatory `stream_concatenation`, then failed without backtracking); per LRM a slice_size is always followed by the `{` of the mandatory stream_concatenation, so the guard is transparent (the `{op, slice_size, body}` shape is byte-identical — `slice_size` is unpolluted by the lookahead).
- **Fixes a real parse bug + a shape correction (ledger `SV-0002`):** the LRM with-bracket form `{<< 4 {a with [3:0]}}` now parses (was REJECTED at every release ≤ 1.0.139); the canonical brace forms `{>> {data}}` / `{<< 8 {a, b}}` now parse with the *correct* `stream_concatenation` structure (the inner `{ }` is the stream-concatenation, not a mis-parsed concatenation expression / mis-bound `slice_size`); a bare `with X` without brackets is now correctly REJECTED per the LRM.
- **CONSUMER NOTICE — schema 3 → 4 (shape-changing):** consumers that read streaming-concatenation ASTs must adopt the schema-`4` `body` array shape (`stream_concatenation.body` raw → array; `stream_expression.with_clause` inner shape now carries the bracket tokens). Inputs containing no streaming concatenation are byte-identical.
- Closes the certificate-coverage `array_range_expression` `UNKNOWN`: SV cert `UNKNOWN 89 → 88` (witness `1202 → 1203`; deterministic at seeds 0/7/42; `spf=0`; zero newly-unknown rules).
- Full verification matrix in ledger row `SV-0002` (`docs/contracts/PGEN_RELEASED_PARSER_BUG_LEDGER.md`).

## Release 1.0.139 / Contract 1.0.139 Highlights — GRAMMAR-WELLFORMED.H.11.5-FIX: layout-skipper `#`-arm token theft fixed (ledger `SV-0001`); schema stays 3

- Engine fix (`rust/src/ast_pipeline/ast_based_generator.rs`, parser-agnostic): the generated layout skippers' comment arms are suppressed statically at emit time for every introducer the grammar assigns a non-comment meaning. SV defines `hash := trivia "#"`, so the `#`-to-EOL arm is no longer emitted; the `//`/`/*` arms remain because SV's `line_comment`/`block_comment` tokens are comment-defining and agree with them.
- The fixed defect (the dual case the `VHDL-0002` dynamic guard does not cover): during a speculative `line_comment` attempt, the `#` arm swallowed a real `#` token's line as a "comment", the bogus trivia span was memoized, and the true parse path died on the poisoned memo — rejecting grammar-valid SV on every release since the family shipped.
- Acceptance widening only for grammar-valid inputs; non-breaking; AST-dump schema stays `3` (every previously-parsed input yields a byte-identical AST).
- Full verification matrix in ledger row `SV-0001` (`docs/contracts/PGEN_RELEASED_PARSER_BUG_LEDGER.md`).

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
     (since BRANCH-PREDICATE-LOCALITY.2, 2026-07-06: **2 kinds — class_scope / package_scope**;
      the hierarchical branch was retired — it had been predicate-dead since it landed, so
      `kind:"hierarchical"` was never observable from this rule; identifier-headed hierarchical
      tf calls ride the method-call carriers, `$root.`-headed ones ride `rooted_tf_call_sv_only`)

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
bit_select_expression            -> 3 kinds (method / dollar / expression)
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
  - `docs/book/src/roadmap-and-live-status.md`
- Reference IEEE 1800 LRM corpus (read-only):
  - `docs/systemverilog/2017/` (Annex A formal syntax, plus other annexes)
  - `docs/systemverilog/2023/` (delta + 2023-specific annexes)

## Stable Integration Surface
- Grammar family:
  - `systemverilog`
- Stable host profiles:
  - `sv_2017`
  - `sv_2023`
- Stable convenience entry points (since embedding API `2.0.0` under
  `pgen::families::systemverilog::embed`, one line each over the generic API; the
  `pgen::embedding_api::parse_systemverilog_*` spellings of `1.x` are gone):
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
- **Standard-library bundle (embedding API `2.1.0`, `ENGINE-UNIVERSAL-SERVICES.73`, 2026-09-30).** The IEEE 1800 §G.2 predefined `std::` classes (`process`, `semaphore`, `mailbox`) are preloaded before every parse from a bundle **embedded in the binary at build time** (`parser_libs/sv_2017_std/package/std.facts.json` and the `sv_2023` twin, tracked); no path is resolved at run time, so the binary works wherever it is deployed. `PGEN_PARSER_LIBS_DIR=<dir>` overrides it with `<dir>/sv_<profile>_std/package/std.facts.json`; a directory named there that lacks the bundle REFUSES the parse with a diagnostic naming the knob, the bundle and the expected path — never a silent empty stdlib. ⛔ Before `2.1.0` the bundle was resolved from a compile-time absolute path and a moved tree parsed with no stdlib at all, rejecting `mailbox m;` at module scope and the corpus rows `sv-tests/tests/chapter-15/15.4--mailbox-{blocking,non-blocking}.sv` (both `must_accept`) with a diagnostic that named the wrong cause.
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

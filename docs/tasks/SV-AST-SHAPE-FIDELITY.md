# SV-AST-SHAPE-FIDELITY: eliminate `<invalid_sequence_access>` AST-shape corruption on REACHABLE SystemVerilog constructs

## Metadata

- Tree ID: `SV-AST-SHAPE-FIDELITY`
- Status: `active`
- Roadmap lane: SystemVerilog main-parser AST-shape correctness (return-annotation fidelity) — post-audit residuals
- Created: `2026-07-01`
- Owner: repo-local workflow
- Related: `docs/POST_SV_AUDIT_LEDGER.md` (the completed static `{first/rest:$N}` iteration-lead audit), `docs/tasks/POST-SV-AUDIT.md` (`done`), `docs/contracts/PGEN_RELEASED_PARSER_BUG_LEDGER.md` (the `<invalid_sequence_access>` corruption ledger — `SVPP-0001`, `RTL-FE-0002`, `VHDL-0001`, `RTL-CE-0001`).

## Goal

Eliminate every `<invalid_sequence_access>` AST-shape corruption the SystemVerilog
parser emits on **reachable, valid** input. This is the *return-annotation
positional-model corruption* class (`ast_return_transform.rs:193/438` emits the
`<invalid_sequence_access>` sentinel) that the completed `POST-SV-AUDIT` static audit
did **not** cover: that audit swept only iteration-lead `{first:$N, rest:$M}` list/binop
rules, so an inline alternation `( A | B )` (or `( A | B )?`) referenced by a bare
positional `$N` **outside** an iteration was never classified. A parser that emits a
malformed AST (the `<invalid_sequence_access>` sentinel) on valid, common input is a
signoff-blocking correctness defect — the typed AST **is** the product for downstream
consumers (Nexsim). Per [[feedback_be_alert_root_cause_fishy_immediately]] this is a
foundational fishy result to root-cause and fix immediately, not classify-and-route; per
[[project_ebnf_is_single_source_of_truth]] the shape is defined by the EBNF return
annotation only.

Restore each affected rule's intended shape via the **established, precedent-proven
idiom**: lift the inline alternation into a NAMED rule so the bare `$N` binds cleanly
(the `SVPP-0001` `pp_if_keyword` / `RTL-FE-0002` `event_separator` playbook).

## Non-Goals

- NOT the SV certificate-coverage closure (`context_member_method_call`) or the
  closed-loop replay-debt lane (`SV-EXH-PROOF.7.4.x`) — those are separate, deferred-hard
  workstreams on the tuned generation surface.
- NOT a re-run of the completed `POST-SV-AUDIT` iteration-lead list-rule sweep.
- NOT touching `net_port_header` / `interface_port_header` themselves (they are correct);
  only the corrupted *reference* to them is repaired.

## Acceptance Criteria

- Every fixed rule: `<invalid_sequence_access>` count **0** in the parsed AST for a
  minimal reachable repro (measured before→after), with the intended shape restored.
- No regression: SV external corpus **14/14**; SV canonical cert `UNKNOWN=20` / sound
  recognized 4-config union `UNKNOWN=1` unchanged (or improved) at seeds 0/7/42 (`spf=0`);
  the 6 fully-certified grammars byte-identical `fully_certified=true`; `cargo test --lib`
  green; clippy source-clean; `ast_shape_contract` green (manifest re-locked to the
  corrected shape + a port-bearing sample added so the class cannot silently re-drift).
- Full COMMIT.md lockstep per leaf: shape-contract manifest re-locked, SV parser book +
  integration contract "AST-Shape Corrections" entry, released-parser bug-ledger row
  (corruption class), release/schema bump as the consumer-visible shape change warrants.

## Task Tree

- ID: `SV-AST-SHAPE-FIDELITY.1`
  Status: `done` (2026-07-01)
  Goal: Fix the `ansi_port_declaration` inline-alternation-`$1` corruption — the most
  common SV construct (a typed ANSI module port) currently yields
  `<invalid_sequence_access>` for the port `header` sub-shape.
  Rule: `grammars/systemverilog.ebnf:519-520`
  `ansi_port_declaration := ( net_port_header | interface_port_header )? port_identifier unpacked_dimension* ( assign constant_expression )? -> {kind: "net_or_interface", header: $1, name: $2, dims: $3, default: $4}`
  Fix (precedent-proven): lift `( net_port_header | interface_port_header )` into a NAMED
  un-annotated rule `ansi_port_header := net_port_header | interface_port_header`, then
  `ansi_port_declaration := ansi_port_header? port_identifier unpacked_dimension* ( assign constant_expression )? -> {kind: "net_or_interface", header: $1, name: $2, dims: $3, default: $4}`.
  `$1` now binds the clean named rule (annotation text unchanged); the branch result
  (`net_port_header` `{direction, port_type}` or `interface_port_header`
  `{kind:"named", name, modport}`) passes through intact.

  ### Acceptance Checklist (enforced)
  - [x] **REPRODUCE / ISSUE** — `module m (input logic a); endmodule` →
    `./rust/target/release/parseability_probe --parse-dump-ast-pretty systemverilog … --profile sv_2017`
    yields `ports[…].header = {kind:"net_or_interface", header:{direction,port_type}, default:"<invalid_sequence_access>", dims:"<invalid_sequence_access>", name:"<invalid_sequence_access>"}` — **3 `<invalid_sequence_access>` per typed port** (6 for a 2-port module).
  - [x] **ROOT CAUSE (WHY + WHERE)** — `grammars/systemverilog.ebnf:519-520`
    (`ansi_port_declaration`): `header: $1` binds a bare positional `$1` to the **inline
    alternation inside an optional group** `( net_port_header | interface_port_header )?`.
    The inline-alt-`$N` corrupts the positional model in
    `rust/src/ast_pipeline/ast_return_transform.rs` (sentinel emitted at `:193`/`:438`), so
    the rule's own annotation is mis-recursed onto the branch result — `net_port_header`'s
    correct `{direction, port_type}` is re-wrapped as `header.header` and the out-of-range
    `dims`/`default`/`name` slots emit `<invalid_sequence_access>`. IDENTICAL mechanism to
    the ledgered `SVPP-0001` (`pp_if_branch`'s `keyword:$1` over `(kw_ifdef|kw_ifndef)`),
    `RTL-FE-0002` (`event_control_list`), `RTL-CE-0001`, `VHDL-0001`. NOT covered by
    `POST-SV-AUDIT` (that audit was scoped to `{first/rest:$N}` iteration leads).
  - [x] **FIX** — lifted the inline alternation into the new un-annotated named rule
    `ansi_port_header := net_port_header | interface_port_header` (`grammars/systemverilog.ebnf:519`);
    `ansi_port_declaration` (`:521-522`) now reads `ansi_port_header? port_identifier
    unpacked_dimension* ( assign constant_expression )? -> {kind:"net_or_interface",
    header:$1, name:$2, dims:$3, default:$4}` (annotation text unchanged). Fix-hierarchy
    tier = Level-1 declarative grammar edit; the `SVPP-0001` `pp_if_keyword` /
    `RTL-FE-0002` `event_separator` precedent idiom. SV parser regenerated (`make -C rust
    focus_systemverilog`).
  - [x] **ADDRESSED (verified)** — `module m (input logic a); endmodule` via
    `parseability_probe --parse-dump-ast-pretty systemverilog … --profile sv_2017`:
    `<invalid_sequence_access>` count per typed port **3 → 0** (6 → 0 for the 1-port
    module's full dump); `header` now the clean `{direction:{kind:"input"}, port_type:{…}}`
    (`net_port_header`), the outer port object keeps `{kind:"net_or_interface", header,
    name, dims, default}`.
  - [x] **NO REGRESSION** — SV external corpus **14/14** (`parse_pass_total=14
    parse_fail_total=0`, both uvm cases through the regenerated parser); SV canonical cert
    `total 1304→1305 / witness 1283→1284 / UNKNOWN=20` unchanged — IDENTICAL 20-rule
    residual (`ansi_port_header` witnessed; ZERO newly-unknown; `comm -13` empty), byte-identical
    seeds 0/7/42 (`spf=0`); 6 fully-certified grammars byte-identical `fully_certified=true`
    (SV-only regen); `cargo test --lib --features generated_parsers` **739 passed / 0 failed**;
    `systemverilog_ast_shape_contract` **1 passed** (ran, not compiled-out; validates the 2
    new samples against the running regenerated parser); `--lint-grammar`
    `ordered_choice_shadowing=0` **1426 rules** (`always_matches 7→8` = the now-visible
    pre-existing `net_port_header`-always-matches shadow, behavior-preserving; documented);
    clippy source-clean.
  - [x] **LOCKSTEP** — shape-contract manifest `systemverilog_v1.json` re-locked with 2
    port-bearing samples (`ansi_port_header_typed` rule_under_test=`ansi_port_header`,
    `ansi_port_typed` rule_under_test=`ansi_port_declaration`) + `ast_shape_contract.rs`
    callback arms; released-parser bug-ledger row `SV-0014` (corruption class); integration
    contract release/contract `1.0.151 → 1.0.152` + schema `6 → 7` + § "AST-Shape Corrections
    — 1.0.152" + Highlights; SV parser book (`schema-versioning.md` schema-`7` row + a
    backfilled schema-`6` bridging row + intro fix `now 3`→`now 7`; `welcome.md` version
    `4`→`7` + "early phase" drift corrected to "mature"; `json-carrier.md` `ansi_port_header`
    + `ansi_port_declaration` rows) rebuilt via `systemverilog_parser_book_gate`; CHANGES /
    DEVELOPMENT_NOTES / MEMORY / LIVE_ACHIEVEMENT_STATUS updated.

- ID: `SV-AST-SHAPE-FIDELITY.2`
  Status: `in_progress` (2026-07-01)
  Goal: Sweep `grammars/systemverilog.ebnf` for OTHER reachable inline-alternation-`$N`
  corruptions (a bare positional `$N` referencing an inline `( A | B )` / `( A | B )?`
  group **outside** an iteration lead, the class `POST-SV-AUDIT` did not cover). Enumerate
  candidates statically, then probe each for reachability + `<invalid_sequence_access>`
  tools-first (`--parse-dump-ast-pretty`), one rule per sub-leaf. Fix each via the named-lift
  idiom. `ansi_port_declaration` (`.1`) was the first found; confirm whether it was the only
  common one.

  ### Enumeration result (2026-07-01, delegated static sweep — HIGH confidence)
  A tools-first static audit of `grammars/systemverilog.ebnf` found **21 candidate
  rule-branches** with a **bare `$N`** whose body-position N is an inline alternation group
  `( A | B | … )` (top-level `|`), the exact `.1` class. (6 further inline-alt branches are SAFE
  — they carry no per-branch return annotation, so the `<invalid_sequence_access>` arm cannot
  fire: `trivia` L471, `function_body_declaration` br2/3 L2132, `net_type_declaration_sv_2017`
  br1 L3295, `task_body_declaration` br2/3 L4981.) `ansi_port_declaration` (fixed in `.1`) is
  correctly absent. **These are STATIC candidates — each MUST be verified tools-first
  (`--parse-dump-ast-pretty` on a minimal reachable repro: is it reachable? does it actually
  emit the sentinel?) BEFORE any fix** ([[feedback_no_codebase_change_without_tool_backed_facts]]).
  NOTE candidate #21 `class_scoped_call_prefix` is the `SV-0013` rule — `SV-0013` gated a branch
  INSIDE the alt (predicates) but did NOT change the inline-alt structure, so its `head: $1` may
  still corrupt; verify.

  | # | rule (branch) | line | `$N` | inline-alt group (summary) |
  |---|---|---|---|---|
  | 1 | `class_scope_type` br0 | 1067 | $1 | scoped/known-unscoped class-scope identifiers (4-way) |
  | 2 | `base_class_type` br0 | 1094 | $1 | scoped/known-unscoped base-class identifiers (3-way) |
  | 3 | `constraint_primary_sv_2017` br0 | 1422 | $1 | `( implicit_class_handle dot \| class_scope )?` |
  | 4 | `constraint_primary_sv_2023` br0 | 1426 | $1 | `( implicit_class_handle dot \| class_scope )?` |
  | 5 | `scoped_block_type_identifier` br0 | 1634 | $1 | `( class_scope \| non_typedef_package_scope )` |
  | 6 | `scoped_data_type_identifier` br0 | 1641 | $1 | `( class_scope \| non_typedef_package_scope )` |
  | 7 | `hierarchical_btf_identifier` br2 | 2253 | $1 | `( hierarchical_identifier dot \| class_scope )?` |
  | 8 | `interface_class_type` br0 | 2451 | $1 | scoped/known-unscoped interface-class ids (2-way) |
  | 9 | `split_hierarchical_callable_receiver` br0 | 2827 | $1 | `( kw_class_qualifier… \| non_typedef_package_scope )?` |
  | 10 | `method_call_receiver_sv_2017` br1 | 2839 | $1 | `( kw_class_qualifier… \| non_typedef_package_scope )?` |
  | 11 | `method_call_receiver_sv_2017` br13 | 2839 | $6 | `( implicit_class_handle dot \| class_scope )?` |
  | 12 | `method_call_receiver_sv_2023` br1 | 2858 | $1 | `( kw_class_qualifier… \| non_typedef_package_scope )?` |
  | 13 | `method_call_receiver_sv_2023` br13 | 2858 | $6 | `( implicit_class_handle dot \| class_scope )?` |
  | 14 | `nettype_declaration_sv_2023` br1 | 3344 | $2 | `( non_typedef_package_scope \| class_scope )?` |
  | 15 | `nonrange_variable_lvalue` br0 | 3433 | $1 | `( implicit_class_handle dot \| package_scope \| class_scope )?` |
  | 16 | `ps_or_hierarchical_array_identifier` br0 | 4281 | $1 | `( implicit_class_handle dot \| class_scope \| non_typedef_package_scope )?` |
  | 17 | `ps_type_identifier_sv_2017` br0 | 4326 | $1 | `( kw_local… scope_resolution kw_n… \| non_typedef_package_scope \| class_scope )?` |
  | 18 | `ps_type_identifier_sv_2023` br0 | 4330 | $1 | `( kw_local… scope_resolution kw_n… \| non_typedef_package_scope \| class_scope )?` |
  | 19 | `type_declaration_sv_2017` br5 | 5199 | $2 | `( kw_enum \| kw_struct \| kw_union )?` |
  | 20 | `type_declaration_sv_2023` br5 | 5215 | $2 | `( kw_enum \| kw_struct \| kw_union )?` |
  | 21 | `class_scoped_call_prefix` br0 | 6282 | $1 | scoped/known-unscoped class-scoped-call ids (4-way); **= SV-0013 rule** |

  **Fix strategy (from `.1` precedent):** lift each inline-alt into a NAMED rule; several
  candidates share the SAME scope-prefix alternation (`implicit_class_handle dot | class_scope
  | …`), so a handful of shared named rules (e.g. `class_or_package_scope_prefix`) cover most.
  Sequence as small verified clusters (one leaf per rule or per shared-named-rule cluster),
  accept/reject + `<invalid_sequence_access>` before→after proven tools-first per leaf, full
  NO-REGRESSION (corpus 14/14 + cert unchanged + fully-certified byte-identical) + lockstep
  each. Schema/release bump only where a fix changes a REACHABLE consumer-visible shape (like
  `.1`); latent-but-unreachable candidates get a manifest calibration note, not a bump.

  **RESUME (fresh session):** for each of the 21, construct a minimal reachable repro →
  `parseability_probe --parse-dump-ast-pretty systemverilog <file> --profile sv_2017` → confirm
  the sentinel → named-lift fix → re-verify → lockstep. Start with the reachability-obvious,
  consumer-common ones (`type_declaration` forward-typedef #19/#20; the `method_call_receiver`
  #10-13; the `scoped_*_type_identifier` #5/#6).

- ID: `SV-AST-SHAPE-FIDELITY.2.1`
  Status: `done` (2026-07-01)
  Goal: Fix the `.2` enumeration candidates **#19 + #20** — the `type_declaration`
  forward-typedef `keyword` inline-alternation-`$2` corruption. A forward typedef with a
  keyword (`typedef enum e_t;` / `typedef struct s_t;` / `typedef union u_t;`) — a common SV
  idiom used to break declaration cycles — currently yields `<invalid_sequence_access>` in the
  `keyword` sub-shape. Both profile rules (`type_declaration_sv_2017` br6 `:5209`,
  `type_declaration_sv_2023` br6 `:5225`) carry the IDENTICAL inline-alt
  `( kw_enum_e338e8e3 | kw_struct_d118e5a3 | kw_union_67ad5a07 )?` referenced by a bare `$2`, so
  ONE shared un-annotated named rule (`forward_type_keyword`) fixes both — the `.1`
  `ansi_port_header` named-lift precedent.

  ### Acceptance Checklist (enforced)
  - [x] **REPRODUCE / ISSUE** — `module m; typedef enum e_t; endmodule` →
    `./rust/target/release/parseability_probe --parse-dump-ast-pretty systemverilog … --profile sv_2017`
    yields the `forward` node `keyword:{kind:"class_alias", class_type:"<invalid_sequence_access>",
    dims:"<invalid_sequence_access>", name:"<invalid_sequence_access>"}` — **3 `<invalid_sequence_access>`**
    (`enum`/`struct` = 3, `union` = 4). The keyword-less `typedef e_t;` is clean (0) — the optional
    group is simply absent, so the corruption only fires when a branch matches.
  - [x] **ROOT CAUSE (WHY + WHERE)** — `grammars/systemverilog.ebnf:5209` / `:5225`
    (`type_declaration_sv_2017`/`_sv_2023` br6): `keyword: $2` binds a bare positional `$2` to the
    inline alternation-inside-optional-group `( kw_enum_e338e8e3 | kw_struct_d118e5a3 |
    kw_union_67ad5a07 )?`. The inline-alt-`$N` corrupts the positional model in
    `rust/src/ast_pipeline/ast_return_transform.rs` (sentinel `:193`/`:438`): the rule's own
    branch-1 return annotation (`{kind:"class_alias", class_type, name, dims}`) is mis-recursed onto
    the `keyword` slot with out-of-range positions → `<invalid_sequence_access>`. IDENTICAL mechanism
    to `.1` (`ansi_port_declaration`), `SVPP-0001`, `RTL-FE-0002`, `RTL-CE-0001`, `VHDL-0001`. `name: $3`
    resolves correctly (`{body:"e_t"}`) — only the `$2` inline-alt slot corrupts.
  - [x] **FIX** — lifted the shared inline alternation into the new un-annotated named rule
    `forward_type_keyword := kw_enum_e338e8e3 | kw_struct_d118e5a3 | kw_union_67ad5a07`
    (`grammars/systemverilog.ebnf`, placed just before the outer `type_declaration`); both br6 now read
    `kw_typedef_6dc2082b forward_type_keyword? declared_type_identifier semi -> {kind:"forward",
    keyword:$2, name:$3}` (annotation text + positions unchanged; the two br6 lines were byte-identical
    ⇒ one `replace_all` edit + one shared rule fix BOTH profiles). Fix-hierarchy tier = Level-1
    declarative grammar edit; `.1` `ansi_port_header` precedent. SV parser regenerated (`make -C rust
    focus_systemverilog`), release `parseability_probe` + debug `ast_pipeline` rebuilt.
  - [x] **ADDRESSED (verified)** — before→after on the keyword repros
    (`--parse-dump-ast-pretty` on `sv_2017` AND `sv_2023`): `typedef enum/struct/union e_t;`
    `<invalid_sequence_access>` **3/4 → 0** (both profiles); `keyword` now the clean fused-token
    `[[], "enum"]` (`[trivia, text]`, the grammar's direct-keyword convention); `typedef e_t;` (no keyword)
    still clean (`keyword: []`). `name` stays `{body:"e_t"}`. `parse_full` still passes.
  - [x] **NO REGRESSION** — SV external corpus **14/14** (`parse_pass_total=14 parse_fail_total=0`,
    `sv_external_corpus_triage_gate`); SV canonical cert `total 1305→1306 / witness 1284→1285 /
    UNKNOWN=20` unchanged seeds 0/7/42 (`spf=0`; `forward_type_keyword` witnessed; identical 20-rule
    residual — 19 `no_path` + `context_member_method_call`; ZERO newly-unknown, `forward_type_keyword`
    absent from the UNKNOWN set); 6 fully-certified grammars byte-identical `fully_certified=true`
    (SV-only regen; codegen untouched); `cargo test --lib --features generated_parsers` **739/0**;
    `systemverilog_ast_shape_contract` **18/0** (the 2 new samples validated against the regenerated
    parser); `--lint-grammar` `ordered_choice_shadowing=0 non_terminating=0 unreachable=0
    profile_orphans=0` (1426→1427 rules, `always_matches=8` unchanged); clippy source-clean.
  - [x] **LOCKSTEP** — shape-contract manifest `systemverilog_v1.json` (+2 samples
    `forward_typedef_keyword`/`forward_typedef_enum`, +1 calibration note) + `ast_shape_contract.rs`
    dispatch arms (`forward_type_keyword`, `type_declaration_sv_2017`); released-parser bug-ledger row
    `SV-0015`; SV integration contract `1.0.152 → 1.0.153` + schema `7 → 8` + § "AST-Shape Corrections
    — 1.0.153"; SV parser book (`schema-versioning.md` schema-8 row + intro `now 8`, `welcome.md`
    version `8`, `json-carrier.md` 2 rows, `changelog-index.md` `1.0.153` entry) rebuilt GREEN via
    `systemverilog_parser_book_gate`; CHANGES / DEVELOPMENT_NOTES / MEMORY / LIVE_ACHIEVEMENT_STATUS
    updated.

- ID: `SV-AST-SHAPE-FIDELITY.2.2`
  Status: `done` (2026-07-01)
  Goal: Fix the `.2` enumeration candidate **#2** — the `base_class_type` head
  inline-alternation-`$1` corruption. A derived class whose base is a
  package-scoped class (`class D extends pkg::B;` — a ubiquitous SV/UVM idiom,
  e.g. `class my_driver extends uvm_pkg::uvm_driver;`) currently yields
  `<invalid_sequence_access>` for the `base_class_type` `params` + `scope_chain`
  sub-shapes. ONE un-annotated named rule (`base_class_type_head`) fixes both
  profiles (both `class_declaration_sv_2017:955` and `class_declaration_sv_2023:959`
  reference the single shared `base_class_type` rule via `extends`), mirroring the
  already-correct sibling `class_type_head` idiom (`systemverilog.ebnf:1099-1104`)
  and the `.1` `ansi_port_header` / `.2.1` `forward_type_keyword` precedent.

  ### Acceptance Checklist (enforced)
  - [x] **REPRODUCE / ISSUE** — `class D extends pkg::B; endclass` →
    `./rust/target/release/parseability_probe --parse-dump-ast-pretty systemverilog … --profile sv_2017`
    (also `sv_2023`) yields the `extends[1].head` (`base_class_type`) node
    `{head:{name:{body:"B"}, scope:{…pkg…}}, params:"<invalid_sequence_access>",
    scope_chain:"<invalid_sequence_access>"}` — **2 `<invalid_sequence_access>` per
    scoped-base extends**, both profiles. The declared-bare form
    `class B; endclass class D extends B;` is already clean (`{head:{body:"B"},
    params:[], scope_chain:[]}`) — the bare branch carries no per-branch annotation,
    so only the annotated `scoped_base_class_type_identifier` branch triggers the
    positional-model corruption.
  - [x] **ROOT CAUSE (WHY + WHERE)** — `grammars/systemverilog.ebnf:1094`
    (`base_class_type`): `head: $1` binds a bare positional `$1` to the inline 3-way
    alternation `( scoped_base_class_type_identifier |
    known_unscoped_base_class_type_identifier |
    known_unscoped_base_class_type_parameter_identifier )`. The inline-alt-`$N`
    corrupts the positional model in
    `rust/src/ast_pipeline/ast_return_transform.rs` (sentinel `:193`/`:438`): when the
    self-annotated `scoped_base_class_type_identifier` branch (`-> {scope:$1,name:$2}`)
    wins, the rule's own annotation `{head:$1, params:$2, scope_chain:$3}` is
    mis-recursed so `params` (`$2` = `( parameter_value_assignment )?`) and
    `scope_chain` (`$3` = `( scope_resolution class_identifier … )*`) resolve to
    out-of-range positions → `<invalid_sequence_access>`. IDENTICAL mechanism to `.1`
    (`ansi_port_declaration`), `.2.1` (`type_declaration` br6), `SVPP-0001`,
    `RTL-FE-0002`, `RTL-CE-0001`, `VHDL-0001`. The sibling `class_type` (`:1099`) is
    already correct precisely because it lifts its head into the named `class_type_head`
    (`:1102`) — `base_class_type` is the un-lifted twin.
  - [x] **FIX** — lifted the inline alternation into the new un-annotated named rule
    `base_class_type_head := scoped_base_class_type_identifier |
    known_unscoped_base_class_type_identifier |
    known_unscoped_base_class_type_parameter_identifier`
    (`grammars/systemverilog.ebnf`, placed just before `base_class_type`); rewrote
    `base_class_type := base_class_type_head ( parameter_value_assignment )? (
    scope_resolution class_identifier ( parameter_value_assignment )? )* -> {head:$1,
    params:$2, scope_chain:$3}` (annotation text + positions unchanged). Fix-hierarchy
    tier = Level-1 declarative grammar edit; the `.1`/`.2.1` named-lift precedent + the
    in-file `class_type_head` idiom. SV parser regenerated (`make -C rust
    focus_systemverilog`), release `parseability_probe` + debug `ast_pipeline` rebuilt.
  - [x] **ADDRESSED (verified)** — before→after on both profiles
    (`--parse-dump-ast-pretty` on `class D extends pkg::B; endclass`):
    `<invalid_sequence_access>` **2 → 0** (both `sv_2017` AND `sv_2023`); the
    `base_class_type` node's `params`/`scope_chain` go from the corrupted sentinels to
    the clean empty arrays `[]`, `head` stays the correct `scoped_base_class_type_identifier`
    `{name, scope}`. The declared-bare `class B; class D extends B;` stays clean
    (`{head:{body:"B"}, params:[], scope_chain:[]}`) — no regression on the bare branch.
    `parse_full` still passes.
  - [x] **NO REGRESSION** — SV external corpus **14/14** (`sv_external_corpus_triage_gate`,
    `parse_pass_total=14 parse_fail_total=0`); SV canonical cert `total 1306→1307 /
    witness 1285→1286 / UNKNOWN=20` unchanged seeds 0/7/42 (`spf=0`; identical 20-rule
    residual — 19 `no_path` + `context_member_method_call`; `base_class_type_head`
    witnessed, ZERO newly-unknown); 6 fully-certified grammars byte-identical
    `fully_certified=true` (SV-only regen; codegen untouched — json/regex spot-checked
    `fully_certified=true`; git diff = grammar + docs only); `cargo test --lib --features
    generated_parsers` **739/0**; `systemverilog_ast_shape_contract` **11/11** aligned
    (2 new samples validated against the running regenerated parser); `--lint-grammar`
    `ordered_choice_shadowing=0 non_terminating=0 unreachable=0 profile_orphans=0`
    (1427→1428 rules, `always_matches=8` unchanged); clippy source-clean.
  - [x] **LOCKSTEP** — shape-contract manifest `systemverilog_v1.json` (+2 samples
    `base_class_type_head_scoped`/`base_class_type_scoped`, +1 calibration_history entry)
    + `ast_shape_contract.rs` dispatch arms (`base_class_type_head`, `base_class_type`);
    released-parser bug-ledger row `SV-0016`; SV integration contract `1.0.153 → 1.0.154`
    + schema `8 → 9` + § "AST-Shape Corrections — 1.0.154"; SV parser book
    (`schema-versioning.md` schema-9 row + intro `now 9`, `welcome.md` version `9`,
    `json-carrier.md` +2 rows, `changelog-index.md` `1.0.154` entry) rebuilt GREEN via
    `systemverilog_parser_book_gate`; CHANGES / DEVELOPMENT_NOTES / MEMORY /
    LIVE_ACHIEVEMENT_STATUS updated.

- ID: `SV-AST-SHAPE-FIDELITY.2.3`
  Status: `done` (2026-07-01)
  Goal: Fix the `.2` enumeration **scoped-type-reference cluster** — candidates **#1
  `class_scope_type`**, **#5 `scoped_block_type_identifier`**, **#6
  `scoped_data_type_identifier`** — which co-occur in the single ubiquitous repro
  `module m; C::D::E v; endmodule` (a package/class-scoped type reference on a variable
  declaration — extremely common in real SV/UVM). That one repro emits **4
  `<invalid_sequence_access>` sentinels** from TWO distinct inline-alt-`$N` corruptions,
  so the cluster is fixed and verified together (two shared named-lifts) to drive the
  repro to 0. `class_scope_type` (#1) is the EXACT pre-`.2.2` `base_class_type` pattern;
  `scoped_block_type_identifier`/`scoped_data_type_identifier` (#5/#6) are byte-identical
  and share ONE inline alt, so one shared prefix rule fixes both — the `.2.1`
  two-identical-lines / `.2.2` shared-`base_class_type` precedent.

  ### Acceptance Checklist (enforced)
  - [x] **REPRODUCE / ISSUE** — `module m; C::D::E v; endmodule` →
    `./rust/target/release/parseability_probe --parse-dump-ast-pretty systemverilog … --profile sv_2017`
    yields the `data_type` `{kind:"scoped_data_type", body:{…}}` node with **4
    `<invalid_sequence_access>`**: the OUTER `scoped_data_type_identifier` `scope` value is a
    corrupted `{scope:<real class_scope>, type:"<invalid_sequence_access>",
    dims:"<invalid_sequence_access>"}` (self-annotation mis-recursed onto the `class_scope`
    branch result); the NESTED `class_scope_type` head is
    `{head:{name:{body:"D"},scope:{…C…}}, params:"<invalid_sequence_access>",
    scope_chain:"<invalid_sequence_access>"}`.
  - [x] **ROOT CAUSE (WHY + WHERE)** — two rules, IDENTICAL mechanism to `.1`/`.2.1`/`.2.2`
    (`SVPP-0001`, `RTL-FE-0002`, `RTL-CE-0001`, `VHDL-0001`), the inline-alt-`$N`
    positional-model corruption in `rust/src/ast_pipeline/ast_return_transform.rs` (sentinel
    `:193`/`:438`):
    - **#6 `scoped_data_type_identifier`** (`grammars/systemverilog.ebnf:1649-1650`) and its
      byte-identical block twin **#5 `scoped_block_type_identifier`** (`:1642-1643`):
      `scope: $1` binds a bare positional `$1` to the inline alt
      `( class_scope | non_typedef_package_scope )`; the `class_scope` branch is self-annotated
      (`class_scope -> {body:$1}`, `:1071`), so when it wins the rule's own annotation
      `{scope:$1, type:$2, dims:$3}` is mis-recursed onto the branch result → `type`/`dims`
      resolve to out-of-range positions → `<invalid_sequence_access>`.
    - **#1 `class_scope_type`** (`grammars/systemverilog.ebnf:1067-1068`): `head: $1` binds a
      bare positional `$1` to the inline 4-way alt
      `( scoped_class_scope_identifier | known_unscoped_class_scope_class_identifier |
      known_unscoped_class_scope_interface_class_identifier |
      known_unscoped_class_scope_type_parameter_identifier )`; the self-annotated
      `scoped_class_scope_identifier` branch (`-> {scope:$1, name:$2}`, `:1049`) wins →
      `params: $2` and `scope_chain: $3` corrupt → `<invalid_sequence_access>`. This is the
      un-lifted twin of the already-correct `base_class_type` (fixed in `.2.2`) and
      `class_type` (in-file precedent) — same `{head, params, scope_chain}` shape.
    The inline alt `( class_scope | non_typedef_package_scope )` occurs ONLY at #5/#6 (grep-proven),
    so one shared prefix rule fixes both with no other reference affected.
  - [x] **FIX** — two Level-1 declarative named-lifts (the `.1`/`.2.1`/`.2.2` precedent):
    (a) `class_scope_type_head := scoped_class_scope_identifier | known_unscoped_class_scope_class_identifier | known_unscoped_class_scope_interface_class_identifier | known_unscoped_class_scope_type_parameter_identifier`;
    `class_scope_type := class_scope_type_head ( parameter_value_assignment )? ( scope_resolution class_identifier ( parameter_value_assignment )? &scope_resolution )* -> {head:$1, params:$2, scope_chain:$3}`
    (annotation text + `&scope_resolution` tail-lookahead unchanged; `grammars/systemverilog.ebnf:1100`);
    (b) `scoped_type_scope_prefix := class_scope | non_typedef_package_scope` (`:1651`); both
    `scoped_block_type_identifier` (`:1653`)/`scoped_data_type_identifier` (`:1659`) become
    `scoped_type_scope_prefix type_identifier packed_dimension* -> {scope:$1, type:$2, dims:$3}`.
    +2 new named rules only. SV parser regenerated (`make -C rust focus_systemverilog`); release
    `parseability_probe` + debug `ast_pipeline` rebuilt.
  - [x] **ADDRESSED (verified)** — before→after on `module m; C::D::E v; endmodule`
    (`sv_2017` AND `sv_2023`): `<invalid_sequence_access>` **4 → 0** BOTH profiles;
    `class_scope_type` head `params`/`scope_chain` → `[]`, `scoped_data_type_identifier` `scope`
    → clean class_scope, `type`→`{body:"E"}`, `dims`→`[]`; `parse_full` still passes. The
    `var C::D::E` block forms also parse clean (0 sentinels).
  - [x] **NO REGRESSION** — SV external corpus **14/14** (`sv_external_corpus_triage_gate`
    `GATE_EXIT=0`, `parse_pass_total=14 parse_fail_total=0`, 18 parse_full logs incl. bootstrap
    variants all PASS); SV canonical cert `total 1307→1309 / witness 1286→1288 / UNKNOWN=20`
    unchanged, seeds 0/7/42 byte-identical (`spf=0`, `proof_reverify_failures=0`; IDENTICAL 20-rule
    residual — 19 `no_path` + `context_member_method_call`, `PGEN_CERT_COVERAGE_DUMP_ALL` diff empty;
    the 2 new named rules + the 3 rewritten rules ALL witnessed, absent from UNKNOWN); 6
    fully-certified grammars byte-identical `fully_certified=true` (SV-only regen — only
    `generated/systemverilog_parser.rs` mtime changed; json `UNKNOWN=0`/regex `UNKNOWN=0`
    spot-checked `fully_certified=true`); `cargo test --lib --features generated_parsers`
    **739/0/21ignored**; `systemverilog_ast_shape_contract` **PASS** (4 new samples validated
    against the regenerated parser); `--lint-grammar` `ordered_choice_shadowing=0 non_terminating=0
    unreachable=0 profile_orphans=0` (1428→1430 rules, `always_matches=8` unchanged); clippy
    source stage `ok` (generated non-strict = established debt).
  - [x] **LOCKSTEP** — shape-contract manifest `systemverilog_v1.json` (+4 scoped-type samples
    `class_scope_type_head_scoped`/`class_scope_type_scoped`/`scoped_type_scope_prefix_class`/`scoped_data_type_identifier_scoped`
    + calibration_history entry) + `ast_shape_contract.rs` dispatch arms (4); released-parser
    bug-ledger row `SV-0017`; SV integration contract `1.0.154 → 1.0.155` + schema `9 → 10` +
    § "AST-Shape Corrections — 1.0.155"; SV parser book (`schema-versioning.md` schema-10 row +
    intro `now 10`, `welcome.md` version `10`, `json-carrier.md` +4 rows, `changelog-index.md`
    `1.0.155` entry) rebuilt GREEN via `systemverilog_parser_book_gate` (mdbook_build + tracked_html
    both pass); CHANGES / DEVELOPMENT_NOTES / MEMORY / LIVE_ACHIEVEMENT_STATUS / TASK_TREE updated.

- ID: `SV-AST-SHAPE-FIDELITY.2.4`
  Status: `done` (2026-07-01, session #12)
  Goal: TOOL-BUILD (general, parser-agnostic debug capability) + candidate
  reclassification. Session #12's tools-first probing of the resume-pointer's
  "next" cluster (`method_call_receiver` #10–13) established that the REMAINING
  `.2` candidates are NOT the same easy class as the first six fixed candidates
  (#1/#2/#5/#6/#19/#20). The first six all sit on **declaration surfaces**
  (module ports, forward typedefs, base-class-of, scoped data-type on a var decl)
  that are NOT shadowed. The remaining candidates sit in **expression / receiver /
  constraint / lvalue** positions that the general SV expression grammar
  **PEG-shadows**, so a natural minimal repro parsed from the canonical entry does
  not reach them — and the `<invalid_sequence_access>` sentinel is a **runtime**
  condition (`ast_return_transform.rs:193`: fires for `$N` N≥2 only when the matched
  body is not a `Sequence` long enough; `$1` has an `other => other.clone()` fallback
  and never hits it), so the candidates **cannot be classified statically**. To
  classify each remaining candidate rigorously (reachable-corrupt / latent-corrupt /
  benign) I must dump the AST of a scoped instance **parsed from that candidate rule
  as the entry** — but the AST-dump path does not honor `--entry-rule` (a tool gap).
  This leaf (a) builds that capability, then (b) uses it to produce a verdict table
  for the 15 remaining candidates. Any candidate proven reachable-AND-corrupt gets its
  own follow-on fix sub-leaf (the `.2.1`/`.2.2`/`.2.3` named-lift idiom); latent-corrupt
  gets a manifest calibration note; benign is dropped from the candidate list.

  ### Tools-first diagnosis already established (session #12, read-only — the REPRODUCE/ROOT-CAUSE evidence)
  - `pkg::obj.foo()` / `x = pkg::obj.foo();` (canonical entry) → `parse_full` PASS,
    **0 sentinels, 0 `method_call_receiver` nodes**; the AST footprint shows it routed
    via `split_direct_callable` → `split_hierarchical_callable_receiver` (**candidate
    #9**), because `method_call_initial`'s FIRST branch
    `split_direct_callable_method_call := split_hierarchical_callable_receiver dot …`
    PEG-wins over `direct_method_call` (which is the only path to
    `method_call_receiver` #10). So **#10 is shadowed by #9**.
  - **#9 itself fires but is CLEAN** on `pkg::obj.foo()` (0 sentinels) despite carrying
    the SAME `scope: $1`-over-`( kw_class_qualifier | non_typedef_package_scope )?`
    inline-alt pattern — proof the inline-alt-`$N` pattern is NOT universally corrupting;
    corruption depends on the winning branch's runtime `Sequence` packaging.
  - `constraint_primary` (#3/#4) via `this.x` / `B::x` in a constraint, and
    `nonrange_variable_lvalue` (#15) via `this.x = …`, both parse clean and DO NOT reach
    their specialized rules (general expression path shadows them): 0 sentinels, 0 rule
    nodes.
  - Sentinel mechanism pinned: `rust/src/ast_pipeline/ast_return_transform.rs:193`
    (`_ => ParseContent::Terminal("<invalid_sequence_access>")`), reached for a positional
    `$N` with element_index ≥ 1 (i.e. `$2`+) when the base is not a `Sequence` with
    `len > element_index`. `$1` (element_index 0, L159–181) has an `other => other.clone()`
    fallback and cannot emit the sentinel.
  - Tool gap pinned: `parseability_probe --parse` honors `--entry-rule`
    (`parser_registry::parse_sample_detail_from_entry` → `parser.parse_full_from(entry)`),
    but `--parse-dump-ast[-pretty]` does NOT — `command_parse_dump_ast`
    (`rust/src/bin/parseability_probe.rs:484`) calls
    `parser_registry::parse_sample_ast_json_with_profile(…)` which parses from the CANONICAL
    entry (`parse_with_<g>_ast_json[_profile]` → `parser.parse_full_<entry>()`), and the
    `--parse-dump-ast[-pretty]` dispatch arms (`:641`/`:669`) never pass `options.entry_rule`.
    Demonstrated: `--parse pkg::obj --entry-rule method_call_receiver_sv_2017` → PASS, but
    `--parse-dump-ast-pretty pkg::obj --entry-rule method_call_receiver_sv_2017` → rejected at
    position 0 (fell back to canonical `systemverilog_file`).

  ### Fix scope (registry-only, no codegen — confirmed by reading `parse_with_systemverilog_ast_json_profile`)
  Add an entry-aware AST-JSON path that mirrors the existing canonical one but calls
  `parser.parse_full_from(entry)` in place of `parser.parse_full_<canonical>()` and feeds
  the same `parse_node_to_json(&parsed)`:
  - `parser_registry`: `parse_sample_ast_json_from_entry(grammar, sample, profile, entry)`
    + per-grammar `parse_with_<g>_ast_json_from_entry(sample, profile, entry)` variants
    (at minimum `systemverilog`; general dispatcher returns `None` for grammars lacking a
    variant, byte-identical for every other path).
  - `parseability_probe`: thread `entry_rule: Option<&str>` into `command_parse_dump_ast`
    and pass `options.entry_rule.as_deref()` from both `--parse-dump-ast` / `--parse-dump-ast-pretty`
    dispatch arms; `None` keeps the byte-identical canonical path. Update the usage block.

  ### Acceptance Checklist (enforced)
  - [x] **REPRODUCE / ISSUE** — the tool gap (dump path ignores `--entry-rule`), demonstrated by the PASS-vs-reject asymmetry on `pkg::obj` @ `method_call_receiver_sv_2017`: `--parse … --entry-rule` PASS but `--parse-dump-ast-pretty … --entry-rule` → "rejected at position 0" (fell back to canonical `systemverilog_file`).
  - [x] **ROOT CAUSE (WHY + WHERE)** — `command_parse_dump_ast` (`rust/src/bin/parseability_probe.rs:484`) → `parser_registry::parse_sample_ast_json_with_profile` (canonical entry only; `parse_with_<g>_ast_json[_profile]` → `parse_full_<canonical>()`); the `--parse-dump-ast[-pretty]` dispatch arms (`:641`/`:669`) never forwarded `options.entry_rule`.
  - [x] **FIX** — registry-only, no codegen: added `parser_registry::parse_sample_ast_json_from_entry` (dispatcher) + `parse_with_systemverilog_ast_json_from_entry` (`parse_full_from(entry)` + `parse_node_to_json`); threaded `entry_rule: Option<&str>` through both `command_parse_dump_ast` variants + both dump dispatch arms; usage string + `--entry-rule` help updated. Fix-hierarchy tier = tool/observability code.
  - [x] **ADDRESSED (verified)** — release `parseability_probe` rebuilt; `--parse-dump-ast-pretty pkg::obj --entry-rule method_call_receiver_sv_2017` now dumps the AST (was reject) and reveals **1 sentinel**. Full isolation + canonical-reachability classification of all 15 remaining candidates produced (verdict table below). The tool surfaced **3 REACHABLE+CORRUPT bugs (#8, #14, #21)** that natural canonical-entry repros entirely missed.
  - [x] **NO REGRESSION** — omitting `--entry-rule` is byte-identical (canonical dump on `module m (input logic a); endmodule` → 0 sentinels, unchanged); `cargo test --lib --features generated_parsers` **739 passed / 0 failed / 21 ignored**; the 6 fully-certified grammars unaffected (registry/probe-only — no grammar/codegen touch, no parser regen); clippy — [pending background `clippy_on_rust_change`].
  - [x] **LOCKSTEP** — `TOOLBOX.md` (1.2 entry) + book debug chapter (`docs/book/src/parseability-probe-debug.md`) note the new `--entry-rule` on the dump; probe usage string + `--entry-rule` help updated; CHANGES / DEVELOPMENT_NOTES / MEMORY / LIVE_ACHIEVEMENT_STATUS / TASK_TREE updated.

  ### Verdict table — 15 remaining `.2` candidates (tool-proven, session #12)
  Reachability = canonical `systemverilog_file` entry; isolation = `--entry-rule <rule>` (empty store, so only non-store-gated `this.` / `pkg::` scoped branches fire).

  | # | rule | isolation repro → sentinels | canonical-entry reachable? | VERDICT |
  |---|---|---|---|---|
  | 8 | `interface_class_type` | `pkg::IC` → 1 | **YES** — `class C implements pkg::IC;` → 1 | **REACHABLE-CORRUPT → fix (`.2.5`)** |
  | 14 | `nettype_declaration_sv_2023` (br1 `with`) | `nettype logic n with pkg::f;` → 3 | **YES** — `module m; nettype logic n with pkg::f; endmodule` → 3 (also pkg/unit scope) | **REACHABLE-CORRUPT → fix (`.2.6`)** |
  | 21 | `class_scoped_call_prefix` | `pkg::C::` → 2 | **YES** — `pkg::C::foo()` (stmt + asn-RHS) → 2 | **REACHABLE-CORRUPT → fix (`.2.7`)** |
  | 3 | `constraint_primary_sv_2017` | `this.x` → 1 | NO — `constraint cc { this.x < 5; }` → 0 (general-expr shadow) | LATENT (calibration note) |
  | 4 | `constraint_primary_sv_2023` | `this.x` → 2 | NO (same shadow) | LATENT |
  | 10 | `method_call_receiver_sv_2017` (br1) | `pkg::obj` → 1 | NO — `pkg::obj.foo()` → 0 (shadowed by #9 `split_hierarchical_callable_receiver` via `split_direct_callable_method_call`, the first `method_call_initial` branch) | LATENT |
  | 11 | `method_call_receiver_sv_2017` (br13 null_class) | (obscure; not constructed) | NO — same rule shadowed by #9 | LATENT |
  | 12 | `method_call_receiver_sv_2023` (br1) | `pkg::obj` → 1 | NO (shadowed by #9) | LATENT |
  | 13 | `method_call_receiver_sv_2023` (br13) | (obscure) | NO (shadowed) | LATENT |
  | 15 | `nonrange_variable_lvalue` | `this.x` → 1 (`pkg::x` → 0) | NO — `this.x = 1;` → 0 (general-expr shadow) | LATENT |
  | 9 | `split_hierarchical_callable_receiver` | `pkg::x` → 0 | reached (`pkg::obj.foo()`) but 0 | **BENIGN** (0 in isolation + reachable-clean) |
  | 16 | `ps_or_hierarchical_array_identifier` | `pkg::arr` → 0, `this.arr` → 0 | — | **BENIGN** |
  | 17 | `ps_type_identifier_sv_2017` | `pkg::T` → 0 | — | **BENIGN** |
  | 18 | `ps_type_identifier_sv_2023` | `pkg::T` → 0 | — | **BENIGN** |
  | 7 | `hierarchical_btf_identifier` | pattern ABSENT (now `tf|block`, `body:$1` only) | — | **STALE** (static enumeration drifted) |

  Key runtime insight (from `ast_return_transform.rs:193`): the sentinel is branch-sensitive — a self-annotated *single-rule* alt branch (`non_typedef_package_scope`) and a *sequence* alt branch (`implicit_class_handle dot`) package differently at runtime, so e.g. `nonrange_variable_lvalue` corrupts on `this.x` but not `pkg::x`. Only `$N` N≥2 can hit the sentinel; `$1` has the `other => other.clone()` fallback.

- ID: `SV-AST-SHAPE-FIDELITY.2.5`
  Status: `open` (2026-07-01, session #12)
  Goal: Fix candidate **#8 `interface_class_type`** — `class C implements pkg::IC; endclass`
  (a class implementing a package-scoped interface class — a real, reachable SV/UVM idiom)
  emits **1 `<invalid_sequence_access>`** (proven reachable in `.2.4`). Rule
  `grammars/systemverilog.ebnf:2478` `interface_class_type := ( scoped_interface_class_type_identifier
  | known_unscoped_interface_class_type_identifier ) ( parameter_value_assignment )? -> {name:$1, params:$2}`
  — IDENTICAL `{name/head:$1, params:$2}` shape to `base_class_type` (`.2.2`). Fix: named-lift
  `interface_class_type_head := scoped_interface_class_type_identifier | known_unscoped_interface_class_type_identifier`,
  then `interface_class_type := interface_class_type_head ( parameter_value_assignment )? -> {name:$1, params:$2}`.
  Full acceptance checklist + release/schema bump + lockstep per the `.2.1`/`.2.2`/`.2.3` precedent.

- ID: `SV-AST-SHAPE-FIDELITY.2.6`
  Status: `open` (2026-07-01, session #12)
  Goal: Fix candidate **#14 `nettype_declaration_sv_2023`** br1 `with`-clause — `nettype logic n
  with pkg::f;` (a nettype with a package-scoped resolution function — reachable at module/package/
  compilation-unit scope, proven in `.2.4`) emits **3 `<invalid_sequence_access>`**. The inline alt
  `( non_typedef_package_scope | class_scope )?` sits inside `( kw_with ( … )? tf_identifier )?` bound
  by `with_clause:$4`. Fix: named-lift the scope alt (shared `nettype_with_scope_prefix` or reuse
  `scoped_type_scope_prefix` if the alt is byte-identical — grep-verify first) so the `$4` positional
  model is restored. Full checklist + lockstep. (`sv_2023`-gated — verify the `sv_2017` nettype rule
  for a twin.)

- ID: `SV-AST-SHAPE-FIDELITY.2.7`
  Status: `open` (2026-07-01, session #12)
  Goal: Fix candidate **#21 `class_scoped_call_prefix`** (the `SV-0013` rule) — `pkg::C::foo()`
  (a package-class-scoped static method call — reachable in statement + assignment-RHS context,
  proven in `.2.4`) emits **2 `<invalid_sequence_access>`**. Rule `grammars/systemverilog.ebnf:6317`
  `class_scoped_call_prefix := ( scoped_class_scoped_call_prefix_identifier | known_unscoped_class_scoped_call_class_identifier
  | known_unscoped_class_scoped_call_interface_class_identifier | known_unscoped_class_scoped_call_type_parameter_identifier )
  ( parameter_value_assignment )? scope_resolution ( … )* -> {head:$1, params:$2, scope_chain:$4}` —
  IDENTICAL 4-way `{head:$1, params:$2, scope_chain:$N}` shape to `class_scope_type` (`.2.3`). Fix:
  named-lift `class_scoped_call_prefix_head := <4 alts>`. NOTE: `SV-0013` gated branches INSIDE the alt
  but did not lift it — this completes that rule. Full checklist + release/schema bump + lockstep.

- ID: `SV-AST-SHAPE-FIDELITY.3`
  Status: `open` (2026-07-01)
  Goal: Backfill the SV parser book `schema-versioning.md` per-release timeline rows for
  releases `1.0.148`–`1.0.151` (the schema-`6` era: ledger `SV-0010`/`SV-0011`/`SV-0012`/
  `SV-0013`), which were never added to the book timeline (pre-existing lockstep drift
  discovered during `.1`). `.1` added the schema-`7` row, a bridging schema-`6` row (the
  `1.0.147` 5→6 bump, `SV-0009`), and corrected the stale intro (`now 3` → `now 7`) +
  `welcome.md` (`version 4` → `7`, "early phase" → "mature"); this leaf completes the
  per-release backfill so the book timeline is row-complete against the contract Contract
  Identity schema note. **Scope extended (`.2.1`, 2026-07-01):** the SV book
  `changelog-index.md` was also found to be missing per-release entries for `1.0.151`–`1.0.152`
  (its newest entry was `1.0.150`; `.2.1` added its own `1.0.153` entry above the gap) — backfill
  the `1.0.151`/`1.0.152` `changelog-index.md` entries here too, alongside the
  `schema-versioning.md` `1.0.148`–`1.0.151` rows, so both SV-book timelines are row-complete.

## Decisions

- 2026-07-01: The corruption class is the same one the released-parser bug ledger reserves
  a row for (unlike the clean Category-A list-shape improvements). `ansi_port_declaration`
  is REACHABLE and consumer-visible (every typed module port), so this is a genuine
  released-parser bug → a bug-ledger row + a release bump, mirroring `SVPP-0001`.
- 2026-07-01: Schema bumped `6 → 7` (not "no bump"): unlike the strictly-more-permissive
  `SV-0010`..`SV-0013`, this fix CHANGES the emitted AST for a common construct (a typed
  ANSI port's `header`) — a consumer-visible structural change warrants a schema bump per the
  contract's schema-versioning policy, exactly as the `SV-0002` / `SV-0007` shape bumps.
- 2026-07-01: Pre-existing SV-book drift discovered during `.1` lockstep (per
  [[feedback_be_alert_root_cause_fishy_immediately]] — flagged + partially corrected, not
  routed away). `schema-versioning.md` intro said the schema "is now `3`" and the timeline
  topped out at schema `5`/`1.0.146`; `welcome.md` said schema `4` and called the campaign
  "early phase / mostly un-annotated". `.1` corrected the intro (`now 7`), `welcome.md`
  (`version 7`, "mature"), and added the schema-`7` + bridging schema-`6` rows; the full
  per-release backfill of `1.0.148`–`1.0.151` is scoped to `.3` to keep `.1` focused on the
  port fix.

## Open Questions

- (during `.1`) Are there OTHER reachable inline-alt-`$N` corruptions on common SV
  constructs beyond `ansi_port_declaration`? A follow-up leaf `.2` will sweep the grammar
  for the pattern and probe each for reachability (tools-first, one rule per leaf).

## Blockers

- none.

## Verification Log

- 2026-07-01 (`.1`): tools-first before→after on `module m (input logic a); endmodule`
  (`--parse-dump-ast-pretty systemverilog --profile sv_2017`): `<invalid_sequence_access>`
  **6 → 0**; `port.header` corrupted `{header:{direction,port_type}, default/dims/name:
  "<invalid_sequence_access>"}` → clean `{direction:{kind:"input"}, port_type:{…}}`.
- 2026-07-01 (`.1`) NO-REGRESSION: SV external corpus triage gate
  `parse_pass_total=14 parse_fail_total=0` (**14/14**); SV canonical cert `total 1304→1305
  witness 1283→1284 UNKNOWN=20 spf=0` seeds 0/7/42 byte-identical (identical 20-rule
  residual, `comm -13` empty, `ansi_port_header` witnessed); 6 fully-certified grammars
  byte-identical; `cargo test --lib --features generated_parsers` 739/0;
  `systemverilog_ast_shape_contract` 1/0 (ran; 2 new samples validated); `--lint-grammar`
  `ordered_choice_shadowing=0` 1426 rules (`always_matches 7→8` pre-existing shadow, documented);
  clippy source-clean.
- 2026-07-01 (`.2.1`): tools-first before→after on `module m; typedef enum e_t; endmodule` (also
  `struct`/`union`), `--parse-dump-ast-pretty` on `sv_2017` AND `sv_2023`: forward node `keyword`
  `<invalid_sequence_access>` **3/4 → 0** both profiles; corrupted `{kind:"class_alias",
  class_type/dims/name:"<invalid_sequence_access>"}` → clean fused-token `[[], "enum"]`; keyword-less
  `typedef e_t;` still clean (`keyword: []`); `name` stays `{body:"e_t"}`; `parse_full` passes.
- 2026-07-01 (`.2.1`) NO-REGRESSION: SV external corpus triage gate `parse_pass_total=14
  parse_fail_total=0` (**14/14**, regenerated parser, no timeouts); SV canonical cert `total 1305→1306
  witness 1284→1285 UNKNOWN=20 spf=0` seeds 0/7/42 byte-identical (identical 20-rule residual — 19
  `no_path` + `context_member_method_call`; `comm -13` empty; `forward_type_keyword` witnessed, absent
  from the UNKNOWN set); 6 fully-certified grammars byte-identical (codegen untouched, SV-only regen);
  `cargo test --lib --features generated_parsers` 739/0; `systemverilog_ast_shape_contract` 18/0 (2 new
  forward-typedef samples validated against the regenerated parser); `--lint-grammar`
  `ordered_choice_shadowing=0 non_terminating=0 unreachable=0 profile_orphans=0` 1426→1427 rules
  (`always_matches=8` unchanged); `systemverilog_parser_book_gate` GREEN (mdbook_build + tracked_html);
  clippy source-clean.
- 2026-07-01 (`.2.2`): tools-first before→after on `class D extends pkg::B; endclass`,
  `--parse-dump-ast-pretty` on `sv_2017` AND `sv_2023`: `base_class_type` node
  `<invalid_sequence_access>` **2 → 0** both profiles; corrupted `{head:{name,scope},
  params/scope_chain:"<invalid_sequence_access>"}` → clean `{head:{name,scope}, params:[],
  scope_chain:[]}`; declared-bare `class B; class D extends B;` still clean
  (`{head:{body:"B"}, params:[], scope_chain:[]}`); `parse_full` passes.
- 2026-07-01 (`.2.2`) NO-REGRESSION: SV external corpus triage gate `parse_pass_total=14
  parse_fail_total=0` (**14/14**); SV canonical cert `total 1306→1307 witness 1285→1286 UNKNOWN=20
  spf=0` seeds 0/7/42 byte-identical (identical 20-rule residual — 19 `no_path` +
  `context_member_method_call`; `comm -13` empty; `base_class_type_head` witnessed, absent from the
  UNKNOWN set); 6 fully-certified grammars byte-identical (json/regex spot-checked `fully_certified=true`;
  codegen untouched, SV-only regen; git diff = grammar + docs only); `cargo test --lib --features
  generated_parsers` 739/0; `systemverilog_ast_shape_contract` 11/11 aligned (2 new base_class_type
  samples validated against the regenerated parser); `--lint-grammar` `ordered_choice_shadowing=0
  non_terminating=0 unreachable=0 profile_orphans=0` 1427→1428 rules (`always_matches=8` unchanged);
  `systemverilog_parser_book_gate` GREEN (mdbook_build + tracked_html); clippy source-clean.
- 2026-07-01 (`.2.3`): tools-first before→after on `module m; C::D::E v; endmodule`,
  `--parse-dump-ast-pretty` on `sv_2017` AND `sv_2023`: the `scoped_data_type` node
  `<invalid_sequence_access>` **4 → 0** both profiles — the outer `scoped_data_type_identifier`
  `scope` is now the clean `class_scope`, `type`→`{body:"E"}`, `dims`→`[]`, and the nested
  `class_scope_type` head `params`/`scope_chain`→`[]`; `parse_full` passes. The `var C::D::E`
  block forms also parse clean (0 sentinels).
- 2026-07-01 (`.2.3`) NO-REGRESSION: SV external corpus triage gate `GATE_EXIT=0`
  `parse_pass_total=14 parse_fail_total=0` (**14/14**, 18 parse_full logs incl. bootstrap variants);
  SV canonical cert `total 1307→1309 witness 1286→1288 UNKNOWN=20 spf=0 proof_reverify_failures=0`
  seeds 0/7/42 byte-identical (IDENTICAL 20-rule residual — 19 `no_path` + `context_member_method_call`;
  `PGEN_CERT_COVERAGE_DUMP_ALL` diff empty; `class_scope_type_head` + `scoped_type_scope_prefix`
  witnessed, absent from the UNKNOWN set); 6 fully-certified grammars byte-identical (json/regex
  spot-checked `fully_certified=true`; codegen untouched, SV-only regen — only
  `generated/systemverilog_parser.rs` mtime changed); `cargo test --lib --features generated_parsers`
  739/0/21ignored; `systemverilog_ast_shape_contract` PASS (4 new scoped-type samples validated
  against the running regenerated parser); `--lint-grammar` `ordered_choice_shadowing=0
  non_terminating=0 unreachable=0 profile_orphans=0` 1428→1430 rules (`always_matches=8` unchanged);
  `systemverilog_parser_book_gate` GREEN (mdbook_build + tracked_html); clippy source stage `ok`.

- 2026-07-01 (`.2.4`, session #12): TOOL-BUILD verified — release `parseability_probe` rebuilt;
  `--parse-dump-ast-pretty pkg::obj --entry-rule method_call_receiver_sv_2017` now dumps the AST (was
  "reject at position 0") revealing 1 sentinel; canonical dump (no `--entry-rule`) byte-identical
  (0 sentinels on `module m (input logic a); endmodule`); `cargo test --lib --features
  generated_parsers` **739/0/21ignored**; `clippy_on_rust_change` exit 0 (source-clean);
  registry/probe-only (no grammar/codegen regen). Classification of all 15 remaining candidates
  (isolation `--entry-rule` + canonical-entry reachability): **3 REACHABLE-CORRUPT** (#8
  `interface_class_type` `implements pkg::IC` → 1; #14 `nettype…with pkg::f` → 3; #21
  `class_scoped_call_prefix` `pkg::C::foo()` → 2), **7 LATENT** (#3/#4/#10-13/#15 — PEG-shadowed),
  **4 BENIGN** (#9/#16/#17/#18), **1 STALE** (#7). Fix leaves `.2.5`/`.2.6`/`.2.7` opened.

## Commit Log

- 2026-07-01 (`.1`): `PGEN-SV-AST-SHAPE-FIDELITY-0001 (SV-AST-SHAPE-FIDELITY.1)` — committed `e0a672f4`.
- 2026-07-01 (`.2.1`): `PGEN-SV-AST-SHAPE-FIDELITY-0002 (SV-AST-SHAPE-FIDELITY.2.1)` — committed `88ffe2b3`.
- 2026-07-01 (`.2.2`): `PGEN-SV-AST-SHAPE-FIDELITY-0003 (SV-AST-SHAPE-FIDELITY.2.2)` — committed `5854cbad`.
- 2026-07-01 (`.2.3`): `PGEN-SV-AST-SHAPE-FIDELITY-0004 (SV-AST-SHAPE-FIDELITY.2.3)` — committed `12e8905c`.
- 2026-07-01 (`.2.4`): `PGEN-SV-AST-SHAPE-FIDELITY-0005 (SV-AST-SHAPE-FIDELITY.2.4)` — pending commit.

## Changelog

- 2026-07-01: Tree created; `.1` root-cause established tools-first (`ansi_port_declaration`
  inline-alt-`$1` corruption on typed ANSI ports).
- 2026-07-01: `.2` enumeration DONE (delegated static sweep) — **21 HIGH-confidence candidate
  rule-branches** recorded in the `.2` leaf (bare `$N` over an inline-alt group), + 6 safe
  no-annotation branches. Per-candidate tools-first verify + named-lift fix PENDING (fresh
  session); `.2` stays `in_progress`.
- 2026-07-01: `.1` FIXED + fully verified + lockstepped (named-lift `ansi_port_header`;
  release/schema `1.0.152`/schema `7`; ledger `SV-0014`). During `.1` lockstep discovered
  pre-existing SV-book `schema-versioning.md` drift (intro said "now 3"; timeline missing the
  `1.0.147` 5→6 bump + releases `1.0.148`–`1.0.151`) and `welcome.md` drift (schema "4";
  "early phase" claim) — corrected the intro/version/status + added the schema-`7` & bridging
  schema-`6` rows; opened `.3` to complete the `1.0.148`–`1.0.151` per-release backfill and
  `.2` for the grammar-wide inline-alt-`$N` sweep.
- 2026-07-01: `.2.1` FIXED + fully verified + lockstepped (candidates #19+#20 — the
  `type_declaration_{sv_2017,sv_2023}` br6 forward-typedef `keyword` corruption). Shared named-lift
  `forward_type_keyword` fixes both profiles; release/schema `1.0.153`/schema `8`; ledger `SV-0015`.
  `<invalid_sequence_access>` 3/4→0; corpus 14/14; cert `UNKNOWN=20` unchanged; `ast_shape_contract`
  18/0 (2 new samples); book gate GREEN. `.2` stays `in_progress` — 19 of 21 candidates remain; the
  SV-book `changelog-index.md` `1.0.151`/`1.0.152` gap discovered during `.2.1` lockstep was folded
  into `.3`'s scope.
- 2026-07-01: `.2.2` FIXED + fully verified + lockstepped (candidate #2 — the `base_class_type` head
  inline-alternation-`$1` corruption on a package-scoped base class, `class D extends pkg::B;`). Shared
  named-lift `base_class_type_head` fixes both profiles (both `class_declaration` rules reference the one
  `base_class_type` via `extends`), mirroring the already-correct in-file sibling `class_type_head`;
  release/schema `1.0.154`/schema `9`; ledger `SV-0016`. `<invalid_sequence_access>` 2→0 both profiles
  (`params`/`scope_chain` → `[]`); declared-bare `extends B;` stays clean; corpus 14/14; cert
  `UNKNOWN=20` unchanged (`base_class_type_head` witnessed); `ast_shape_contract` 11/11 (2 new samples);
  book gate GREEN. `.2` stays `in_progress` — **18 of 21 candidates remain** (tools-first probing this
  session also found #5/#6 `scoped_block/data_type_identifier` REACHABLE + corrupt via `C::D::E v;` — a
  shared-named-lift cluster queued next; #1 `class_scope_type` did NOT reproduce a sentinel with common
  repros this session — its head-alt is clean when the winning branch is an un-annotated bare identifier,
  as in `C::D::E`; remains a candidate pending a reachable annotated-branch repro).
- 2026-07-01: `.2.3` FIXED + fully verified + lockstepped (scoped-type-reference **cluster** —
  candidates **#1 `class_scope_type`**, **#5 `scoped_block_type_identifier`**, **#6
  `scoped_data_type_identifier`**). Session #11 tools-first `--parse-dump-ast-pretty` on the single
  ubiquitous repro `module m; C::D::E v; endmodule` surfaced **4** `<invalid_sequence_access>` sentinels
  from TWO nested inline-alt-`$N` corruptions — CORRECTING the `.2.2`-era tentative note above: candidate
  **#1 `class_scope_type` DOES reproduce** (its `head:$1` over a 4-way alt corrupts `params`/`scope_chain`
  when the self-annotated `scoped_class_scope_identifier` branch wins, nested inside the `class_scope` of
  `C::D::E`), alongside #6 `scoped_data_type_identifier` (`scope:$1` over the shared alt
  `( class_scope | non_typedef_package_scope )` corrupts `type`/`dims`). Fixed together via TWO shared
  named-lifts — `class_scope_type_head` (#1) + `scoped_type_scope_prefix` (#5/#6, the shared alt
  grep-proven to occur ONLY at those two byte-identical rules) — the `.1`/`.2.1`/`.2.2` idiom mirroring
  the in-file `class_type_head`. `<invalid_sequence_access>` **4 → 0** both profiles; release/schema
  `1.0.155`/schema `10`; ledger `SV-0017`. Corpus 14/14; cert `total 1307→1309 witness 1286→1288 UNKNOWN=20`
  unchanged (both new rules witnessed; zero newly-unknown); `ast_shape_contract` PASS (4 new samples);
  book gate GREEN; `cargo test --lib` 739/0; clippy source-clean. `.2` stays `in_progress` — **15 of 21
  candidates remain** (fixed: #2, #19, #20, #1, #5, #6); next = `method_call_receiver` #10-13.
- 2026-07-01: `.2.4` (session #12) TOOL-BUILD + classification DONE. Tools-first probing of the
  resume-pointer's "next" cluster (`method_call_receiver` #10-13) revealed the remaining candidates are
  NOT the easy declaration-surface class — they sit in expression/receiver/constraint/lvalue positions
  the general SV grammar PEG-shadows, and the `<invalid_sequence_access>` sentinel is a RUNTIME
  condition (`ast_return_transform.rs:193`, `$N` N≥2 vs a too-short Sequence), so candidates cannot be
  classified statically. Built a GENERAL entry-aware AST-dump capability
  (`parseability_probe --parse-dump-ast[-pretty] --entry-rule RULE`, registry-only via
  `parse_full_from` — no grammar/codegen change) to isolate each rule, then classified all 15 by
  isolation-corruption × canonical-entry reachability. RESULT: **3 REACHABLE-CORRUPT bugs** the natural
  repros entirely missed — **#8 `interface_class_type`** (`class C implements pkg::IC;` → 1), **#14
  `nettype_declaration_sv_2023` `with`-clause** (`nettype logic n with pkg::f;` → 3), **#21
  `class_scoped_call_prefix`** (`pkg::C::foo()` → 2), all the `base_class_type`/`class_scope_type`
  `.2.2`/`.2.3` shape → named-lift fixes queued as `.2.5`/`.2.6`/`.2.7`; **7 LATENT** (calibration
  note); **4 BENIGN**; **1 STALE** (#7, pattern removed). `cargo test --lib` 739/0; clippy exit 0;
  canonical dump byte-identical. `.2` stays `in_progress` — the remaining sweep is now proof-driven:
  3 real fixes + latent calibration, not 15 unknowns.

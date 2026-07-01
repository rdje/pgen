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

- ID: `SV-AST-SHAPE-FIDELITY.3`
  Status: `open` (2026-07-01)
  Goal: Backfill the SV parser book `schema-versioning.md` per-release timeline rows for
  releases `1.0.148`–`1.0.151` (the schema-`6` era: ledger `SV-0010`/`SV-0011`/`SV-0012`/
  `SV-0013`), which were never added to the book timeline (pre-existing lockstep drift
  discovered during `.1`). `.1` added the schema-`7` row, a bridging schema-`6` row (the
  `1.0.147` 5→6 bump, `SV-0009`), and corrected the stale intro (`now 3` → `now 7`) +
  `welcome.md` (`version 4` → `7`, "early phase" → "mature"); this leaf completes the
  per-release backfill so the book timeline is row-complete against the contract Contract
  Identity schema note.

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

## Commit Log

- 2026-07-01 (`.1`): `PGEN-SV-AST-SHAPE-FIDELITY-0001 (SV-AST-SHAPE-FIDELITY.1)` — pending commit.

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

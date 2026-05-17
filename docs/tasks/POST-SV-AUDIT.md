# POST-SV-AUDIT: Holistic post-campaign AST-shape correctness audit

## Metadata

- Tree ID: `POST-SV-AUDIT`
- Status: `active`
- Roadmap lane: shape audit (TaskList #49)
- Created: `2026-05-17`
- Last updated: `2026-05-17`
- Owner: repo-local workflow

## Goal

Perform the deferred holistic AST-shape correctness audit (TaskList
#49) across **all** tracked `grammars/*.ebnf` typed surfaces: enumerate
and classify every `{first/lhs … rest: $N}`-style annotation (and any
bare positional `$N` over a quantified iteration) into the
Category A / B / C taxonomy of
[[feedback_quantified_group_extraction]], fix the objective bugs
(Category-A raw-envelope misuse on pure separator lists; any residual
Category-B inline-alternation-`$N` instance not covered by the
now-closed `INLINE-ALT-FIX` class), and record dispositions for the
benign Category-C / "suboptimal-but-working" cases — each fix in
lockstep with regen + shape-contract manifest + book + contract per
the proven playbook.

## Non-Goals

- Not re-opening the closed systemic inline-alternation binop class
  (`RTL-CE-0001`/`SVPP-0001`/`RTL-FE-0001`/`VHDL-0001` — already fixed)
  except to confirm exhaustiveness.
- Not re-shaping working annotations purely for taste — only objective
  correctness bugs are fixed; benign style differences are recorded,
  not churned.
- Not auditing `return_annotation.ebnf` / `semantic_annotation.ebnf`
  (the annotation meta-grammars) for product AST shape.

## Acceptance Criteria

- Every `{first/lhs … rest:$N}` + raw-`$N`-over-iteration occurrence
  across all product grammars is enumerated and classified A/B/C with
  empirical evidence (`parseability_probe` for any suspected
  `<invalid_sequence_access>` / raw-envelope leak).
- Each confirmed objective bug (Cat-A misuse / residual Cat-B) is
  fixed with regen + manifest + schema/release bump + book + contract
  + bug-ledger lockstep, `*_ast_shape_contract` + `*_parser_book_gate`
  green, independently verified.
- Benign Cat-C / accepted-as-is cases are recorded with rationale.
- The audit ledger is committed; the tree closes with a holistic
  green re-verification.

## Task Tree

- ID: `POST-SV-AUDIT`
  Status: `active`
  Goal: `Holistic post-campaign AST-shape correctness audit + objective-bug fixes.`
  Children: `POST-SV-AUDIT.1`, `POST-SV-AUDIT.2`, `POST-SV-AUDIT.3`

- ID: `POST-SV-AUDIT.1`
  Status: `done`
  Goal: `Audit & classify: enumerate every {first/lhs..rest:$N} + raw-$N-over-iteration annotation across all product grammars; classify A/B/C with parseability_probe evidence; produce the prioritized objective-bug fix worklist (docs/POST_SV_AUDIT_LEDGER.md).`
  Acceptance: `Ledger covers all occurrences in regex/systemverilog/systemverilog_preprocessor/vhdl/rtl_frontend/rtl_const_expr (others justified out-of-scope); each classified A/B/C with evidence; Cat-A misuse + residual Cat-B candidates explicitly listed (incl. the known rtl_frontend event_control_list ( comma|kw_or ) finding); .2 frontier populated from the findings.`
  Verification: `2026-05-17: docs/POST_SV_AUDIT_LEDGER.md created (static analysis; subagent + independent parent verification). 12 ## headers, no dups; counts reconcile vs pre-scope (rtl_const_expr 10, rtl_frontend 28[=27+1 doc-comment], svpp 1, sv 38, vhdl 22; raw {first:$1,rest:$2} rtl_frontend 3/sv 7/vhdl 7); scope = 6 product grammars, 11 non-product/meta/LRM-extracted grammars justified-excluded. Classifications independently spot-checked against the grammar (rtl_frontend event_control_list:162-163, sv unsigned_number:345, sv net_alias:2889, vhdl {first,rest} 30/32/43) — all ACCURATE. Findings: ~35 static-conclusive Cat-A misuse (svpp 1 macro_formals, rtl_frontend 16, vhdl 17, sv 1 net_alias); 6 HIGH inline-alt-$N suspects (rtl_frontend event_control_list + 5 sv number rules digit(kw_sv_rule_c82a06f6|digit)*) — structurally the EXACT signature that empirically produced <invalid_sequence_access> 4× this session, marked "probe-confirm at .2-start" (the proven before/after playbook step, not a .1 gap; SV/rtl_frontend are on-demand parsers needing .2-time regen+wire); 11 structured-per-iteration Cat-A (sv list_of_*identifiers family — need per-rule record-rule design judgement); 30 Cat-B-resolved confirmed (binop class exhaustively re-verified clean across all product grammars); 16 Cat-C benign; Cat-A-already-correct idioms identified for .3. Worklist is concrete, prioritized, per-grammar.`
  Commit: `PGEN-POST-SV-AUDIT-0001`

- ID: `POST-SV-AUDIT.2`
  Status: `done`
  Goal: `Fix the confirmed objective bugs from the .1 ledger (Cat-A raw-envelope misuse on pure separator lists; residual Cat-B inline-alternation-$N), one commit per grammar via the proven regen+manifest+schema/release-bump+book+contract+ledger playbook. Split per grammar (.1 quantified a large per-grammar worklist).`
  Children: `POST-SV-AUDIT.2.1`, `POST-SV-AUDIT.2.2`, `POST-SV-AUDIT.2.3`, `POST-SV-AUDIT.2.4` (all `done`: `.2.4`→`.2.4a`+`.2.4b` both done)

- ID: `POST-SV-AUDIT.2.1`
  Status: `done`
  Goal: `sv_preprocessor: fix macro_formals Cat-A raw-envelope misuse (-> extraction-spread); regen + manifest + schema/release bump + book + contract lockstep. SMALLEST blast radius — proves the Cat-A shape-change lockstep playbook before the larger grammars.`
  Acceptance: `parseability_probe before/after on a multi-formal \`define; macro_formals emits a clean item list (no raw [[comma,item],...] envelope); systemverilog_preprocessor_ast_shape_contract + book gate green; lockstep complete; independently verified.`
  Verification: `2026-05-17: macro_formals := lparen macro_formal (comma macro_formal)* rparen -> {first:$2,rest:$3} (Cat-A raw-envelope misuse) fixed to -> [$2, $3::2*] (canonical object_properties extraction-spread, drop comma). BEFORE probe (\`define M(a, b, c) a+b+c): pp_define.formals = {first:{..a},rest:[[[[],","],{..b}],[[[],","],{..c}]]} (raw comma-envelope). AFTER: [{..a},{..b},{..c}] clean flat macro_formal list; no <invalid_sequence_access> (clean Cat-A, not corruption). Parser+frontend-JSON+inventory regenerated; count UNCHANGED 66/28 (macro_formals annotation_type return_object->return_array only). Manifest dai rebuilt via dump_declared_annotation_inventory + new macro_with_formals sample + extracted_at 2026-05-17 -> systemverilog_preprocessor_ast_shape_contract PASSES. Schema 2->3, release/contract 1.0.2->1.0.3. Contract (new "AST-Shape Corrections — 1.0.3" section + schema-3 row + macro_formals loci, 1.0.2/schema-2 kept historical) + book (8 chapters, walker pins =3, schema-versioning 2->3 row, 66/28 unchanged) + docs/POST_SV_AUDIT_LEDGER.md (macro_formals RESOLVED-FIXED + worklist DONE, original kept as history) lockstep. NO PGEN_RELEASED_PARSER_BUG_LEDGER row (Cat-A clean shape improvement, not corruption — per the recorded Decision). Independently verified: contract no dup ## headers, 1.0.3/schema-3 current (1.0.2/schema-2 historical only), 66/28 unchanged, no fabricated AstDumpPayload (DOC-ENVELOPE not regressed), book gate independently re-run green (searchindex deterministic), clippy strict source passed. docs/book/ checked — no sv_preprocessor mention, no edit. Subagent honestly flagged pre-existing grammar-line-ref drift (predates this fix, partial-corrected in edited loci) + a stale "27-distinct" corrected to verified 28 — both acceptable, not blockers.`
  Commit: `PGEN-POST-SV-AUDIT-0002`

- ID: `POST-SV-AUDIT.2.2`
  Status: `done`
  Goal: `rtl_frontend: fix the 16 static-conclusive Cat-A misuses + the event_control_list inline-alt-$N HIGH finding (named event_separator rule + extraction-spread); regen + manifest + schema/release bump + book + contract + ledger lockstep.`
  Acceptance: `parseability_probe before/after: event_control_list no <invalid_sequence_access>, all 16 Cat-A rules emit clean item lists; rtl_frontend_ast_shape_contract + book gate green; lockstep complete; independently verified.`
  Verification: `2026-05-17: 15 Cat-A raw-envelope misuses fixed to extraction-spreads (parameter_declaration_sequence/port_list/genvar_declaration/scoped_identifier/concatenation_expr -> bare [$N,$M::2*]; port_group ports:[$4,$5::3*]; module_instantiation/net_declaration/repetition_expr/enum_type/struct_union_field/assignment_target -> {siblings, items/names:[$F,$R::2*]}; parameter_override_list+port_connection_list named&positional -> {kind, items:[$2,$3::3*]}). event_control_list RTL-FE-0002: inline ( comma | kw_or ) iteration-lead -> lifted to NEW un-annotated event_separator := comma | kw_or, [$3,$4::2*]. Comprehensive AFTER parseability_probe (module w/ ports/genvar/override named+pos/event_control/concat/repetition): invalid_sequence_access count 0; ZERO [[], ","] separator-envelope leaks; per-rule precise checks confirm port_group.ports = clean port_item list (['dims','name'] keys, $5::3* index CORRECT — !port_direction_token negative-lookahead occupies a slot, inference probe-confirmed), parameter_override/port_connection .items = clean {kind,name} list ($3::3* &dot/!dot index CORRECT), event_control = clean [{edge,expr},...]. Inventory UNCHANGED 156/74 (bare-list rules return_object->return_array; {.,items} stay return_object; event_separator un-annotated). Manifest dai rebuilt + cat_a_shapes sample + extracted_at 2026-05-17 -> rtl_frontend_ast_shape_contract PASSES. Schema 2->3, release/contract 1.0.2->1.0.3. Contract (new "## AST-Shape Corrections — 1.0.3" section + schema-3 row + 15 rule loci, 1.0.1/1.0.2/RTL-FE-0001 kept historical) + 9 book chapters (walker pins =3) + POST_SV_AUDIT_LEDGER (16 FIXED + worklist DONE) lockstep. RTL-FE-0002 added to PGEN_RELEASED_PARSER_BUG_LEDGER (EXACTLY 1 insertion, 0 deletions — RTL-FE-0001/SVPP-0001/VHDL-0001 untouched); the 15 Cat-A are NOT bug-ledger rows (recorded Decision: clean shape improvement vs corruption class). Independently verified: contract no dup ## headers, 1.0.3/schema-3 current (1.0.2 historical only), 156/74 unchanged, no fabricated AstDumpPayload (DOC-ENVELOPE not regressed); book gate independently re-run green (searchindex deterministic, no toc rename); clippy strict source passed. docs/book/ checked — no drifting rtl_frontend fact, no edit.`
  Commit: `PGEN-POST-SV-AUDIT-0003`

- ID: `POST-SV-AUDIT.2.3`
  Status: `done`
  Goal: `vhdl: fix the 17 static-conclusive Cat-A misuses (-> extraction-spread, vhdl convention); regen + manifest + schema/release bump + book + contract lockstep.`
  Acceptance: `parseability_probe before/after: all 17 vhdl Cat-A rules emit clean item lists; vhdl_ast_shape_contract + book gate green; lockstep complete; independently verified.`
  Verification: `2026-05-17: 17 vhdl Cat-A raw-envelope misuses fixed to extraction-spreads (14 bare-list {first,rest}->[$F,$R::2*]: library_clause/use_clause/selected_name/identifier_list/generic_interface_list/port_interface_list/parameter_list/enumeration_type_definition/index_constraint/association_list/sensitivity_list/actual_parameter_part/choices/aggregate_choice_list; target aggregate-branch ->{kind:"aggregate",items:[$2,$3::2*]}; aggregate 2 branches keep first_choices/first_value/second, rest:$5->rest:[$5::2*]). All single-token separators (comma/semi/dot/bar) — NO inline-alternation, NO corruption. Comprehensive AFTER parseability_probe (library/use/entity+generics+ports/process/case/aggregate/instantiation): invalid_sequence_access 0; ZERO separator-envelope leaks for ALL 4 sep types ([[],","]/[[],";"]/[[],"."]/[[],"|"] all 0); aggregate named_first rest = clean aggregate_element_association[] list; no leftover {first,rest}-only objects. Inventory UNCHANGED 256/112 (bare-list return_object->return_array; target-agg+aggregate stay return_object). Manifest dai rebuilt + cat_a_shapes sample + extracted_at 2026-05-17 -> vhdl_ast_shape_contract PASSES. Schema 2->3, release/contract 1.0.2->1.0.3. Contract (new "## AST-Shape Corrections — 1.0.3" section + schema-3 row + 17 rule loci, 1.0.1/1.0.2/VHDL-0001 kept historical) + 9 book chapters (walker pins =3) + POST_SV_AUDIT_LEDGER (17 FIXED + worklist DONE) lockstep. NO PGEN_RELEASED_PARSER_BUG_LEDGER row (pure Cat-A, no corruption — per recorded Decision; ledger git-diff empty/untouched). Independently verified: contract no dup ## headers, 1.0.3/schema-3 current (1.0.2 historical only), 256/112 unchanged, no fabricated AstDumpPayload (DOC-ENVELOPE not regressed); book gate independently re-run green (searchindex deterministic); clippy strict source passed. docs/book/ checked — no drift, no edit.`
  Commit: `PGEN-POST-SV-AUDIT-0004`

- ID: `POST-SV-AUDIT.2.4`
  Status: `done`
  Goal: `systemverilog (flagship, released 1.0.115) Cat-A/inline-alt fixes. Split: .2.4a (net_alias + 5 number rules — proven playbooks, decidable) + .2.4b (11 structured-per-iteration — per-rule record-rule design, ledger-flagged "needs parent judgement").`
  Children: `POST-SV-AUDIT.2.4a`, `POST-SV-AUDIT.2.4b`

- ID: `POST-SV-AUDIT.2.4a`
  Status: `done`
  Goal: `systemverilog: net_alias single-token-sep Cat-A (-> {lvalues:[…]}) + the 5 number-rule inline-alt-$N (lift to named *_tail rules); regen + manifest + schema/release bump + book + contract + ledger lockstep.`
  Acceptance: `parseability_probe: net_alias clean {lvalues:[…]}; number-rule corruption resolved (or honestly dispositioned if unreachable); systemverilog_ast_shape_contract + book gate green; lockstep; independently verified.`
  Verification: `2026-05-17: net_alias := … net_lvalue assign net_lvalue ( assign net_lvalue )* … {first:$2,second:$4,rest:$5} -> {lvalues:[$2,$4,$5::2*]} (Cat-A clean flat list); probe-verified on \`module m; wire a,b,c; alias a=b=c;\` -> {lvalues:[{a},{b},{c}]} (3-elem list). 5 number rules (unsigned_number/non_zero_unsigned_number/binary_value/octal_value/hex_value): inline-alt ( kw_sv_rule_c82a06f6 | <digit> ) lifted into NEW un-annotated <rule>_tail rules; {first:$1,rest:$2} annotation UNCHANGED; the 6x-proven inline-alt-$N transformation, correct by construction + clean regen. HONEST FINDING: the number-rule corruption is structurally present but NOT consumer-reproducible — SV systemverilog_file root rejects ALL numeric top-level constructs (parameter/localparam/assign/$display/[15:0]/module-#-headers) in default/sv_2017/sv_2023 (empirically established; pre-existing SV-root coverage limitation, separate, OUT OF AUDIT SCOPE) -> defensive structural correction, NOT a PGEN_RELEASED_PARSER_BUG_LEDGER row (no unreproducible-defect claim — faithful reporting). kw_sv_rule_c82a06f6:=/sv_rule\\b/ is a degenerate LRM-extraction artifact (noted, out of scope). Inventory UNCHANGED 2290/999 (net_alias return_object text-only; number rules unchanged; *_tail un-annotated). Manifest: net_alias sample + calibration_history #117 -> systemverilog_ast_shape_contract PASSES. Schema 1->2, release/contract 1.0.115->1.0.116. Contract (new "## AST-Shape Corrections — 1.0.116" + Release-1.0.116-Highlights w/ embedded schema row, 1.0.115/schema-1 historical kept) + 4 book chapters (walker pins =2) + POST_SV_AUDIT_LEDGER (net_alias+5-number RESOLVED, 11-structured untouched/OPEN) lockstep. NO bug-ledger row (verified PGEN_RELEASED_PARSER_BUG_LEDGER untouched). Independently verified: contract no dup ## headers, 1.0.116/schema-2 current, 2290/999 unchanged, no fabricated AstDumpPayload (the 4 public-api.md grammar/profile:String are real NamedGrammar* fields, untouched, legit per DOC-ENVELOPE closeout); SV book gate independently re-run green (searchindex deterministic); clippy strict source passed; docs/book/ no drift.`
  Commit: `PGEN-POST-SV-AUDIT-0005`

- ID: `POST-SV-AUDIT.2.4b`
  Status: `done`
  Goal: `systemverilog: the 11 structured-per-iteration Cat-A rules (list_of_*identifiers family etc.) — factor each repeated unit into a named record-emitting rule + extraction-spread; regen + manifest + schema/release bump + book + contract lockstep.`
  Acceptance: `Per-rule record-rule design (proven idioms, fields mirroring macro_formal/net_lvalue precedents); parseability_probe before/after clean structured-record lists (no raw multi-field envelope); systemverilog_ast_shape_contract + book gate green; lockstep; independently verified; user reviews the committed result.`
  Verification: `2026-05-17 (user-authorized "design+implement, review after"): 11 structured-per-iteration Cat-A misuses fixed by factoring the repeated multi-field unit into 9 NEW annotated named record rules + extraction-spread, field names preserved. 5 list_of_*_identifiers -> new *_identifier_decl ({name,dims[,init]}) + list [$1,$2::2*]; let/property/sequence_list_of_arguments -> new <x>_named_arg ({name,value}) + mixed {kind:"mixed",head:$1,ordered_tail:[$2::2*],named_tail:[$3::2*]} & named_only {kind:"named_only",items:[$1,$2::2*]}; parameter_port_list type_only {kind:"type_only",items:[$4,$5::3*]}; assignment_pattern named (x2 occurrences) -> shared assignment_pattern_entry ({name,pattern}) + entries:[$3,$4::2*]. Probe-verified reachable list_of_*_identifiers path (module m; wire a,b,c; logic x,y,z;): clean {name,dims,init} record list, 0 invalid_sequence_access, 0 raw [[],","] envelope, no leftover {first:{...},rest} structured envelope. Inventory 2290->2299 / 999->1008 (+9 annotated record rules — DELIBERATE count change, documented; the *_list_of_arguments/parameter_port_list-type_only/assignment_pattern-named branches are likely SV-root-unreachable -> honest defensive-correct-by-construction disposition like .2.4a, not a fresh probe, NOT bug-ledger'd). Manifest structured_decls sample + calibration #118 -> systemverilog_ast_shape_contract PASSES. Schema 2->3, release/contract 1.0.116->1.0.117. Contract (new "## AST-Shape Corrections — 1.0.117" + Release-1.0.117-Highlights w/ embedded schema-3 row, 1.0.116/earlier historical kept) + 5 book chapters (walker pins =3) + POST_SV_AUDIT_LEDGER (11 RESOLVED + worklist DONE; POST-SV-AUDIT.2 fully dispositioned) lockstep. NO PGEN_RELEASED_PARSER_BUG_LEDGER row (Cat-A structured, no corruption — ledger untouched, verified). Independently verified: contract no dup ## headers, 1.0.117/schema-3 current (1.0.116 historical only), 2290->2299/999->1008 stated as +9 change, no fabricated AstDumpPayload (the 4 public-api.md residuals = real NamedGrammar* fields, untouched, legit); SV book gate independently re-run green (searchindex deterministic); clippy strict source passed; docs/book/ no drift.`
  Commit: `PGEN-POST-SV-AUDIT-0006`

- ID: `POST-SV-AUDIT.3`
  Status: `pending`
  Goal: `Cat-C + residual review & close: confirm X X* {first,rest} correctness, disposition remaining suboptimal-but-working shapes (accepted vs deferred, with rationale), holistic green re-verification, close the tree + TaskList #49.`
  Acceptance: `Cat-C uses confirmed correct; every remaining flagged shape has a recorded disposition; all per-family ast_shape_contract + book gates green; tree closed; LIVE/CHANGES/DEVELOPMENT_NOTES/memory updated.`
  Verification: `pending`
  Commit: `pending`

## Current Frontier

| Order | Leaf | Status | Why next |
| --- | --- | --- | --- |
| 1 | `POST-SV-AUDIT.3` | `pending` | **`POST-SV-AUDIT.2` fully done** (`.2.1` svpp + `.2.2` rtl_frontend + `.2.3` vhdl + `.2.4a`/`.2.4b` SV). Final leaf: Cat-C/benign confirmation + accepted-as-is dispositions + holistic green re-verify + close the tree & TaskList #49. |

## Decisions

- `2026-05-17`: User explicitly **activated** the proposed
  `POST-SV-AUDIT` tree (was workflow-gated as `proposed` per
  [[feedback_post_campaign_audit]] / TaskList #49). The whole bounded
  campaign being complete is the precondition the deferral required.
- `2026-05-17`: Empirical pre-scope (recorded so `.1` starts from
  evidence, not zero): audit surface `{first/lhs..rest:$N}` per grammar
  = rtl_const_expr 10, rtl_frontend 28, systemverilog_preprocessor 1,
  systemverilog 38, vhdl 22; raw `{first:$1, rest:$2}` candidates =
  rtl_frontend 3 / systemverilog 7 / vhdl 7; one known residual Cat-B
  inline-alt-separator: `rtl_frontend.ebnf:162` `event_control_list :=
  … ( ( comma | kw_or ) event_control_item )*` (NOT covered by the
  binop `INLINE-ALT-FIX` class — must probe + classify in `.1`).
- `2026-05-17`: Cat-A misuse is the objective-bug priority; Cat-B
  systemic binop class is already closed (confirm exhaustiveness only);
  Cat-C `{first,rest}` is benign (verify, do not churn).
- `2026-05-17`: `.1` complete → `.2` **split per grammar**
  (`.2.1` svpp → `.2.2` rtl_frontend → `.2.3` vhdl → `.2.4`
  systemverilog) because `.1` quantified a large per-grammar worklist
  (~35 static-conclusive Cat-A + 6 inline-alt-$N + 11 structured).
  Order = smallest-blast-radius-first (svpp `macro_formals` proves the
  Cat-A→extraction-spread + schema/release-bump lockstep playbook)
  then ascending size, flagship `systemverilog` last. Each `.2.x` is a
  full per-grammar lockstep slice (regen + manifest + schema/release
  bump + book + contract + ledger), schema bump because the
  `{first,rest}`→`[$1,$2::2*]` shape change is consumer-visible.
- `2026-05-17`: inline-alt-$N suspects (event_control_list, 5 sv
  number rules) are probe-confirmed at the START of their `.2.x`
  slice (the proven before/after playbook step), not in `.1` — SV /
  rtl_frontend are on-demand parsers requiring `.2`-time regen+wire;
  the structural signature is identical to the 4 empirically-confirmed
  binop fixes this session, so the static classification is
  high-confidence.

## Open Questions

- Is `rtl_frontend` `event_control_list`'s `( comma | kw_or )` a
  Cat-A separator-list (→ `[…, $N::M*]`) or does its current
  annotation emit `<invalid_sequence_access>` (residual Cat-B)?
  Resolve empirically at the start of `.1` (probe). Does not block
  `.1` — it is the first item `.1` classifies.

## Blockers

- None. (`POST-SV-AUDIT.2.4b` was blocked on user design input
  2026-05-17 → **RESOLVED same day**: user chose "Design + implement,
  you review after"; `.2.4b` proceeds autonomously with the proven
  named-record-rule idiom + rigorous probe verification, user reviews
  the committed result. Push: user chose "keep holding" past the
  30-cap — recorded in memory `feedback_push_pacing`.)

  (`.3` Cat-C close still waits until `.2.4b` lands.)

## Verification Log

| Date | Leaf | Checks | Result |
| --- | --- | --- | --- |
| `2026-05-17` | `POST-SV-AUDIT` (setup) | tree decomposition vs workflow splitting rules; empirical pre-scope; precondition (campaign complete) confirmed | `pass — activated on explicit user direction; decomposed into audit→fix→close; pre-scope recorded with one concrete known finding` |
| `2026-05-17` | `POST-SV-AUDIT.1` | ledger structure (12 ## headers, no dups) + scope justification + count reconciliation vs pre-scope + independent classification spot-checks vs grammar (event_control_list/unsigned_number/net_alias/vhdl) | `pass — docs/POST_SV_AUDIT_LEDGER.md accurate; ~35 static-conclusive Cat-A + 6 HIGH inline-alt-$N + 11 structured-Cat-A + 30 Cat-B-resolved + 16 Cat-C; worklist concrete & per-grammar; .2 split per grammar; inline-alt-$N probe-confirm deferred to .2-start per the proven playbook (on-demand-parser infra)` |
| `2026-05-17` | `POST-SV-AUDIT.2.1` | before/after parseability_probe; regen (count UNCHANGED 66/28, macro_formals return_object->return_array); manifest dai-rebuild + macro_with_formals sample; systemverilog_preprocessor_ast_shape_contract; contract dup-header+numbers+no-fabricated-AstDumpPayload grep; book walker-pin + fabricated-residual grep; POST_SV_AUDIT_LEDGER FIXED + NO bug-ledger row check; independent book-gate re-run; clippy; docs/book check | `pass — macro_formals Cat-A fixed ({first,rest}->[$2,$3::2*]); probe raw-comma-envelope -> clean macro_formal[] list, no <invalid_sequence_access>; shape-contract PASSES; 66/28 UNCHANGED; schema 2->3 / release 1.0.2->1.0.3; contract+8-book-chapters+audit-ledger lockstep; NO bug-ledger row (Cat-A clean improvement per Decision); no dup ## headers; DOC-ENVELOPE not regressed; book gate independently green. Cat-A playbook proven for .2.2/.2.3/.2.4.` |
| `2026-05-17` | `POST-SV-AUDIT.2.2` | comprehensive AFTER parseability_probe + precise per-rule shape checks (port_group $5::3*, override/connection $3::3*, event_control $4::2* — all CORRECT); 0 separator-envelope-leak + 0 invalid_sequence_access; regen (156/74 UNCHANGED); manifest dai-rebuild + cat_a_shapes sample; rtl_frontend_ast_shape_contract; contract dup-header+numbers grep; book walker-pin + fabricated-residual grep; POST_SV_AUDIT_LEDGER 16-FIXED; bug-ledger EXACTLY-1-insertion(RTL-FE-0002)-0-deletions check; independent book-gate re-run; clippy; docs/book check | `pass — 15 rtl_frontend Cat-A misuses fixed (extraction-spreads, all 3 inferred-index edge cases probe-confirmed) + event_control_list RTL-FE-0002 inline-alt fix (named event_separator); 156/74 UNCHANGED; schema 2->3 / release 1.0.2->1.0.3; contract(new AST-Shape-Corrections-1.0.3 section)+9-book-chapters+audit-ledger lockstep; RTL-FE-0002 only-new bug-ledger row (others untouched); 15 Cat-A NOT bug-ledger (Decision); no dup ## headers; DOC-ENVELOPE not regressed; book gate independently green` |
| `2026-05-17` | `POST-SV-AUDIT.2.3` | comprehensive AFTER parseability_probe (all 4 sep types) + aggregate-rest precise check; 0 separator-envelope-leak (`,`/`;`/`.`/`\|`) + 0 invalid_sequence_access; regen (256/112 UNCHANGED); manifest dai-rebuild + cat_a_shapes sample; vhdl_ast_shape_contract; contract dup-header+numbers grep; book walker-pin + fabricated-residual grep; POST_SV_AUDIT_LEDGER 17-FIXED; bug-ledger-untouched check; independent book-gate re-run; clippy; docs/book check | `pass — 17 vhdl Cat-A misuses fixed (extraction-spreads, single-token seps); 256/112 UNCHANGED; schema 2->3 / release 1.0.2->1.0.3; contract(new AST-Shape-Corrections-1.0.3 section)+9-book-chapters+audit-ledger lockstep; NO bug-ledger row (pure Cat-A, ledger untouched — Decision); no dup ## headers; DOC-ENVELOPE not regressed; book gate independently green` |
| `2026-05-17` | `POST-SV-AUDIT.2.4b` | probe reachable list_of_*_identifiers path; regen (2290->2299/999->1008 +9 record rules); structured_decls sample + calibration #118; systemverilog_ast_shape_contract; contract dup-header+1.0.117/schema-3+count-change grep; book walker-pin (=3) + fabricated-residual (4=legit untouched public-api NamedGrammar) grep; POST_SV_AUDIT_LEDGER 11-FIXED + POST-SV-AUDIT.2-dispositioned; bug-ledger-untouched check; independent SV book-gate re-run; clippy; docs/book check | `pass — 11 structured-per-iteration Cat-A fixed via 9 factored named record rules + extraction-spread (field names preserved); reachable list_of_*_identifiers probe-clean (clean {name,dims,init} list, 0 invalid_seq, 0 raw-envelope, no leftover {first,rest}); 2290->2299/999->1008 deliberate +9 change documented; schema 2->3 / release 1.0.116->1.0.117; contract(new AST-Shape-Corrections-1.0.117)+5-book-chapters+audit-ledger lockstep; NO bug-ledger row (Cat-A structured, ledger untouched); unreachable branches honestly defensive-dispositioned; no dup ## headers; DOC-ENVELOPE not regressed; book gate independently green. Closes .2.4 -> POST-SV-AUDIT.2.` |
| `2026-05-17` | `POST-SV-AUDIT.2.4a` | net_alias probe ({lvalues:[a,b,c]}); 5 number rules proven-transformation + clean regen; numeric-reachability empirically tested (all numeric top-level constructs rejected, all profiles); manifest net_alias sample + calibration #117; systemverilog_ast_shape_contract; contract dup-header+1.0.116/schema-2+2290/999 grep; book walker-pin (=2) + fabricated-residual (4=legit public-api NamedGrammar fields, untouched) grep; POST_SV_AUDIT_LEDGER net_alias+5-number-FIXED & 11-structured-OPEN check; bug-ledger-untouched check; independent SV book-gate re-run; clippy; docs/book check | `pass — net_alias Cat-A {lvalues:[$2,$4,$5::2*]} probe-clean; 5 number rules defensive structural fix (named *_tail; corruption structurally-present-but-unreachable-via-SV-root, honestly NOT bug-ledger'd — faithful); 2290/999 UNCHANGED; schema 1->2 / release 1.0.115->1.0.116; contract(new AST-Shape-Corrections-1.0.116)+4-book-chapters+audit-ledger lockstep; bug-ledger untouched; no dup ## headers; DOC-ENVELOPE not regressed (4 residuals = real NamedGrammar fields); book gate independently green. .2.4b (11 structured) BLOCKED on user design input.` |

## Commit Log

| Leaf | Commit subject or reference | Notes |
| --- | --- | --- |
| `POST-SV-AUDIT` (setup) | `PGEN-POST-SV-AUDIT-0000` | tree created + activated (Proposed→Active); frontier `.1`; empirical pre-scope recorded |
| `POST-SV-AUDIT.1` | `PGEN-POST-SV-AUDIT-0001` | ledger `docs/POST_SV_AUDIT_LEDGER.md` (all 6 product grammars classified A/B/C, verified accurate); `.2` split per grammar; frontier → `.2.1` |
| `POST-SV-AUDIT.2.1` | `PGEN-POST-SV-AUDIT-0002` | sv_preprocessor `macro_formals` Cat-A fix (`{first,rest}`→`[$2,$3::2*]`); schema 2→3 / release 1.0.2→1.0.3; 66/28 unchanged; contract+book+audit-ledger lockstep, NO bug-ledger row; Cat-A playbook proven |
| `POST-SV-AUDIT.2.2` | `PGEN-POST-SV-AUDIT-0003` | rtl_frontend 15 Cat-A misuses → extraction-spreads + `event_control_list` `RTL-FE-0002` inline-alt fix (named `event_separator`); schema 2→3 / release 1.0.2→1.0.3; 156/74 unchanged; contract(AST-Shape-Corrections-1.0.3)+book+audit-ledger lockstep; `RTL-FE-0002` only-new bug-ledger row; 15 Cat-A not bug-ledger |
| `POST-SV-AUDIT.2.3` | `PGEN-POST-SV-AUDIT-0004` | vhdl 17 Cat-A misuses → extraction-spreads (single-token seps); schema 2→3 / release 1.0.2→1.0.3; 256/112 unchanged; contract(AST-Shape-Corrections-1.0.3)+book+audit-ledger lockstep; NO bug-ledger row (pure Cat-A, ledger untouched) |
| `POST-SV-AUDIT.2.4a` | `PGEN-POST-SV-AUDIT-0005` | SV net_alias Cat-A `{lvalues:[…]}` (probe-clean) + 5 number-rule defensive structural fix (named `*_tail`; corruption unreachable-via-SV-root, honestly NOT bug-ledger'd); schema 1→2 / release 1.0.115→1.0.116; 2290/999 unchanged; contract(AST-Shape-Corrections-1.0.116)+book+audit-ledger lockstep; bug-ledger untouched. `.2.4b` (11 structured) blocked on user design input |
| `POST-SV-AUDIT.2.4b` | `PGEN-POST-SV-AUDIT-0006` | SV 11 structured-per-iteration Cat-A → 9 factored named record rules + extraction-spread (field names preserved); reachable path probe-clean; schema 2→3 / release 1.0.116→1.0.117; 2290→2299 / 999→1008 (+9 record rules, deliberate); contract(AST-Shape-Corrections-1.0.117)+book+audit-ledger lockstep; NO bug-ledger row. **Closes `.2.4` → `POST-SV-AUDIT.2` fully done** |

## Changelog

- `2026-05-17`: Created + activated the task tree on explicit user
  direction (the campaign-complete precondition from
  [[feedback_post_campaign_audit]] / TaskList #49 is met). Decomposed
  into `.1` audit/classify → `.2` objective-bug fixes → `.3`
  Cat-C/residual review + close. Frontier `.1`. Empirical pre-scope +
  one known concrete finding (`rtl_frontend` `event_control_list`)
  recorded.
- `2026-05-17`: `.1` done — `docs/POST_SV_AUDIT_LEDGER.md` produced
  (subagent) + independently verified (counts reconcile, no dup
  headers, classifications spot-checked accurate vs grammar). Findings:
  ~35 static-conclusive Cat-A misuse + 6 HIGH inline-alt-$N + 11
  structured-Cat-A + 30 Cat-B-resolved-confirmed + 16 Cat-C-benign.
  `.2` split per grammar (`.2.1` svpp → `.2.2` rtl_frontend → `.2.3`
  vhdl → `.2.4` systemverilog), smallest-first. Frontier advances
  `.1`→`.2.1`. Tree stays `active`.
- `2026-05-17`: `.2.4b` **done** (`PGEN-POST-SV-AUDIT-0006`,
  user-authorized "design+implement, review after"): 11
  structured-per-iteration SV Cat-A misuses fixed by factoring the
  repeated multi-field unit into **9 new annotated named record rules**
  + extraction-spread (field names preserved). Reachable
  `list_of_*_identifiers` path probe-verified clean; 0
  `<invalid_sequence_access>`, 0 raw-envelope, no leftover
  `{first,rest}` structured envelope. Inventory **2290→2299 /
  999→1008** (deliberate +9 record-rule change, documented honestly —
  unlike pure Cat-A; SV-root-unreachable branches defensive-disposition
  per the .2.4a precedent). Schema `2→3`, release/contract
  `1.0.116→1.0.117`; manifest `structured_decls` sample +
  calibration #118 → `systemverilog_ast_shape_contract` passes;
  contract + 5 book chapters + `POST_SV_AUDIT_LEDGER` lockstep; NO
  bug-ledger row (Cat-A structured). Independently verified; SV book
  gate re-run green. **This closes `.2.4` and `POST-SV-AUDIT.2`
  entirely** (`.2.1`–`.2.4` all done). Frontier advances to `.3`
  (Cat-C review + close TaskList #49 — the final leaf).
- `2026-05-17`: `.2.4` split into `.2.4a` (net_alias + 5 number rules
  — proven playbooks, decidable) + `.2.4b` (11 structured-per-iteration
  — ledger-flagged design fork). `.2.4a` **done** (`PGEN-POST-SV-AUDIT-0005`):
  net_alias Cat-A `{lvalues:[$2,$4,$5::2*]}` probe-verified clean
  `[a,b,c]`; 5 number rules inline-alt-$N lifted to named `*_tail`
  (annotation unchanged, 6×-proven transformation) — corruption found
  **structurally present but NOT consumer-reproducible** (SV root
  rejects all numeric top-level constructs in every profile;
  pre-existing/out-of-scope) → defensive structural fix, honestly NOT
  a bug-ledger entry. Schema `1→2`, release/contract `1.0.115→1.0.116`;
  2290/999 unchanged; contract+book+`POST_SV_AUDIT_LEDGER` lockstep;
  bug-ledger untouched. Independently verified; SV book gate re-run
  green. **`.2.4b` BLOCKED on user design input** (the 11 structured
  record-rule shapes change the flagship released-SV public AST
  contract — surfaced via AskUserQuestion per the PNT directive).
- `2026-05-17`: `.2.3` done — vhdl: 17 Cat-A raw-envelope misuses
  fixed to extraction-spreads (all single-token separators
  comma/semi/dot/bar; 14 bare-list `[$F,$R::2*]`, `target`-aggregate
  `{kind,items:[…]}`, `aggregate` 2 branches `rest:[$5::2*]`).
  Comprehensive AFTER probe: 0 `<invalid_sequence_access>`, **zero
  separator-envelope leaks across ALL 4 sep types**; aggregate.rest
  clean list. Inventory **256/112 unchanged**. Schema `2→3`,
  release/contract `1.0.2→1.0.3`; manifest + `cat_a_shapes` sample →
  `vhdl_ast_shape_contract` passes; contract (new
  `## AST-Shape Corrections — 1.0.3`) + 9 book chapters +
  `POST_SV_AUDIT_LEDGER` (17 FIXED) lockstep. **NO bug-ledger row**
  (pure Cat-A, no corruption — `PGEN_RELEASED_PARSER_BUG_LEDGER`
  untouched, per the recorded Decision). Independently verified; book
  gate re-run green. Frontier advances `.2.3`→`.2.4` (systemverilog,
  flagship — net_alias + 5 number-rule inline-alt-$N + 11 structured).
  Tree stays `active`.
- `2026-05-17`: `.2.2` done — rtl_frontend: 15 Cat-A raw-envelope
  misuses fixed to extraction-spreads + `event_control_list`
  `RTL-FE-0002` inline-alternation-`$N` corruption fixed (lifted
  `( comma | kw_or )` into a new un-annotated `event_separator` rule).
  Comprehensive AFTER probe: 0 `<invalid_sequence_access>`, 0
  separator-envelope leaks; the 3 inferred-index edge cases
  (port_group `$5::3*`, override/connection `$3::3*`,
  event_control `$4::2*`) all precise-probe-confirmed correct.
  Inventory **156/74 unchanged**. Schema `2→3`, release/contract
  `1.0.2→1.0.3`; manifest + `cat_a_shapes` sample →
  `rtl_frontend_ast_shape_contract` passes; contract (new
  `## AST-Shape Corrections — 1.0.3`) + 9 book chapters +
  `POST_SV_AUDIT_LEDGER` (16 FIXED) lockstep; `RTL-FE-0002` the only
  new `PGEN_RELEASED_PARSER_BUG_LEDGER` row (others untouched), the
  15 Cat-A NOT bug-ledger (Decision). Independently verified; book
  gate re-run green. Frontier advances `.2.2`→`.2.3` (vhdl, 17 Cat-A).
  Tree stays `active`.
- `2026-05-17`: `.2.1` done — sv_preprocessor `macro_formals` Cat-A
  raw-envelope misuse fixed (`{first:$2,rest:$3}` → `[$2,$3::2*]`).
  Before/after probe: raw `[[comma,formal],…]` envelope → clean
  `macro_formal[]` list. Count UNCHANGED 66/28
  (`return_object`→`return_array` only). Schema `2→3`, release/contract
  `1.0.2→1.0.3`; manifest + `macro_with_formals` sample →
  `systemverilog_preprocessor_ast_shape_contract` passes;
  contract + 8 book chapters + `POST_SV_AUDIT_LEDGER` lockstep; **NO
  `PGEN_RELEASED_PARSER_BUG_LEDGER` row** (Cat-A clean improvement, not
  the corruption class — recorded Decision). Independently verified;
  book gate re-run green. **The Cat-A `{first,rest}`→extraction-spread
  + schema/release-bump lockstep playbook is now proven** — `.2.2`
  (rtl_frontend), `.2.3` (vhdl), `.2.4` (systemverilog) follow it.
  Frontier advances `.2.1`→`.2.2`. Tree stays `active`.

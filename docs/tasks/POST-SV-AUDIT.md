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
  Status: `active`
  Goal: `Fix the confirmed objective bugs from the .1 ledger (Cat-A raw-envelope misuse on pure separator lists; residual Cat-B inline-alternation-$N), one commit per grammar via the proven regen+manifest+schema/release-bump+book+contract+ledger playbook. Split per grammar (.1 quantified a large per-grammar worklist).`
  Children: `POST-SV-AUDIT.2.1`, `POST-SV-AUDIT.2.2`, `POST-SV-AUDIT.2.3`, `POST-SV-AUDIT.2.4`

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
  Status: `pending`
  Goal: `vhdl: fix the 17 static-conclusive Cat-A misuses (-> extraction-spread, vhdl convention); regen + manifest + schema/release bump + book + contract lockstep.`
  Acceptance: `parseability_probe before/after: all 17 vhdl Cat-A rules emit clean item lists; vhdl_ast_shape_contract + book gate green; lockstep complete; independently verified.`
  Verification: `pending`
  Commit: `pending`

- ID: `POST-SV-AUDIT.2.4`
  Status: `pending`
  Goal: `systemverilog (flagship, released 1.0.115): fix net_alias (single-token-sep Cat-A) + the 5 number-rule inline-alt-$N HIGH finds + the 11 structured-per-iteration Cat-A (per-rule record-rule design); regen + manifest + schema/release bump + book + contract + ledger lockstep. LAST — largest, most careful, flagship.`
  Acceptance: `parseability_probe before/after: number rules no <invalid_sequence_access>, net_alias + structured rules emit clean lists; systemverilog_ast_shape_contract + book gate green; lockstep complete; independently verified.`
  Verification: `pending`
  Commit: `pending`

- ID: `POST-SV-AUDIT.3`
  Status: `pending`
  Goal: `Cat-C + residual review & close: confirm X X* {first,rest} correctness, disposition remaining suboptimal-but-working shapes (accepted vs deferred, with rationale), holistic green re-verification, close the tree + TaskList #49.`
  Acceptance: `Cat-C uses confirmed correct; every remaining flagged shape has a recorded disposition; all per-family ast_shape_contract + book gates green; tree closed; LIVE/CHANGES/DEVELOPMENT_NOTES/memory updated.`
  Verification: `pending`
  Commit: `pending`

## Current Frontier

| Order | Leaf | Status | Why next |
| --- | --- | --- | --- |
| 1 | `POST-SV-AUDIT.2.3` | `pending` | `.2.1` (svpp) + `.2.2` (rtl_frontend, 15 Cat-A + RTL-FE-0002) done — the Cat-A extraction-spread + schema/release-bump lockstep playbook is proven at scale. vhdl next (17 Cat-A). |
| 2 | `POST-SV-AUDIT.2.4` | `pending` | systemverilog (flagship, released 1.0.115; net_alias + 5 number-rule inline-alt + 11 structured-per-iteration needing record-rule design) — last, largest, most careful. |
| 3 | `POST-SV-AUDIT.3` | `pending` | Cat-C/residual review + close TaskList #49 after all `.2.*` fixes land. |

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

- None.

## Verification Log

| Date | Leaf | Checks | Result |
| --- | --- | --- | --- |
| `2026-05-17` | `POST-SV-AUDIT` (setup) | tree decomposition vs workflow splitting rules; empirical pre-scope; precondition (campaign complete) confirmed | `pass — activated on explicit user direction; decomposed into audit→fix→close; pre-scope recorded with one concrete known finding` |
| `2026-05-17` | `POST-SV-AUDIT.1` | ledger structure (12 ## headers, no dups) + scope justification + count reconciliation vs pre-scope + independent classification spot-checks vs grammar (event_control_list/unsigned_number/net_alias/vhdl) | `pass — docs/POST_SV_AUDIT_LEDGER.md accurate; ~35 static-conclusive Cat-A + 6 HIGH inline-alt-$N + 11 structured-Cat-A + 30 Cat-B-resolved + 16 Cat-C; worklist concrete & per-grammar; .2 split per grammar; inline-alt-$N probe-confirm deferred to .2-start per the proven playbook (on-demand-parser infra)` |
| `2026-05-17` | `POST-SV-AUDIT.2.1` | before/after parseability_probe; regen (count UNCHANGED 66/28, macro_formals return_object->return_array); manifest dai-rebuild + macro_with_formals sample; systemverilog_preprocessor_ast_shape_contract; contract dup-header+numbers+no-fabricated-AstDumpPayload grep; book walker-pin + fabricated-residual grep; POST_SV_AUDIT_LEDGER FIXED + NO bug-ledger row check; independent book-gate re-run; clippy; docs/book check | `pass — macro_formals Cat-A fixed ({first,rest}->[$2,$3::2*]); probe raw-comma-envelope -> clean macro_formal[] list, no <invalid_sequence_access>; shape-contract PASSES; 66/28 UNCHANGED; schema 2->3 / release 1.0.2->1.0.3; contract+8-book-chapters+audit-ledger lockstep; NO bug-ledger row (Cat-A clean improvement per Decision); no dup ## headers; DOC-ENVELOPE not regressed; book gate independently green. Cat-A playbook proven for .2.2/.2.3/.2.4.` |
| `2026-05-17` | `POST-SV-AUDIT.2.2` | comprehensive AFTER parseability_probe + precise per-rule shape checks (port_group $5::3*, override/connection $3::3*, event_control $4::2* — all CORRECT); 0 separator-envelope-leak + 0 invalid_sequence_access; regen (156/74 UNCHANGED); manifest dai-rebuild + cat_a_shapes sample; rtl_frontend_ast_shape_contract; contract dup-header+numbers grep; book walker-pin + fabricated-residual grep; POST_SV_AUDIT_LEDGER 16-FIXED; bug-ledger EXACTLY-1-insertion(RTL-FE-0002)-0-deletions check; independent book-gate re-run; clippy; docs/book check | `pass — 15 rtl_frontend Cat-A misuses fixed (extraction-spreads, all 3 inferred-index edge cases probe-confirmed) + event_control_list RTL-FE-0002 inline-alt fix (named event_separator); 156/74 UNCHANGED; schema 2->3 / release 1.0.2->1.0.3; contract(new AST-Shape-Corrections-1.0.3 section)+9-book-chapters+audit-ledger lockstep; RTL-FE-0002 only-new bug-ledger row (others untouched); 15 Cat-A NOT bug-ledger (Decision); no dup ## headers; DOC-ENVELOPE not regressed; book gate independently green` |

## Commit Log

| Leaf | Commit subject or reference | Notes |
| --- | --- | --- |
| `POST-SV-AUDIT` (setup) | `PGEN-POST-SV-AUDIT-0000` | tree created + activated (Proposed→Active); frontier `.1`; empirical pre-scope recorded |
| `POST-SV-AUDIT.1` | `PGEN-POST-SV-AUDIT-0001` | ledger `docs/POST_SV_AUDIT_LEDGER.md` (all 6 product grammars classified A/B/C, verified accurate); `.2` split per grammar; frontier → `.2.1` |
| `POST-SV-AUDIT.2.1` | `PGEN-POST-SV-AUDIT-0002` | sv_preprocessor `macro_formals` Cat-A fix (`{first,rest}`→`[$2,$3::2*]`); schema 2→3 / release 1.0.2→1.0.3; 66/28 unchanged; contract+book+audit-ledger lockstep, NO bug-ledger row; Cat-A playbook proven |
| `POST-SV-AUDIT.2.2` | `PGEN-POST-SV-AUDIT-0003` | rtl_frontend 15 Cat-A misuses → extraction-spreads + `event_control_list` `RTL-FE-0002` inline-alt fix (named `event_separator`); schema 2→3 / release 1.0.2→1.0.3; 156/74 unchanged; contract(AST-Shape-Corrections-1.0.3)+book+audit-ledger lockstep; `RTL-FE-0002` only-new bug-ledger row; 15 Cat-A not bug-ledger |

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

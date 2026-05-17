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
  Status: `pending`
  Goal: `sv_preprocessor: fix macro_formals Cat-A raw-envelope misuse (-> extraction-spread); regen + manifest + schema/release bump + book + contract lockstep. SMALLEST blast radius — proves the Cat-A shape-change lockstep playbook before the larger grammars.`
  Acceptance: `parseability_probe before/after on a multi-formal \`define; macro_formals emits a clean item list (no raw [[comma,item],...] envelope); systemverilog_preprocessor_ast_shape_contract + book gate green; lockstep complete; independently verified.`
  Verification: `pending`
  Commit: `pending`

- ID: `POST-SV-AUDIT.2.2`
  Status: `pending`
  Goal: `rtl_frontend: fix the 16 static-conclusive Cat-A misuses + the event_control_list inline-alt-$N HIGH finding (named event_separator rule + extraction-spread); regen + manifest + schema/release bump + book + contract + ledger lockstep.`
  Acceptance: `parseability_probe before/after: event_control_list no <invalid_sequence_access>, all 16 Cat-A rules emit clean item lists; rtl_frontend_ast_shape_contract + book gate green; lockstep complete; independently verified.`
  Verification: `pending`
  Commit: `pending`

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
| 1 | `POST-SV-AUDIT.2.1` | `pending` | `.1` done (ledger produced + verified). `.2` split per grammar; `.2.1` (sv_preprocessor `macro_formals`, 1 rule) is the smallest blast radius — proves the Cat-A `{first,rest}`→extraction-spread + schema/release-bump lockstep playbook before the larger rtl_frontend/vhdl/sv grammars. |
| 2 | `POST-SV-AUDIT.2.2` | `pending` | rtl_frontend (16 Cat-A + event_control_list inline-alt-$N) — parser-regen mechanism already proven this session. |
| 3 | `POST-SV-AUDIT.2.3` | `pending` | vhdl (17 Cat-A). |
| 4 | `POST-SV-AUDIT.2.4` | `pending` | systemverilog (flagship, released 1.0.115; net_alias + 5 number-rule inline-alt + 11 structured-per-iteration needing record-rule design) — last, largest, most careful. |
| 5 | `POST-SV-AUDIT.3` | `pending` | Cat-C/residual review + close TaskList #49 after all `.2.*` fixes land. |

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

## Commit Log

| Leaf | Commit subject or reference | Notes |
| --- | --- | --- |
| `POST-SV-AUDIT` (setup) | `PGEN-POST-SV-AUDIT-0000` | tree created + activated (Proposed→Active); frontier `.1`; empirical pre-scope recorded |
| `POST-SV-AUDIT.1` | `PGEN-POST-SV-AUDIT-0001` | ledger `docs/POST_SV_AUDIT_LEDGER.md` (all 6 product grammars classified A/B/C, verified accurate); `.2` split per grammar; frontier → `.2.1` |

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

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
  Status: `pending`
  Goal: `Audit & classify: enumerate every {first/lhs..rest:$N} + raw-$N-over-iteration annotation across all product grammars; classify A/B/C with parseability_probe evidence; produce the prioritized objective-bug fix worklist (docs/POST_SV_AUDIT_LEDGER.md).`
  Acceptance: `Ledger covers all occurrences in regex/systemverilog/systemverilog_preprocessor/vhdl/rtl_frontend/rtl_const_expr (others justified out-of-scope); each classified A/B/C with evidence; Cat-A misuse + residual Cat-B candidates explicitly listed (incl. the known rtl_frontend event_control_list ( comma|kw_or ) finding); .2 frontier populated from the findings.`
  Verification: `pending`
  Commit: `pending`

- ID: `POST-SV-AUDIT.2`
  Status: `pending`
  Goal: `Fix the confirmed objective bugs from the .1 ledger (Cat-A raw-envelope misuse on pure separator lists; residual Cat-B inline-alternation-$N), one commit per grammar via the proven regen+manifest+schema/release-bump+book+contract+ledger playbook.`
  Acceptance: `Each fixed grammar: parseability_probe clean (no <invalid_sequence_access>, no raw separator envelope where a clean list is contractually expected); *_ast_shape_contract + *_parser_book_gate green; lockstep complete; independently verified. (May split per grammar once .1 quantifies the worklist.)`
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
| 1 | `POST-SV-AUDIT.1` | `pending` | The audit/classify pass produces the evidence-based worklist that `.2` (fixes) and `.3` (closeout) depend on. Cannot fix before classifying. |

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

## Commit Log

| Leaf | Commit subject or reference | Notes |
| --- | --- | --- |
| `POST-SV-AUDIT` (setup) | `PGEN-POST-SV-AUDIT-0000` | tree created + activated (Proposed→Active); frontier `.1`; empirical pre-scope recorded |

## Changelog

- `2026-05-17`: Created + activated the task tree on explicit user
  direction (the campaign-complete precondition from
  [[feedback_post_campaign_audit]] / TaskList #49 is met). Decomposed
  into `.1` audit/classify → `.2` objective-bug fixes → `.3`
  Cat-C/residual review + close. Frontier `.1`. Empirical pre-scope +
  one known concrete finding (`rtl_frontend` `event_control_list`)
  recorded.

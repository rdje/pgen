# RTL-FE-CLOSURE: rtl_frontend Parser-Family Closure (Phase S)

## Metadata

- Tree ID: `RTL-FE-CLOSURE`
- Status: `active`
- Roadmap lane: `Phase S — rtl_frontend synthesizable-RTL subset: parser-family closure (distinct from the existing RTL-FE-MDBOOK / RTL-FE-CONTRACT-BODY book/contract trees)`
- Created: `2026-05-31`
- Last updated: `2026-06-13`
- Owner: repo-local workflow

## Goal

Drive the `rtl_frontend` family from its current `In Progress` LIVE status to
PGEN closure: generated-grammar exhaustiveness + elaboration-facing closure,
beyond the already-landed generated-contract/handwritten-parity work. The
existing `RTL-FE-MDBOOK` + `RTL-FE-CONTRACT-BODY` trees (done) cover the book +
integration-contract surfaces; THIS tree owns the parser/elaboration CLOSURE
the LIVE row says is still open.

## Non-Goals

- Not the book or integration-contract surfaces (owned by the existing RTL-FE
  trees) — this is parser/elaboration closure.
- Per Phase S rule: handwritten `parse_design` baseline is scaffolding;
  generated-parser exhaustiveness is the closure bar.

## Acceptance Criteria

- Generated `rtl_frontend` grammar exhaustiveness proven.
- Elaboration-facing closure proven (the `rtl_frontend_generated_contract_gate`
  + elaboration replay layer extended to the closure bar).
- LIVE "rtl_frontend synthesizable subset baseline" row → Done with evidence.
- Each completed leaf committed through `COMMIT.md`; any code change owns a leaf.

## Task Tree

- ID: `RTL-FE-CLOSURE`
  Status: `active`
  Goal: `rtl_frontend generated-grammar exhaustiveness + elaboration closure → LIVE Done.`
  Children: `RTL-FE-CLOSURE.1`, `RTL-FE-CLOSURE.2`, `RTL-FE-CLOSURE.3`, `RTL-FE-CLOSURE.4`, `RTL-FE-CLOSURE.5`, `RTL-FE-CLOSURE.6`, `RTL-FE-CLOSURE.7`, `RTL-FE-CLOSURE.8`

- ID: `RTL-FE-CLOSURE.1`
  Status: `done`
  Goal: `SCOPING (pure docs): read the LIVE rtl_frontend row + README rtl_frontend notes + rtl_frontend_generated_contract_gate to pin the exact remaining exhaustiveness/elaboration gap to Done; produce an ordered leaf plan. Tools-first; no code until a code leaf owns it.`
  Acceptance: `Remaining-to-Done gap + ordered leaf plan recorded.`
  Verification: `2026-06-13 — SCOPING COMPLETE (tools-first, fresh measured against the prebuilt debug ast_pipeline + generated/rtl_frontend_parser.rs). DONE BAR (RTL-FE-CLOSURE acceptance + locked-program 2026-06-08 "ALL EXISTING PARSERS → Done = cert-coverage clean + UNKNOWN=0"): rtl_frontend cert-coverage fully_certified=true (UNKNOWN=0, spf=0) deterministic at canonical seeds 0/7/42, + elaboration-facing closure (rtl_frontend_generated_contract_gate green — ALREADY at contract 0.2.0 since .3), + LIVE row → Done with evidence. MEASURED (--report-certificate-coverage --entry-rule rtl_frontend_file --count 40): total=170 proof=0 witness=99 UNKNOWN=71 fully_certified=false; witness=99/UNKNOWN=71 BYTE-IDENTICAL at seeds 0/7/42 (deterministic); spf=0 at seeds 0/7, spf=1 ONLY at seed 42 (= the pre-existing over-generation residual ticketed at -0072). REACH-PASS ATTRIBUTION (seed 0): 131 UNKNOWN rules targeted → 50 witnessed, 67 parsed-but-routed-elsewhere, 37 probe-didn't-reparse, 20 generation-failures; 0 NO-REACH-PATH, 0 LEFT-UNATTEMPTED (no dead-rule candidates, no cap drops). LINTER (--lint-grammar): rtl_frontend (170 rules) WELL-FORMED on every decidable axis — unreachable_rules=0, ordered_choice_shadowing=0, always_matches_shadowing=0, non_terminating=0, unbound_fact_kinds=0, nullable_repetition=0, profile_orphans=0. ⇒ ATTRIBUTION-RULE VERDICT: the ENTIRE UNKNOWN=71 is a GENERATOR-REACH (constructor) deficiency, NOT a grammar defect (no dead/shadowed/unreachable rules) — the same class rtl_const_expr/svpp/regex/vhdl each closed. UNKNOWN families (25 shown of 71; report caps the print at 25): generate (if/for/body), ports (list/group/item/direction/direction_token), parameter_override (list/override), port_connection (list/connection), continuous_assign, event_control (separator/list/item/edge), always_ff (statement/assignment_operator), operators+targets (assignment_operator/assignment_target/value_expr/equality_op/relational_op/shift_op). ORDERED LEAF PLAN: .4 root-cause investigation (categorize all 71 by reach-failure mode + the seed-42 spf; attribution per cluster incl. any latent parser bug the reach probes surface — fix-parser-bugs-ASAP) → .5 generator-reach fix umbrella (parser-agnostic; children authored by .4) → .6 seed-42 spf=1 over-generation residual fix → .7 elaboration-facing closure confirmation/ratchet → .8 CLOSURE (LIVE row → Done + docs/book/contract lockstep). EBNF meta-grammar lockstep (feedback_ebnf_meta_grammar_lockstep) binds any leaf that introduces a new EBNF-format construct.`
  Commit: `PGEN-RTL-FE-CLOSURE-0003`

- ID: `RTL-FE-CLOSURE.4`
  Status: `pending`
  Goal: `INVESTIGATION (pure docs / tools-first): root-cause the cert-coverage UNKNOWN=71 backlog. Categorize all 71 UNKNOWN rules by reach-failure mode (the 67 parsed-but-routed-elsewhere / 37 probe-didn't-reparse / 20 generation-failure buckets) and by construct family, using the reach-pass detail + rule-scoped --trace probes + the (already-clean) linter. Apply the attribution rule per cluster: generator-reach-plan gap vs any latent parser bug the reach probes surface (per the H.7.2/H.9 precedent where the reach pass found a real svpp parser bug). Also root-cause the seed-42 spf=1 sample. Produce per-cluster WHY+WHERE + fix-leaf tickets that .5/.6 own.`
  Acceptance: `Every UNKNOWN cluster has a pinned cause (generator-reach vs parser-bug) with decisive tool evidence; the seed-42 spf=1 sample is root-caused; per-cluster fix tickets authored under .5/.6 (any parser bug found → its own highest-priority fix leaf per fix-parser-bugs-ASAP). No code changed (investigation only).`
  Verification: `pending`
  Commit: `pending`

- ID: `RTL-FE-CLOSURE.5`
  Status: `pending`
  Goal: `FIX umbrella (container): drive cert-coverage UNKNOWN 71→0 by closing the generator-reach deficiencies .4 pins. Concrete child leaves (each owns its code change) are authored by .4 — expected shapes: parser-agnostic reach-pass / stimuli-generator capability extensions so the routed-elsewhere rules get forced entry, plus any grammar/engine fix for a parser bug the reach probes surface. Parser-agnostic per feedback_features_parser_agnostic; EBNF meta-grammar lockstep if any new construct lands.`
  Acceptance: `cert-coverage UNKNOWN=0 deterministic at seeds 0/7/42; zero cert regression on any other grammar (decisive git-stash baseline); cross-family + oracle gates green; all children done/deferred/superseded with recorded reasons.`
  Verification: `pending`
  Commit: `pending`

- ID: `RTL-FE-CLOSURE.6`
  Status: `pending`
  Goal: `FIX (code leaf): the seed-42 sample_parse_failures=1 over-generation residual (pre-existing, stash-proven at -0072). Constrain the generator so the offending sample re-parses ("the closed loop must not out-generate its own parser") without loosening the parser_rejections==0 precondition; spf=0 at seeds 0/7/42. May fold into a .5 child if .4 shows the same mechanism.`
  Acceptance: `spf=0 at canonical seeds 0/7/42 (and a wider sweep); no cross-grammar regression; round-trip constraint is general (grammar/HIR-shape driven), not a per-sample patch.`
  Verification: `pending`
  Commit: `pending`

- ID: `RTL-FE-CLOSURE.7`
  Status: `pending`
  Goal: `ELABORATION-FACING CLOSURE confirmation/ratchet: confirm the rtl_frontend_generated_contract_gate elaboration replay layer (59 samples, 46/13; green at contract 0.2.0) meets the RTL-FE-CLOSURE elaboration Done bar; ratchet the minimums if the acceptance demands more elaboration evidence; record as closure evidence.`
  Acceptance: `Elaboration Done bar pinned + met (gate green); any ratchet landed with evidence; README/book elaboration descriptions truthful.`
  Verification: `pending`
  Commit: `pending`

- ID: `RTL-FE-CLOSURE.8`
  Status: `pending`
  Goal: `CLOSURE (mixed verify + docs): with cert-coverage fully_certified=true (UNKNOWN=0, spf=0, deterministic seeds 0/7/42) + the generated-contract gate green + elaboration closure (.7), promote the LIVE rtl_frontend row → Done with evidence and lockstep README / docs/book / the rtl_frontend integration contract / the per-parser book.`
  Acceptance: `LIVE rtl_frontend row = Done with the machine-checkable evidence cited; all books/contracts re-synced same-wave; mdbook + per-parser-book gates PASS.`
  Verification: `pending`
  Commit: `pending`

- ID: `RTL-FE-CLOSURE.2`
  Status: `done`
  Goal: `INVESTIGATION (pure docs, the -0067/-0075 red-gate ticket): root-cause WHY+WHERE rtl_frontend_generated_contract_gate fails ("always_ff_well_formed missing required rule 'module_declaration'") tools-first; adjudicate parser-bug vs proof-surface defect; record the fix design for its own leaf.`
  Acceptance: `WHY + WHERE pinned with decisive tool evidence; parser-bug-or-not adjudicated; fix leaf ticketed with design + acceptance.`
  Verification: `2026-06-11 — VERDICT: NOT a parser bug — a STALE PROOF SURFACE. (1) WHY: the curated parity manifest (rust/test_data/grammar_quality/rtl_frontend_generated_parity_contract_v0.json, authored 2026-04-08, last touched 2026-04-20 — when grammars/rtl_frontend.ebnf had ZERO return annotations) asserts raw-envelope AST retention: required_rule_names / required_rule_texts / expected_rule_texts walk the dumped AST for rule_name+span keys (probe rtl_frontend_generated_contract_probe.rs collect_rule_names/collect_rule_spans). The RELEASED typed-AST campaign (RTL-FE Slices 1–7 starting 2026-05-14 ad822637..2728d5de, then 1.0.2 617d6ad2 + 1.0.3/schema-3 84624543 on 2026-05-17) annotated the structural rules (design_item -> {kind, body:$1}, module_declaration -> {name:$2, …}), folding those subtrees into typed ParseContent::Json with NO rule_name keys — MEASURED: the always_ff_well_formed dump contains exactly ONE rule_name (the entry rtl_frontend_file); all 14 required rules absent. The AST-retention layer is structurally unsatisfiable for all 93 require_ast_json samples; forbidden_rule_names (82 samples) is vacuous. (2) WHY SILENT: the typing campaign validated via the NEWER typed proof surfaces (rtl_frontend_ast_shape_contract manifest + auto return-annotation gate — commit bodies of ad822637/84624543 name only those); the hosted workflow rtl-frontend-generated-contract-gate.yml is manual-only since the 2026-04-14 Actions pause ⇒ red from ~2026-05-14 unnoticed until the -0067 sweep (2026-06-10). (3) HEALTH OF THE OTHER LAYERS — both GREEN: parse-acceptance replayed over ALL 125 samples = 0 mismatches (parseability_probe per-sample, expected_parse_ok exact); stage-2 handwritten replay + elaboration replay (cargo test generated_contract_manifest_matches_handwritten in ../rtl_frontend) 2/2 PASS. (4) The -0067 framing "broken by some earlier engine wave" is CORRECTED: broken by the deliberate, released GRAMMAR typing campaign with the April-era gate left un-migrated; the generated parser's typed output is the documented schema-3 contract (rtl_frontend book + ast_shape_contract). No release/ledger action (no shipped-parser defect).`
  Commit: `PGEN-RTL-FE-CLOSURE-0001`

- ID: `RTL-FE-CLOSURE.3`
  Status: `done`
  Goal: `FIX (code leaf): migrate rtl_frontend_generated_contract_gate's stage-1 proof surface to the typed-AST era so the gate is GREEN and DISCRIMINATING again. Design (from .2): keep expected_parse_ok over all 125 samples (proven healthy) + stage-2 handwritten/elaboration replay (proven green) unchanged; replace the raw-envelope checks with typed-era assertions; bump contract_version 0.1.0→0.2.0; expected values from the SPEC side (feedback_corpus_expected_from_spec_not_fix); README + book gate descriptions re-synced to what the gate actually proves.`
  Acceptance: `make -C rust SHELL=/bin/bash rtl_frontend_generated_contract_gate exits 0; the typed checks FAIL on a deliberately mutated shape (discriminating-power probe); stage-2 ratchets preserved (59 expected_elaboration samples incl. the 46/13 accept/reject split); README + book gate descriptions truthful; no parser/grammar change (proof-surface-only — schema/release untouched).`
  Verification: `2026-06-11 — ALL ACCEPTANCE MET. DESIGN REFINEMENT during implementation: required_rule_names/forbidden_rule_names migrated to the parser's TRANSACTIONAL COVERAGE RECORD (parse_and_cover_rtl_frontend — entry testimony, sound under backtracking, complete under annotation folding; the engine facility built exactly for "did rule X participate in the accepted parse") ⇒ those manifest fields survive UNCHANGED (93 required + 82 forbidden lists, semantics preserved 1:1, green first-try); required_rule_texts/expected_rule_texts retired and re-expressed as required_typed_string_values — the curated (human-authored, spec-side) texts that survive as EXACT string values of the released schema-3 typed carrier: 214 locks across 69/93 samples (1050 multi-token span texts adjudicated superseded by coverage testimony + handwritten parity + elaboration replay; recorded in the manifest provenance). MEASURED: gate exits 0 end-to-end (probe + handwritten replay stages both pass); DISCRIMINATION A/B 3/3 — bogus required rule, forbidden=module_declaration, bogus typed value each FAIL loudly with exact messages, restored manifest green; probe unit tests 4/4 (new collect_string_values multiplicity lock); handwritten crate suite 91/91 (contract_version asserts 0.1.0→0.2.0); clippy strict-source clean (generated stage = pre-existing tolerated debt); mdbook_docs_gate PASS. Blast radius: NO parser/grammar/engine change (probe bin + manifest + 2 handwritten-test asserts + README + 2 book chapters); schema stays 3, release stays 1.0.3, no ledger row (not a shipped-parser defect).`
  Commit: `PGEN-RTL-FE-CLOSURE-0002`

## Current Frontier

| Order | Leaf | Status | Why next |
| --- | --- | --- | --- |
| 1 | `RTL-FE-CLOSURE.4` | `pending` | Root-cause the UNKNOWN=71 backlog (all generator-reach per `.1`'s attribution verdict) + the seed-42 spf — produces the per-cluster fix tickets `.5`/`.6` own. Pure docs / tools-first. |
| 2 | `RTL-FE-CLOSURE.6` | `pending` | seed-42 spf=1 over-generation residual fix (small, bounded; can land independently of the UNKNOWN drive). |
| 3 | `RTL-FE-CLOSURE.5` | `pending` | Generator-reach fix umbrella (children authored by `.4`) — drives UNKNOWN 71→0. |
| 4 | `RTL-FE-CLOSURE.7` | `pending` | Elaboration-facing closure confirmation/ratchet (gate already green at 0.2.0). |
| 5 | `RTL-FE-CLOSURE.8` | `pending` | CLOSURE: LIVE row → Done + docs/book/contract lockstep (after `.5`/`.6`/`.7`). |
| — | `RTL-FE-CLOSURE.1` | `done` (`PGEN-RTL-FE-CLOSURE-0003`) | SCOPING: pinned the Done bar + the measured/attributed gap + the ordered leaf plan. |
| — | `RTL-FE-CLOSURE.2` | `done` (`PGEN-RTL-FE-CLOSURE-0001`) | The -0067/-0075 red-gate ticket: adjudicated NOT-a-parser-bug (stale April-era manifest vs the released typed schema-3 carrier); fix design recorded in `.3`. |
| — | `RTL-FE-CLOSURE.3` | `done` (`PGEN-RTL-FE-CLOSURE-0002`) | The typed-era migration landed: gate GREEN end-to-end + discrimination A/B 3/3; coverage-testimony preserves required/forbidden semantics 1:1; 214 typed-value locks; README + book truthful. |

## Decisions

- `2026-05-31`: Created as a thin owning skeleton by `TASKTREE-GOV.2` to own the rtl_frontend parser/elaboration CLOSURE gap, distinct from the done book/contract trees. Not started; activate when prioritized (after SV-EXH-PROOF).
- `2026-06-11`: **Tree ACTIVATED** by the `-0067`/`-0075` ticket (the red `rtl_frontend_generated_contract_gate` "needs its own leaf — RTL-FE lane"). `.2` adjudication: the red gate is a STALE PROOF SURFACE (April-era raw-envelope manifest vs the released typed schema-3 carrier), NOT a parser bug — so the fix-parser-bugs-ASAP escalation does not apply; the fix (`.3`) is a proof-surface migration, not a release event. The seed-42 cert-coverage spf=1 over-generation residual (stash-proven pre-existing at `-0072`) remains a SEPARATE future leaf of this tree (generator-side), to be opened when the rtl_frontend `UNKNOWN`→0 drive resumes.
- `2026-06-13` (`.1` SCOPING, `PGEN-RTL-FE-CLOSURE-0003`): **the Done bar is cert-coverage fully_certified=true (UNKNOWN=0, spf=0) deterministic at seeds 0/7/42 + elaboration-facing closure (contract gate, already green at 0.2.0) + LIVE row promotion** — matching the locked-program rule (ALL EXISTING PARSERS → Done = cert-coverage clean + UNKNOWN=0). **Decisive attribution decision:** fresh measurement (`total=170 witness=99 UNKNOWN=71`, deterministic) + the reach pass reporting **0 no-reach-path / 0 left-unattempted** + the linter reporting **0 unreachable / 0 shadowing / 0 always-matches / 0 non-terminating / 0 unbound-fact-kinds** ⇒ the whole UNKNOWN=71 is a **GENERATOR-REACH (constructor) deficiency, NOT a grammar defect**. So the drive belongs in the generator/reach-pass (parser-agnostic), exactly the rtl_const_expr/svpp/regex/vhdl pattern — UNLESS `.4`'s per-rule probing surfaces a latent parser bug (then that becomes a highest-priority fix leaf per fix-parser-bugs-ASAP). The seed-42 spf=1 residual is split out as its own code leaf (`.6`). Ordered plan recorded in the task tree (`.4`→`.8`); frontier → `.4`.

## Open Questions

- Which of the three reach-failure buckets dominate the 71 (67 routed-elsewhere is the largest sample) — is the routed-elsewhere class a reach-plan force-entry gap, a PEG earlier-alternative absorption the linter's *sound subset* does not flag, or a real parser mis-route? (resolve per-cluster in `.4`)
- Does the RTL-FE-CLOSURE elaboration Done bar require any ratchet beyond the current 59-sample (46/13) replay, or is cert-coverage UNKNOWN=0/spf=0 the gating criterion with elaboration already satisfied? (resolve in `.7`)

## Blockers

- None (In Progress baseline; closure deferred).

## Verification Log

| Date | Leaf | Checks | Result |
| --- | --- | --- | --- |
| `2026-05-31` | `RTL-FE-CLOSURE.1` | `pending` | `pending` |
| `2026-06-11` | `RTL-FE-CLOSURE.2` | `gate repro (probe fail signature exact); AST dump walk (1 rule_name total, 14/14 required absent); grammar git -S (module_declaration annotation = f70b8976 Slice-5 2026-05-14; manifest last touch 486db2bf 2026-04-20); campaign commit bodies (ad822637/84624543 validate via typed surfaces only); hosted workflow manual-only; parse-acceptance replay 125/125 = 0 mismatches; stage-2 handwritten+elaboration cargo test 2/2 PASS` | `VERDICT: stale proof surface, NOT a parser bug; fix design → .3` |
| `2026-06-11` | `RTL-FE-CLOSURE.3` | `gate end-to-end exit 0 (probe + handwritten stages); discrimination A/B 3/3 (bogus required rule / forbidden module_declaration / bogus typed value each FAIL loudly, restored green); typed-value survival measurement (214 locks / 69 samples; 1050 span texts adjudicated); probe unit tests 4/4; rtl_frontend crate suite 91/91; clippy strict-source clean; mdbook_docs_gate PASS` | `Gate GREEN + discriminating; contract 0.2.0; no parser/grammar change` |
| `2026-06-13` | `RTL-FE-CLOSURE.1` | `cert-coverage --report-certificate-coverage --entry-rule rtl_frontend_file --count 40 at seeds 0/7/42 (prebuilt debug ast_pipeline + generated parser): total=170 witness=99 UNKNOWN=71 deterministic, spf 0/0/1; reach-pass 131 targeted → 50 witnessed / 67 routed-elsewhere / 37 didn't-reparse / 20 gen-failure, 0 no-path, 0 unattempted; --lint-grammar all decidable axes 0` | `Done bar + measured/attributed gap + ordered leaf plan recorded; UNKNOWN=71 attributed GENERATOR-REACH (linter clean); frontier → .4. No code changed.` |

## Commit Log

| Leaf | Commit subject or reference | Notes |
| --- | --- | --- |
| `RTL-FE-CLOSURE.1` | `PGEN-RTL-FE-CLOSURE-0003` | Pure-docs scoping; Done bar + measured/attributed gap + ordered leaf plan (`.4`→`.8`). |
| `RTL-FE-CLOSURE.2` | `PGEN-RTL-FE-CLOSURE-0001` | Pure-docs investigation; corrects the -0067 "earlier engine wave" framing. |
| `RTL-FE-CLOSURE.3` | `PGEN-RTL-FE-CLOSURE-0002` | Typed-era gate migration: coverage testimony + typed-value locks; README/book re-sync. |

## Changelog

- `2026-05-31`: Created thin skeleton (TASKTREE-GOV.2 roadmap-coverage).
- `2026-06-11`: Activated; `.2` (red-gate root-cause investigation) done — stale April-era manifest vs released typed schema-3 carrier; `.3` (typed-era proof-surface migration) ticketed as frontier.
- `2026-06-11`: `.3` done — `rtl_frontend_generated_contract_gate` GREEN end-to-end at contract `0.2.0` (coverage-record rule testimony + 214 typed-value locks + unchanged handwritten/elaboration stages); README + cli-and-workflows + parser-families re-synced. Frontier → `.1` (closure scoping).
- `2026-06-13`: `.1` done (`PGEN-RTL-FE-CLOSURE-0003`, pure docs) — SCOPING complete tools-first: Done bar pinned (cert-coverage UNKNOWN=0/spf=0 deterministic + elaboration closure + LIVE promotion); fresh measurement `total=170 witness=99 UNKNOWN=71` (deterministic seeds 0/7/42, spf=1 only at 42); reach-pass + linter attribute the whole UNKNOWN=71 to GENERATOR-REACH (no grammar defect); ordered leaf plan `.4`→`.8` added (children .4–.8). Frontier → `.4` (UNKNOWN root-cause investigation).

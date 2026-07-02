# SV-DOLLAR-LRM-FIDELITY: restore LRM `$`-spelling fidelity for the mangled `sv_dollar_*` keyword-token family

## Metadata

- Tree ID: `SV-DOLLAR-LRM-FIDELITY`
- Status: `active`
- Roadmap lane: SystemVerilog parser-family LRM fidelity (released-parser correctness defect
  family; sibling of `SV-COVERGROUP-FIDELITY` / `SV-SVA-PROPERTY-FIDELITY` /
  `SV-AST-SHAPE-FIDELITY`)
- Created: `2026-07-02`
- Last updated: `2026-07-02`
- Owner: repo-local workflow
- Origin: discovered tools-first during `VERILOG-2005-PROFILE.6.4`
  (`PGEN-VERILOG-2005-PROFILE-0017`) — see that tree's "`.6.4` Findings" for the full
  evidence record.
- Ledger rows: `SV-0029` (the 19-token mangled-literal family), `SV-0030` (`scalar_constant`
  digit loss + `scalar_timing_check_condition` PEG shadowing).

## Goal

Make the flattened `grammars/systemverilog.ebnf` accept the IEEE LRM `$` spellings (and stop
accepting the mangled nonsense spellings) for every keyword-like `$` token, with correct
per-profile gating:

1. The **19 `kw_sv_dollar_*` tokens** (`grammars/systemverilog.ebnf:6248-6284`) currently match
   the literal TEXT `sv_dollar_*` (e.g. `kw_sv_dollar_setup_b58bdaae := trivia
   /sv_dollar_setup\b/`) instead of the LRM `$*` spellings. Proven consequences (probes recorded
   in `VERILOG-2005-PROFILE` "`.6.4` Findings", 2026-07-02, release `1.0.158`):
   - LRM `$setup(d, posedge clk, 1);` in a specify block REJECTS under `sv_2017`
     (`furthest_position=26`) AND under `verilog_2005` — although specify timing checks are
     CORE surface in BOTH IEEE 1800-2017 §31 and IEEE 1364-2005 §15.
   - LRM `$unit::y` REJECTS under `sv_2017` (IEEE 1800 §3.12.1); nonsense `sv_dollar_unit::y`
     ACCEPTS.
   - LRM module-level `$fatal;` (IEEE 1800 §20.11 elaboration severity task) REJECTS;
     nonsense `sv_dollar_fatal;` ACCEPTS.
   - `$root.top.f(1);` ACCEPTS but MIS-ROUTED: the AST carries `kind:"system_tf"` only —
     `$root` is consumed by the generic `system_tf_identifier` token (`:449`), never the
     `$root`-anchored `hierarchical_identifier` (`:2356`) / `hierarchical_tf_identifier`
     (`:2372`) forms, so downstream consumers never see the LRM `$root` anchor.
   - Nonsense `sv_dollar_setup(…)` ACCEPTS (over-acceptance of non-language text).
2. The **`scalar_constant` digit loss** (`SV-0030`): IEEE `scalar_constant ::= 1'b0 | 1'b1 |
   1'B0 | 1'B1 | 'b0 | 'b1 | 'B0 | 'B1 | 1 | 0` (faithful in
   `grammars/systemverilog_2017_lrm_extracted.ebnf:951`) was flattened into digit-less prefixes
   (`kw_n_1_tick_b_f4c81681 := trivia "1'b"`, `:6068`; rule `:4713`). Additionally
   `scalar_timing_check_condition` (`:4720`) lists bare `expression` FIRST, PEG-shadowing the
   `expression equal scalar_constant` branches (probes: `e == 1'b0` ACCEPTS via the bare
   expression branch; defective `e == 1'b` REJECTS at the outer sequence).

Provenance (WHY+WHERE of the mangling): the extracted LRM snapshots carry the true `$`
spellings (zero `sv_dollar` matches in both `systemverilog_2017_lrm_extracted.ebnf` and
`verilog_2005_lrm_extracted.ebnf`); the mangling entered via the profiled-synthesis rule-name
canonicalization (`tools/extract_systemverilog_lrm_profiles.py:315`, `$name` → `sv_dollar_name`
— correct for RULE NAMES, leaked into keyword-token LITERALS). No pre-parse rewrite exists
(`grep -rn "sv_dollar" rust/src/` → zero hits), so the defect is live in released `1.0.158`.

## Non-Goals

- No re-run of the LRM extraction/synthesis pipeline: the flattened
  `grammars/systemverilog.ebnf` is the actively-maintained artifact (census 1450 with many
  post-synthesis hand-landed fixes); the fix lands there directly. The synthesis tool's
  name-vs-literal confusion is recorded for provenance; fixing the *tool* is only worth doing
  if the profiles are ever re-synthesized (tracked as an open question, not a leaf).
- No behavioral change to the generic `system_tf_identifier` surface (`$display`, `$random`, …)
  — that token (`/\$[a-zA-Z0-9_$]+/`) is correct and is how the external corpus parses today.
- Witness/cert ratcheting beyond what the fixes naturally earn (owned by the per-profile trees).

## Profile map (from the `.6.4` adjudication — drives the gating design)

| Token group | LRM home | `verilog_2005` | `sv_2017`/`sv_2023` |
| --- | --- | --- | --- |
| 12 timing checks (`$setup`,`$hold`,`$setuphold`,`$recovery`,`$recrem`,`$removal`,`$skew`,`$timeskew`,`$fullskew`,`$period`,`$width`,`$nochange`) | 1364-2005 §15 + 1800 §31 | **in-profile** | in-profile |
| `$root` (`:2356` optional prefix, `:2372` mandatory) | 1800 only | must REJECT | in-profile |
| `$unit` (`:3762`) | 1800 only | must REJECT | in-profile |
| `$fatal`/`$error`/`$warning`/`$info` elaboration tasks (`:2019+`, `:4871+`) | 1800 only (module-level elaboration form) | must REJECT | in-profile |
| bare `$` primary (`kw_sv_dollar_04da59ec`, `:2976`/`:2993`/`:4026`/`:4081`) | 1800 (`$` as unbounded literal) | adjudicate in `.1` | in-profile |

## Acceptance Criteria

- Every affected LRM spelling ACCEPTS in its correct profiles and every mangled nonsense
  spelling REJECTS, proven by a per-token probe matrix (accept + reject, both directions).
- `scalar_constant` accepts exactly the 10 LRM alternatives, and the
  `scalar_timing_check_condition` eq/case_eq/ne/case_ne branches are reachable (shadowing
  removed) — with the AST-shape/schema impact adjudicated and, if shapes change, the release/
  schema/ledger/book lockstep performed.
- No regression: canonical cert (seeds 0/7/42, `spf=0`) with residual-set drift adjudicated
  (witness-count drift is EXPECTED — new tokens become witnessable; every delta must be
  set-diff-proven honest), union gate + `verilog_2005` conformance gate re-pinned in the same
  commit as any drift, `ast_shape_contract` green, external corpus 14/14, the 6 fully-certified
  grammars byte-identical, clippy source clean.
- Ledger `SV-0029`/`SV-0030` → `Fixed`/`Released`; SV integration contract + SV parser book +
  top-level book lockstep.
- Each leaf passes the `TOOLBOX.md` acceptance checklist and commits per `COMMIT.md`.

## Task Tree

- ID: `SV-DOLLAR-LRM-FIDELITY`
  Status: `active`
  Goal: restore LRM `$`-spelling fidelity for the mangled token family (SV-0029) + the
  `scalar_constant` digit loss (SV-0030), with correct per-profile gating.
  Children: `.1`, `.2`, `.3`

- ID: `SV-DOLLAR-LRM-FIDELITY.1` — **frontier** pending: DESIGN leaf (tools-first, zero code):
  (a) per-token consequence audit — for each of the 19 tokens, probe the LRM spelling and the
  mangled spelling under all 3 profiles at HEAD (the `.6.4` probes cover `$setup`/`$unit`/
  `$fatal`/`$root`; complete the matrix incl. the bare-`$` primary sites); (b) collision audit —
  changing a token literal from identifier-shaped text to a `$` spelling can re-route parses
  (e.g. `$setup` currently lexes as `system_tf_identifier` in expression/statement contexts;
  `$root` as the anchor vs generic system-tf; check PEG order at every referencing site);
  (c) AST-shape/schema impact — which typed shapes change (e.g. `$root.` chains gain the
  anchored kind; `scalar_timing_check_condition` eq forms move from flat-expression to
  `kind:"eq"`), whether a release/schema bump is required, and which shape-contract samples to
  add; (d) profile-gating design per the profile map (which HOST rules need `@profiles` gates
  under `verilog_2005` once the literals are real — `$root`/`$unit`/severity vs the in-profile
  timing checks); (e) wave plan — slice `.2`/`.3` (and further if needed) so each wave is one
  surgical, fully-gated commit.

- ID: `SV-DOLLAR-LRM-FIDELITY.2` — proposed: CODE leaf, wave 1 per the `.1` design (expected:
  the 12 timing-check tokens — in-profile in BOTH dialects, so no new gating needed — plus the
  `SV-0030` scalar_constant digit restoration + eq-branch de-shadowing, since the two surfaces
  compose inside `specify`).

- ID: `SV-DOLLAR-LRM-FIDELITY.3` — proposed: CODE leaf, wave 2 per the `.1` design (expected:
  `$root`/`$unit`/elaboration-severity/bare-`$` — the SV-only group, each literal fix paired
  with its `verilog_2005` profile gate so the strict profile REJECTS the now-real `$` SV
  spellings).

## Current Frontier

| Order | Leaf | Status | Why next |
| --- | --- | --- | --- |
| 1 | `SV-DOLLAR-LRM-FIDELITY.1` | `pending` | design/audit must precede any grammar edit (19 tokens × 3 profiles blast radius; AST-shape + schema + gating consequences need adjudication first) |

## Decisions

- `2026-07-02`: fix lands in the flattened `grammars/systemverilog.ebnf` (the maintained
  artifact), NOT via re-running the synthesis pipeline (see Non-Goals). The synthesis tool
  defect is provenance only.
- `2026-07-02`: tree named `SV-DOLLAR-LRM-FIDELITY` to sit beside the existing SV fidelity
  trees; the defect family is ONE mechanism (name-canonicalization leaked into literals) so it
  gets ONE ledger row (`SV-0029`), with the `scalar_constant` content loss as a distinct
  second mechanism (`SV-0030`).

## Open Questions

- Whether `tools/extract_systemverilog_lrm_profiles.py` should be fixed too (only matters on a
  future re-synthesis; does not block the frontier).
- Whether the bare-`$` primary sites (`kw_sv_dollar_04da59ec`) are redundant with the correct
  `kw_dollar := /\$/` covergroup/queue-bound sites or a genuine additional LRM surface —
  adjudicate in `.1` (b).

## Blockers

- None.

## Verification Log

| Date | Leaf | Checks | Result |
| --- | --- | --- | --- |
| `2026-07-02` | (origin) | the `VERILOG-2005-PROFILE.6.4` probe matrix (LRM-vs-mangled × profiles) | recorded in that tree's `.6.4` Findings + Verification Log |

## Commit Log

| Leaf | Commit subject or reference | Notes |
| --- | --- | --- |
| (origin) | `PGEN-VERILOG-2005-PROFILE-0017` (`VERILOG-2005-PROFILE.6.4`) | discovery + ledger rows `SV-0029`/`SV-0030`; zero code |

## Changelog

- `2026-07-02`: Created task tree from the `VERILOG-2005-PROFILE.6.4` adjudication findings.

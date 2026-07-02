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

- ID: `SV-DOLLAR-LRM-FIDELITY.1` — Status: `done` (2026-07-02, session #21,
  `PGEN-SV-DOLLAR-LRM-FIDELITY-0001`, ZERO code — DESIGN/AUDIT leaf, tools-first): the full
  per-token × per-profile probe matrix, the collision audit, the shape/schema adjudication, the
  gating map, and the wave plan are all recorded (see "`.1` Findings"). Headlines: all 12
  timing checks are literal-only defects (LRM REJ / mangled ACC, 24/24 probes, both dialects;
  rule bodies faithful to IEEE 1800-2017 A.7.5.1 incl. `$width`'s mandatory `threshold`);
  `specify_item` has NO generic system-TF alternative → wave 1 is collision-free; `$unit` +
  the 4 severity tasks are LRM-REJ/mangled-ACC on all 3 profiles, and the mangled severity
  spellings ACCEPT under `verilog_2005` TODAY (their host rides the v2005-admitted
  `module_common_item_sv_2017`) → wave-3 gates required; `$root` accepts in BOTH spellings but
  the LRM spelling mis-routes (`kind:"system_tf"`) — wave 3 is PEG-ordered-choice-commit
  collision work; **`SV-0030` has a SECOND site**: `init_val` (`:2429`) lost the same digits —
  UDP `initial q = 1'b0;` (LRM) REJECTS while `initial q = 1'b;` ACCEPTS (x-forms `1'bx` etc.
  intact). Token-naming decision: literals change, names stay (set-comparability of pinned
  residual lists; the sha1-of-literal hash suffix mismatch is accepted cosmetic debt).

- ID: `SV-DOLLAR-LRM-FIDELITY.2` — **frontier** pending: CODE leaf, wave 1 (`SV-0029` timing
  checks): correct the 12 timing-check token literals `sv_dollar_X` → `$X`
  (`/sv_dollar_setup\b/` → `/\$setup\b/`, etc.). In-profile for BOTH dialects — no new gates;
  collision-free (specify-only surface, `specify_item:4934` has no competing system-TF route).
  Accept-set change on all profiles ⇒ SV release bump; typed kinds pre-exist (no carrier
  reshape expected — adjudicate schema on the diff); add conformance-corpus LRM-spelling accept
  locks + mangled-spelling reject locks; cert/union/v2005 re-pins with set-diff-proven deltas
  (mangled-spelling witnesses re-route to the LRM spellings).

- ID: `SV-DOLLAR-LRM-FIDELITY.3` — proposed: CODE leaf, wave 2 (`SV-0030`, both sites):
  restore the LRM digits — `scalar_constant` (`:4713`) to the ten 1364-2005/1800 alternatives
  and `init_val` (`:2429`) to the ten UDP alternatives (new `1'b0`/`1'b1`/`1'B0`/`1'B1` tokens,
  sha1-of-literal names; the digit-less `1'b`/`1'B` tokens are removed with their last
  referencing sites) + reorder `scalar_timing_check_condition` (`:4720`) so the
  eq/case_eq/ne/case_ne branches precede the bare-`expression` branch. SHAPE-AFFECTING:
  `e == 1'b0` in a timing-check condition moves from flat `kind:"expression"` to `kind:"eq"`
  ⇒ schema bump + shape-contract samples + book/contract lockstep.

- ID: `SV-DOLLAR-LRM-FIDELITY.4` — proposed: CODE leaf, wave 3 (`SV-0029` SV-only group):
  `$root` (2 sites) / `$unit` (`package_scope:3760`) / the 4 elaboration-severity tokens /
  bare-`$` literal fixes + the PEG-order adjudication per referencing context (the
  ordered-choice COMMIT hazard: `system_tf_identifier` matches `$root`/`$unit` prefixes, and a
  committed choice does not re-enter on outer failure — each `primary`/statement site needs its
  order proven, not assumed) + the `verilog_2005` gates for the now-real SV-only spellings
  (incl. closing the TODAY-leak: mangled `sv_dollar_fatal;` accepts under `verilog_2005`).
  AST-shape impact: `$root.`-anchored names move from `kind:"system_tf"` to the hierarchical
  kinds ⇒ schema adjudication + samples.

## Current Frontier

| Order | Leaf | Status | Why next |
| --- | --- | --- | --- |
| 1 | `SV-DOLLAR-LRM-FIDELITY.2` | `pending` | wave 1 (12 timing-check literals) is the audited collision-free slice: pure literal swap, no gates, no shape change — the lowest-risk, highest-value fix (LRM specify surface restored on BOTH dialects) |
| 2 | `SV-DOLLAR-LRM-FIDELITY.3` | `pending` | wave 2 (`SV-0030` digits + reorder) is shape-affecting (schema bump) — lands after the literal wave so each re-pin has one cause |
| 3 | `SV-DOLLAR-LRM-FIDELITY.4` | `pending` | wave 3 (SV-only group) needs per-site PEG-order proofs + `verilog_2005` gates — the most delicate slice, last |

## `.1` Findings (tools-first, 2026-07-02 — the per-token audit)

All probes: `parseability_probe --parse systemverilog <file> --profile {sv_2017, sv_2023,
verilog_2005}` on HEAD binaries (release `1.0.158`), files preserved under the session
scratchpad (`sv0029_matrix/`). "LRM" = the IEEE `$` spelling; "MAN" = the mangled
`sv_dollar_*` spelling.

- **(a) The 12 timing checks — literal-only defect, 24/24 probes:** every LRM spelling REJECTS
  and every mangled spelling ACCEPTS, under BOTH `sv_2017` and `verilog_2005` (minimal legal
  instance per rule shape, e.g. `$setup(d, posedge clk, 1);`, `$width(posedge clk, 1, 0);`,
  `$nochange(posedge clk, d, 0, 0);`). The rule BODIES are faithful to IEEE 1800-2017 A.7.5.1
  — including `$width`'s MANDATORY `threshold` (verified against
  `docs/systemverilog/2017/txt/section-31-timing-checks.txt:71-72`:
  `$width ( controlled_reference_event , timing_check_limit , threshold [ , [ notifier ] ] )`;
  the threshold-less `$width(posedge clk, 1);` REJECTS in BOTH spellings — correct per 1800,
  a 1364-2005-vs-1800 arg-shape nuance recorded as an open question for the v2005 profile,
  NOT a defect).
- **(b) Wave-1 collision audit — CLEAN:** `specify_item` (`:4934`) = specparam | pulsestyle |
  showcancelled | path_declaration | system_timing_check — NO generic `system_tf_call`
  alternative, so inside `specify` the fixed `$setup` literal has no competing route. In
  procedural/expression contexts `$setup(…)` continues to lex via `system_tf_identifier`
  (`:449`) — the keyword tokens are referenced ONLY by the 12 timing-check rules
  (`:5074-5107`), so no procedural behavior changes.
- **(c) `$unit` / severity / bare-`$` / `$root` (3-profile matrix):** `$unit::y` REJ/REJ/REJ
  vs `sv_dollar_unit::y` ACC/ACC/ACC (host `package_scope:3760`); module-level `$fatal;` /
  `$error;` / `$warning;` / `$info;` all REJ×3 vs mangled ACC×3 — **note the mangled severity
  spellings ACCEPT under `verilog_2005` TODAY** (their hosts `elaboration_system_task_sv_2017`
  (`:2019`) / `severity_system_task_sv_2023` (`:4871`) ride `module_common_item_sv_2017`,
  which is `@profiles ["sv_2017","verilog_2005"]`-admitted — a live v2005 over-acceptance of
  nonsense text, and post-fix the real `$fatal;` would leak into v2005 unless the branch is
  gated in wave 3); bare `x = $;` REJ×3 vs `x = sv_dollar;` ACC×3 (per IEEE 1800 A.8.4
  `primary ::= … $ …`, the fixed literal would make `x = $;` grammatically parseable —
  1800-faithful; semantic restriction to queue-bound contexts is out of parser scope);
  `$root.m.y` / `$root.m.f(1);` ACCEPT in BOTH spellings on ALL 3 profiles — but the LRM
  spelling routes through the generic system-TF token (`kind:"system_tf"`, AST-proven in
  `.6.4`), NOT the `$root`-anchored forms.
- **(d) The wave-3 hazard named — PEG ordered-choice COMMIT:** `system_tf_identifier`
  (`/\$[a-zA-Z0-9_$]+/`) matches `$root`/`$unit` as prefixes; a `primary`/statement alternative
  that succeeds on the system-TF reading COMMITS, and an outer failure (`::`-tail,
  hierarchical tail) backtracks past the WHOLE choice without re-entering later alternatives.
  So merely fixing the `$root`/`$unit` literals may be INERT (the steal continues) or may flip
  accept→reject in tail contexts — every referencing context needs its order proven with
  before→after probes in wave 3, not assumed.
- **(e) `SV-0030` second site:** `init_val` (`:2429`) — UDP probes (both profiles):
  `initial q = 1'b0;` REJ, `initial q = 1'b1;` REJ (the LRM spellings!), `initial q = 1'b;`
  ACC (nonsense), `initial q = 1'bx;` ACC, `initial q = 1;` ACC. Same prefix-merge digit loss
  as `scalar_constant` — the x-forms (`1'bx`/`1'bX`/`1'Bx`/`1'BX`, `:6060-6070`) survived
  because they were not prefix-mergeable. Ledger row `SV-0030` extended with this site.
- **(f) Token naming:** the `kw_*_<hash>` suffix is `sha1(literal)[:8]` (verified:
  `c2543fff`=sha1("this"), `b58bdaae`=sha1("sv_dollar_setup"), `f4c81681`=sha1("1'b")).
  Decision: change literals, KEEP the existing token names — renames would churn the pinned
  cert residual/NO-reach name lists and break exactly the set-diff signal the no-regression
  proofs rely on; the resulting name-hash/literal mismatch is accepted, documented cosmetic
  debt (new tokens introduced by wave 2 DO follow the sha1-of-literal convention).

## Decisions

- `2026-07-02`: fix lands in the flattened `grammars/systemverilog.ebnf` (the maintained
  artifact), NOT via re-running the synthesis pipeline (see Non-Goals). The synthesis tool
  defect is provenance only.
- `2026-07-02`: tree named `SV-DOLLAR-LRM-FIDELITY` to sit beside the existing SV fidelity
  trees; the defect family is ONE mechanism (name-canonicalization leaked into literals) so it
  gets ONE ledger row (`SV-0029`), with the `scalar_constant` content loss as a distinct
  second mechanism (`SV-0030`).
- `2026-07-02` (`.1`): waves re-sliced by RISK CLASS, one concern per commit — `.2` = pure
  literal swap (12 timing checks, no gates, no shape change), `.3` = shape-affecting `SV-0030`
  (digits + reorder ⇒ schema bump), `.4` = SV-only group (PEG-order proofs + `verilog_2005`
  gates). The originally sketched ".2 = timing + SV-0030" bundle was split so every cert/shape
  re-pin has exactly one cause.
- `2026-07-02` (`.1`): token literals change, token NAMES stay — renames would churn the
  pinned residual/NO-reach name lists and destroy the set-diff no-regression signal; the
  sha1-of-literal hash-suffix mismatch is accepted, documented cosmetic debt (wave-2's NEW
  tokens do follow the convention).

## Open Questions

- Whether `tools/extract_systemverilog_lrm_profiles.py` should be fixed too (only matters on a
  future re-synthesis; does not block the frontier).
- The bare-`$` primary sites (`kw_sv_dollar_04da59ec` at `:2976`/`:2993`/`:4026`/`:4081`) are a
  genuine IEEE 1800 A.8.4 surface (`primary ::= … $ …`), distinct from the correct
  `kw_dollar := /\$/` covergroup/queue-bound sites — the wave-3 literal fix makes `x = $;`
  grammatically parseable (1800-faithful; semantic queue-bound restriction is out of parser
  scope). Resolved in `.1` (c).
- `verilog_2005`-profile nuance: 1364-2005 §15.5.4 allows `$width(controlled_ref, limit)`
  (threshold optional) while our 1800-faithful rule mandates the threshold — a potential
  future v2005-profile arg-shape refinement, NOT a defect in the SV grammar (verified against
  the 1800-2017 section-31 BNF). Does not block any wave.

## Blockers

- None.

## Verification Log

| Date | Leaf | Checks | Result |
| --- | --- | --- | --- |
| `2026-07-02` | (origin) | the `VERILOG-2005-PROFILE.6.4` probe matrix (LRM-vs-mangled × profiles) | recorded in that tree's `.6.4` Findings + Verification Log |
| `2026-07-02` | `.1` | 12 timing checks × {LRM, mangled} × {sv_2017, verilog_2005} = 26 probes (incl. the two `width_min` controls); `$unit`/severity×4/bare-`$`/`$root`×2 × {LRM, mangled} × 3 profiles; UDP `init_val` × 5 digit forms × 2 profiles; `specify_item` alternatives read; severity-host profile-admission cross-checked against the `man_sev_fatal` v2005=ACC probe; `$width` BNF verified in `section-31-timing-checks.txt`; sha1 naming convention verified on 3 samples | all recorded in "`.1` Findings"; zero code |

## Commit Log

| Leaf | Commit subject or reference | Notes |
| --- | --- | --- |
| (origin) | `PGEN-VERILOG-2005-PROFILE-0017` (`VERILOG-2005-PROFILE.6.4`) | discovery + ledger rows `SV-0029`/`SV-0030`; zero code |
| `.1` | `PGEN-SV-DOLLAR-LRM-FIDELITY-0001` (`SV-DOLLAR-LRM-FIDELITY.1`) | design/audit closed; wave plan re-sliced `.2` literals / `.3` SV-0030 / `.4` SV-only; `SV-0030` extended with the `init_val` site; zero code |

## Changelog

- `2026-07-02`: Created task tree from the `VERILOG-2005-PROFILE.6.4` adjudication findings.
- `2026-07-02`: `.1` design/audit leaf closed (`PGEN-SV-DOLLAR-LRM-FIDELITY-0001`); frontier →
  `.2` (wave 1, the 12 timing-check literals).

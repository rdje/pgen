# CORPUS-GRAD-ALL: every PGEN parser graduates from its external official/recognized corpora before `Done` — exhaustive per-family corpus discovery, audit, and gate wiring

## Metadata

- Tree ID: `CORPUS-GRAD-ALL`
- Status: **`active`** (created 2026-07-22, session #190, on the director's
  cross-family directives — verbatim: "All PGEN parser to be exersized against
  their external and official and recognized test corpus before claiming they
  are done." + "So you will need to do an exhaustive search for each parser and
  pin all the applicable external test corpus that will increase the confidence
  in the fact that the corresponding parser is really Done and is sota,
  signoff-grade." — [[project_all_parsers_done_requires_corpus_graduation]])
- Roadmap lane: the Nexsim delivery directive
  ([[project_nexsim_sv_signoff_delivery_focus]]) + the locked program + the
  external-corpus doctrine ([[project_external_corpus_doctrine]]).
- Created: `2026-07-22`
- Owner: repo-local workflow
- Cross-links: generalizes [[project_sv_done_requires_external_corpus_graduation]]
  (SV = the template campaign, tree `SV-CORPUS-GRAD`); supersedes the
  EXTERNAL-CORPUS "`.3+` not blocking all-parsers-Done" framing (they now DO
  block, per family); supersedes-by-GO the `REJECTS-VALID-DEFENSE` cross-family
  corpus clause (its non-corpus instruments stay parked); acquisition +
  characterization precedents = `EXTERNAL-CORPUS` (`regex_corpus_bundle/`,
  `json_corpus_bundle/`, `stimuli/sv/subs/`, `stimuli/vhdl/subs/`).

## Goal (the tree's single deliverable)

Every parser family's `Done` claim is backed by graduation from an
**exhaustively-discovered, pinned, frozen roster** of external official/
recognized corpora — adjudicated by each corpus's own answer key / governing
spec with **zero unexplained divergences**, proven by standing deterministic
per-family graduation gates wired into each family's `Done` computation — or an
honest, tracked **N/A-with-cause** adjudication where no external corpus can
exist. Never gamed ([[feedback_corpus_expected_from_spec_not_fix]]).

## Per-family state (2026-07-22 snapshot; `.1` refines and freezes)

| Family | LIVE row | Vendored today | Known gap / note |
|---|---|---|---|
| regex | Done | PCRE2 `regex_corpus_bundle/` + live gates (2,189-cell oracle corpus, divergences tracked) | LIKELY already graduated — formal audit + roster check (candidates beyond PCRE2: cross-engine/Unicode suites — `.1` adjudicates applicability under the PCRE2-faithful contract) |
| systemverilog | Mostly Done | `stimuli/sv/subs/` (sv-tests/verible/slang/verilator + designs) | campaign ACTIVE = `SV-CORPUS-GRAD` (roster additions, e.g. ivtest for `verilog_2005`, adjudicated there) |
| systemverilog_preprocessor | Done | (shares SV suites' preprocessor cases) | mapping to pin in `.1` |
| vhdl | **Done — AT RISK** | `stimuli/vhdl/subs/` (ghdl incl. VESTS+gna / nvc / OSVVM / UVVM / vunit + designs; characterized **29.4%**) | **NOT graduated** — VESTS ≈2,042 compliant-but-rejected real gaps; the material demotion candidate; second campaign (Nexsim also needs VHDL) |
| json (built-in) | (no family row) | `json_corpus_bundle/` (JSONTestSuite) | graduation inherently gated on the parked `JSON-RFC8259` commitment (`.4` conformance gate) |
| rtl_frontend / rtl_const_expr | Done / Mostly Done | — | internal subset grammars — N/A-with-cause candidates (`.1` adjudicates; possible mapped SV-subset slice) |
| return_annotation / semantic_annotation / ebnf | Done | — | PGEN-defined DSLs — N/A-with-cause candidates (`.1` adjudicates) |

## Leaves

### `.1` — EXHAUSTIVE per-family corpus discovery + pinning + `Done`-claim audit (research + read-only)

- **Status: `todo`** (the director-mandated exhaustive search). For EACH family:
  research-grounded discovery ([[feedback_research_grounded_sota_no_trial_and_revert]]:
  primary sources, citations) of ALL applicable official/recognized suites —
  standards-body suites, reference-implementation regression corpora,
  community conformance aggregations — each candidate pinned (upstream repo/URL,
  license, size, answer-key/metadata form) and adjudicated for applicability
  against the family's contract (e.g. regex is PCRE2-faithful by contract —
  other-engine suites apply only where dialect-compatible,
  [[project_regex_pcre2_faithful_by_default]]). Output per family: the FROZEN
  graduation roster + the honest audit of the current LIVE row against it
  (graduated / campaign-needed / N/A-with-cause). Evidence in
  `docs/tasks/artifacts/corpus_grad_all/`.

### `.2` — VHDL graduation campaign opening (the second family campaign)

- **Status: `todo`** — the largest measured gap (29.4%; VESTS answer-key
  adjudication first: compliant[must-parse] / non_compliant[must-reject] /
  gna expected-fail drivers). Opens the VHDL analogue of `SV-CORPUS-GRAD`
  (own tree) scoped from `.1`'s frozen VHDL roster; the VHDL `Done` row is
  re-adjudicated honestly there.

### `.3` — Per-family graduation-gate wiring (code; per-family leaves)

- **Status: `todo`** — each family's `Done` computation gains its
  corpus-graduation criterion (the SV `.5` pattern generalized: deterministic
  gate over pinned corpus commits; family-status gates / LIVE-row oracles).

### `.4` — Lockstep

- **Status: `todo`** — LIVE tracker per-row re-adjudication from the audits,
  books (top-level corpus doctrine chapter + per-parser books), integration
  contracts.

## Delegations (owned elsewhere, tracked here)

- SV → `SV-CORPUS-GRAD` (active; axis 2 of the SV `Done` bar).
- JSON → `JSON-RFC8259` (parked; director-gated).
- The non-corpus REJECTS-VALID instruments → `REJECTS-VALID-DEFENSE` (parked).

## Acceptance Criteria (tree)

1. Every family has a FROZEN, exhaustively-researched, pinned graduation roster
   (or a tracked N/A-with-cause adjudication).
2. Every family's `Done` claim is either backed by a green graduation gate over
   its roster or honestly re-adjudicated (no silent demotions, no gamed passes).
3. Gates are standing, deterministic, wired into each family's `Done`
   computation; full lockstep.

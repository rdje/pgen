# project — EVERY PGEN parser's `Done` requires external-corpus graduation (the SV bar GENERALIZED to all families)

- Date: 2026-07-22 (session #190)
- Category: project (director directive — the cross-family `Done` bar)
- Status: ACTIVE — binding on every parser family's `Done` claim

## Context

Director directive (2026-07-22, verbatim): **"All PGEN parser to be exersized
against their external and official and recognized test corpus before claiming
they are done."** — followed same-day (verbatim): **"So you will need to do an
exhaustive search for each parser and pin all the applicable external test
corpus that will increase the confidence in the fact that the corresponding
parser is really Done and is sota, signoff-grade."**

This generalizes the same-day SV directive
([[project_sv_done_requires_external_corpus_graduation]]) to EVERY family, and
is the cross-family GO for the corpus clause parked in the
`REJECTS-VALID-DEFENSE` charter. It also supersedes the earlier EXTERNAL-CORPUS
framing that "other-grammar external corpora are not blocking all-parsers-Done"
— they now ARE blocking, per family.

## Decision

1. **No parser family claims `Done` without graduating from its external,
   official, recognized test corpus** — exercised, adjudicated by the corpus's
   own answer key / the governing spec, zero unexplained divergences
   ([[feedback_corpus_expected_from_spec_not_fix]]; characterize-don't-game).
2. **Where no external official corpus can exist** (PGEN-defined internal DSLs:
   the annotation grammars, the `ebnf` meta-grammar, the `rtl_*` internal
   subset grammars), the family's corpus surface is adjudicated
   **N/A-with-cause** in the tracked mapping — named, justified, and
   director-visible, never silently assumed.
3. **EXHAUSTIVE per-family corpus DISCOVERY first** (director's follow-up): for
   EACH family, an exhaustive, research-grounded search
   ([[feedback_research_grounded_sota_no_trial_and_revert]]: citations + worked
   mapping, primary sources) enumerating ALL applicable official/recognized
   corpora — beyond what is already vendored — each pinned (upstream, license,
   size, answer-key/metadata form, applicability adjudication). The resulting
   per-family roster is FROZEN as that family's graduation roster; additions
   later re-open the roster leaf, never silently.
4. Owned by the new umbrella tree **`CORPUS-GRAD-ALL`**
   (`docs/tasks/CORPUS-GRAD-ALL.md`): the exhaustive per-family discovery +
   `Done`-claim audit, family campaign delegation (SV → `SV-CORPUS-GRAD`
   active; JSON → the parked `JSON-RFC8259.4` conformance gate), and the
   per-family graduation-gate wiring into each family's `Done` computation.

## Per-family mapping (initial; the `.1` audit refines it)

| Family | Current LIVE row | External corpus | Graduation state |
|---|---|---|---|
| regex | Done | PCRE2 (`regex_corpus_bundle/`, vendored; textsafe + compile-oracle gates; the 2,189-cell oracle corpus in continuous ratchet use, divergences tracked) | LIKELY MET — formal audit to confirm + state it |
| systemverilog | Mostly Done | sv-tests/verible/slang/verilator (`stimuli/sv/subs/`, vendored 2026-06-17) | campaign ACTIVE (`SV-CORPUS-GRAD`) |
| systemverilog_preprocessor | Done | mapping to adjudicate (sv-tests/verilator preprocessor cases; UVM macro surface) | AUDIT PENDING |
| vhdl | **Done — AT RISK** | ghdl(VESTS+gna)/nvc/OSVVM/UVVM/vunit + designs (`stimuli/vhdl/subs/`, vendored 2026-06-17; characterized **29.4%**, VESTS ≈2,042 compliant-but-rejected real gaps) | **NOT graduated — the material demotion candidate; needs its own campaign** |
| json (built-in) | (no family row) | JSONTestSuite (`json_corpus_bundle/`, vendored) | gated on the parked `JSON-RFC8259` full-standard commitment (`.4` = the conformance gate) |
| rtl_frontend / rtl_const_expr | Done / Mostly Done | none exists (PGEN-internal subset grammars) | N/A-with-cause adjudication in `.1` (or a mapped SV-subset slice) |
| return_annotation / semantic_annotation / ebnf | Done (annotation row) | none can exist (PGEN-defined DSLs) | N/A-with-cause adjudication in `.1` |
| verilog_2005 (SV profile) | (profile of SV) | ivtest (Icarus) = the natural candidate | roster adjudication in `SV-CORPUS-GRAD.1` |

## Consequences

- **Existing `Done` rows are NOT silently rewritten** — the `.1` audit
  re-adjudicates each row with evidence; but the LIVE tracker carries a dated
  note that vhdl / svpp / rtl_* / annotation `Done` claims are pending
  corpus-graduation audit under this bar (VHDL the material case).
- Sequencing stays Nexsim-first ([[project_nexsim_sv_signoff_delivery_focus]]):
  SV graduation is the active campaign; **VHDL is the natural second campaign**
  (the other Nexsim target, and the largest measured gap).
- The `REJECTS-VALID-DEFENSE` cross-family corpus clause is superseded-by-GO
  (its non-corpus instruments — grammar-edit differential, context-dual grid,
  mutation-differential, spec traceability — remain parked awaiting their own GO).

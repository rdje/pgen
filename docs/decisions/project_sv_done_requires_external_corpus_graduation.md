# project — SV `Done` REQUIRES external-corpus graduation (verible, slang, sv-tests, …)

- Date: 2026-07-22 (session #190)
- Category: project (director directive — the SV `Done` bar AMENDED)
- Status: ACTIVE — binding on the SV Done campaign

## Context

Director directive (2026-07-22, verbatim): **"AS part of th SV Done campaign we
should include the external official and recognized SV test corpus (verible,
slang, ...). The SV parser shall cleanly pass all of them with flying colors. No
Done with the SV parser graduating from those external official and recognized
SV corpus."** — **CLARIFIED by the director same-day (verbatim): "I meant 'No
Done WITHOUT the SV parser graduating from those external official and
recognized SV corpus'. Without. I meant Before claiming SV is Done it should
have passed all the external and official test corpus."** (The recorded
semantics were already this reading — graduation is a hard PRECONDITION for
`Done`.)

Until now the SV `Done` bar was the machine-computed 7-criterion
`sv_parser_family_status_gate` (last unmet: `focused_replay_target_debt_zero`,
owned by tree `SV-REPLAY-DEBT`). ⭐ **The corpora themselves were ALREADY
acquired and characterized** (director-prompted sweep re-surfaced the capture):
`EXTERNAL-CORPUS.3.1` (`PGEN-EXTERNAL-CORPUS-0007`, 2026-06-17) vendored
sv-tests (`25e4d275`, ISC, 1028) + verible (`a0a8d8eb`, Apache) + slang
(`4106501b`, MIT) + verilator (`a534a1d1`, ⚠️ copyleft, director-ruled GPL-OK
for test-input use) as pinned sparse submodules under `stimuli/sv/subs/` (with
`PROVENANCE.md`), runner `stimuli/run_external_corpus.sh sv`, and an honest
baseline characterization (5128 files, 58.0% pass; report
`stimuli/sv/characterization/characterization.md`); the roster reference is
`docs/decisions/reference_sv_external_corpus_and_oracle_repos.md`. The
follow-up adjudication ("expected-vs-actual per each corpus's own answer key")
was lane 3 of the BINDING 2026-06-17 director sequencing — (1) SV `UNKNOWN→0`
✅ done 2026-07-22, (2) `GRAMMAR-WELLFORMED.H.12.5.8` infix parse bug (open),
(3) the corpus gap-drive. **This directive ACTIVATES lane 3 into the SV Done
campaign** (and supersedes-by-GO the parked `REJECTS-VALID-DEFENSE` SV clause).

## Decision

1. **The SV `Done` bar has TWO axes**, both mandatory:
   - the existing family-status gate (7 criteria; `SV-REPLAY-DEBT` closes the last), and
   - **external-corpus graduation**: the SV parser cleanly passes the vendored
     official/recognized SV corpora — **sv-tests, verible, slang, verilator**
     (+ the real-design corpora VeeR/scr1/friscv and any `.1`-adjudicated
     roster additions, e.g. ivtest/yosys).
2. Owned by the new tree **`SV-CORPUS-GRAD`** (`docs/tasks/SV-CORPUS-GRAD.md`),
   building on the EXISTING assets (no re-acquisition): fresh characterization
   at HEAD vintage, the expected-verdict adjudication manifest (suite metadata
   as the answer key — sv-tests `:should_fail_because:`/`:tags:`, verilator
   expected-fail conventions), defect burn-down (first known member:
   `H.12.5.8`), full-design chaining, and a standing graduation gate wired into
   the family-status `Done` computation (an 8th criterion — a gate-spec change
   owned by its own leaf).
3. **"Cleanly pass with flying colors" is earned, never gamed** (per
   [[feedback_corpus_expected_from_spec_not_fix]]): every corpus case gets a
   spec-derived expected verdict (accept-valid / reject-invalid /
   out-of-scope-with-cause, e.g. preprocessor-required or tool-specific
   extensions — each classification tool-proven against IEEE 1800, exclusions
   named and justified in the tracked triage manifest, never silently dropped).
   Graduation = zero unexplained divergences across the vendored suites.

## Consequences

- The LIVE tracker's SV `Done` flip now requires BOTH axes green; the
  family-status gate's Done computation gains the graduation criterion when the
  gate leaf lands (until then the tracker carries the two-axis bar in prose).
- `SV-REPLAY-DEBT` (axis 1) and `SV-CORPUS-GRAD` (axis 2) are sibling
  workstreams of the one campaign ([[project_nexsim_sv_signoff_delivery_focus]]);
  the parked `REJECTS-VALID-DEFENSE` charter's SV clause is superseded-by-GO
  (the cross-family generalization for OTHER parsers stays parked there).
- These corpora are also exactly the REJECTS-VALID defense for SV that the
  charter anticipated: grammar-derived stimuli cannot find rejects-valid bugs
  ([[feedback_stimuli_gen_rejects_valid_blindspot]]) — external corpora can.

# project — Nexsim needs a SOTA, signoff SV parser: FOCUS and DELIVER

- Date: 2026-07-22 (session #190)
- Category: project (director directive — delivery focus)
- Status: ACTIVE — the standing mission framing for all SystemVerilog work

## Context

Mid-session directive from the director (2026-07-22, verbatim): **"NEXSIM needs a
sota, signoff SV parser, so we need to focus on that and deliver."**

Nexsim is a primary near-term integration target (README: SystemVerilog + VHDL
parsing). The directive arrived while the session was executing exactly the SV
`Done` workstream (the `f48a60b2` handoff frontier): re-baselining the
`focused_replay_target_debt` — the LAST unmet criterion of SV's machine-computed
family-status `Done` bar.

## Decision

The SystemVerilog parser is the DELIVERY FOCUS. Concretely, in order:

1. **Close the SV family-status `Done` bar** — tree `SV-REPLAY-DEBT` drives
   `focused_replay_target_debt_zero` to true (or an honestly-proven irreducible
   remainder), flipping `sv_parser_family_status_gate` to `Done`. The other 6
   criteria are already green; the certificate axis is complete (SV recognized
   `fully_certified`, `STRUCTURED-WITNESS-SYNTH`, 2026-07-22).
2. **Signoff trajectory beyond the Done bar** (per the standing closure doctrine +
   the north star): the external-corpus axis (the sv-tests candidate captured in
   the parked `REJECTS-VALID-DEFENSE` charter — suite acquisition director-owned)
   and the per-family SPEED campaign (the RGX apparatus is the template;
   "milk on fire" monitoring) follow once the `Done` bar closes.
3. Nexsim-facing surfaces stay in lockstep every slice: the SV integration
   contract (`docs/contracts/PGEN_SYSTEMVERILOG_PARSER_INTEGRATION_CONTRACT.md`),
   the SV parser book, and the LIVE tracker.

## Consequences

- PNT frontier selection is now pinned to the SV delivery path; the deferred
  backlog / horizon audit stay behind it (unchanged from the `f48a60b2` ranking,
  now director-reinforced).
- Signoff-grade means: no gamed metrics, tool-proven classifications only,
  the full TOOLBOX acceptance checklist on every landing leaf.
- Related: [[project_vision_and_discipline]] (sign-off-grade trust goal),
  [[feedback_correctness_before_speed]] (accuracy first, speed campaign after),
  `docs/tasks/SV-REPLAY-DEBT.md` (the owning tree).

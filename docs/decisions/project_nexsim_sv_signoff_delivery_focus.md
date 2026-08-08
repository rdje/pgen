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

## REAFFIRMED AND SHARPENED — 2026-08-08 (session #213)

Director, verbatim, in three escalating statements: *"NEXSIM is waiting on the SV parser
initially and the VHDL parser later so the SV parser has a higher priority then the VHDL
one, at least for now. So, to me all our time shall be devoted to make the SV parser
release ready."* — on scope: *"all SV axis shall be handled."* — and, as a standing lock:
**"We should stay on the SV parser until it is released to NEXSIM."**

⛔ **THIS IS A LANE LOCK, NOT A RANKING.** PNT frontier selection does not leave the SV
delivery path until SV is RELEASED to Nexsim. A new finding in another family is ROUTED
(a tracked leaf, parked), never worked — including findings this session's VHDL work
queued. The only admissible exception is a defect that BLOCKS the SV release itself,
which by definition is SV work.

**What changes:**

- **SV outranks VHDL explicitly.** This resolves an ordering the cross-family corpus
  directive ([[project_all_parsers_fully_pass_stimuli_and_external_corpora]]) had left open:
  that record forbids letting a lane ROT while another advances, and it is satisfied by
  VHDL being parked at a clean, measured boundary — not by round-robin. `CORPUS-GRAD-ALL.2`
  (VHDL grammar growth) is **PARKED, not abandoned**: corpus 31.6 %, worklist regenerated,
  next classes ranked and queued. "at least for now" is the director's own hedge — the
  ordering is revisitable, the parking is not a demotion.
- **ALL SV axes are in scope, not just the corpus one.** Release-ready is the whole
  family-status bar, and both axes must be green before the SV row flips `Done`
  (`SV-CORPUS-GRAD` metadata). Concretely: axis 1 stimuli duality (closed at literal 0,
  but replayed from a FROZEN universe that enforces nothing — freshness is unproven);
  axis 2 external corpus (`SV-CORPUS-GRAD` `.3`/`.4`/`.5`/`.6`/`.8c.3`/`.9` open); the
  Nexsim-facing surfaces the record already requires in lockstep (integration contract,
  SV parser book, LIVE tracker, bug ledger); and the gate surface the claim RIDES on —
  ⛔ `--lib` RED on HEAD read by no gate (`CI-PARITY-GATE-ROT.21`),
  `ci_workflow_local_gate` unable to complete (`.20`), `sota_exit_gate` not re-proven
  end-to-end since `.7`. A release claim standing on an unproven gate surface is not a
  release claim.
- **Every recorded SV number is STALE and must be re-measured before it is quoted.** The
  external-corpus baseline is `2026-06-17` vintage (the tree itself says ~25 SV releases
  stale) and the `403` unexplained-divergence adjudication is `2026-07-25` with untracked
  raw results. This is the exact defect class `CORPUS-GRAD-ALL.2.1` just fixed for VHDL,
  and the instruments are now generalized — `stimuli/run_external_corpus.sh` emits
  repo-root-relative paths, `stimuli/sv/cluster_rejects_valid.py` is family-parameterized,
  and `furthest_position` is universal — so the SV re-measure is cheap by construction.

# project — EVERY PGEN parser must 100% parse its own stimuli AND 100% of its external corpus, and no lane may go stale

- Date: 2026-08-08 (session #212)
- Category: project (director directive — the PROGRAM-WIDE bar, generalizing the SV-only one)
- Status: ACTIVE — binding on every parser family, not just SystemVerilog

## Context

Director directive (2026-08-08, verbatim):

> *"Whatever we are doing now, we are doing it for objectives reasons, but do not lose focus
> about the ultimate goal, all PGEN parsers need to 100% parser their stimuli generated samples
> and 100% all these external test corpus in case they have one. So, I do not see the point in
> have external corpuses if we are not using them fully to find and fix weakness or flaws in
> their corresponding PGEN parsers. We need to move everything forward not only one part and
> leaving the other parts standing and becoming stale, outdated, ..., see what I mean ?"*

It was given while `SV-EXH-PROOF.7.4.6.19` was closing the SV closed-loop residual at literal
zero — i.e. precisely at the moment one axis of one family looked finished. That timing is the
point: the directive exists to stop a closed axis from being mistaken for a closed program.

## Decision

Two invariants, both at **100%**, for **every** parser family:

1. **AXIS 1 — stimuli duality.** A parser parses 100% of the samples its OWN generator emits.
   (This is what the closed-loop residual measures: SV is now `0` residual over `5 461`
   enumerated coverage targets, both LRM profiles, ratcheted two-sided.)
2. **AXIS 2 — external corpus.** Where a family HAS an external corpus, the parser passes 100%
   of it. "In case they have one" is the director's own scoping — a family with no recognized
   external corpus is not held to a corpus it does not have, and that absence must be *stated*
   per family rather than left implicit.

And a third clause, which is the one with teeth:

3. **A CORPUS EXISTS TO FIND AND FIX, NOT TO CHARACTERIZE AND SHELVE.** Acquiring corpora and
   reporting a pass-rate is not the deliverable; driving the divergences to zero is. A
   characterization that is not being burned down is not evidence of quality — it is a bug
   backlog wearing a report's clothing.

4. **NO LANE MAY GO STALE WHILE ANOTHER ADVANCES.** Moving one family (or one axis) forward
   while the others rot is explicitly refused. A measurement that is not periodically re-run
   against the current tree is not a measurement, it is a memory.

## Consequences

- **It GENERALIZES [[project_sv_done_requires_external_corpus_graduation]]** from "SV `Done`
  requires external-corpus graduation" to *every* family. That record stays valid and becomes
  the SV instance of this one.
- **`Done` remains first-tier only** ([[feedback_done_bar_is_first_tier_only]]): both axes,
  measured, never a snapshot. Literal-zero on axis 1 does NOT license a `Done` flip on its own —
  `SV-EXH-PROOF.7.4.6.6`'s closure explicitly disclaims it.
- **Staleness becomes a defect class, not an inconvenience.** Three independent instances were
  measured in this single session, which is what promoted it from an observation to this record:
  1. the closed-loop **replay stage** replays a FROZEN `initial_gap` + gen-AST out of the last
     canonical gate's work dir, and enforces nothing — so for **ten** generator commits the
     project held "residual 0, ratcheted" on a probe that cannot refuse. (Re-measured
     `2026-08-08`: the universe was in fact byte-identical, so nothing was wrong — but nothing
     would have *told* us if it had been.)
  2. the SV **external-corpus adjudication** (`403` unexplained divergences) was last
     regenerated `2026-07-25`, with `4` intervening `systemverilog.ebnf` commits, and its
     `results.tsv` is **not tracked in git at all**.
  3. the **`--lib` suite is RED on HEAD** and has been since before `2026-08-01`
     (`CI-PARITY-GATE-ROT.21`) — because `--lib` is read by NO gate in the registry.
  The common shape: *a measurement surface that no gate reads goes stale or red in silence.*
  This is [[feedback_enumerating_instrument_must_refuse]] applied to freshness rather than to
  enumeration, and it is the mechanizable core of the director's "becoming stale" clause.
- **Therefore the enforcement direction** (to be designed in its own tree, not assumed here): a
  measurement artifact should declare the tree-state it was produced against and REFUSE when the
  tree has moved, the way the parse harness already refuses a stale single-feature binary
  (`PARSE-HARNESS.10`). Prefer ONE shared mechanism over per-probe patches.
- **Per-family status must be explicit**, so "which parts are standing still" is answerable at a
  glance rather than by archaeology: for each family, axis-1 residual, axis-2 corpus + unexplained
  divergence count + the date it was last measured, or an explicit "no external corpus exists".

## Honest bounds at the time of writing

No family is at 100% on axis 2 today. Known state: SV `403` unexplained (stale); regex carries a
pinned PCRE2 oracle tuple whose false-reject ratchet sits at `48`; VHDL has a triage gate whose
adjudicated state has not been re-measured this session. These are the numbers the program has to
move, and the first honest act is to re-measure them rather than to quote them.

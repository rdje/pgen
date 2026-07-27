---
name: project-waiver-is-a-gate-bug-report
description: DOCTRINE (2026-07-27, session #216, director-ordered) — an author writing a waiver ("the diagnosis signatures do not apply to my defect") is the GATE reporting a missing capability, not an author cutting a corner; it is the highest-signal defect report a gate can receive, and RGX-0090's sat unread for months. Mechanized as the 10th enforced doctrine WAIVER-ROUTING: a waiver stays legal, but it must name the leaf that owns fixing the gate.
metadata:
  node_type: memory
  type: project
---

**The founding case.** `docs/tasks/RGX-0090.md:131` carried this, hand-written *inside a ticked
ROOT CAUSE box*:

> *"(Waiver note: like RGX-0091 this is a BUILD-FLOW defect — the parse/perf diagnosis-toolbox
> signatures do not apply; root cause is backed by the recorded scratch reproduction runs above.)"*

That author was **right**, and precise, and nothing happened. `docs/tasks/RGX-0091.md:147` did the
same, and even wrote a line headed *"Diagnosis tool signatures:"* citing real `grep -n` output —
and got no credit for it. Months later `GENERATED-LINT-CORRECTNESS.4` re-derived the identical gap
from scratch, measured that the acceptance gate modelled four diagnosis families and none fitted a
build-flow defect, and added the **ops/build-flow** family under which both leaves would have
passed with no waiver at all.

## ⭐ The law

**An author writing a waiver is the GATE reporting a missing capability.** It is the highest-signal
defect report a gate can receive, because it comes from someone who *did the work*, *hit the
boundary*, and *wrote down exactly where the boundary was*. Treating it as an author cutting a
corner inverts the diagnosis: the author complied; the instrument was incomplete.

Director, 2026-07-27, on being shown the unread note: *"What should we do here to correct that and
prevent that it does not happen ever again."*

## The mechanization — `WAIVER-ROUTING`, the 10th enforced doctrine

`scripts/check_waiver_routing.sh`, registered in `scripts/check_doctrines.sh`. If a staged
`docs/tasks/*.md` **adds** a claim that a gate's signature surface cannot express its evidence,
that claim must name an owning leaf id (`TREE.4`) or slice id (`PGEN-FAMILY-0001`) in its
immediate neighbourhood. Otherwise the commit is blocked, with the offending line quoted.

⛔ **It must not punish honesty — this is the load-bearing design constraint**, inherited from
`DOCTRINE-GAP-OWNERSHIP.2`. Forbidding waiver language would simply delete the signal: authors
would stop writing the note and the gap would become invisible again, which is strictly *worse*
than an unread note. So a waiver stays entirely **legal**. It just has to name an owner — one
token — and the inert note becomes tracked work.

## Two boundaries the implementation had to learn, both found by USING it

1. ⭐ **SCOPE ≠ CAPABILITY.** The first draft triggered on any *"the gate does not apply"*, and the
   corpus sweep immediately fired on a dozen **honest scope statements** — *"this slice is
   pure-docs, so the code-change gate does not apply"*. Those are correct; the gate is behaving as
   designed and there is nothing to fix. The signal worth routing is narrower: a claim that the
   gate **does** apply but its **signature surface** cannot express the author's evidence. Only
   that is bound.
2. ⭐ **Markdown wraps, so same-line discharge is unsatisfiable.** Requiring the owner citation on
   the identical physical line failed on every wrapped paragraph — including the two real waivers
   being retro-routed. A strict rule there would have pushed authors toward *deleting* the waiver
   to pass, the exact outcome the doctrine exists to prevent. Discharge is therefore checked in a
   small window (±6 lines) — the same paragraph a human reads as one thought — kept small so a
   leaf id elsewhere in the file cannot vacuously discharge it.

## Verification

`docs/tasks/artifacts/generated_lint_correctness/run_waiver_routing_probes.sh` — **8/8** (3 RED /
2 GREEN / 3 CONTROL), including the **verbatim RGX-0090 text** as a RED arm and a pre-existing
untouched waiver as a CONTROL proving the historical record is never retro-bound. The driver also
sweeps the tracked corpus and **sources `WAIVER_RE`/`OWNER_RE` live from the enforcer**, so the
sweep can never measure a different rule than the gate applies.

Corpus at adoption: 6 waiver-shaped lines, **4 routed, 2 residual** (both meta-prose *quoting* the
founding case rather than making a claim).

## Retro-routing, marked as such

`RGX-0090` and `RGX-0091` were routed **in place**, annotated as **CLOSED by
`GENERATED-LINT-CORRECTNESS.4`**, with the retro-fit explicitly labelled — the
`LEX-ADJACENCY.1` precedent, so the failure stays visible instead of being back-dated.

**Honest limit:** this verifies an owner was *named*, not that the owner is real or that the work
happens — the same bound `DESIGN-PRIOR-ART` carries. Ratification pending.

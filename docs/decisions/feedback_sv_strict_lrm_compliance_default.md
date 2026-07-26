---
name: feedback_sv_strict_lrm_compliance_default
description: "STANDING POLICY, REAFFIRMED NON-NEGOTIABLE (director 2026-07-26 session #208) — the SV parser works toward 100% IEEE LRM compliance and strictness BY DEFAULT: no exception, no compromise, non-negotiable. Over-acceptance is a defect to fix, never a feature to keep. Dialect tolerance may be ADDED later as an opt-in layer if a real need appears, but it is never a reason to relax the default or to leave an over-acceptance unfixed."
metadata:
  node_type: memory
  type: feedback
---

## ⭐ REAFFIRMATION (director 2026-07-26, session #208) — this is the operative wording

Director, verbatim: *"PGEN SV needs to be 100% compliant to the LRM by default. We
can add dialect-tolerance later if need be, but we need to work towards LRM full
compliance and strictness, **no exception, no compromise and non-negotiable** — but
still allowing dialect-tolerance, if need be at a later time in the future."*

Two things this settles, both of which the earlier wording left softer than the
director intended:

1. **Strictness is NOT contingent on the accepts-invalid triage.** The earlier
   "RE-OPEN TRIGGER" framing below read as *"find a construct real designs depend
   on ⇒ reconsider strictness."* That is **not** the ruling. The ruling is: fix it
   strictly regardless; a bucket-(a) row is evidence for **adding an opt-in
   tolerance layer later**, never for leaving the grammar non-compliant.
2. **Dialect tolerance is ADDITIVE and FUTURE.** It is a possible future *opt-in*
   on top of a compliant default — never a fork of it, never a reason to defer a
   fix.

⛔ **Provenance correction the director asked to have on record.** The director's
earlier openness to tolerance (*"if other simulators accept it, why wouldn't
PGEN"*) was **caused by an unverified claim of mine** — "mainstream simulators
accept `10 ns`" — asserted from general knowledge in session #206 and never
measured. It was measured shortly after and is **false in the way that matters**
(see the numbers below: 0 spaced vs 273 tight across 16,336 real-world files). The
retraction was recorded, but the phrase leaked onward into `MEMORY.md` and was
repeated back to the director in session #208 as if it were *their* position. **A
claim about what external tools accept is a measurement, not a recollection** — do
not state one without a run behind it, and purge a retracted claim from every
layer, not just the one where it was retracted.

## Historical framing (session #206) — superseded in emphasis by the reaffirmation above

**Director decision (2026-07-25, session #206), stated after reviewing the `.3.11`
measurement:** *"So, we will stick to strict-LRM compliance then, good I prefer
that."* — following their earlier lean *"I would incline to stick to the LRM"* and
their conditional interest in a switch (*"I want the SV parser to be flexible …
so I would go for a switch"*), which the measurement then made unnecessary for the
case at hand.

## The policy

1. **The IEEE LRM is the acceptance authority for the SV parser.** Where PGEN
   accepts text the standard forbids, that is a DEFECT to be fixed, not a
   tolerated convenience — consistent with the project's sign-off-grade north star
   ("make PGEN sign-off-grade when parsing correctness materially affects
   downstream flows") and with [[feedback_correctness_before_speed]].
2. **Annex A footnotes are normative and in scope**, not just the BNF. See
   [[project_lrm_extractor_sota_and_guardrails]]; footnote 48
   (`unbased_unsized_literal`) and footnote 44 (`time_literal`) are the worked
   examples that established this.
3. **The dialect-tolerance switch is DEFERRED, not rejected — and ADDITIVE when it
   comes.** Building it is a future option layered on top of a compliant default;
   it is never a precondition for fixing an over-acceptance, and never a fork of
   the default. (Reaffirmed #208.)

## ⚠️ THE SCOPE LIMIT — what is measured vs what is assumed

⚠️ **Read this as a statement about COST, not about whether to comply.** Per the
#208 reaffirmation, compliance is not conditional on what this triage finds; the
triage tells us how much *care* a given tightening needs, not whether to do it.

The decision was taken on the back of ONE measured case (`SV-CORPUS-GRAD.3.11`,
`time_literal` / footnote 44), where strictness was proven free: **0 of 16,336
corpus files** write a spaced time literal against **273** that write it tight, and
the 4 apparent hits were all false positives (`#1 ps[idx]`, `#2 s = ~s`, `##1 s` —
delays followed by *signals* named `ps`/`s`).

**That result does NOT generalize on its own, and it must not be cited as if it
did.** The engineer explicitly flagged this to the director rather than letting the
n=1 result harden into an unexamined blanket rule — the same over-generalization
that, in the OPPOSITE direction, produced the false "mainstream simulators accept
`10 ns`" claim this decision supersedes.

**What is actually unmeasured:** the **35 accepts-invalid rows** (21 `sv_2017`
lane + 14 `verilog_2005` lane) are, by construction, every place PGEN currently
accepts what the standard forbids. They have never been triaged into:

- **(a) genuine dialect tolerance** — constructs the ecosystem really relies on,
  where strictness would break currently-PASSING corpus files; versus
- **(b) plain over-acceptance bugs** — like `.3.11`, where strictness is free.

Until that triage runs, the honest statement is *"strict-LRM is the default and
every known case so far is bucket (b)"*, not *"strict-LRM costs nothing."*

## Operating rules that follow

- **Fix over-acceptance strictly by default.** No switch, no flag, no exception
  branch — just make the grammar match the LRM.
- **⛔ A tightening fix carries a regression risk that a widening fix does not.**
  Every prior `SV-CORPUS-GRAD.3.x` leaf WIDENED acceptance, which is structurally
  incapable of turning a passing file into a failing one. A strict/tightening fix
  CAN. So the per-FILE pass-set diff (the `.3.4` LAW) is not a formality on these
  leaves — it is the primary safety instrument, and any `pass -> fail` must halt
  the leaf and be routed to bucket (a) rather than argued away.
- **THE TRIGGER FOR DESIGNING THE (additive) SWITCH:** if the accepts-invalid
  triage finds ANY bucket-(a) row — a construct real designs depend on that the LRM
  forbids — that row motivates designing the opt-in tolerance layer, with that row
  as its evidence. One real case is worth more than the whole hypothetical design.
  ⛔ **It does NOT pause or weaken the strict fix** (#208): the grammar is brought
  into compliance either way, and the tolerance layer — if built — sits on top as
  an explicit opt-in. "We found a bucket-(a) row" is never a reason to ship a
  non-compliant default.

## If/when the switch IS built — constraints banked up front

Recorded so a future session does not have to rediscover them (from the same
session's review):

- **It MUST be EBNF-native.** `EBNF-SOURCE-OF-TRUTH` is a *mechanically enforced*
  doctrine (`scripts/check_doctrines.sh`: "no new out-of-band acceptance validator
  wired outside the EBNF") — a runtime parser flag would fail the pre-commit hook.
  The existing `@profiles` annotation proves an EBNF-native switch is achievable.
- **It MUST be ORTHOGONAL to `@profiles`, not folded into it.** Profiles answer
  *which standard* (`sv_2017` / `sv_2023` / `verilog_2005`); strictness answers
  *how pedantically to enforce it*. Folding the second into the first yields a
  combinatorial explosion (`sv_2017_strict`, `sv_2017_lax`, … ×3) and would muddle
  a mechanism that is currently clean, gate-verified (`profile_orphans=0`) and
  load-bearing for two cert contracts.
- **Design it against the measured 35 rows**, never against a single speculative
  case.

Owner if re-opened: `LRM-GRAMMAR-FIDELITY` (cross-family fidelity infrastructure),
not `SV-CORPUS-GRAD` (corpus graduation).

See also: [[feedback_correctness_before_speed]],
[[feedback_corpus_expected_from_spec_not_fix]],
[[project_lrm_extractor_sota_and_guardrails]],
[[project_sv_done_requires_external_corpus_graduation]].

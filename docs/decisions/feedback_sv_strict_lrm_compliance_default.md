---
name: feedback_sv_strict_lrm_compliance_default
description: STANDING POLICY — the SV parser is STRICT-LRM by default; over-acceptance is a defect to fix, not a feature to keep. A dialect-tolerance switch is DEFERRED until the measured accepts-invalid population proves a real need.
metadata:
  node_type: memory
  type: feedback
---

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
3. **The dialect-tolerance switch is DEFERRED, not rejected.** It is re-openable
   the moment evidence demands it (see the trigger below).

## ⚠️ THE SCOPE LIMIT — this policy is a DEFAULT, not a proven-free blanket

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
- **THE RE-OPEN TRIGGER for the switch:** if the accepts-invalid triage finds ANY
  bucket-(a) row — a construct real designs depend on that the LRM forbids — the
  strictness-axis design is re-opened immediately, with that row as its motivating
  evidence. One real case is worth more than the whole hypothetical design.

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

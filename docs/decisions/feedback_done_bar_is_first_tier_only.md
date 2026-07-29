---
name: feedback_done_bar_is_first_tier_only
description: "STANDING DIRECTIVE, NON-NEGOTIABLE (director 2026-07-29, session #221) — `Done` for a parser family means FIRST-TIER done and nothing less: proven by the stimuli generators' own samples, by ALL our gates, and by ALL the external test corpus. No compromises, no exceptions, no 2nd- or 3rd-tier Done. SHARPENED SAME SESSION: `Done` means SOTA/signoff-level — a user can BLINDLY TRUST the parser without re-verifying it. The three legs are the mechanism; blind-trustworthiness is the arbiter and can fail on its own. A family that does not meet all of them is not `Done`, whatever the tracker currently says."
metadata:
  node_type: memory
  type: feedback
---

## The directive (director 2026-07-29, session #221, verbatim)

> *"I need the highest bar possible for all parsers. do not want compromises, exceptions.
> The Done bar have to be a DONE bar, It shall me really done with our samples from the
> stimuli generators, with all our gates and with all the external test corpus. I do not
> 2nd or 3rd tier Done. I want only 1st tier Done."*

## ⭐⭐ THE SHARPENING (director 2026-07-29, same session, verbatim) — this is the OPERATIVE definition

> *"A Done shall means sota, signoff level Done. it shall me user can now blindly trust the
> parser. Done that should means highly professional, very high level quality, sota, signoff
> level quality."*

⇒ **`Done` is defined by what a downstream consumer can rely on WITHOUT verifying it themselves.**
The three legs below are the *mechanism*; **blind trustworthiness is the arbiter.** This ordering
matters, and it closes a loophole the three-leg list would otherwise have left open:

- ⛔ **Passing today's three legs does NOT by itself confer `Done`** while the consumer-facing gates
  are unbuilt — a known open defect, a stale published contract, an undocumented boundary or a
  silent-failure mode all leave a green parser a consumer would be burned by. Those checks are
  **gates to write** (`DONE-BAR.5`), not a separate tier of judgement; until they exist, a row
  clearing the three legs is **`PROVISIONAL`**, never `Done`.
- ⭐ **The test to apply to any candidate row:** *would I tell a downstream team to build on this
  parser without re-verifying it, and would I be right?* If the honest answer is no, it is not
  `Done`, whatever the gates say.
- ⭐ **"Signoff level" is the repo's existing quality word**, already binding on code
  (`CLAUDE.md` §3). This directive extends it to **parser closure claims**: same standard, same
  refusal to accept "good enough".

## ⭐⭐⭐ AND THE FLOW SHALL GUARANTEE IT 100% (director, same session, verbatim)

> *"So the flow shall guarantee this 100%"*

⇒ the bar is an **invariant the flow enforces**, not a checklist someone remembers. A `Done` row
that does not meet the legs must be impossible to hold, because a gate fails while it is held.
⛔ **The flow cannot deliver that today, measured:** `sota_exit_gate` has never completed green, the
AUTOMATIC tier over all 123 gate targets is **zero**, and the family status gates are reachable only
through that aggregate — which is precisely how `regex` held `Done` against a gate that disagreed.
⇒ the guarantee is blocked on `CI-PARITY-GATE-ROT.7` and on the escalated hosted-auto-trigger call.
⚠️ Honest bound on "100%": a pre-commit hook is bypassable and a local run proves one machine, so the
strongest honest form is *enforced on every commit through the hook AND re-proved by an automatic
lane no contributor controls* — anything less is stated, never quietly claimed.

## What triggered it

`CI-PARITY-GATE-ROT.13` found the `regex` family asserting `Done` while its own
`regex_parser_family_status_gate` computed `In Progress` — measured `final_targets = 31`
against a criterion of `0`. Reporting that, I observed that `regex` earned `Done` against
**355** targets and the universe is now **1033**, and asked whether closure should be
re-earned when the target universe grows. The director's answer is broader than the question:
the bar itself is being raised and fixed.

## The legs — ALL of them, for every family

`Done` requires **all of these**, simultaneously and currently:

1. **Stimuli-generator proof** — the family's own generated samples close the loop. Residual
   actionable-target debt is not `Done`. (This is the leg `regex` currently fails.)
2. **All our gates** — every gate that covers the family is green *now*, not "was green when
   the claim was made". A gate nothing runs does not count as green
   ([[project_gate_reachability_is_a_doctrine]] territory: *a check nothing invokes is
   indistinguishable from one that does not exist*).
3. **All the external test corpus** — an officially-recognized third-party corpus, passing.
   This is the leg the repo is furthest from, and the README already states the doctrine
   (*"every parser proven by BOTH the stimuli generator AND an officially-recognized external
   corpus"*) without it being enforced.

⭐ **The "blind trust" purpose is NOT a fourth leg** — that framing was challenged by the director
(*"I don't really understand (4). Why do we need it?"*) and it was wrong: it stated a GOAL, not a
criterion, and an unmeasurable leg cannot be guaranteed by a flow. What it pointed at is real and
measured (silent-success sentinel paths that return `Ok` with zero diagnostics; a user guide
publishing a ~75-release-stale version pair; open ledger entries; undocumented boundaries — and
**no gate covers any of it**). Those are concrete consumer-facing checks, so they are **written as
gates and absorbed into leg 2**. The bar stays THREE legs, all measurable, all gateable.

## What this SETTLES that was previously soft

- ⛔ **`Done` is not a snapshot.** It is a claim about the tree as it stands. A family whose
  grammar, target universe, or corpus has grown since the claim must re-earn it. "It was true
  in March" is not a defence.
- ⛔ **A TRIAGE gate is not a conformance gate.** Measured at directive time: of the 6
  corpus-facing gate targets, `sv_external_corpus_triage_gate` and
  `vhdl_external_corpus_triage_gate` are triage — they classify findings; they do not assert
  conformance. Triage does not satisfy leg 3.
- ⛔ **A characterization is not a corpus pass.** `json_corpus_bundle/results/characterization.md`
  is explicitly *"a characterization, not a conformance gate, because `json.ebnf` is a
  deliberately simplified subset"*. That is honest, and it is also not leg 3.
- ⛔ **Absence of a corpus is not satisfaction of leg 3.** A family with no external corpus has
  an *unmet* leg, not an *inapplicable* one. If a family genuinely cannot have one, that needs a
  recorded, director-visible justification — not silence.
- ⭐ **Demotion is the correct action, not a failure.** When a family stops meeting the bar, the
  tracker row moves. Leaving a contested `Done` in place to avoid a red row is exactly the
  2nd-tier Done the director is refusing.

## How it is applied

- Every `Done` row in `LIVE_ACHIEVEMENT_STATUS.md`'s parser-family tables is audited against the
  four legs, and the audit is re-runnable rather than one-shot — a one-shot census is the same
  disease one level up ([[feedback_instrument_needs_ground_truth]] and
  `DOCTRINE-GAP-OWNERSHIP.1`'s ratchet lesson).
- Owned by the `DONE-BAR` task tree.
- ⚠️ The expected near-term outcome is that **several current `Done` rows do not survive**. That
  is the directive working, not the audit malfunctioning. The claim to protect is the bar, not
  the row count.

## Related

[[project_regex_pcre2_faithful_by_default_relaxed_optout]] · [[feedback_sv_strict_lrm_compliance_default]]
(the same "no exception, no compromise" posture, applied to SV acceptance) ·
[[feedback_instrument_needs_ground_truth]]

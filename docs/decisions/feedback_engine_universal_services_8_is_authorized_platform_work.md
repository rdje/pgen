# `ENGINE-UNIVERSAL-SERVICES.8` is AUTHORIZED platform work — the SV lane lock does not park it

**Category:** feedback · **Established:** 2026-08-11 (director, session #215) · scopes
[[project_nexsim_sv_signoff_delivery_focus]]'s lane lock and extends
[[feedback_capability_work_is_greenlit_by_standing_authorization]].

## The directive (director, verbatim)

Given the surfaced caution that *"`.8` is bigger than it looks … it's a platform-wide change, not an
SV leaf, and it will pull the lane lock's edges"*:

> *"Go ahead with [it] … after the /exit."*

## 1. What this authorizes

`ENGINE-UNIVERSAL-SERVICES.8` — fold the LR-elimination `_pgen_lr_chain` blob into the **declared**
annotation shape, plus the missing **annotation-vs-emitted-AST conformance oracle** — is the
**next work unit**, and it is authorized *as platform work* even though it is not an SV leaf.

⛔ **This is the point of the record.** The standing SV lane lock says do not leave SV until it ships
to Nexsim, and a session resuming from `MEMORY.md` would otherwise read `.8`'s platform scope as
out-of-lane, park it, and return to the SV burn-down. That would be **wrong**: `.8` blocks
`GRAMMAR-WELLFORMED.A2.5`, which blocks `SV-CORPUS-GRAD.13c.2a.2`/`.3`/`.4`. It is the lane lock's
own named exception — *a defect that BLOCKS the SV release*.

## 2. What it does NOT authorize

The **schema consequences** are not pre-approved, because they were not knowable when the
authorization was given, and the director was told so:

- Folding the chain changes the emitted AST of **every LR-eliminated rule in every grammar**,
  including `return_annotation`, which is `fully_certified` today and has been emitting the raw blob
  at 10 sites since it shipped.
- ⇒ this is likely a schema move for **more than one family**, each with its own contract, ledger
  row and consumer migration.

⭐ So: **implement and measure freely; publish a multi-family schema move only after saying what it
costs.** The measurement is execution ([[feedback_sequence_answer_your_own_technical_questions]]
applies — how to fold is a technical question, answer it); the decision to break more than one
family's contract at once is a scope call and gets surfaced with the measured blast radius, not a
menu of options.

## 3. Why the fold is not optional

PGEN's second doctrine is *"every generated parser returns an AST; **return annotations shape that
AST**"* (`README.md`). An LR-eliminated rule currently violates it: the consumer receives the
eliminator's internal `{initial, suffixes, type: "_pgen_lr_chain", wrapper_specs}` instead of the
shape the grammar author declared. That is the engine leaking an implementation detail through a
published contract, and it is exactly the class of thing
[[a-directly-left-recursive-alternative-inside-a-choice-is-dead-code]] says the engine must absorb
so no EBNF author has to care.

## 4. The oracle is the more valuable half

⛔ No gate caught this, and the reason generalizes: the differential suites assert the interpreter
and the generated parser are byte-identical **to each other**. Both fold identically — both leak
identically — so agreement is total and every suite is green. **Two implementations agreeing is not
evidence either one is right.** Nothing in the gate set compares the emitted AST against the
**declared** annotation. `ast_shape_contract_gate` does not close it either: measured 18/18 green
with the fix prototyped, because its pins do not reach the changed rule — a green gate that does not
cover the change is not evidence of safety.

⇒ ship the conformance oracle **with** the fold, not after it.

## How to apply

- Resume at `ENGINE-UNIVERSAL-SERVICES.8`; the blocked fix is preserved and re-appliable at
  `docs/tasks/artifacts/engine_universal_services/A2.5_direct_lr_normalization.patch`
  (verified `git apply --check` clean against `474f40df`).
- Land `.8` first, then the patch: that ordering makes the SV schema move **additive** (three
  previously-unreachable kinds appear) rather than a shape replacement.
- ⚠️ `generated/` and the release probe are stale from the prototype build — regenerate before
  trusting any local parse.

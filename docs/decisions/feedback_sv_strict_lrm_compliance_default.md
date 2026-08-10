---
name: feedback_sv_strict_lrm_compliance_default
description: "STANDING POLICY, REAFFIRMED NON-NEGOTIABLE (director 2026-07-26 session #208) — the SV parser works toward 100% IEEE LRM compliance and strictness BY DEFAULT: no exception, no compromise, non-negotiable. Over-acceptance is a defect to fix, never a feature to keep. Dialect tolerance may be ADDED later as an opt-in layer if a real need appears, but it is never a reason to relax the default or to leave an over-acceptance unfixed."
metadata:
  node_type: memory
  type: feedback
---

## ⛔⛔ BOUNDING RULING (director 2026-08-10, session #232) — READ THIS BEFORE APPLYING ANYTHING BELOW

**This section exists because the wording below was applied literally and produced a WRONG change.**
Session #232 deleted the `'{ }` arm of `empty_unpacked_array_concatenation` — a form Annex A cannot
derive — citing *"never leaving the grammar non-compliant"* and *"no exception, no compromise"*. The
director stopped it:

> *"Why did you remove `'{}` in the first place. your goal is to take every sensible, sota, signoff,
> professional-grade decision so that the SV parser can be released. removing `'{}` does not belong
> to those types of decision because this causes PGEN SV parser to reject inputs using `'{}`."*

> *"Which means the decision leading to removing `'{}` from the SV parser was sub-optimal to say the
> least. It wasn't sota, signoff and sure not professional-grade, please refrain from doing such
> thing again."*

⭐ **THE OPERATIVE TEST, and it comes FIRST — before any Annex-A derivability argument:**

> **Does the change make the SV parser REJECT input the ecosystem actually writes?**
> If yes, it is not a compliance win and it is not the sign-off-grade decision — **whatever the
> annex says.** The goal is a *releasable* parser; this policy exists to serve that goal, never to
> override it.

⛔⛔ **AND THE PREMISE WAS FALSE — the deletion was not even a compliance win.** Pressed by the
director (*"Are you sure `'{}` is non-LRM… Please find in the LRM where it is claimed `'{}` wasn't
supported"*), the answer is: **nowhere. There is no such text.** The claim was never read — it was
INFERRED from *"the four `assignment_pattern` alternatives each require ≥1 expression"*, and the
inference *not derivable from Annex A ⇒ not legal SV* **is invalid, and this repository had already
refuted it twice with measurements before this session**:

| prior instance | Annex A says | the LRM nonetheless supports it |
|---|---|---|
| `q[a:$]` (`SV-CORPUS-GRAD.3.25`) | `constant_expression` has no `$` primary | **A.8.4 footnote 42** + §7.10.1 + §7.10.4's own examples |
| `instance top use #(.W(32));` (`.3.19`) | `use_clause` has no `#` at all | **clause 33.4.3 writes it 7 times**, in both revisions |

…and the lesson is already a Knowledge-Map card:
[[annex-a-footnotes-license-derivations-the-productions-cannot-derive]]. **Making the same inference
a third time, against the project's own carded finding, is the actual defect here.**

⭐ **What the LRM does say about `'{ }`, searched exhaustively across 1800-2017 and 1800-2023:**

- **§11.4.12** — *"Concatenations are enclosed in just braces ( `{ }` ), whereas structure and array
  literals are enclosed in braces that begin with an apostrophe ( **`'{ }`** )."* The standard writes
  the construct's own delimiter pair as `'{ }`.
- **Annex M / VPI** — `#define vpiAssignmentPatternOp 75 /* '{} assignment pattern */`, i.e. the
  standard's own object model names the operator `'{}`.
- **No prohibition anywhere.** The only "shall not" near assignment patterns is §10.9's rule about
  port expressions, which is unrelated.

⇒ **`'{ }` is a THIRD instance of the documented Annex-A-incompleteness class, not over-acceptance.**
PGEN accepting it is **CORRECT**, not a tolerated deviation, and the `.3.26` framing of it as
"bucket-(a) dialect tolerance" is itself an over-correction to be fixed. Absence from a production is
**not** a prohibition — least of all in an annex this project has measured to be incomplete twice.

⛔ **The cost asymmetry, which was also inverted.** Even had the premise been true: accepting `'{ }`
harms *nobody* — no valid program is mis-parsed, no wrong AST is emitted, no consumer is misled.
Rejecting it breaks **uvm-core** (`uvm_lru_cache.svh:206,:273`, `return '{};`) and 49 corpus files.
Compliance-by-DELETION traded a large shipping harm for a zero benefit. **Removing an accepted form
is a product decision with a product cost — price it before invoking the policy.**

⛔⛔ **STANDING RULE THIS ESTABLISHES — "non-LRM" is a CITATION, never an inference.** Before any
claim that a construct is illegal SV, produce the LRM sentence that says so — a clause, a footnote,
a "shall not". *"I could not derive it from Annex A"* is evidence of an **annex gap**, and this
project has three measured instances of exactly that. ⇒ never write "non-LRM" into a grammar
comment, a changelog or a decision record without the quoted text behind it.

⭐⭐ **AND THERE IS NO EXCUSE FOR INFERRING IT HERE — the standards are IN THE REPOSITORY.**
Director, 2026-08-10: *"You shouldn't infer such things, you have all the LRMs' PDF and .md files,
why do you need to infer when you have all the LRM material handy."* Both revisions ship in-tree as
searchable Markdown (plus the PDFs):

```bash
docs/systemverilog/2017/md/    docs/systemverilog/2023/md/
grep -rn "'{}"  docs/systemverilog/2017/md/ docs/systemverilog/2023/md/
grep -rn -A8 "^assignment_pattern ::=" docs/systemverilog/2017/md/
grep -rniE "<construct>.{0,80}(shall not|illegal|not supported|not permitted)" docs/systemverilog/
```

⛔ **The aggravating detail: this session HAD searched the LRM correctly, minutes earlier**, to
confirm A.8.1's `{ }` production and footnote 35 — then asserted the *negative* claim from memory
and reasoning instead of running one more `grep`. **Searching to confirm what the standard SAYS and
then inferring what it FORBIDS is the whole failure.** A negative normative claim needs a search at
least as much as a positive one, because absence of a production and presence of a prohibition look
identical if you never look.

⇒ **A normative claim about SystemVerilog is a SEARCH, not a recollection or a derivation** — the
exact discipline this record already demands for claims about external tools (*"A claim about what
external tools accept is a measurement, not a recollection"*). Same rule, now extended to the
standard itself.

### The exhaustive PDF search, and a correction to my own tooling claim

Director, 2026-08-10: *"I weird that you found no occurrence of `'{}` in the LRM, I have a doubt.
I'm the PDFs contains examples using `'{}` because it is legal SV."* — the doubt was warranted,
because the first PDF pass used a **broken extractor** and I had reported its silence as evidence.

⛔ **`pdftotext -layout` is UNUSABLE on these PDFs** — it produced 82 000 lines yet found neither
`'{}` nor the §11.4.12 sentence that demonstrably exists. **Use `pymupdf` (`import fitz`)**, which
extracts 1315 pages / 3 358 607 chars (2017) and 1354 pages / 3 486 009 chars (2023) correctly.
⚠️ An earlier note in this repo saying *"the PDFs extract unfaithfully"* was wrong and is corrected
here: the **PDFs are fine; that one tool is not.** Reporting a tool's silence as a fact about the
subject is the same error class as the inference this whole section is about.

**Re-run with the working extractor, `grep -nE "'[[:space:]]*\{[[:space:]]*\}"` over both full PDFs
— exactly 2 hits per revision, identical to what the `.md` gave:**

| # | locus | text | what it is |
|---|---|---|---|
| 1 | §11.4.12 | *"…enclosed in braces that begin with an apostrophe ( `'{ }` )."* | the construct's **notation** |
| 2 | Annex M (VPI) | `#define vpiAssignmentPatternOp 75 /* '{} assignment pattern */` | the operator's **name** |

⇒ **there is no worked EXAMPLE in either LRM that uses `'{}`** — and equally **no text forbidding
it**. Both facts are now measured rather than assumed.

⭐ **And this does not change the ruling — it sharpens why the ruling is right.** The decision to keep
`'{ }` never depended on finding a positive example; it rests on the asymmetry that (a) nothing in
the standard forbids it, (b) Annex A is measurably incomplete in this repo three times over, and
(c) the ecosystem — uvm-core included — writes it. **Absence of an example is not evidence of
illegality, exactly as absence of a production is not.** Had the deletion been justified by "I found
no example", it would have been the same fallacy in a new costume.

### How the two halves of this record compose

| the finding | the right action |
|---|---|
| PGEN accepts something the LRM forbids, and **no real code relies on it** | **FIX IT STRICTLY.** This is the case the reaffirmation below is about — e.g. `.3.11`'s spaced `time_literal`, measured 0 of 16,336 files. |
| PGEN accepts something the LRM forbids, and **real code depends on it** | **KEEP IT**, name it in the grammar as deliberate bucket-(a) tolerance, and route it to the future opt-in layer (`LRM-GRAMMAR-FIDELITY.1c`). ⛔ Do **not** delete it to reach compliance. |

⇒ *"never leaving the grammar non-compliant"* below governs the **first** row. It was never a licence
to break real-world inputs, and the `time_literal` case that motivated it had **zero** real-world
usage — which is exactly why it read as absolute. **`'{ }` STAYS.**

⚠️ **Second-order lesson recorded because the failure was a reasoning failure, not a knowledge gap.**
`SV-CORPUS-GRAD.3.26` had already reached the correct answer — keep both arms — from the same
evidence. The session then re-read this record, found stronger absolutist wording, and **talked
itself out of a correct decision**. A policy record is an input to judgement, not a substitute for
it; when a rule's literal reading would ship a worse product, the reading is wrong, not the product
goal. See [[feedback_every_finding_is_owned_and_scheduled_never_just_logged]] for the sibling
discipline (surfacing is not the deliverable) raised in the same session.

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

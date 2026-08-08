# Derived-State Containment — a field another system already owns exactly must not be hand-maintained

A small, project-neutral, harness-neutral standard for a failure mode that both the Durable
Agent Memory Architecture and the Live-Document Size-Containment Doctrine *name in prose and
leave unenforced*: **hand-written copies of facts a tool can compute exactly.**

> One-line thesis: **a stored fact that a tool owns exactly is not information — it is a cache
> with no invalidation.** Either derive it at read time, or pin it with a verifier that fails
> when it drifts. A third option — store it and trust discipline — is the defect.

It is deliberately narrow. It governs one property (is this field's value knowable from a
canonical system?) of one class of content (fields inside maintained documents). It is designed
to be dropped into any repository that already has a commit-time or CI check runner.

---

## 1. Why this needs its own contract

Two existing doctrines already *recommend* the behaviour, and neither *checks* it:

| Source | What it says | What it enforces |
|---|---|---|
| Memory architecture, §6 | "**Prefer derived over hand-written** — a small script can regenerate the current-state block … so it cannot drift." | nothing |
| Memory architecture, §12 | anti-pattern: "**Hand-maintained current-state that drifts from reality (prefer derived).**" | nothing |
| Size-containment doctrine | "**Size checks prove boundedness, not semantic truth.** A surface claiming currency needs a separate, lifecycle-specific verifier calibrated against its canonical source." | defers to that verifier |

The size doctrine is explicit that boundedness and truth are different properties, and that the
truth half needs its own instrument. This standard **is** that instrument for the one class where
truth is decidable mechanically and cheaply: fields whose canonical source is a program.

It is not a competitor to either doctrine. It is the missing verifier both point at.

---

## 2. The three field kinds

Classification is the whole standard. Only one kind is forbidden.

**(a) Derivable-exact — FORBIDDEN as a hand-maintained field.**
A canonical system can compute the value exactly, on demand, cheaply. Examples: the current
commit hash; the number of commits ahead of a remote; a file's line or byte count; a test count;
"files changed since X"; a directory listing; anything a one-line command answers.
⇒ **Delete the field. Store the command instead.** A reader who needs the value runs it and gets
a value that is true *at the moment they need it*, which is the only moment that matters.

**(b) Derivable-but-pinned — ALLOWED, and only with a verifier.**
The value is derivable, but a copy is deliberately kept because something depends on the copy
itself — an anchor another check compares against, a published contract, a frozen baseline.
⇒ **Allowed if and only if a check fails when the copy and its source disagree.** A pinned
duplicate with a verifier is a ratchet. A pinned duplicate without one is shadow state wearing a
ratchet's clothes.

**(c) Non-derivable — ALLOWED, unconstrained by this standard.**
No system owns the value: a judgement, an intent, a next action, a rationale, a blocker's
ownership, a decision. These are exactly what a bounded pointer *should* hold.

⭐ **The classification is the deliverable.** Most arguments about "should this field be here?"
dissolve once the field is placed in (a), (b) or (c) — and the placement is a question of fact,
not taste.

---

## 3. The sharpest sub-class: fields that are wrong *by construction*

A subset of (a) is worse than drift-prone. It is **unsatisfiable**: the act of recording the
value changes the value.

Any field whose source is *"the state of the repository including this document"* has this
shape. The canonical instance is a commit-distance counter stored in a tracked file:

```
record "N commits ahead"  ->  requires a commit  ->  the repository is now N+1 ahead
```

The field is stale the instant the commit that writes it lands. No discipline, review, cadence
or reminder can fix it, because the failure is not a lapse — it is arithmetic. The same shape
covers "total commits", "files in this repo", "lines of code", and "last updated" when written
by hand into the file being updated.

⛔ **Test for this class:** *does writing the value invalidate it?* If yes, no policy short of
deletion is coherent, and any effort spent maintaining it is definitionally wasted.

### Worked evidence (evidence about one project, not portable policy)

The repository that authored this standard carried a commit-distance field in its bounded
layer-A resume pointer, annotated in the file itself with a warning to re-derive rather than
increment it, *because it had already drifted once*:

- measured: the field read **185**; `git rev-list --count origin/main..HEAD` said **187**;
- the file had been corrected by **four commits whose subject line is that counter**, each of
  which invalidated its own result on landing;
- **12 of the last 400 commits** touched only the resume pointer;
- the pointer sat at **98.7 % of its byte cap** with **88 bytes** of headroom, so the
  maintenance tax was being paid *and* the space was being consumed.

A field carrying its own "do not trust this" warning is a field that has already failed review.
The warning is the bug report.

---

## 4. The rule (normative)

> **R1.** A maintained document MUST NOT contain a derivable-exact field as hand-written content.
>
> **R2.** A derivable-but-pinned value MUST be covered by a check that fails when the pinned copy
> and its canonical source disagree. The check must be executed by the unconditional gate, not
> merely declared.
>
> **R3.** A document that needs a derivable value MUST carry the *derivation* — the exact command
> or accessor — instead of the value.
>
> **R4.** A field that is invalidated by the act of writing it MUST be deleted, never scheduled
> for periodic correction.
>
> **R5.** Removing a field under R1/R4 MUST NOT remove the reader's ability to answer the
> question. The derivation replaces the value in the same place, so the resume path is unchanged.

R5 is what keeps this from being a deletion campaign. The reader still gets the answer; they get
a correct one.

---

## 5. The check contract

An implementation is conforming if it fails on all of:

1. a governed document containing a pattern matching a declared derivable-exact field;
2. a declared pinned value with no registered verifier;
3. a registered verifier that the gate did not actually execute;
4. a pinned value that disagrees with its canonical source.

And if it satisfies the instrument requirements:

- **Ground truth, or it does not run.** The check MUST carry a positive control (a compliant
  fixture it passes) and a negative control (a planted violation it must catch, at exactly the
  expected location) and MUST refuse — nonzero, before publishing a verdict — if either misses.
  An instrument with no ground truth is a confident guess.
- **Declared, not inferred.** The set of derivable-exact patterns is an explicit, reviewable
  list in the repository, not a heuristic. A heuristic that guesses which prose is a commit count
  will fail open, silently, in the passing direction.
- **Scoped.** It governs named documents, not every file. A changelog *should* contain historical
  counts; that is its job. The standard targets surfaces claiming to describe *now*.

⚠️ **Honest bound.** This check proves a field is absent or agrees with its source. It cannot
prove a *non-derivable* field is true — no mechanical check can, which is why (c) is
unconstrained here and governed by review instead.

---

## 6. Adoption

1. **Own it under a task/work unit first**, with the measurement that motivated it. Adopting an
   enforcement rule without a measured local instance produces ceremony.
2. **Inventory the governed surfaces** — the bounded pointer first, then any document that
   describes current state.
3. **Classify every field** into (a), (b), (c). Publish the classification; it is the artifact
   that survives, more than the check.
4. **Delete (a), register a verifier for (b), leave (c).** Replace each deleted field with its
   derivation in place (R5).
5. **Wire the check into the unconditional gate** — commit hook and CI — with its controls.
6. **Re-measure the surface.** The freed space is the point; report it.

---

## 7. Relationship to the two neighbouring doctrines

- **Memory architecture** — supplies the layer model and the bounded pointer. This standard
  enforces its §6 "prefer derived" advice and closes its §12 anti-pattern.
- **Live-document size containment** — supplies lifecycle, ceilings, routes and transition debt.
  This standard supplies the *content-truth* verifier it explicitly defers to, for the decidable
  class. A `bounded_snapshot` in that doctrine's vocabulary is the natural governed surface.
- **Neither is superseded.** Size and truth are independent axes: a document can be small and
  false, or large and true. Enforce both or claim neither.

⭐ **Why this ordering matters in practice.** Derivable-exact fields are usually *also* scaling
terms — they are the ones a project updates most often, so they attract the most surrounding
narration. Removing them relieves size pressure as a side effect, which means this check is a
cheap first move *before* a size-containment migration, not a refinement after one.

---

## 8. Anti-patterns

- ❌ Storing a value and adding a comment telling readers not to trust it.
- ❌ Scheduling periodic correction of a field that writing invalidates.
- ❌ Counting a co-staged edit as proof that a stored value is now true.
- ❌ A pinned duplicate whose "verifier" is a human habit.
- ❌ Declaring a verifier in a registry that the gate never runs.
- ❌ Inferring derivable fields by heuristic instead of a declared list.
- ❌ Deleting the field without leaving the derivation, so the reader loses the answer.
- ❌ Treating this as licence to delete judgement, rationale, or next actions — class (c) is the
  content a bounded pointer exists to carry.

---

## 9. Completion test

The adoption is complete when: every governed surface's fields are classified and published; no
derivable-exact field remains as stored content; every pinned duplicate has an executed verifier
that fails on disagreement; the gate refuses on a planted violation; and a reader resuming from
the surface can still answer every question it previously answered — from a derivation whose
value is correct at the moment they ask.

---

*Forwardable. A receiving project should take an owned copy, replace this paragraph with its own
authority note, and build its own declared pattern list, registry and controls. Do not copy
another project's field lists, measurements or thresholds — those are evidence about the donor,
not policy.*

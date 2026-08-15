# Claim Verification — verify a claim three ways before you publish it

A portable, **project- and harness-agnostic** standard for the moment an agent (or a human) turns a
measurement into a **claim someone else will act on**. Drop it into any repository; it assumes
nothing but a version-control system.

> One-line thesis: **checking a claim twice does not make it twice as verified — a repeated pass
> repeats its own blind spot.** Verification must be *dimensionally different*, not merely more.

This file is the **5th portable architecture** a project adopts, alongside the four it already has:

| # | Portable architecture | Owns | Standard |
|---|---|---|---|
| 1 | **Task-trees** | per-unit work memory (goal/frontier/acceptance/verification) | `docs/TASK_TREE.md` |
| 2 | **Memory-architecture** | durable harness-agnostic agent memory (4 layers) | `MEMORY_ARCHITECTURE.md` |
| 3 | **Knowledge-map** | a retrieval layer over fact cards | `knowledge-map/` |
| 4 | **Doctrine-enforcement** | turning every rule into a mechanically-gated check | `DOCTRINE_ENFORCEMENT.md` |
| 5 | **Claim-verification** | what "checked" means before a number is published | **this file** |

It is the sibling of `DOCTRINE_ENFORCEMENT.md`. That standard asks *"is this rule enforced?"*; this
one asks *"is this **number** earned?"* — the question that comes first, because a gate built on an
unverified measurement enforces the wrong thing precisely and forever.

---

## 0. How to use this file

1. Adopt the **three legs** (§3) as the definition of "checked".
2. Adopt the **publishing contract** (§4): a claim ships with its legs named, and a *missing* leg is
   stated, never hidden.
3. Mechanize what you can (§5). Prose that nothing checks is a suggestion.
4. Run the adoption checklist (§7) once; keep §6 next to your review template.

If you remember one rule: **re-derive · falsify · make durable — three different questions, in that
order.**

---

## 1. The problem: "check it again" does not work

The instruction *"double-check your claims"* is almost always already being followed. Claims that
fail review are rarely unchecked — they are checked by a procedure **structurally incapable of
catching the defect**.

Measured in the reference deployment, within a single session, every corrected claim had already
been checked once, carefully, by its author:

| the claim | it was checked by… | why that check could not fail |
|---|---|---|
| a profiler attributing **18 %** of CPU to a subsystem | a ground-truth control, green | the control asserted **conservation** (`sum(parts) == total`); the bug was a **misassignment**, which conserves the total. Real value **2 %** — an **8×** error |
| "this rule family is **0.68 %** of the workload" | a 10-case control suite that **refuses** on any miss | all 10 cases were drawn from the same **prose** as the classifier they tested, so they could only ever agree with it. Real value **2.74 %** |
| "two independent instruments agree, 75 % vs 76 %" | cross-checking two measurements | the **quantities** were independent; the **classifier was shared** — and the classifier was the defect |
| a corrected constant, freshly re-derived from source | a full re-measurement | correct — then written into a hand-carried constant guarded by a **comment**. Stale-able on day two |

⇒ Four checks, four authors' worth of care, and **zero** chance of catching the respective defect.
Asking for a second pass of the same kind buys nothing. ⛔ This is why restating *"be careful"* does
not work as a remedy, and why what follows is a procedure with **named legs**: so *"did I check?"*
becomes answerable instead of felt.

---

## 2. The taxonomy of checks that cannot fail

Before trusting a check, ask the only question that matters: **what class of bug does this still
permit?**

| the check | catches | still permits |
|---|---|---|
| `sum(parts) == total` | dropped rows, double counts, arithmetic slips | ❌ **any redistribution between parts** |
| `count(rows) == expected` | truncation, a missed input | ❌ a wrong value in every row |
| a hash of the inputs | stale inputs | ❌ every logic bug downstream of them |
| tests written from the spec, over an implementation written from the same spec | typos | ❌ **every misreading of the spec** |
| a ticked checklist box | "I forgot the step" | ❌ the step being done wrong |
| **per-bucket agreement with an independent source** | ✅ misassignment, redistribution, wrong bucketing | genuinely little |

**The general form: a check and the thing it checks must not share a parent.** When the test and the
implementation descend from the same understanding, their agreement carries no information. That is
why *"I wrote tests and they pass"* is strong evidence about a **transcription** error and nearly no
evidence about a **specification** error.

---

## 3. The three legs

### Leg 1 — RE-DERIVE. Does it reproduce, by command, from the source?

Not *"do I remember measuring this"*, and not *"is it written consistently in three places"* —
consistency propagates errors faithfully. Run the command; keep the output.

- Applies to numbers you are **quoting from your own project's documentation**. In the reference
  deployment a live status line had been stale in **two of its three numbers** for days: everyone
  read it, nobody re-ran it.
- A number appearing in *N* places has *N* chances to be stale. Prefer **one derived source** over
  *N* synchronized copies.

### Leg 2 — FALSIFY. What would make this false, and is there an oracle you did not build?

- **Name the competing hypothesis, then find evidence that separates them.** "10 families, 1 symbol"
  is equally consistent with *the linker merged them* and *this family's copy was inlined away*. A
  symbol census cannot separate those; a **call-target address** can. ⛔ If your evidence is
  consistent with both hypotheses, you have not tested — you have illustrated.
- **Prefer an oracle you did not build.** A profiler's own per-symbol table caught an 8× error that
  the author's own control had passed. Look for one already sitting in your inputs; it is free.
- **Make the control go RED on purpose.** A control never observed failing is not known to work. A
  21-case suite became trustworthy only once it was run against the *old, broken* predicate and
  correctly missed **8 of 21**.
- **Derive classifiers, shape lists and membership tests from the PRODUCER** — the code that emits
  the thing — never from a description of the producer. One `grep` over the emitting call sites beat
  a carefully-reasoned pattern built from the design document's own sentence.

### Leg 3 — DURABILITY. Can the reader re-run it, and does anything fail when it goes stale?

⭐ **This is the leg that gets skipped, and it is what separates *signoff-grade* from *true-today*.**

- **Is the producer tracked?** A measured number whose instrument lives in a scratch or ignored
  directory is a *"trust me"* with extra steps. Measured: four analysis instruments and eight
  profile artifacts, `git ls-files` → **0**, leaving published intervals permanently unreproducible.
- **Is the claim watched?** A number nothing re-derives goes stale silently. ⛔ **Replacing a wrong
  unwatched number with a right unwatched number is not a fix.** If re-deriving costs 71 seconds,
  "it was expensive" is not available as a reason.

---

## 4. The publishing contract

State the claim, then the legs that earn it. **When a leg is missing, name it.**

> *"Re-derived and falsified; **not durable** — the instrument is untracked."*

That is a signoff-grade sentence. **A claim with a named gap is usable; a claim with a hidden gap is
the defect.** Two riders:

- **Intervals, not point estimates, for anything stochastic.** A single run per arm produced *"these
  agree within 0.6 points"* — when the within-arm spread alone was **3.1 points**. The agreement was
  luck, and it read as precision.
- ⛔ **The auditor's asymmetry.** When a re-derivation disagrees with a published number, **the
  re-derivation is the newer instrument and carries the heavier burden of proof.** It has been run
  once; the thing it contradicts has at least been read. A reviewer who forgets this writes *"your
  number is wrong"* when the correct sentence is *"one of these two is wrong, and it might be mine."*

---

## 5. Mechanizing it

Prose is discoverable, not enforceable (`DOCTRINE_ENFORCEMENT.md` §1). Two cheap mechanizations, in
increasing strength:

**A — the claim tag.** Any published number carries an inline provenance tag naming its legs:

```
throughput 2.741 %  [rederive: scripts/census.sh --family | falsify: RED-probe 8/21 | durable: NO — see #412]
```

Grep-able, reviewable, and it makes a missing leg **visible** instead of **absent**.

**B — the derived-constant rule.** Any constant that is a function of the repository is **derived or
gated**, never carried. If deriving is too slow to run every time, gate it: hash the inputs it
depends on and fail when they move.

```bash
# the general shape: a cheap identity tier + an expensive re-measure on demand
recorded=$(sed -n 's/^inputs_sha: //p' baseline.md)
live=$(cat $(cat inputs.list) | sha256sum | cut -d' ' -f1)
[ "$recorded" = "$live" ] || { echo "baseline no longer describes this tree — re-measure"; exit 1; }
```

⛔ **Name every input the ARTIFACT depends on, not every input the HEADLINE METRIC depends on.** A
baseline whose identity covered grammar + binary + inputs was still silently staled by a change to
*the instrument that wrote it* — the identity was sound for the headline number and insufficient for
the file.

---

## 6. Anti-patterns

- ❌ "I checked it twice" — twice the same way is once.
- ❌ A control that only ever passes; a control never seen RED.
- ❌ Tests written from the same document as the implementation, cited as evidence about that document.
- ❌ A conservation or checksum control on a tool whose job is **allocation between buckets**.
- ❌ A measured number whose producing script is untracked, ad-hoc, or in a scratch directory.
- ❌ Correcting a stale constant to a fresh constant and changing nothing about *why* it went stale.
- ❌ A comment reading *"do not edit by hand — re-derive it"*. That is prose; write the check.
- ❌ A point estimate for a stochastic quantity.
- ❌ Treating your own re-derivation as automatically authoritative over the thing it contradicts.

---

## 7. Adoption checklist

1. Adopt §3 as the working definition of "checked" and §4 as the reporting format.
2. **Sweep published constants**: for each, can it be re-derived by one command? If not, derive it or
   gate it (§5B).
3. **Sweep for untracked producers**: run your VCS's ignored-file listing over scratch/output paths.
   Anything there that produced a *published* number is a leg-3 breach.
4. **Fire every control**: run each against a known-bad input and confirm it goes RED. Delete or fix
   any that cannot.
5. Add the claim tag (§5A) to your review or pull-request template.

---

*This document is an instance of what it describes: every quantitative claim in it was measured in a
real deployment, and the section that matters most — leg 3 — exists because its author skipped it and
had it caught in review.*

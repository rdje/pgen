---
id: a-gate-must-be-able-to-fail-and-able-to-run
title: A freshness gate that regenerates the artifact it compares can never fail, and a gate in the wrong lane never runs — settle both properties before writing the assertion
answers:
  - "how do I gate a generated artifact against going stale"
  - "my freshness check re-runs the generator and then diffs — why is it always green"
  - "should a new check be a make target or a registered doctrine enforcer"
  - "where does a check in this repo actually run — pre-commit, CI, or only when a human asks"
  - "how do I hold a published number equal to its derivation without false-firing on historical citations"
  - "my anchor check reports the anchor missing but I can see it in the document"
  - "how do I keep a denominator published beside a defect bar"
tags: [gate-design, instrument-honesty, doctrine-enforcement, staleness, measurement, ci]
date: 2026-08-11
status: current
evidence: scripts/check_sv_corpus_denominator.sh (the scratch-directory indirection, the marker-scoped paragraph reader, the NOT-EVALUATED leg); docs/tasks/SV-CORPUS-GRAD.md leaf .13b (the six fired arms); docs/book/src/gate-flow.md §6 (the measured tier table — automatic = 0 of 123 make targets); scripts/check_regex_oracle_anchor_sync.sh (the bold-marker precedent); docs/tasks/SV-CORPUS-GRAD.md leaf .13a (the artifact that rotted in one day, publishing bar 319 against HEAD's 318)
reverify: "bash scripts/check_sv_corpus_denominator.sh && grep -q 'rust/target/sv_corpus_denominator_gate' scripts/check_sv_corpus_denominator.sh && grep -q 'SCRATCH directory' scripts/check_sv_corpus_denominator.sh && echo SCRATCH-INDIRECTION-LIVE"
---

A tracked artifact holding a derived number will go stale. In this repository the SV corpus
verdict-coverage artifact went stale **within one day** of being committed — it published
`match 5 804` and defect bar **319** while the manifest at HEAD said `5 805` and **318** — and nothing
could see it, because `git ls-files 'scripts/*.sh' 'rust/scripts/*.sh' '.github/workflows/*.yml'
'rust/Makefile' '.githooks/*' | xargs grep -ln <the instrument>` returned nothing at all.

Gating that is three decisions, and two of them are easy to get silently wrong.

## 1. ⛔ A check that regenerates the artifact it compares always passes

The obvious implementation is *re-run the generator, then diff the working tree*. It is green forever:
the re-run overwrites the evidence before the comparison reads it. **Regenerate into a scratch
directory and compare that against the tracked file.** The indirection is not a detail — it is the
whole check. Same shape as any verification that mutates its own subject: a test that fixes the data
before asserting on it, a linter run with `--fix` before the check step, a "reproducibility" gate that
rebuilds in place.

Corollary: prove the gate can go red *before* trusting it green. The denominator gate had six arms
fired on purpose — stale artifact, stale published anchor, missing anchor, generator refusing,
inputs absent, and a deliberately broken ground-truth control (which must exit 2, not 1).

## 2. ⛔ A gate's LANE is part of its design

`make` targets look like the natural home for a new gate. In this repo the automatic tier is **0 of
123 `make` targets**: hosted Actions are paused, 14 of 15 workflows are `workflow_dispatch`-only, and
the single auto-running workflow invokes the doctrine driver rather than any target. A `make` gate
therefore satisfies the task leaf and runs never — which is indistinguishable from not writing it
(`GATE-REACHABILITY` exists because three such gates were found *by accident*, one per session).

⇒ ask **"which lane will this actually run in?"** before choosing the shape. Here that meant shipping
a registered doctrine enforcer (`scripts/check_*.sh` in the `check_doctrines.sh` registry, ~1 s), so it
binds on every commit through the pre-commit hook and on every push through the one live workflow.
Family-specific doctrines are legitimate; `REGEX-ORACLE-ANCHOR-SYNC` is the precedent.

## 3. A published number needs an anchor, and the anchor needs a discriminator

Holding a document equal to a derivation runs into the document's own history: the chapter publishing
the current bar also narrates `293 → 319 → 318`. "The current value appears somewhere" is too weak;
"no other value appears" is unusable. **Mark the live anchor and scope the check to the marker**
(`Live verdict-coverage tuple — a/b/c/d/e`), exactly as the regex oracle anchor uses bold. Era-dated
citations stay history; the anchor stays strict — *every* value in the marker's paragraph must equal the
derivation, so a stray number fails loudly instead of averaging into agreement.

⚠️ **And scope it to the PARAGRAPH, not the line.** Prose wraps, so a marker and its value legitimately
land on different lines; a line-scoped reader reported the anchor MISSING against a perfectly correct
document on its first run. Same family as [[a-furthest-position-names-a-region-not-a-token]]: an
instrument that indexes text must be told the shape of the text it indexes.

## The general rule

Before writing the assertion, answer three questions in this order: **can it fail?** (do not let the
check touch its own subject), **will it run?** (name the lane, and check what that lane actually
executes), **what exactly does it compare?** (a marked live anchor, never "the number appears
somewhere"). A check that misses any one of them is decoration that reads as proof.

⚠️ And when an input is legitimately unavailable — here the corpora are git submodules a hosted
checkout may not have — report that leg as **NOT EVALUATED** and keep the others binding. Skipping the
whole check, or passing quietly, are both worse:
[[an-instrument-that-prints-an-unjudged-column-reports-a-defect-as-green]].

See also [[a-check-whose-inputs-all-pass-has-not-been-tested]] (manufacture the control when every
real input agrees), [[a-measurement-that-cannot-name-its-instrument-cannot-be-checked-for-staleness]],
and [[a-cap-with-no-headroom-is-a-cap-about-to-be-raised]].

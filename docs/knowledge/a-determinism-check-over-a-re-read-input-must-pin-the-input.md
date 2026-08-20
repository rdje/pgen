---
id: a-determinism-check-over-a-re-read-input-must-pin-the-input
title: A determinism check that re-reads a mutable input once per iteration must record WHICH input it read — otherwise its failure message cannot tell "the tool is nondeterministic" from "the file changed under me", and the two send you to opposite places
answers:
  - "my gate says 'signature drift vs seed 0' — is the tool nondeterministic?"
  - "how do I prove a rule count is deterministic rather than assuming it"
  - "what does a cross-seed determinism check actually prove"
  - "why did a count differ between seeds when the code that computes it takes no seed"
  - "how do I de-confound seed from process when each seed runs in its own process"
  - "what identity does a multi-iteration gate need beyond its output signature"
tags: [gates, determinism, evidence, confounding, provenance, instruments, methodology]
date: 2026-08-20
status: current
evidence: "SV-CORPUS-GRAD.13c.2x.1 (PGEN-SV-CORPUS-GRAD-0255). sv_cert_recognized_union_gate read canonical total=1434 at seed 0 and 1433 at seeds 7/42 on the SV-0065 arm, and its determinism check fired. certificate_coverage() is PURE and its total is grammar.rule_order.len(), so no witness-search seed can move it — which made the observation look like nondeterministic rule SYNTHESIS and put every rule-count baseline in the repository in doubt. The discriminating probe (3 processes at one fixed seed, then one process per seed) measured 1434 six times out of six on the arm and 1433 three times out of three at HEAD, refuting BOTH recorded hypotheses. The gate re-reads $GRAMMAR_FILE once per seed (:255) and its signature (:307) is ten OUTPUT fields with no input identity; its only identity call (:177) is hoisted out of the loop and checks the contract, not the grammar. The gate's own arithmetic agrees: unmet=29 on the arm vs 27 at HEAD, where 27 is nine criteria on each of three seeds, so the +2 is exactly the two drift entries the loop appends. Census over the repo: 5 gates assert determinism across re-reads and 5 of 5 record no input identity inside the comparison loop."
reverify: "bash docs/tasks/artifacts/sv_corpus_grad/cert_count_determinism/gate_input_pin_census.sh   # GATE-INPUT-PIN-CENSUS: asserts_determinism=5 blind_to_input_change=4 (was 5 of 5; the SV cert gate was fixed by PGEN-SV-CORPUS-GRAD-0258, the remaining four are ENGINE-UNIVERSAL-SERVICES.41). The attribution helper is proven by docs/tasks/artifacts/sv_corpus_grad/cert_count_determinism/attribution_probe.sh -> ATTRIBUTION-PROBE: passed=20 failed=0"
---

**"Every iteration agreed" is a claim about the TOOL. It only means that if the INPUT was the same
every time — and a gate that re-reads a file per iteration and records nothing about it has not
established that.** When such a check fails it reports the only thing it can see, a difference in
outputs, and that message is equally true of a nondeterministic tool and of a file someone edited
while the gate was running. Those two causes have nothing in common: one is a defect in the code
under test, the other is not a defect at all.

## The shape of the mistake

A gate loops over seeds, and per seed it re-reads a grammar from disk, runs a minutes-long
analysis, and appends the run's numbers to a signature. At the end it asserts every seed's
signature equals the first seed's. The signature is built from outputs only:

```text
canon_total|canon_proof|canon_witness|canon_unknown|canon_spf|union_total|…|residual
```

That is a perfectly good *comparison* and a perfectly bad *attribution*. Three properties conspire:

1. **The input is mutable and re-read.** It is not snapshotted, hashed, or held open.
2. **The window is long.** Minutes per iteration, so an ordinary human action — applying an
   experimental patch, reverting it — lands inside the window routinely.
3. **The failure message names the axis it varied, not the axis that moved.** It says
   "seed 7 differs from seed 0", so the reader reasons about seeds.

The result is a hypothesis space anchored on the wrong variable. In the instance above, both
hypotheses that got written down — per-process nondeterminism and genuine seed-dependence — were
about the tool, because the message was about the tool. Both were false. The real explanation was
not in the space at all, and could not be, because nothing in the evidence recorded the input.

## Why "it's pure, so this is impossible" is the tell, not the answer

The count that moved is computed by a pure function of the loaded grammar. That was established
early and correctly, and it is exactly the point at which the investigation should have turned
outward. When a value that *provably cannot* depend on the varied axis appears to depend on it, the
strongest inference is not "the impossible happened" but **"something you are not recording
changed"**. A pure function with a moving output is a statement about its argument.

## What to do instead

- **Pin the input inside the loop.** Record a digest of every mutable input per iteration and put
  it in the comparison. On drift the gate can then say *"the GRAMMAR CHANGED between iteration 1 and
  iteration 2"* — which is a different sentence, sending the reader to a different place, in one
  line instead of two sessions.
- ⛔ **Make the digest EVIDENCE, not a standalone failure condition.** A digest that fails on its own
  fires on edits the consumer provably cannot see — a comment the frontend strips, a reordered
  key — and a guard that blocks work for a non-difference gets bypassed. Consult it only once the
  comparison has already failed; then it can only ever *improve* a message, never invent a failure.
- ⭐ **Take the digest at the point of use.** Hoisting it into a helper defined above the loop is
  functionally identical and measurably worse: any instrument that scans the loop body for evidence
  of pinning will report the gate as still blind. When that happened here, the tempting fix was to
  teach the instrument about the helper — **which is tuning the measure to flatter the change it is
  measuring.** Move the code instead.
- **De-confound before you hypothesise.** If each iteration is also a separate process, then
  "iteration" and "process" are one axis, not two. Vary them independently — N processes at one
  fixed value, then one process per value — and the two hypotheses separate mechanically instead of
  by argument.
- **Make the control prove it can go RED.** A probe reporting "stable" is worthless until you have
  shown it distinguishes the two values in question. Run it against a state you know differs; if it
  cannot see that difference, its stability finding is about the probe.
- **Cross-check with instruments that share nothing.** Three unrelated code paths reporting the same
  delta for the same change (a certificate pass, a linter, a frontend graph derivation) turn
  "repeatable" into "correct".

## The generalisation

This is the same defect as a stale baseline with no identity block, one level up. There, an artifact
stored numbers derived from a tree and could not say *which* tree; here, a gate compares numbers
derived from a file and cannot say *which* file. Both fail in the direction that reads as a
substantive finding — a regression, a nondeterminism — when the honest reading is "I cannot
attribute this". ⛔ An unattributable RED is not a small defect: it costs exactly as much
investigation as a real one, and it spends that budget on a hypothesis space that does not contain
the answer.

See also [[a-provenance-block-must-say-whether-the-numbers-were-ever-right]] and
[[a-check-whose-inputs-all-pass-has-not-been-tested]].

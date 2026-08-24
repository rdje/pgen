---
id: the-first-real-consumer-is-a-test-of-the-producer
title: An option nobody has exercised is not a feature yet — wiring the first real consumer is a test of the producer, and it is not substitutable
answers:
  - "my tool passed its own self-test but broke when a gate called it — why"
  - "why did wiring a gate to a working instrument break the instrument"
  - "is a green --self-test enough to trust a new doctrine"
  - "where should a gate write its scratch files in this repo"
  - "my script printed the right output and still exited non-zero"
  - "how do I prove a doctrine actually blocks a commit"
  - "should I test a refusal at the enforcer or at the driver"
  - "TMPDIR vs a repo-derived scratch path — which is correct here"
  - "how do I check an instrument before gating on its verdict"
tags: [gates, doctrines, instruments, consumers, controls, exit-codes, data-locality, corpus-key-audit]
date: 2026-08-24
status: current
evidence: "CORPUS-KEY-AUDIT.1(e) (PGEN-CORPUS-KEY-AUDIT-0003). The census had shipped one commit earlier, measured, self-tested and doctrine-green. Wiring a gate to it broke it twice in ten minutes. (1) `--md` is a documented option, but every prior call pointed it at the default path INSIDE the repo; a gate must point it at scratch, and `args.md.relative_to(ROOT)` raises ValueError on any path outside — after the summary had printed and the file had been written, so the work succeeded and only the exit code said failure. (2) The enforcer's first cut wrote scratch to ${TMPDIR:-/tmp}; on this host TMPDIR happens to sit on the repository volume, so it would have passed every check run locally while violating the same-volume data policy by construction elsewhere. Both fixed; doctrine CORPUS-KEY-INTEGRITY landed as the repository's 27th, 13/13 refusal arms."
reverify: "bash scripts/check_corpus_key_integrity.sh --self-test   # 13 passed, 0 failed. Then the claim this record is about, at the DRIVER not the enforcer: printf '\\nDRIFT\\n' >> docs/tasks/artifacts/corpus_key_audit/census.md && bash scripts/check_doctrines.sh   # -> FAIL CORPUS-KEY-INTEGRITY, commit/merge blocked; then git checkout docs/tasks/artifacts/corpus_key_audit/census.md && bash scripts/check_doctrines.sh   # -> ALL 27 enforced doctrines PASS"
---

A tool that passes its own tests has been tested by the person who wrote it, in the shapes that
person imagined. **The first real consumer tests it in the shapes the job actually has** — and that
is a different set, reliably.

## The measured case

A corpus-audit census shipped complete: tracked, re-runnable, five synthetic self-test arms, a green
doctrine run. One commit later, wiring a gate to it broke it twice inside ten minutes.

**Defect 1 — a documented option nobody had exercised.** `--md OUT` had always been called with the
default, in-repo path. A gate must point it at scratch so the tracked artifact is never mutated, and
the reporting line did:

```python
print(f"wrote {args.md.relative_to(ROOT)}")   # ValueError on any path outside the repo
```

⛔ **The failure shape is the one worth remembering: the work succeeded and only the exit code said
otherwise.** The summary line had already printed. The output file had already been written. A reader
skimming stdout would have seen a correct census beside a red gate and gone hunting in the wrong
place.

**Defect 2 — a default that is correct on your machine.** The enforcer's first cut wrote scratch to
`${TMPDIR:-/tmp}`. On this host `TMPDIR` happens to sit on the repository's own volume, so it would
have passed every check run locally — and been wrong by construction anywhere it does not. The
repository already had the right answer in every other self-test: a **repo-derived** scratch path
(`rust/target/`, git-ignored, resolved from the script's own location).

## Two habits

1. **Treat "wire up the first consumer" as a test phase, not as plumbing.** Budget for finding
   defects there. The producer's own tests cannot cover the calling convention a consumer imposes,
   because that convention did not exist when they were written.
2. **Never take a default from the environment when the repository has a convention.** `$TMPDIR`,
   `$HOME`, the current working directory: each is a value that varies per host and therefore cannot
   be checked by running it on yours. Copy the house pattern.

## And prove the refusal where it is enforced, not one layer below

Thirteen green arms in a gate's `--self-test` show the *enforcer's* logic refuses. They say nothing
about whether the **driver** the commit hook actually runs surfaces that refusal — a gap this
repository has already been bitten by, when a flag outlived its implementation's deletion and went on
exiting 0. So the control was run end to end:

```text
printf '\nDRIFT\n' >> <the tracked artifact>   ->  ✗ FAIL CORPUS-KEY-INTEGRITY … commit/merge blocked
git checkout <the tracked artifact>            ->  ALL 27 enforced doctrines PASS
```

This is [[a-control-that-cannot-fail-is-not-a-control]] applied to the *wiring* rather than to the
check: an unwired gate and a wired one that never fires are indistinguishable from the outside. See
also [[prove-each-refusal-path-with-an-input-only-it-can-trigger]] for the per-arm version, and
[[a-gate-must-be-able-to-fail-and-able-to-run]].

## The corollary that shaped the gate itself

If building a consumer can break a producer, then a gate that simply reads an instrument's verdict is
gating on an oracle whose own health is unestablished. So the doctrine checks the instrument
**first** — a corpus-independent tier that runs the census's self-test and refuses a *shrunken* arm
set, a *zero-arm* run reporting `failed=0`, and an *unparsable* summary — and only then reads the
verdict. That third refusal is the subtle one:
[[an-answer-key-that-records-why-it-decided-can-audit-itself]] is a lane founded on errors that fail
in the passing direction, and **a reader returning "no findings" for text it cannot parse is exactly
that**.

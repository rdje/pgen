---
id: a-published-block-derived-from-untracked-state-fails-on-a-clean-clone
title: A co-publication gate holds a document equal to a DERIVATION, so the derivation must read only TRACKED inputs — mix in build-output state and the gate fails on a fresh clone while passing on the machine that ran the build
answers:
  - "my derive-and-diff currency gate passes locally and fails in CI — what differs"
  - "should a published status block include what the gate artifacts in my tree say"
  - "how do I keep a --check style documentation gate portable across clones"
  - "why does a status page disagree between two checkouts of the same commit"
  - "can I publish a number that comes from something under target/"
  - "how do I report machine-local state without making it a gate-held claim"
tags: [gates, currency, documentation, evidence, portability, ci, instruments]
date: 2026-08-30
status: current
evidence: "DONE-BAR-VISIBILITY.1 (PGEN-DONE-BAR-VISIBILITY-0003). scripts/audit_done_bar.sh derives Done-bar leg 1 and the green-NOW half of leg 2 from `*_parser_family_status_gate/summary.txt` artifacts under rust/target/, which .gitignore excludes. On the machine that had run the gate the audit reported leg 1 UNPROVEN with reason `artifact … STALE — older than its own inputs`; driven through its own PGEN_DONE_BAR_TARGET_DIR seam at an empty directory — the state of any fresh clone — the same commit reported `has produced no non-empty summary.txt on this machine`. Same verdict, different sentence: a block publishing either one would fail --check for the other reader. Measured on the same run: with the seam applied the audit's whole report carries no mtimes and no build-output paths, so the tracked-only derivation is byte-stable across clones, and all 13 of the audit's ground-truth controls still reproduce."
reverify: "mkdir -p rust/target/empty && PGEN_DONE_BAR_TARGET_DIR=$PWD/rust/target/empty bash scripts/audit_done_bar.sh | grep -A2 'leg 1' | head -4   # then diff against `bash scripts/audit_done_bar.sh` — the verdicts agree, the REASONS do not"
---

**A derive-and-diff gate is a promise about a document, but it is enforced by re-running a
derivation.** The document is tracked; the derivation is not automatically anything. If any input it
reads lives outside the tracked tree — a build artifact, a log, a gate summary, an installed tool
version — then the gate is not comparing the document against *the repository*. It is comparing the
document against *this machine*.

That has a very specific failure shape, and it is worse than a plain false alarm:

- On the machine that produced the build output, the gate is **green**, and stays green.
- On a fresh clone — a new contributor, a CI runner, a hosted job that skipped a build — the same
  commit derives something different, so the gate is **red on an untouched tree**.
- The remedy the gate suggests ("re-derive and republish") makes it green *there* and red *back on
  the first machine*. The two readers cannot both be satisfied, and neither of them changed anything.

A gate that fails on a clean clone of its own repository teaches exactly one lesson: bypass it.

## The fix is a policy about which half is published, not a fix to the gate

Split the derivation by the portability of its inputs:

- **The published half is derived from tracked inputs only.** It is what the repository proves to
  anybody who clones it, and it is the only half `--check` compares. Every clone agrees on it by
  construction.
- **The machine-local half is reported, never published.** It belongs in the terminal output the
  operator reads, clearly labelled as *this tree, right now*, alongside the command that would
  establish it.

Both halves come from **one run of one producer**, so they cannot tell different stories about a
shared fact — the split is about what is *gate-held*, not about deriving twice.

## Prefer a seam the instrument already has to a fork of it

The instrument that reads untracked state usually has a testability seam for exactly this — an
environment variable naming the artifact root, because its own ground-truth controls needed to
construct the state they observe. Point that seam at an empty directory and the same instrument,
unmodified, answers the tracked-only question. Forking it, or re-implementing its attribution logic
in the renderer, buys a second copy that drifts; the seam buys none.

## How to notice you have this defect

Ask of every number in a gate-held block: *what would a fresh clone read here?* If the answer needs
a build first, the number is not a claim about the commit. Two cheap tells:

- The rendered text contains an mtime, a build-output path, or a phrase like *on this machine*.
- The gate's own instructions include running something under `make` before the check will pass.

⚠️ **The honest limit**: publishing tracked-only makes some legs permanently unprovable in the
document — a proof that lives only in an untracked artifact can never be published as met. That is
information, not a loss. It says the proof surface itself is not part of the repository, which is a
finding the reader should have.

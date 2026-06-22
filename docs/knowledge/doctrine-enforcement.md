---
id: doctrine-enforcement
title: The general doctrine enforcer — make a rule provable + gated, not prose
answers:
  - "how do I enforce a project doctrine mechanically"
  - "how do I make a rule provable and re-checkable instead of trust-me"
  - "where is the general doctrine enforcer / driver"
  - "how do I add a new enforced doctrine"
  - "how is the use-the-debug-toolbox rule enforced"
  - "how do I prove a code change followed the root-cause procedure"
  - "what is the check-script contract for a doctrine"
  - "how do I enforce reasoned-from-evidence"
  - "is the doctrine enforcer portable to other projects"
tags: [enforcement, doctrine, gate, git-hook, ci, toolbox, reference]
date: 2026-06-22
status: current
evidence: scripts/check_doctrines.sh (driver+registry); scripts/check_diagnosis_evidence.sh (evidence archetype); .githooks/pre-commit (E3); DOCTRINE_ENFORCEMENT.md (portable standard)
reverify: `bash scripts/check_doctrines.sh` (prints the per-doctrine PASS/FAIL report); `git config core.hooksPath` (expect .githooks)
---

A doctrine that is not mechanically checked is a suggestion. PGEN enforces doctrines via a
registry+driver + git hook + CI (MEMORY_ARCHITECTURE.md §9 layering, generalized). Portable model:
`DOCTRINE_ENFORCEMENT.md`.

## The pieces
- **Driver/registry:** `scripts/check_doctrines.sh` — runs EVERY registered `check_*.sh`, prints a
  per-doctrine PASS/FAIL report, exits nonzero on any breach; meta-checks that each registered
  enforcer exists+is executable.
- **Git hook (E3):** `.githooks/pre-commit` runs the driver (activate once: `git config core.hooksPath .githooks`).
- **CI (E4):** the same driver server-side (un-bypassable). HONEST GAP: hosted CI is currently
  manual-only — re-enabling an auto job is the true "no matter what".

## The three check archetypes (pick by what makes the proof real)
- **Structural** — re-derive an invariant from the tree (allowlist match, derived-artifact sync). Cannot be faked; cheap → pre-commit.
- **Oracle (re-run)** — re-EXECUTE a deterministic tool at fixed seeds/inputs and assert the result (cert-coverage at seeds 0/7/42, shape-contract). Strongest; heavy ones → CI.
- **Evidence (artifact)** — for a process that leaves no other trace, require a re-checkable artifact; make it oracle-like (re-run the cited command).

## The check-script contract (so any project can add doctrines)
`exit 0` = compliant / nonzero = breach; explain on stderr; DETERMINISTIC; reads repo+git, mutates
nothing (idempotent derive-and-stage ok); scope-aware (look at `git diff --cached`, exempt
non-governed changes); resolve root from `BASH_SOURCE`; fast or CI-deferred. Register = add one
`id|proves|path` line to the driver's `DOCTRINES` array.

## The toolbox / task-acceptance enforcer
`scripts/check_diagnosis_evidence.sh` (evidence archetype): a CODE change (grammars/*.ebnf,
rust/src/**, generated/**, ast_shape_contract manifests) is BLOCKED unless its staged owning
`docs/tasks/*.md` leaf passes the ACCEPTANCE CHECKLIST — the required boxes **ROOT CAUSE** (why+where),
**ADDRESSED** (verified), and **NO REGRESSION** must each be TICKED `[x]` and backed by a tool
signature; an unticked or missing required box blocks the commit (the task is not done). Pure-docs
commits are exempt. **A box is EARNED, not ticked:** the `[x]` is a claim — the proof is the
deterministic-gate ORACLE RE-RUN (cert at seeds 0/7/42, ast_shape_contract, byte-identical across the
fully-certified grammars, external corpus) in CI; a self-ticked-but-false box fails when the oracle
re-runs (DOCTRINE_ENFORCEMENT.md §6.1). Checklist template in TOOLBOX.md. See
[[cert-coverage-unknown-diagnostics]] for the diagnosis tools.

## Honest limits
Local hooks are bypassable (`--no-verify`); CI is the backstop. Evidence-presence is fakeable unless
the cited command is re-run (so prefer structural/oracle). A check proves artifacts+oracles
reproduce, not intent — which is the point.

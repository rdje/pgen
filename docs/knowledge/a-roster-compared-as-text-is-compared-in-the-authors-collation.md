---
id: a-roster-compared-as-text-is-compared-in-the-authors-collation
title: A roster compared as text is compared in the author's collation — an exact-set comparison sorts both sides under LC_ALL=C, because a set has no order and `sort`'s order belongs to the host
answers:
  - "why did the hosted memory-architecture-gate say root markdown allowlist drift when the files were all there"
  - "why does CI-MIRROR-DOCS-SURFACES pass locally and fail on ubuntu-latest"
  - "why does sort put MEMORY.md before MEMORY_ARCHITECTURE.md on the runner and after it on the Mac"
  - "does the order of entries in a docs-surface allowlist roster matter"
  - "how should an exact-set comparison of file lists be written in a shell gate"
  - "what did CI-PARITY-GATE-ROT.122 fix"
tags: [ci, collation, locale, sort, allowlist, push-mirror, hosted-runner, instrument]
date: 2026-10-20
status: current
evidence: "PGEN-CI-PARITY-GATE-ROT-0127 (CI-PARITY-GATE-ROT.122): hosted run 39421495286 on a0bec1ab6 printed two root-markdown lists holding the same seventeen files in different orders and refused the tree; LC_ALL=C reproduced the identical refusal on the author's host, the default en_US.UTF-8 locale passed. Five audits in rust/scripts/ci_workflow_local_gate.sh compared a hand-ordered roster verbatim against the host's `sort`; both sides are now sorted under LC_ALL=C, verified under C, POSIX and en_US.UTF-8 with the doctrine's four RED arms intact."
reverify: "LC_ALL=C PGEN_CI_WORKFLOW_LOCAL_AUDITS_ONLY='audit_root_markdown_surface audit_top_level_docs_surface audit_contract_docs_surface audit_reference_docs_surface audit_docs_book_surface audit_active_docs_rehome_paths' bash rust/scripts/ci_workflow_local_gate.sh 2>&1 | grep -F 'audits-only mode: 6 working-tree audit(s) passed'"
---

**Two lists that hold the same files and differ in order are the same set. A gate that compares them as strings has an opinion about collation it did not mean to have.**

## The rules

1. **`sort`'s order belongs to the host.** In `en_US.UTF-8`, `sort` ignores punctuation at the first
   level, so `MEMORY_ARCHITECTURE.md` (read as `MEMORYARCHITECTUREmd`) precedes `MEMORY.md`
   (`MEMORYmd`); in `C`, `POSIX` and `C.UTF-8` (a stock `ubuntu-latest` runner) `.` (0x2E) precedes
   `_` (0x5F) and the order flips. A roster whose order was copied from the author's `sort` output
   is correct on the author's machine and wrong on the runner.
2. **An exact-set comparison sorts BOTH sides under `LC_ALL=C` first.** Then the order of the
   roster on the page means nothing, the comparison is the same on every host, and a comment such
   as "ORDER IS LOAD-BEARING: this locale sorts X before Y" can never be written again — "this
   locale" was the tell.
3. **The first refusal hides the rest.** A gate that `fail`s at its first audit evaluated nothing
   after it; the top-level-docs roster carried the same hazard (`TASK_TREE_README.md` versus
   `TASK_TREE.md`) and would have been the next hosted red. Fix the class, not the instance.
4. **A hosted verdict is read defect by defect against the local one.** The two printed lists were
   the whole diagnosis; `LC_ALL=C` reproduced them locally in two seconds, so no CI-shaped checkout
   was needed for this member of the class (`CI-PARITY-GATE-ROT.121` needed one for the other five).

## Where it lives

- `rust/scripts/ci_workflow_local_gate.sh` — the five set audits (`audit_root_markdown_surface`,
  `audit_top_level_docs_surface`, `audit_contract_docs_surface`, `audit_reference_docs_surface`,
  `audit_docs_book_surface`): `LC_ALL=C sort` on both sides of every comparison.
- `scripts/check_ci_mirror_docs_surfaces.sh` — the doctrine that runs them at every commit; its
  `--self-test` proves the four RED arms.
- `docs/tasks/CI-PARITY-GATE-ROT.md` leaf `.122` — the hosted log, the reproduction, the fix.

# DOCTRINE-GAP-OWNERSHIP: a known defect written into a decision record is not tracked work — 58 commits proved it

## Metadata

- Tree ID: `DOCTRINE-GAP-OWNERSHIP`
- Status: `active` (opened 2026-07-27, session #215, **by direct director order**)
- Family / slice-id prefix: `PGEN-DOCTRINE-GAP-OWNERSHIP-<NNNN>`
- Created: `2026-07-27`
- Owner: repo-local workflow
- **Frontier: `.1`** (sweep every recorded-but-unowned gap and give each one a leaf)

## ⛔ THE DIRECTOR'S ORDER (2026-07-27, session #215, verbatim)

On being shown that the enforcer's box-scoping hole had been known and unfixed:

> *"Why did you we address this issue in the enforcer in the first place ? To me that's a
> critical defect, it shouldn't have happened. please make sure they are all task-tree owned,
> tracked and addressed head on when time permit."*

Two obligations, and they are separable: **(1) OWNERSHIP + TRACKING is due now** — every
recorded-but-unowned gap gets a task-tree leaf; **(2) addressing them head-on is "when time
permits"** — the director explicitly allowed that half to be scheduled, not rushed.

## ⭐ THE PROVENANCE (measured, not asserted)

| fact | evidence |
|---|---|
| the gap was FOUND and WRITTEN DOWN | commit `82e8ff32`, **2026-07-22** (`PGEN-BIN-BUILD-INTEGRITY-0002`) |
| where it was written | a **"Watch item"** paragraph in `docs/decisions/project_build_integrity_compiler_root_cause_signature.md` |
| what it said | *"the checks grep whole files, not the ticked bullet … that whole-file looseness affects all three signature groups and is worth a future box-scoped hardening slice"* |
| was it ever given a task-tree leaf? | **No.** `git log --all -S "box-scoped" -- docs/tasks/` returns only `5c5a0ca0`, the fix itself |
| commits that elapsed | **58** |
| how it was eventually fixed | an UNRELATED leaf (`GENERATED-LINT-CORRECTNESS.3`) tripped over the same hole and re-derived it from scratch |

⇒ **ROOT CAUSE: `docs/decisions/` is a MEMORY surface, not a WORK QUEUE.** A defect recorded
there is durable but inert — no frontier points at it, no index lists it as open, no gate asks
about it. The author who records a gap discharges their honesty obligation and the project
silently absorbs a known defect. The task-tree doctrine exists precisely to stop this, and it
was bypassed not by carelessness but because **there is no rule saying a recorded gap must
become a leaf.**

## ⭐⭐ WHY THIS IS THE SESSION'S THIRD INSTANCE OF ONE PATTERN

1. `GENERATED-LINT-CORRECTNESS.1` — `ast_dump_contract_gate`: a check **nothing runs**.
2. `CI-PARITY-GATE-ROT` — `ci_workflow_local_gate`: a gate **nothing runs**, 1,371 commits.
3. **This tree** — a defect **nothing tracks**, 58 commits.

The unifying statement, now covering both axes: **an instrument that nothing invokes, and a
defect that nothing tracks, are both indistinguishable from absent.** `CI-PARITY-GATE-ROT.2`
owns the mechanization for the first; this tree owns it for the second.

## Leaves

### `.1` — sweep every recorded-but-unowned gap and give each one a leaf (`todo`)

- **Status: `todo`** — frontier. This is the half the director wants **now** (ownership +
  tracking), ahead of fixing.
- Sweep for recorded-gap language across `docs/decisions/`, `docs/tasks/` and the live docs —
  *watch item*, *known gap*, *known limitation*, *worth a future*, *hardening slice*,
  *not fixed here*, *residual*, *deferred*, *left as-is*, *re-open if* — and for each hit
  determine whether a task-tree leaf already owns it.
- Every orphan gets an owning leaf (new or existing) and appears in `docs/TASK_TREE.md`.
  ⛔ Do **not** fix them in this leaf — ownership first, so the backlog becomes visible and
  countable before it becomes work.
- Deliverable: a census (how many gaps, how many orphaned, oldest-first by the commit that
  recorded them) plus the leaves. Expect the age distribution to be the alarming part.
- ⚠️ Known members already: this tree's own founding case (now closed by `-0004`);
  `GENERATED-LINT-CORRECTNESS.2`'s note that `always_succeeds_alternatives` cannot see through
  the allowlist ("re-open if a member is ever proposed"); `QUANT-PLUS-ITER`'s include-provenance
  bound routed to `.4`. Those three were found incidentally — the sweep is the point.

### `.2` — mechanize it: a recorded gap must name its owning leaf (`todo`)

- **Status: `todo`**, after `.1`.
- Candidate enforced doctrine (would be the **11th**, alongside `CI-PARITY-GATE-ROT.2`'s
  reachability doctrine): a staged `docs/decisions/*.md` or `docs/tasks/*.md` that ADDS
  recorded-gap language must cite an owning leaf ID, or the commit is blocked — the same shape
  as `DESIGN-PRIOR-ART`, which already blocks a novel `@name` without a `PRIOR ART` section.
- ⚠️ Honest limit to state up front, exactly as `DESIGN-PRIOR-ART` does: this can only check
  that an owner was NAMED, not that the owner is real or that the work will happen. That is
  still strictly better than prose nobody indexes.
- ⛔ Design constraint: it must not punish honesty. If recording a gap becomes expensive,
  authors stop recording gaps — which is far worse than an untracked one. Prefer a one-token
  citation over a required section.

## Evidence

- `git log -1 -S "whole-file grep, not box-scoped" -- docs/decisions/project_build_integrity_compiler_root_cause_signature.md`
  → `82e8ff32 2026-07-22`.
- `git log --all -S "box-scoped" -- docs/tasks/` → only `5c5a0ca0` (the fix), i.e. no leaf ever
  owned it.
- 58 commits between the record and the fix.

## Commit log

| slice | leaf | commit subject |
|---|---|---|
| `PGEN-DOCTRINE-GAP-OWNERSHIP-0001` | (tree opened) | a known defect filed in a decision record is not tracked work — 58 commits proved it |

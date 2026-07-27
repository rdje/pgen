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

⛔⛔ **ROOT-CAUSE FRAMING CORRECTED IN PLACE (director, same session).** The first version of
this tree said the cause was that *"no rule says a recorded gap must become a leaf"*. **That is a
deflection and it is false.** The rule already exists, twice over: `CLAUDE.md` item 6 (*"Track ALL
work in task-trees"*) and the standing surfacing directive (*"open a tracked task for it. I should
never be the one who has to notice"*). Director, verbatim: *"But it is your job to do that,
systematically, that's your job. If you see something that isn't correct, you should systematically
task-tree own it. But for some reason you didn't nothing. that's not cool."*

⇒ **ROOT CAUSE: a COMPLIANCE failure, not a missing rule** — and not only in 2026-07-22. This
session repeated it: `GENERATED-LINT-CORRECTNESS.3` *read* that watch item during its prior-art
search, *used* it, and *fixed* it — and still never asked "what else is sitting orphaned like
this?" until the director asked. The mechanization in `.2` is a **backstop for a discipline that
is already mandatory**, never a substitute for it.

⇒ **Contributing structural factor (real, but secondary): `docs/decisions/` is a MEMORY surface,
not a WORK QUEUE.** A defect recorded
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

### `.1` — triage the measured gap census and give every real orphan a leaf (`todo`)

- **Status: `todo`** — frontier. ⛔ **DIRECTOR-SCHEDULED FOR THE NEXT SESSION** (2026-07-27,
  verbatim: *"Please plan all of that for the next session, there is no sufficient context to
  handle those things in this session"*). The sweep is BUILT and RUN; what remains is triage.

#### The census (measured 2026-07-27 at `18be844e`, whole repository)

Driver: `docs/tasks/artifacts/doctrine_gap_ownership/run_gap_ownership_sweep.sh` (tracked).
**225 hits total — 85 OWNED, 140 ORPHAN.** By surface:

| surface | orphans | triage prior |
|---|---|---|
| `history-narrative` (`CHANGES.md` 30, `DEVELOPMENT_NOTES.md` 25, book changelog-indexes 8) | **63** | ⭐ **almost certainly NOT backlog** — these are append-only historical records; *"not fixed here"* in a changelog entry is **provenance of what that commit did**, not open work. Confirm the rule, then exclude the surface wholesale. |
| `root-docs` | 22 | live — triage each |
| `contracts` (`SV` 11, `regex` 9) | 21 | live — likely published support boundaries, not defects; check |
| `task-trees` | 13 | live — a gap in a tree is usually *already* owned by that tree; verify |
| `decisions` | 10 | **SAMPLED — see below** |
| `rust-src` (TODO/FIXME/HACK) | 5 | live code markers — real candidates |
| `reference` / `grammars` / `book` | 3 / 2 / 1 | live — triage each |

#### ⭐⭐ THE SAMPLE THAT SAYS "DO NOT TREAT 140 AS A BACKLOG"

`docs/decisions/` was triaged by hand this session — **10 raw orphans resolved to 1**:

- **6 FALSE POSITIVES** from the regex — doctrine prose (*"the tool-build is its own task-tree
  leaf"*), a leaf ID in a format the owner-pattern missed (`.b.6.2.36.2`), and 4 hits inside
  `project_codegen_emission_root_cause_signature.md` that are the **closure text** for the very
  gap being searched for.
- **2 STALE-CLOSED**, ⭐ **and this is the inversion worth carrying forward: the real defects were
  records claiming work is OPEN when it is DONE**, not untracked open work. (a) the build-integrity
  *"Watch item"* — closed by `5c5a0ca0` but still reading as open; (b)
  `project_every_parser_per_parser_book.md` still listing `ebnf` / `return_annotation` /
  `semantic_annotation` as *"Still missing"* when **all three books exist** and `README.md` already
  calls the directive complete. A future session would have rebuilt three books that already exist.
  **Both corrected in place this session.**
- **1 OWNED-BUT-UNLINKED** — the `eprintln!` shadows latent finding, genuinely owned by the
  `DIAG-SEVERITY` tree, just never cross-referenced. **Cross-reference added this session** (still
  live: 171 sites in `mod.rs`, 76 in `ast_based_generator.rs`).

⇒ **0 genuinely untracked open gaps in `docs/decisions/`.** Expect the same shape elsewhere: the
actionable count is plausibly 10–25, not 140. ⛔ **Do not open 140 leaves.**

#### How to run `.1`

1. Re-run the driver; work surface by surface, **cheapest exclusions first** (`history-narrative`).
2. Classify every hit into exactly one of: **false positive** (tighten the driver's patterns —
   the driver is the deliverable too), **stale-closed** (annotate the record CLOSED + the commit
   that closed it), **owned-but-unlinked** (add the cross-reference), **genuinely orphaned**
   (open/attach a leaf).
3. ⛔ **Own them, do NOT fix them here.** The deliverable is the triaged census + the leaves +
   a tightened driver whose ORPHAN count is then *meaningful*.
4. Record the age of each genuine orphan (the commit that first recorded it). The 58-commit
   latency of the founding case is the number that made this a tree; the distribution is the
   argument for `.2`.

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

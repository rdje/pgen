# LIVE-DOC-CONTAINMENT: adopt the Live-Document Size-Containment Doctrine, and enforce derived-state containment

## Metadata

- Tree ID: `LIVE-DOC-CONTAINMENT`
- Status: **`active`** (created 2026-08-08, session #212, **BY DIRECT DIRECTOR ORDER** —
  > *"I think the live docs containment doctrine shall not be dropped. I might not adopt
  > everything, I understand."*
  and, on sequencing:
  > *"Don't you think queueing it after the corpus program you'll face being hit by the cap
  > issue again, again and again?"*)
- Roadmap lane: documentation governance. Sibling of `README-POLICY` (which governs `README.md`)
  and `LIVE-MEANS-LIVE` (which governs whether a live file's contents are currently TRUE). This
  tree governs the **lifecycle and bounds of every long-lived document family**, and supplies the
  umbrella contract the other two are implementations of.
- Created: `2026-08-08`
- Owner: repo-local workflow

## ⭐ Adopted against a MEASUREMENT, and the measurement is this repo's own anti-pattern

`MEMORY.md` breached its byte cap this session (**7 545 B > 7 168 B**) and the repair applied was
**prose compression** to **7 080 B**. That is the containment doctrine's anti-pattern verbatim —
*"trimming prose repeatedly while leaving the scaling field intact"* — committed by this
repository's own agent, one hour before the doctrine was read. It bought **465 bytes** of
editorial headroom and removed **no scaling term**, leaving **88 bytes (98.7 % full)**.

⛔ **THE SEQUENCING WAS WRONG AND THE DIRECTOR CORRECTED IT.** The proposal was to queue this
behind the cross-family corpus program. That program is inherently **per-family** — ~9 families ×
(axis-1 residual + axis-2 corpus + divergence count + last-measured date) — i.e. it adds a
scaling term to a pointer with 88 bytes of room. It would have breached on slice one or two, with
compression as the only available tool. The adoption guide's own Phase 1 says stabilize the
pointer **before** any project-wide migration; the proposal inverted the guide it was citing.

## ⭐⭐ It is an UMBRELLA, not a second authority — the concern that was withdrawn

The first assessment reserved against adoption on the containment doctrine's own stop condition
(*"two sources plausibly claim canonical authority for the same information"*), since PGEN already
enforces `MEMORY-ARCH`, `README-STABILITY`, `LIVE-DOC-CURRENCY` and `PUBLISHED-VERSION-CURRENCY`.

**That reservation is WITHDRAWN, on evidence.** `LIVE-MEANS-LIVE.1c` had already executed a
migration of exactly this doctrine's shape before PGEN owned its vocabulary: a
**1 563 641 B / 1 684-line** "live status" file that was **94.7 % dated changelog** around 14
status rows, migrated-and-deleted and **proven lossless by re-measurement at the delete**
(467/467 cited slice IDs still reachable from a durable layer, zero orphans), with the causal loop
closed so the README guard no longer routes overflow into an uncapped neighbour. In the doctrine's
terms: `bounded_snapshot` + `generated_projection` + exact-history retention.

⇒ the existing enforcers are the **implementations**; the doctrine supplies the **common lifecycle
contract they were missing**. The adopted copy deliberately does not restate any rule PGEN already
enforces — a duplicated rule is the second authority the stop condition forbids.

## Goal (the tree's single deliverable)

Every long-lived PGEN document family has a declared lifecycle, owner, bounds, retrieval path and
executed verifier; the layer-A pointer holds only non-derivable current state; and a deterministic
check refuses any undeclared surface, over-limit path, or hand-maintained derivable field — with
whatever remains unimplemented recorded as **honest transition debt** with an owner and an ordered
frontier, never as a silent exemption.

## Acceptance Criteria

1. A project-owned `LIVE_DOCUMENT_SIZE_CONTAINMENT.md` at the repo root, neutral body preserved,
   with a PGEN authority note and **zero** donor thresholds, paths, ids, debt or conclusions.
2. The layer-A pointer carries no derivable-exact field, and its headroom is **measured**
   before→after rather than asserted.
3. A registered doctrine check enforcing derived-state containment, with positive **and** negative
   controls that make it REFUSE on a miss, wired into the unconditional gate (E3 + E4).
4. Existing enforcers mapped to lifecycle classes rather than duplicated.
5. Deferred scope recorded as transition debt with owner + ordered frontier.

## Task Tree

- ID: `LIVE-DOC-CONTAINMENT.1`
  Status: `done` (`PGEN-LIVE-DOC-CONTAINMENT-0001`, 2026-08-08)
  Goal: `ADOPT the doctrine as a project-owned copy + author the portable derived-state standard the director asked for, and open this tree. Copy the NEUTRAL body only; replace the donor's fenced local-adoption block with a PGEN authority note recording the measured trigger, the umbrella framing, the locality rule and the never-raise-a-cap rule. Follow the README-POLICY.4 precedent for where a portable standard lives in this repo (neutral standard at the repo root).`
  Acceptance: `neutral body byte-preserved from the donor's line 86 onward; donor-specific block REPLACED not appended; no donor threshold/path/id/debt/conclusion carried over (measured, not asserted); the tree registered in docs/TASK_TREE.md.`
  Verification: `done — LIVE_DOCUMENT_SIZE_CONTAINMENT.md at the repo root, 333 lines / 20 775 B. Donor block (donor lines 1-85) REPLACED by a PGEN authority note; neutral body taken verbatim from donor line 86 onward. DONOR-LEAKAGE MEASURED: grep -ic fsmgen = 3, and all 3 are inside PGEN's own authority note as PROVENANCE statements ("copied from the sibling fsmgen repository once, deliberately" / "no FSMGen threshold, path, surface id, task id, debt allowance, migration, registry value or retention conclusion has been carried over" / "evidence about FSMGen, not policy") — zero donor thresholds, paths, ids or conclusions in the normative body. AMENDED IN-LANE by `-0004` (see below). Also authored docs/DERIVED_STATE_CONTAINMENT.md (project-neutral, forwardable) — the standard the director requested, written so the donor project can incorporate it into its own containment doctrine. ⚠️ PLACEMENT CORRECTED IN-SLICE (director, 2026-08-08): first written to the repo root on the README-POLICY.4 precedent, moved to docs/ because the root had reached 18 markdown files. MEASURED before assenting: of the four portable standards at the root, DERIVED_STATE_CONTAINMENT.md is the ONLY one free to move — LIVE_DOCUMENT_SIZE_CONTAINMENT.md names "repository-root" as its authoritative copy IN ITS OWN adoption note, check_memory_architecture.sh tests for MEMORY_ARCHITECTURE.md at the root, and README_POLICY.md was placed there by director order (README-POLICY.4). The director picked the one that could move. Root 18 -> 17; it also matches the donor's own split (doctrine at root, companion standard under docs/).`
  Commit: `PGEN-LIVE-DOC-CONTAINMENT-0001`

- ID: `LIVE-DOC-CONTAINMENT.2`
  Status: `done` (`PGEN-LIVE-DOC-CONTAINMENT-0003`, 2026-08-08) — ⭐ headroom **88 B → 1 078 B** by removing scaling terms, not prose; and the demotion exposed a wrong frontier in another tree that layer A had been patching.
  Goal: `PHASE 1 — stabilize the resume pointer by removing SCALING TERMS, not prose. Three measured targets in MEMORY.md: (a) the push counter, a derivable-exact field that is wrong BY CONSTRUCTION (measured 185 stored vs 187 actual; recording "N ahead" requires a commit, making it N+1) -- DELETE, leave the derivation in place per docs/DERIVED_STATE_CONTAINMENT.md R3/R5; (b) the latest_commit narrative, 401 B duplicating CHANGES.md and the owning leaf -- reduce to a derivation + the leaf id; (c) the "routed, waiting behind product" roster, 499 B = 7% of the file and GROWING PER ROUTED FINDING (this session added to it) -- demote to a pointer at its canonical home docs/TASK_TREE.md + each tree's own frontier.`
  Acceptance: `every removed field replaced BY ITS DERIVATION so no resume question loses its answer (R5); before→after bytes MEASURED; the cap NOT raised; a fresh-session resume walked end-to-end to prove the pointer still routes correctly.`
  Verification: `done — ⭐ 7 080 B -> 6 090 B (44 -> 38 lines), headroom 88 B -> 1 078 B, a 12x improvement, and the cap was NOT touched. Contrast with the compression-only repair this tree was opened over: that bought 465 B by rewording; removing the SCALING TERMS bought 990 B more and stopped the growth.
  WHAT WENT, BY CLASS (docs/DERIVED_STATE_CONTAINMENT.md §2):
  * (a) DERIVABLE-EXACT, DELETED + derivation left in place per R3/R5 — the push counter (stored 185; `git rev-list --count origin/main..HEAD` said 187 when the slice opened and 189 by the time it landed, i.e. it drifted twice DURING its own repair, which is the by-construction class demonstrating itself) and the latest_commit narrative (401 B duplicating CHANGES.md + the owning leaf).
  * (a) DERIVABLE PROJECTION, DEMOTED — the "routed, waiting behind product" roster, 499 B = 7 % of the file, growing per routed finding. Canonical home docs/TASK_TREE.md; membership PROVEN recoverable before deletion (every routed tree present, every leaf in its own tree file) rather than assumed.
  * (b) PINNED-WITH-VERIFIER, KEPT and now labelled as such — the regex oracle tuple, legal precisely because REGEX-ORACLE-ANCHOR-SYNC fails on drift. It is the worked example of why (b) is not (a).
  * (c) NON-DERIVABLE, KEPT — active work unit, next action, blockers, standing tripwires.
  R5 DEMONSTRATED LIVE, not asserted: `git log -1 --oneline` -> the commit, `git rev-list --count origin/main..HEAD` -> 189, docs/TASK_TREE.md -> 116 tree rows. Every question the deleted fields answered is still answerable, with a value that is correct at the moment it is asked.
  ⭐⭐ AND THE DEMOTION FOUND A LATENT DEFECT — which is the argument for demotion, not merely a bonus. The roster carried "BOOK-PARAGRAPH-SHAPE (`.2` runs BEFORE `.1`)". Checking that it was recoverable before deleting it (the doctrine's "prove what will be retained") showed its tree encodes the order in its leaf headings (`.1` = "ROUTED — do not pull ahead of product", `.2` = "PRIORITY-FIRST when this lane is unparked") while that tree's own Status field read "frontier `.1`" — a CONTRADICTION the bounded resume pointer had been silently compensating for. ⇒ a layer-A roster was patching a wrong canonical source, and every reader who trusted the tree got the wrong frontier. FIXED AT THE SOURCE (BOOK-PARAGRAPH-SHAPE.md frontier `.1` -> `.2`, with the correction recorded there), so no surface has to carry the patch. This is the general lesson: a duplicated projection does not just cost bytes — it HIDES divergence in the thing it duplicates.`
  Commit: `PGEN-LIVE-DOC-CONTAINMENT-0003`

- ID: `LIVE-DOC-CONTAINMENT.3`
  Status: `pending`
  Goal: `Implement + register the DERIVED-STATE doctrine check per docs/DERIVED_STATE_CONTAINMENT.md §6 (the check contract; renumbered from §5 when §4 was added): fail on a governed surface containing a declared derivable-exact pattern, on a pinned duplicate with no registered verifier, on a declared-but-unexecuted verifier, and on a pinned value disagreeing with its source. ⛔ DECLARED pattern list, never a heuristic (a heuristic guessing which prose is a commit count fails OPEN, silently, in the passing direction). Ground truth REQUIRED: positive + negative controls that make it refuse on a miss -- and the negative control must be the REAL pre-fix MEMORY.md replayed from git show, the CTRL-1 shape README-POLICY.2 proved (a check over an already-fixed file returns 0 whether it works or is blind).`
  Acceptance: `registered in the doctrine driver (E3 pre-commit + E4 CI automatic tier); probes RED before / GREEN after; the retired-state control REJECTS the real pre-fix pointer; the <meta:mirror> doctrine count updated in lockstep.`
  Verification: `pending`
  Commit: `pending`

- ID: `LIVE-DOC-CONTAINMENT.4`
  Status: `pending`
  Goal: `Map PGEN's EXISTING enforcers onto the doctrine's lifecycle classes rather than duplicating them -- MEMORY-ARCH -> bounded_snapshot; README-STABILITY -> bounded_snapshot; LIVE-DOC-CURRENCY -> the currency verifier across classes; PUBLISHED-VERSION-CURRENCY -> maintained_reference; CHANGES.md / DEVELOPMENT_NOTES.md -> rolling_ledger; docs/decisions/ -> partitioned_canonical; git -> external_terminal. Publish the map; it is the artifact that prevents the second-authority failure.`
  Acceptance: `every governed surface carries lifecycle + owner + authority + bounds + verifier, or an explicit transition-debt record naming its owner and ordered frontier; no surface silently unclassified.`
  Verification: `pending`
  Commit: `pending`

## Deferred / transition debt (honest, per the doctrine's own §Phase-2.5)

⛔ NOT adopted in this pass, recorded rather than silently skipped: the full Phase-2 registry data
plane (JSONL surface registry, route/archive descriptors, ceiling-increase authorities, archive
terminals with retrieval proofs). PGEN's measured pressure today is the layer-A pointer, and the
doctrine is explicit that ceilings and topology must be derived from a **deliberately reviewed
survivor**, not adopted speculatively. Owner: this tree. Ordered frontier: `.2` → `.3` → `.4`,
then re-measure and decide whether a registry is earned. ⚠️ Re-open this record if live-doc
pressure appears on any surface **other than** `MEMORY.md` — that is the trigger condition, and
it is the evidence a registry would need.

## Blockers

- None.

## Decisions

- `2026-08-08` — **adopt the CONTRACT fully, implement INCREMENTALLY.** The director's *"I might
  not adopt everything, I understand"* is satisfied by the doctrine's own transition-debt
  mechanism rather than by a partial copy: the normative body is taken whole, and what is not yet
  implemented is recorded above with an owner and an ordered frontier. A partial copy would fork
  the standard; tracked debt does not.
- `2026-08-08` — **the repo root is reserved for CONTRACT-BOUND standards; everything else goes under
  `docs/`** (director: *"the repo root is a bit crowded now"*, at 18 root markdown files). A standard
  belongs at the root only when something REQUIRES it there — its own adoption note naming a
  repository-root authoritative copy (`LIVE_DOCUMENT_SIZE_CONTAINMENT.md`), an enforcer testing for
  it there (`check_memory_architecture.sh` → `MEMORY_ARCHITECTURE.md`), or an explicit director
  placement (`README_POLICY.md`, `README-POLICY.4`). `docs/DERIVED_STATE_CONTAINMENT.md` has no such
  binding, so it moves; this also mirrors the donor's own split (doctrine at root, companion standard
  under `docs/`). ⛔ This SUPERSEDES the blanket reading of the decision below — the `README-POLICY.4`
  precedent is about a DIRECTOR-PLACED file, not a general "standards live at the root" rule.
- `2026-08-08` — **the neutral standard lives at the repo root**, following `README-POLICY.4`'s
  precedent for `README_POLICY.md`. Cross-project standards are COPIED IN, never referenced across
  repos.

## Verification Log

- `2026-08-08` — `.1`: doctrine copy 333 lines / 20 775 B; donor-leakage grep = 3, all provenance
  in PGEN's own note, 0 in the normative body. `DERIVED_STATE_CONTAINMENT.md` authored at the repo
  root. Baseline for `.2`: `MEMORY.md` 7 080 B / 7 168 B cap = **88 B headroom (98.7 % full)**;
  measured scaling terms — routed roster 499 B, `latest_commit` 401 B, push counter wrong at rest
  (stored 185 vs `git rev-list --count origin/main..HEAD` = 187).

- `2026-08-08` — `.1` AMENDMENT (`-0004`): `docs/DERIVED_STATE_CONTAINMENT.md` gains **§4 — a duplicate
  HIDES divergence in its source**, the visibility argument, which is STRONGER than the size argument
  the standard was originally built on. Raised by `.2`'s `BOOK-PARAGRAPH-SHAPE` finding and, when the
  director asked whether the standard had actually been amended, measured as **not** amended — it had
  been flagged in conversation and left there, which is the failure [[feedback_every_finding_must_be_fixed_not_logged]]
  and the memory architecture's *"nothing important may live only in this conversation"* both name.
  Sections renumbered 4→5 … 9→10, the one cross-reference in `.3`'s Goal updated in lockstep, and a
  matching anti-pattern added. 209 → 248 lines.

## Commit Log

- `PGEN-LIVE-DOC-CONTAINMENT-0001` — `.1` adoption + the portable derived-state standard + tree opened.

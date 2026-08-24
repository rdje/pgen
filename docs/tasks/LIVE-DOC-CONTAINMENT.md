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
  Status: `done` (`PGEN-LIVE-DOC-CONTAINMENT-0006`, 2026-08-08) — ⭐ derived-state containment is now MECHANICAL (E2.6 sub-check, 6/6 probes, CTRL-1 rejects the real pre-fix pointer, CTRL-2 proves the ground truth refuses when blinded)
  Goal: `Implement + register the DERIVED-STATE doctrine check per docs/DERIVED_STATE_CONTAINMENT.md §6 (the check contract; renumbered from §5 when §4 was added): fail on a governed surface containing a declared derivable-exact pattern, on a pinned duplicate with no registered verifier, on a declared-but-unexecuted verifier, and on a pinned value disagreeing with its source. ⛔ DECLARED pattern list, never a heuristic (a heuristic guessing which prose is a commit count fails OPEN, silently, in the passing direction). Ground truth REQUIRED: positive + negative controls that make it refuse on a miss -- and the negative control must be the REAL pre-fix MEMORY.md replayed from git show, the CTRL-1 shape README-POLICY.2 proved (a check over an already-fixed file returns 0 whether it works or is blind).`
  Acceptance: `registered in the doctrine driver (E3 pre-commit + E4 CI automatic tier); probes RED before / GREEN after; the retired-state control REJECTS the real pre-fix pointer; the <meta:mirror> doctrine count updated in lockstep.`
  Scope_decision_2026-08-08: `⭐ BUILT AS AN E2.6 SUB-CHECK INSIDE scripts/check_memory_architecture.sh, NOT as an 18th doctrine — which is what the director's own phrasing ("MEMORY-ARCH sub-check") already implied, and it is the correct shape: derived-state containment on layer A IS a layer-A memory-architecture invariant, so it belongs to the enforcer that already owns layer A. ⇒ NO new doctrine row, NO driver registry entry, NO <meta:mirror> count change (the Acceptance clause about the mirror is therefore N/A, not skipped), and it inherits E3+E4 wiring automatically because MEMORY-ARCH is already in the automatic tier. The alternative -- a standalone 18th doctrine -- would have added registry churn and a second place to look for one layer-A rule, for zero enforcement benefit.`
  Verification: `done — the rule is now MECHANICAL, not a practice. 6/6 probes, re-runnable from the repo: bash docs/tasks/artifacts/live_doc_containment/run_derived_state_probes.sh.

  ## Acceptance Checklist (enforced)
  - [x] **REPRODUCE / ISSUE** — layer A stored `**push**: 185 ahead` while `git rev-list --count origin/main..HEAD` returned `187`, then `189` — measured twice during the very repair that removed it.
  - [x] **ROOT CAUSE (WHY + WHERE)** — WHY: the field is derivable-exact AND self-invalidating — recording "N ahead" requires a commit, which makes it N+1, so `git rev-list --count origin/main..HEAD` can never agree with a stored copy at rest; no cadence fixes arithmetic. WHERE: `MEMORY.md:41` (`push`) and `MEMORY.md:31` (`latest_commit`) at `ff96f98c~1`, both located by replaying the real pre-fix file with `git show ff96f98c~1:MEMORY.md` and re-scanning it. The class is `docs/DERIVED_STATE_CONTAINMENT.md` §3; ops/build-flow diagnosis family (shell + git), since a shell/docs defect yields no parse, no sample and no `error[EXXXX]`.
  - [x] **FIX** — minimal, declarative tier: a new `E2.6` block in `scripts/check_memory_architecture.sh` with a DECLARED two-pattern list (field-name form + value form, so a rename is still caught) and `bash -n` clean. No new doctrine, no driver change.
  - [x] **ADDRESSED (verified)** — before→after on the symptom: the real pre-fix pointer replayed via `git show ff96f98c~1:MEMORY.md` goes **ACCEPTED → REJECTED (exit 1)**, reporting **exactly 2** findings at lines 31 and 41 (deduped from 3 after a `sort -u`); the current pointer stays GREEN (`memory-arch: OK`, exit 0). Probes P1-P5, **6 passed / 0 failed**.
  - [x] **NO REGRESSION** — E2.6 is PURELY ADDITIVE, proven **byte-identical** rather than asserted: the pre-change enforcer (`git show HEAD:scripts/check_memory_architecture.sh`) and the post-change one were run on the SAME inputs and their verdicts `cmp`-compared — (A) on the current tree, both exit 0, output **byte-identical**; (B) on a fixture breaching the PRE-EXISTING byte cap (`MEMORY_POINTER_BYTE_CAP=100`), both exit 1, output **byte-identical** — so no pre-existing verdict (E2.1-E2.5) moved in either direction. Plus `bash scripts/check_doctrines.sh` → ALL 17 doctrines PASS (count unchanged — sub-check, not a new doctrine), `bash -n` clean, and P5 proves the compliant DERIVATION form is not a false positive, so R3/R5 remains implementable.
  - [x] **LOCKSTEP** — `docs/DERIVED_STATE_CONTAINMENT.md` §6 is the contract this implements; probe script added under `docs/tasks/artifacts/live_doc_containment/`. Book: N/A — no user-visible parser behaviour changes.

  ⚠️ GATE-VOCABULARY NOTE (measured, not waived): the first attempt to land this was BLOCKED by `TASK-ACCEPTANCE` — `NOREGRESS_SIG` accepts only parser-global tokens (`seeds 0/7/42|byte-identical|external corpus|shape-contract|spf=0|fully_certified|clippy`), none of which an ops/build-flow shell change can honestly cite. That is the known `GENERATED-LINT-CORRECTNESS.8` wrong-vocabulary defect (group 2's disease repeated in the NO-REGRESSION box). ⛔ It was NOT waived and NOT satisfied by ceremony: the old-vs-new `cmp` above is a GENUINE and in fact BETTER no-regression proof for an additive check — an added sub-check must not move any existing verdict — and it happens to be expressible in the gate's own vocabulary. Recorded because the next ops change may not be so lucky, and that is evidence for `GENERATED-LINT-CORRECTNESS.8`, not for a waiver.

  ⭐ THE CONTROL THAT MATTERS IS P4, NOT P3. P3 proves the check catches today's known defect. P4 blinds both patterns and proves the ground truth REFUSES (exit 1, "controls MISSED") instead of silently passing — without it, a future edit that breaks the scanner would look identical to a clean tree, which is the exact failure every other bound in this file was written to escape.`
  Commit: `PGEN-LIVE-DOC-CONTAINMENT-0006`

- ID: `LIVE-DOC-CONTAINMENT.4`
  Status: `pending` — ⛔ **PARKED as transition debt (director, 2026-08-08: *"After this I hope we will be done with these administrative documents so we can focus on real parsers' work"*).** It is pure classification with **no measured pressure** behind it and it blocks nothing; the doctrine's own §Phase-2.5 provides exactly this — record the deferral with an owner and an ordered frontier rather than pretend it is done. ⚠️ Re-open when live-doc pressure appears on a surface OTHER than `MEMORY.md`; that is the trigger, and it is the evidence a lifecycle map would need. ⭐⭐ **THE TRIGGER HAS SINCE FIRED (2026-08-10, `.5` below): `CHANGES.md` measured at 34 % omission (41 of the last 120 commits), on a surface `.4`'s own Goal classifies `rolling_ledger`.** So the "no measured pressure" premise of this parking no longer holds — the parking now rests **only** on the SV lane lock, which is a scheduling decision, not a technical one
  Goal: `Map PGEN's EXISTING enforcers onto the doctrine's lifecycle classes rather than duplicating them -- MEMORY-ARCH -> bounded_snapshot; README-STABILITY -> bounded_snapshot; LIVE-DOC-CURRENCY -> the currency verifier across classes; PUBLISHED-VERSION-CURRENCY -> maintained_reference; CHANGES.md / DEVELOPMENT_NOTES.md -> rolling_ledger; docs/decisions/ -> partitioned_canonical; git -> external_terminal. Publish the map; it is the artifact that prevents the second-authority failure.`
  Acceptance: `every governed surface carries lifecycle + owner + authority + bounds + verifier, or an explicit transition-debt record naming its owner and ordered frontier; no surface silently unclassified.`
  Verification: `pending`
  Commit: `pending`

- ID: `LIVE-DOC-CONTAINMENT.5`
  Status: `todo` — ⛔ **PARKED behind the SV lane lock** (routed IN by `SV-CORPUS-GRAD.3.26d`, 2026-08-10, `PGEN-SV-CORPUS-GRAD-0192`). Not worked here: the finding is an ops/live-doc defect, not an SV-release blocker, so it is OWNED and SCHEDULED rather than logged ([[feedback_every_finding_is_owned_and_scheduled_never_just_logged]]). ⭐ **Named re-open trigger: the SV lane lock lifting, OR any session that needs `CHANGES.md` to answer "what changed" and cannot trust it.**
  Goal: `CHANGES.md silently omits a third of the work. MEASURED over the last 120 commits (git log --format='%h%x09%s' -120, each subject's PGEN-<FAMILY>-<NNNN> slice id joined against grep -q on CHANGES.md): 41 MISSING / 79 HAS = 34% with no changelog entry. The omissions are not trivia -- PGEN-SV-CORPUS-GRAD-0189 is the DIRECTOR RULING that retracted a wrong grammar decision, and -0188/-0190 are its follow-ups; the whole retraction was invisible on the changelog surface while the retracted claim (-0187) sat there uncorrected. Also missing: 6 consecutive PGEN-MEMORY-000{1..6}, all 6 PGEN-LIVE-DOC-CONTAINMENT-000{1..6} (this tree's OWN commits), PGEN-HORIZON-0001/0003. DECIDE, then enforce: either (a) CHANGES.md's charter is "every slice", and a doctrine check joins the staged commit subject's slice id against the file -- cheap, structural, exactly the shape of the existing enforcers; or (b) its charter is "notable slices only", in which case that charter must be WRITTEN DOWN in COMMIT.md and the 34% stops being drift. ⛔ What is not acceptable is the current state: COMMIT.md lists CHANGES.md as a required surface with an unstated "as needed" escape, so a third of the history is missing by accident rather than by policy, and nothing measures it.`
  Acceptance: `the charter is written down in COMMIT.md; if (a), a check_*.sh registered in scripts/check_doctrines.sh with RED-before/GREEN-after probes and a ground-truth control (a commit whose id is present must PASS, one whose id is absent must FAIL -- blinding the join must make the control REFUSE, not silently pass); if (b), the exemption criteria are stated and the three SV entries backfilled by -0192 are confirmed as correctly in-charter.`
  Verification: `pending`
  Commit: `pending`

- ID: `LIVE-DOC-CONTAINMENT.6`
  Status: `todo` — ⛔ **PARKED behind the SV lane lock** (routed IN by `ENGINE-UNIVERSAL-SERVICES.8`, 2026-08-11). Not worked there: the *instances* were corrected in-lane as ordinary lockstep, but the *class* is an unenforced derived-state breach and belongs to this tree. ⭐ **Named re-open trigger: the SV lane lock lifting, OR the next time a published suite/roster count is cited as evidence.**
  Goal: `A DERIVED COUNT stored in prose has no verifier, and three of them had silently drifted. MEASURED while doing lockstep for the combinator suite: docs/DERIVED_STATE_CONTAINMENT.md governs exactly this class ("a stored fact that a tool owns exactly is not information -- it is a cache with no invalidation"), and its E2.6 enforcer covers layer-A MEMORY.md ONLY. Outside layer A, nothing checks. Drift found, all three by re-deriving from the source rather than by any gate: TOOLBOX.md 1.7 said "28 small synthetic isolating grammars" and "28/28 CLEAN" while `grep -c "^    CombinatorCase {" rust/src/parse_harness_combinator_suite.rs` = 31 before this session's addition (32 after) -- stale by 3, i.e. the three associativity cases GENERATED-LINT-CORRECTNESS.2 added were never reflected; docs/book/src/parse-harness.md carried the SAME 28 twice, once as a prose count and once as an ITEMISED derivation (16 + 4 + 3 + 2 + 2 + 1) that was internally consistent and simply never extended; and the same chapter said the semantic suite had "29 isolating grammars" while `grep -c "^    SemanticCase {" rust/src/parse_harness_semantic_suite.rs` = 36 -- stale by 7. ⛔ The failure direction is the dangerous one: a published count that UNDERSTATES coverage reads as a smaller, weaker suite, so nobody is prompted to check it, and the drift is invisible until someone happens to re-derive it. DECIDE, then enforce: either (a) these counts are DERIVED at build time (the suites already expose their case arrays, and both gates already assert coverage-completeness internally), or (b) they are PINNED WITH A VERIFIER in the docs/DERIVED_STATE_CONTAINMENT.md class-(b) sense -- a check that re-counts the arrays and fails on disagreement, exactly as REGEX-ORACLE-ANCHOR-SYNC does for the oracle tuple.`
  Acceptance: `the class is decided and written down; if (b), a check re-derives every published suite count from its source array and fails on drift, with RED-before/GREEN-after probes and a ground-truth control (blinding the count source must make the control REFUSE, not silently pass -- the CTRL-2 shape `.3` established); the three corrected instances (TOOLBOX 32/32, book 32 structural, book 36 semantic) are confirmed as re-derived rather than re-typed.`
  Verification: `pending`
  Commit: `pending`

⭐⭐ **AND THIS FIRES `.4`'s OWN RE-OPEN TRIGGER, stated verbatim in its Status: *"Re-open when
live-doc pressure appears on a surface OTHER than `MEMORY.md`."*** `CHANGES.md` is a surface other
than `MEMORY.md`, the pressure is measured (34 % omission), and `.4`'s Goal already assigns
`CHANGES.md` the `rolling_ledger` lifecycle class — a class whose whole contract is *the ledger is
complete for its declared scope*. ⇒ `.4` is no longer "pure classification with no measured pressure
behind it"; it now has its evidence, and `.5` is the concrete instance that would test any map `.4`
publishes. ⛔ Both stay PARKED behind the SV lane lock — recorded so the trigger is not re-derived
from scratch next time, per this tree's own §Phase-2.5 discipline.

- ID: `LIVE-DOC-CONTAINMENT.7`
  Status: `todo` — ⛔ **PARKED behind the SV lane lock** (opened 2026-08-20 session #250 by the
  director's re-verification challenge on `PGEN-SV-CORPUS-GRAD-0254`'s findings callout;
  `PGEN-LIVE-DOC-CONTAINMENT-0008`)
  Goal: `E2.6 declares ONE quantity family — git commit distances — and every OTHER number this
  repository re-derives on every commit can be stored in layer A with nothing objecting.`

  ⛔⛔ **MEASURED, ON A LIVE INSTANCE, NOT HYPOTHESISED.** At `HEAD~1` of
  `PGEN-SV-CORPUS-GRAD-0254`, `MEMORY.md` carried **`axis-2 bar 288`** in its `active_work_unit`
  bullet *and* **`axis-2 bar **282**`** in its standing-facts bullet — the file disagreed with
  itself — while `bash scripts/check_sv_corpus_denominator.sh` derived
  `7556/2459/6321/4393/282` on every commit. Two stored copies of one derived quantity, one of them
  stale by 6, and **every doctrine was GREEN across it**.

  ⭐ **ROOT CAUSE, and it is not that E2.6 is wrong.** `scripts/check_memory_architecture.sh`'s
  `derived_state_patterns` is DECLARED, never inferred, and that is a documented, correct choice —
  a heuristic guessing which prose is a derived number would fail OPEN, silently, in the passing
  direction. The gap is that the declaration covers exactly the quantity `LIVE-DOC-CONTAINMENT.2`
  was repairing (`push` / `unpushed` / `commits ahead|behind`) and **was never extended as new
  derive-every-commit quantities appeared**. ⇒ enumeration without a re-derivation of the
  population is the same failure `LIVE-DOC-CURRENCY`'s instrument B was rebuilt to escape, where
  one population measured 10, then 16, then 18, every miss silent.

  ⭐ **AND THE POPULATION IS ENUMERABLE FROM THE DOCTRINES THEMSELVES**, which is what makes this
  bounded rather than open-ended: the doctrines that publish a derived LIVE ANCHOR already name it
  and already hold designated surfaces equal to it — `SV-CORPUS-DENOMINATOR`'s
  `adjudicated/routed/no-verdict/dark/bar` tuple, `PARSE-COST-RATCHET`'s LR-family share, the
  `REGEX-ORACLE-ANCHOR-SYNC` tuple. ⚠️ The last of those is the sanctioned **class-(b)** case and
  must stay legal: layer A carries `2189/1879/262/48` deliberately, because a gate fails on drift.
  So the rule is not *"ban derived numbers from layer A"* — it is *"a derived number in layer A is
  legal only when a verifier fails on its drift"*.

  Acceptance: `(a) E2.6's declared list is DERIVED from the doctrines that publish a live anchor
  rather than hand-typed, or — if derivation is refused — the hand list carries the anchor marks of
  every such doctrine and a two-sided check fails when a doctrine publishes an anchor the list does
  not name; (b) the class-(b) exemption is expressed as "a verifier exists that fails on drift",
  not as a literal allow-list of digits; (c) probes: the real pre-fix layer A (git show
  HEAD~1:MEMORY.md at PGEN-SV-CORPUS-GRAD-0254) must go ACCEPTED -> REJECTED, the regex tuple must
  stay ACCEPTED, and a compliant pointer that cites the derivation must stay ACCEPTED.`

  ⚠️ **HONEST BOUND on what was already done**: `-0254` removed the stale copy and replaced it with
  its derivation, so there is no number left in layer A to rot — that is the R3/R5 fix and it is
  strictly stronger than correcting the digits. What it does NOT do is stop the next stored copy of
  the next derived quantity, which is why this leaf exists rather than the instance being closed.

- ID: `LIVE-DOC-CONTAINMENT.8`
  Status: `todo` — ⛔ **ROUTED, NOT WORKED** (opened 2026-08-24 by `SV-CORPUS-GRAD.13e.4`,
  `PGEN-SV-CORPUS-GRAD-0288`). The SV lane lock binds work rather than conversation, so this is
  parked here with its measurement rather than fixed in an SV commit.
  Goal: `A TRACKED MARKDOWN TABLE SILENTLY DROPS CELLS, AND 25 ROWS OF THE RELEASED-PARSER BUG LEDGER ARE DOING IT NOW.` GFM's table rule is that a row with MORE cells than the header has the excess **ignored** — not rendered, not warned about. `docs/contracts/PGEN_RELEASED_PARSER_BUG_LEDGER.md` declares **13** columns, and a census of its 192 data rows (splitting on unescaped `|` only) finds **25 rows — 13 % — carrying 14 to 37**. Every one of them is losing its rightmost cells in the rendered book, and the rightmost column is **Notes**, which is where the consumer-impact statement lives. ⇒ a downstream reader of the ledger is being shown a row whose *"what this means for you"* was deleted by the renderer.
  Routing_evidence: `MEASURED, not inferred. The census is one command over the tracked file; the offenders are REGEX-0003/0010/0016/0035/0036/0037/0038/0039/0040/0064/0068/0099/0105/0106/0108/0112 and SV-0008/0034/0035/0042/0043/0047/0048/0050/0052, with REGEX-0105 at 37 columns. ⭐ FOUND BY WRITING ONE: `.13e.4` authored the SV-0069 row by copying the shape of its immediate neighbour SV-0068 — which was itself 14 columns — so the defect REPRODUCED ITSELF THROUGH IMITATION, which is how it reached 25. Both of those rows were repaired in -0288 (cells 11+12 merged into the single Fix Proof column the header declares); the other 23 are this leaf's. ⛔ The two causes are DIFFERENT and a fix must not conflate them: an unescaped `|` inside PROSE (a grammar alternation quoted in a Root Cause cell) needs escaping, while a genuine extra CELL means the author invented a column. Only reading each row separates them, which is why this is a leaf and not a sed.`
  Acceptance: `(a) every data row in every tracked contract/ledger table has exactly the column count its own header declares, measured by a re-runnable census that splits on unescaped pipes; (b) the census is WIRED — an arity check belongs with the other live-doc guards, because this defect is invisible to every existing one: it breaks no build, fails no gate, and the rendered page looks fine (the missing cell leaves no gap); (c) the check is two-sided — a row with FEWER cells than the header is padded silently by GFM and is equally a defect, so it must fail too; (d) prove the control can go RED on a real pre-fix row replayed from git.`
  Blocked_by: `the SV lane lock (director 2026-08-21, "focus on the SV lanes"). Nothing technical.`

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

- `2026-08-08` — `.1` FORWARD-READINESS AUDIT (`-0005`), run because the director asked whether the
  standard could be forwarded NOW. Neutrality VERIFIED by measurement, not assertion: zero project
  names, zero repo-specific identifiers/paths/`[[wiki-links]]`, both worked-evidence blocks labelled
  *"evidence about one project, not portable policy"*, neighbouring doctrines named generically.
  ⛔ BUT THE AUDIT FOUND A REAL DEFECT: §4's corollary cited a *"pre-deletion recoverability check
  (§6)"* — and NEITHER §6 nor §7 defined one. Adoption step 4 covered R5 (the reader keeps the
  answer); nothing covered *prove it is recoverable AND check the source for the divergence the
  duplicate was hiding*. A dangling forward reference to a step the document never wrote down —
  in the very document that had just been amended to make that step the point. FIXED by adding
  the step (new §7·4, steps renumbered 4→5…6→7) rather than by softening the reference, and the
  citation repaired to `§7 step 4`. ⇒ forwardable.

## Commit Log

- `PGEN-LIVE-DOC-CONTAINMENT-0001` — `.1` adoption + the portable derived-state standard + tree opened.

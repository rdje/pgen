# Live-Document Size-Containment Review — PGEN

- Project: PGEN (EBNF-driven parser + stimuli generation)
- Repository type and primary languages: Rust, Bash, Markdown; ~1,770 tracked durable-layer files
- Review date: 2026-07-31
- Reviewer role: sole engineer / guarantor, PGEN
- Material inspected: the packet in full; `live-document-size/scripts/check_live_document_size.pl`
  (856 lines, read for the transition-debt algebra rather than trusting the packet's prose);
  `doctrine/live_document_size/surfaces.jsonl` (all 20 records, measured); `README_POLICY.md`;
  plus PGEN's own measurements from the `LIVE-MEANS-LIVE` and `README-POLICY` task-trees
- Overall verdict: **accept with changes** — two of them before wider reuse

## Executive assessment

The invariant is right and the packet is unusually honest. Ten self-declared limitations, an
explicit "please be adversarial," and L10 volunteering that the doctrine can bias toward
preserving obsolete surfaces — that is not how most architecture packets are written, and it is
the reason this review can be short and specific instead of exploratory.

⭐ **You should know that the "measured adopter" in *Origin of the problem* is us.** The
1,547,057-byte file that was 94.7 % dated changelog is PGEN's `LIVE_ACHIEVEMENT_STATUS.md`. So
this review is not an outside opinion about a hypothetical — it is the downstream report from the
one case history the packet is built on. What happened next is the single most useful thing we can
send back: **we did not partition that file. We deleted it**, three days ago, after proving all
467 slice IDs it cited were reachable from a durable layer without it (0 orphans, re-measured at
the moment of deletion rather than quoted — the figure had moved from 452 since the first census).
The correct containment outcome for the document that motivated this entire architecture was
`delete`. **That is L10, confirmed empirically rather than suspected**, and it is why F-none-of-
the-below matters less than the answer to Q27.

Two findings are worth blocking on. **F1** is a schema defect that makes the registry
self-undermining, and it is already visible in your own committed data. **F2** is a whole failure
mode the architecture does not address: it bounds *size* and *routing*, but never *truth* — a
surface can satisfy all thirteen claimed safety properties and be entirely false. Everything else
is improvement or agreement.

## Findings

### F1 — `budget` holds two incompatible kinds of number, and the sick ones won

- Severity: **blocking**
- Applies to: JSONL schema / checker / doctrine wording
- Evidence: the algebra at `check_live_document_size.pl:492-495` enforces
  `baseline[d] <= budget[d]` and `baseline[d] + max_growth[d] <= budget[d]`. Therefore a surface
  that is *already* oversized at adoption can only be admitted as debt by setting its budget at or
  above its sick size. Measured across all 20 records in `surfaces.jsonl`:

  | surface_id | state | `budgets.lines_each` | `baseline.lines_each` |
  |---|---|---:|---:|
  | `root_documents` | rollover_debt | **38,000** | 34,509 |
  | `engineering_rationale` | rollover_debt | **38,000** | 34,509 |
  | `change_history` | rollover_debt | **35,000** | 31,799 |
  | `fact_index` | structural_debt | **20,000** | 15,541 |
  | `shipped_behavior` | rollover_debt | 20,000 | 18,660 |
  | … | | | |
  | `active_resume` | normal | **60** | — |
  | `enforced_rules` | normal | **300** | — |
  | `diagnostics` | normal | **400** | — |
  | `rationale` | normal | **512** | — |
  | `readme_entrypoint` | warning_debt | 275 | 246 |

- Consequence: the doctrine whose purpose is to bound live documents ships a registry declaring a
  **38,000-line-per-file hard limit**, at `hard_pct: 100`. A 34,509-line mandatory read is
  *compliant*. Meanwhile `active_resume` is bounded at 60 in the same field with the same name and
  the same semantics. **Nothing in the schema distinguishes a reviewed health target from a
  quarantine ceiling** — they differ by a factor of 633 and are indistinguishable to a checker, to
  a reader, and above all to the next adopter, who will copy this registry as a template. Note that
  `readme_entrypoint`'s 275 genuinely *is* a health target (derived from a deliberately reviewed
  246-line survivor, per the packet). So the field demonstrably carries both kinds today.

  L9 is the tell. *"Their current caps are stop-growth ceilings, not recommended healthy defaults …
  seeing 'under hard limit' must not be read as 'well sized'"* is a **prose bandage on a schema
  defect**. It asks every future reader to remember, correctly and forever, which of two visually
  identical numbers they are looking at. That is the same class of thing this architecture exists
  to stop being necessary.

- Recommended change: split the field.
  - `budget[d]` — the reviewed **health target**. May only ever *decrease*. Never derived from a
    measurement of a sick file.
  - `ceiling[d]` — the **stop-growth bound**, defined as `baseline[d] + max_growth[d]`, which is
    what the checker actually enforces today.
  - Drop `baseline <= budget` entirely. `ceiling > budget` becomes the *normal, expected* shape of
    a debt record and is exactly the signal you want visible: this surface is 63× its target.
  - Make `ceiling` a **two-sided ratchet**: fail when actual exceeds it (regression, as today) —
    *and also fail when actual has fallen well below it and the ceiling was not lowered*, with the
    message "lower the ceiling." That converts L9's promise (*"final migration must remeasure and
    lower legacy headroom"*) from an intention into a check that fires.

  PGEN ships this two-sided shape already, in a different domain: our envelope-differential gate
  fails on a count above its per-grammar ceiling **and** on a count below it, and both directions
  were made to fire on purpose before the ratchet was trusted. It is the mechanism that stops a
  legacy allowance from quietly becoming permanent, which is the one thing `baseline`-immutability
  alone cannot do — you froze the floor and left the roof free.

  This also answers **Q10/Q11** as a side effect: once `ceiling` is the enforced bound and `budget`
  is the target, `hard_pct` has no remaining job. Delete it. L2 disappears rather than being
  decided.

### F2 — the architecture bounds size and routing, but never truth

- Severity: **major**
- Applies to: doctrine / checker
- Evidence: measured in PGEN today, over tracked Markdown (`git ls-files`, so submodule-blind by
  construction): **16 files whose own self-declared `Last updated:` is refuted by the newest date
  in their own content.** A sample, worst first:

  | file | declares | newest content date | drift |
  |---|---|---|---|
  | `docs/reference/PGEN_SOTA_IMPLEMENTATION_ROADMAP.md` | 2026-04-22 | 2026-07-31 | ~3.3 months |
  | `PGEN_USER_GUIDE.md` | 2026-04-17 | 2026-07-29 | ~3.4 months |
  | `docs/reference/PGEN_ANNOTATION_NORMATIVE_SPEC.md` | 2026-03-26 | 2026-07-09 | ~3.5 months |
  | `COMMIT.md` | 2026-05-14 | 2026-07-30 | ~2.5 months |

  The first is the roadmap our own session bootstrap makes **mandatory reading**. Every one of
  these 16 files is comfortably inside every size bound, correctly routed, and would satisfy all
  thirteen safety properties in your §"Safety properties claimed" without a single amendment.

- Consequence: "bounded" and "current" are independent properties, and the packet only mechanizes
  the first. The failure mode you set out to fix — *a live document that stopped being live* — has
  two halves. Size rot is the half you measured because it is the half that hurt. Staleness is the
  other half, it is cheaper to detect, and right now nothing in the architecture looks at it. A
  reader who trusts a bounded, route-closed, coverage-complete surface is trusting a claim the
  checker never made.

  L6 (*"semantic quality is not reducible to size"*) reads as pre-emptive cover for this, but it
  does not apply: **self-refutation is not semantic quality.** It requires no judgment, no
  baseline, and no threshold. The document contradicts itself, and a five-line check can say so.

- Recommended change: add two instruments. Both are baseline-free and threshold-free, which is why
  we prefer them to the byte cap we had originally designed for this job.

  **Instrument B — self-refutation.** Extract a `Last updated:` declaration; extract the newest
  `YYYY-MM-DD` anywhere in the body; fail when the declaration is older. No external reference
  point exists or is needed — *the file's two halves contradict each other.* One caveat from our
  own calibration, worth stating because it cost us a wrong number an hour ago: anchor the
  extractor to your project's **actual** declaration spelling. Our first pass required the date to
  follow the colon directly and silently missed ~50 files written as ``- Last updated: `2026-05-31` ``
  — backtick-quoted. We caught it only because a prior measurement said 10 and the new one said 8,
  so the disagreement was visible. An instrument with no ground truth is a confident guess.

  **Instrument A — shape versus declared charter.** Count *distinct* dates in a surface. Measured
  in PGEN: the three healthy book destinations score 2 / 1 / 1, `README.md` scores 0, the rotted
  tracker scored 108, `CHANGES.md` scores 175 and `DEVELOPMENT_NOTES.md` 137. That is a ~50× gap
  with nothing in between, so **no threshold is chosen by a human** — the population is bimodal and
  the split falls out of the data. Pair each surface with its declared lifecycle: `rolling_ledger`
  is *permitted* to be log-shaped, `bounded_snapshot` is not. `CHANGES.md` scoring 175 is correct
  and must not fire; that is what the charter pairing is for.

  ⚠️ Note what a byte cap would have said instead: our `gate-flow.md` is the **largest** of the
  three healthy destinations at 34 KB and would have ranked worst of them. Its date count says it
  is fine. Size and rot are different questions, and only one of them is what you actually want to
  know.

  Put both behind a shrink-only ratchet at adoption (ours would start at 16 and may only go down),
  or every adopter's first run blocks their own commit — including, today, ours.

### F3 — route closure validates the routes you remembered; the rot uses the one you didn't

- Severity: **major**
- Applies to: doctrine / route registry / checker
- Evidence: this is the actual mechanism by which PGEN's 1.5 MB file grew, and it is worth being
  precise about because the packet currently records the *outcome* ("pressure had merely moved")
  without the *mechanism*. Our README guard, on breach, printed a routing hint to the author:
  `family status / Done-bar claims … LIVE_ACHIEVEMENT_STATUS.md`. That hint is a real overflow
  edge, with real traffic, that moved real content — and it is **invisible to a hand-authored route
  registry**, because nobody declared it. It lived in an error message.
- Consequence: **Q4, answered directly — yes, an unbounded sink can still hide.** Not *behind* the
  route graph; *beside* it. Your checker proves that declared edges terminate at declared surfaces
  and that the graph is acyclic. It cannot prove the declared graph is the *whole* graph, and the
  undeclared edge is the one that carries the rot, because an edge nobody wrote down is an edge
  nobody bounded.
- Recommended change: derive candidate edges from the enforcers' own **output text**, not only from
  the registry. Any guard that names a destination path in a hint, a routing table, or a failure
  message is defining a route; fail when such a named destination is not a declared surface. This
  is cheap (grep the enforcer sources for path-shaped literals) and it closes the one gap that the
  packet's own origin story is an instance of. Generalized: *capping a file is half a fix; the
  other half is checking where the overflow lands* — including when the pointer lives in a string.

### F4 — Q6 / L1: do not cap the control plane. Make it incapable of holding prose

- Severity: moderate
- Applies to: JSONL schema / L1 remediation
- Evidence: PGEN's fix for the same problem was to move the one load-bearing value — a single
  hand-authored status word per family — out of a Markdown table cell and into a JSON field with an
  enumerated domain. The containment did not come from a byte cap. It came from the schema: **you
  cannot leak 856 dated changelog entries into a field whose permitted values are six strings.**
- Consequence: L1 lists three remediations and puts *"add bounded registry metadata"* first. That
  is the weakest of the three. A `max_bytes` on the registry is a second number to maintain, and it
  bounds the symptom (size) rather than the mechanism (a container that accepts arbitrary prose).
- Recommended change: prefer the third option. Constrain the *value domains* — enums, integers,
  bounded-length identifiers, no free-text field without a declared max length. Then registry size
  is a consequence of record count, which `max_records` already bounds honestly. Add byte caps only
  where a free-text field genuinely must exist (`owner`, `_why`).

### F5 — prefer `file` retrieval over `version_object` for anything that must actually survive

- Severity: moderate
- Applies to: archive contract / L4 / Q15
- Evidence: your own first migration split the two cases, and — I think accidentally — split them
  correctly in one direction and not the other. The 844 IAL2 nodes went to a content-addressed
  Markdown segment that is **self-sufficient**: filename is the digest, content is in-repo, nothing
  needs a revision to be reachable. The 540 completed-index rows went to a `version_object`, which
  validates only while that revision survives shallow clone, GC, and history rewrite.
- Consequence: two archive contracts with materially different durability are presented as peers.
  PGEN paid a smaller version of this: a ground-truth control that pinned its expected state to
  `rust/target/` — regenerable build output — went silently red for a reason having nothing to do
  with the instrument it guarded, and its sibling that pinned a *tracked* fact is still green. The
  general rule we banked: **a control, or an archive, that depends on a mutable substrate is not a
  control, or an archive.** Git is a far more durable substrate than build output, but it is still
  mutable by policy.
- Recommended change: state a preference order in the doctrine — `file` (content-addressed, in
  repo) > `version_object` (convenience pointer) > `external`. Treat `version_object` as a
  *locator*, not as the retention guarantee, and require anything whose loss would be
  unrecoverable to use `file`. That makes L4 a decision rather than an open risk.

### F6 — L5 is prose where it could be mechanical

- Severity: moderate
- Applies to: checker / portability
- Evidence: L5 correctly identifies that presence-of-verifier is not execution-of-verifier, and
  mitigates it with *"the local doctrine driver must execute the real verifier independently."*
  "Must" is not a check. PGEN adopted a doctrine (`GATE-REACHABILITY`) for precisely this after
  finding **three gates that nothing invoked — by accident, one per session**; one of them had been
  unable to complete for 1,371 commits, and separately, 8 of our 13 registered doctrines had no
  automatic lane because a CI workflow re-typed a list of enforcer names instead of calling the
  driver.
- Consequence: **Q20, answered** — this is the important defect that can still pass. An adopter
  copies the neutral core, declares verifiers, sees green, and never notices that nothing runs
  them. The packet says an adopter "would have a weaker guarantee"; in practice they would have
  *no* guarantee and a green tick, which is worse than none.
- Recommended change: require every declared verifier to be **reachable from the driver registry**,
  and check that mechanically — a check nothing invokes is indistinguishable from a check that does
  not exist. Corollary, learned expensively: the driver must be called *as a driver*; any place
  that re-enumerates enforcer names will drift, and new checks will silently inherit no lane.

### F7 — Q9: do not add a formal JSON Schema

- Severity: minor
- Applies to: schema / portability
- Evidence / consequence: the strict parser **is** the schema, and it is executable. A second
  declarative schema is a second artifact that must agree with the first, which means it is an
  artifact that can disagree with the first. PGEN keeps a `single_source_note` in one of its own
  registers for exactly this reason, after twelve assertions rotted that way.
- Recommended change: none — keep the strict parser as the single source. If you want portability,
  export the schema *from* the parser rather than maintaining it alongside.

### F8 — the evidence map is a 17-row promise that nothing checks

- Severity: observation
- Applies to: packet / repo hygiene
- Evidence: I spot-checked five of the seventeen evidence paths; all five resolve today. That is
  the point — *today*. PGEN added a meta-check comparing its doctrine registry against its
  human-readable mirror and discovered on the first run that 3 registered doctrines had no row and
  1 row named an id the registry no longer carried. The mirror had claimed to be "kept in lockstep"
  in its own header for months.
- Recommended change: a four-line check that every path in the evidence map exists. Cheap, and it
  makes the packet's own claims the same kind of thing it asks of everyone else's documents.

## Credit where it is due

Both instruments from F2, run against the packet itself: **1 distinct date, no self-refutation.**
It practises what it argues. And L10 is the most valuable section in the document — it is the one
that made us re-read our own conclusion rather than nod along, and our data confirms it.

## Question responses

Answering only what we can inform, per your instruction.

- **Q1** — Yes, with F2's amendment: the invariant should be *bounded, current* live view over an
  addressable durable store. Bounded-but-false is a live counterexample, 16 of them here.
- **Q2** — The seven classes covered every PGEN surface we tested. `frozen_legacy` is the one we
  would have most wanted a year ago and would advise adopters to reach for early.
- **Q4** — No, not fully. See F3: the edge that carried our rot lived in an error-message hint and
  no route registry would have contained it.
- **Q5** — JSONL over TSV, agreed, and this is the least consequential question in the packet. The
  format is not where the risk is; F1 and F2 are, and both are format-independent.
- **Q6** — Not metadata caps. Constrain value domains instead — see F4.
- **Q7** — Yes. Strict unknown-key rejection is worth the cost; a typo'd key that silently does
  nothing is the same failure class as a check that never runs.
- **Q9** — No. See F7.
- **Q10 / Q11** — Both dissolve under F1. Once `ceiling` is the enforced bound and `budget` is the
  reviewed target, `hard_pct` has no job; delete it rather than deciding its semantics.
- **Q12** — Five dimensions are sufficient for size. The missing dimension is not a size dimension;
  it is currency (F2).
- **Q13** — Immutable baseline closes the floor and leaves the roof free. The gaming path that
  remains is not gaming at all, it is drift: the budget was set to the sick number to admit the
  debt, and nothing ever forces it down again. Two-sided ratchet (F1).
- **Q15** — Not for anything unrecoverable. See F5.
- **Q19** — F2's instrument A, if shipped without the charter pairing: it would fire on `CHANGES.md`
  forever, which is correct behaviour for a wrong reason and would teach people to waive it. The
  charter pairing is not a refinement, it is what makes the instrument usable.
- **Q20** — L5, per F6.
- **Q21** — Yes, unconditionally, and call the *driver* rather than a list of enforcer names. We
  have the scar: 8 of 13 doctrines had no automatic lane, and every doctrine registered after that
  point inherited none.
- **Q24** — Proportional for F1/F2/F3. Not obviously proportional for the full task-tree sealing
  extension — see "Final recommendation."
- **Q25** — **Accept with named changes**: F1 and F2 before wider reuse.
- **Q26** — F1. It is visible in your committed data right now, and an adopter copying
  `surfaces.jsonl` as a template inherits a 38,000-line "limit."
- **Q27** — **Yes, unconditionally**, and this is the finding we would most want you to take from
  PGEN's experience. The document that generated this entire architecture should have been, and
  ultimately was, `delete`. Had we owned your machinery first, we would have partitioned a 1.5 MB
  file into compliant shards and preserved 856 obsolete tracker notes forever, under budget, route-
  closed, coverage-complete, and worthless. **A containment architecture without a utility gate is
  a very rigorous way to keep the wrong thing.**
- **Q28** — What we required, and would require again: an ID-level census proving every durable
  reference is reachable from another layer *without* the file, run **immediately before** the
  delete rather than quoted from the earlier survey. Ours moved from 452 to 467 between the first
  census and the deletion, five days apart. Quoting the old number would have certified a delete
  against a smaller file than the one actually removed. Also: a negative control proving the census
  can fail — ours refuses on a planted orphan.
- **Q29** — `LIVE_ACHIEVEMENT_STATUS.md` and `ROADMAP_STATUS.md`, in that order; they are the two
  whose role your own table already says has moved. The probe we would run first is not a content
  diff — it is the **ID-level reachability census** from Q28, because it answers "is anything here
  unique?" without anyone having to read 16,618 lines and form a judgment.
- **Q30** — When it is *narrative* rather than *record*: a human explaining why a release matters,
  which task-trees and Git cannot produce. Ours had stopped being that. If a changelog's entries
  are mechanically derivable from commit subjects and task leaves, it has become a projection and
  should be treated as one.
- **Q31** — On a schedule, and cheaply. Instrument A is the scheduler: a surface whose distinct-date
  count crosses into log shape has *told you* its role changed, without anyone reviewing it.

## Measurements from this project

| Measurement | Value |
|---|---|
| The adopter file cited in your §"Origin of the problem" | `LIVE_ACHIEVEMENT_STATUS.md`, PGEN |
| Its size at deletion (2026-07-31) | 1,563,641 B / 1,684 lines / 856 dated tracker notes |
| Share that was dated changelog | 94.7 % |
| Outcome chosen | **`delete`** — not partition, not archive |
| Slice IDs it cited, reachable from a durable layer without it | **467 / 467**, 0 orphans, re-measured at the moment of deletion |
| Durable-layer files scanned to prove it | 1,765 files / 124.3 MB |
| README before → after its own containment | 510 lines / 48,811 B → 178 lines / 8,313 B |
| Resume pointer, line-cap-only era | 60 lines (passing, at its 60-line cap) / **138,403 bytes** |
| Tracked `.md` files self-refuting their `Last updated:` | **16** (worst: ~3.5 months) |
| Distinct-date counts — healthy status surfaces | 0, 1, 1, 2 |
| Distinct-date counts — logs (correct, by charter) | 137, 175 |
| Distinct-date count — the rotted tracker | 108 |

The 60-line / 138,403-byte row is offered as independent confirmation of your "line and byte limits
are both mandatory" argument: that file's own header called it a *bounded resume pointer* while it
carried 2,306 bytes per line, one line of 18,816 bytes, and a green guard the entire time. Your
§"Five budget dimensions" reasoning is correct and we have the scar tissue to prove it.

## Proposed wording or schema changes

Surface record, F1 applied:

```json
{
  "surface_id": "engineering_rationale",
  "lifecycle": "rolling_ledger",
  "budgets":  { "lines_each": 2000,  "bytes_each": 262144,  "…": "the reviewed HEALTH TARGET; may only decrease" },
  "ceiling":  { "lines_each": 38000, "bytes_each": 4194304, "…": "baseline + owned allowance; STOP-GROWTH; two-sided" },
  "baseline": { "lines_each": 34509, "…": "immutable adoption measurement" },
  "transition": { "owner": "…", "max_growth": { "lines_each": 3491 } }
}
```

- drop the `baseline[d] <= budget[d]` constraint; `ceiling > budget` is the expected shape of debt
  and is the signal you want visible;
- enforce `actual[d] <= ceiling[d]` (as today) **and** `ceiling[d]` must be lowered when `actual[d]`
  falls materially below it — the message is "lower the ceiling", and it is what makes L9 real;
- delete `hard_pct` (see Q10/Q11);
- report both numbers side by side, so "34,509 / target 2,000" can never read as "compliant."

Doctrine wording, F2 applied — add to the required behaviour of every non-terminal class:

> A surface that declares its own currency must not be refuted by its own content. A declared
> `Last updated:` older than the newest date in the body is a failure of the surface, not of the
> reader. This check needs no baseline and no threshold: the document contradicts itself.

## Final recommendation

Ship F1 and F2 before offering this for wider reuse. F1 because your committed registry currently
publishes a 38,000-line limit and an adopter will copy it; F2 because the architecture's central
promise — that a live surface is trustworthy — is not something it currently checks.

F3 is the one we would most want in the *doctrine* rather than the checker, because it is the
mechanism behind the case history the packet is built on, and it is currently recorded only as an
outcome.

One caution offered as a peer rather than a finding. The task-tree sealing extension is
impressive engineering — content-addressed segments, exact-revision reconstruction, subtree
closure proofs — and it is the part of this architecture we would adopt *last*, if at all. It
solves the problem of preserving 844 terminal nodes losslessly. Q27 asks whether that problem
should have been solved: our answer, paid for in a 1.5 MB file we deleted rather than sharded, is
that a completed terminal node whose evidence is already in Git, in a task leaf, and in a commit
subject may not need a third home with a digest. The strength of the machinery makes preservation
cheap, and cheap preservation is exactly the bias L10 warns about. **Build the utility gate before
the sealing extension gets any better** — otherwise the best tool in the architecture will be the
one that most efficiently keeps things nobody needs.

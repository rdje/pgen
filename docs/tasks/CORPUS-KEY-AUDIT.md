# CORPUS-KEY-AUDIT: the answer key is an instrument, and nothing was auditing it

## Metadata

- Tree ID: `CORPUS-KEY-AUDIT`
- Status: `active` (opened 2026-08-24, **by director approval** of the finding below)
- Family / slice-id prefix: `PGEN-CORPUS-KEY-AUDIT-<NNNN>`
- Created: `2026-08-24`
- Owner: repo-local corpus-oracle workflow
- **Frontier: `.1`** (the contradiction census — instrument LANDED and measured; the
  deciding-message attribution is the work)

## ⛔ THE DIRECTOR'S APPROVAL (2026-08-24)

Shown the session result that auditing EXPECTATIONS out-yielded burning down the RESIDUAL:

> *"I am ok with your key-auditing suggestion … if this can make PGEN even more sota and
> signoff."*

⇒ the lane is chartered, and the condition is the bar: **every leaf here must make a
published number more trustworthy, not merely produce more work.**

## ⭐ THE PROVENANCE (measured, not asserted — all from session 2026-08-23/24)

| fact | evidence |
|---|---|
| the key manufactured a defect | `br_gh552.v` and `real_invalid_ops.v` use the **same** operator (`~&`) and carried **opposite** expectations; the difference was an `iverilog` flag the key never read (`SV-CORPUS-GRAD.13e.3`(a), `PGEN-SV-CORPUS-GRAD-0285`) |
| the key hid THREE real defects | of 394 ivtest `CE` rows, 84 carry a usable golden and only **6** say the words *"syntax error"* — the sole test applied. Enumerating what the compiler actually says surfaced `br1027a/c/e` as genuine over-acceptances (`.13e.3`(b), `-0286`), fixed as ledger `SV-0068` (`.13e.5`, `-0287`) |
| a wrong `must_accept` is CAMOUFLAGED | a row keyed `must_accept` that the parser accepts reads `match` — the strongest verdict in the file. All three `br1027*` rows sat at `match` for the whole campaign |
| the yield ratio | in one session, auditing expectations **found or corrected 5** defects while residual burn-down **fixed 3** |

⛔ **THE ASYMMETRY IS THE WHOLE ARGUMENT.** Residual burn-down only ever looks at rows already
flagged. An expectation error in the *accept* direction produces no flag at all — it produces a
`match`. So the population this lane audits is, by construction, invisible to the lane that
precedes it.

## Goal

Make the SV (and later every family's) corpus **answer key** a checked instrument rather than a
trusted one, so that the published defect bar means what it says: every `must_accept` /
`must_reject` is traceable to a clause cite or to upstream evidence that has been shown capable of
being wrong, and disagreements inside the corpus are surfaced mechanically instead of by accident.

## Non-goals

- ⛔ **Not a second burn-down.** This tree does not fix parser defects; it routes them to the
  owning family tree (`SV-CORPUS-GRAD` today) with a clause cite. Finding work is not the product.
- ⛔ **Not a re-adjudication of every row by hand.** The corpus is 16 336 rows. Every leaf here
  must be a *mechanical* audit with a stated false-positive rate, not a reading marathon.
- ⛔ **Not a replacement for `SV-CORPUS-DENOMINATOR`.** That doctrine re-derives the published
  NUMBERS from the manifests and says so in its own header (*"it does not re-adjudicate the
  corpus"*). This tree is the layer underneath it.

## Acceptance criteria

1. Every expectation in the SV manifests is classified by the KIND of evidence that produced it —
   clause cite, upstream tool testimony, or suite convention — and the population of each is
   published and re-derivable.
2. A mechanical contradiction check runs over the corpus and its false-positive rate is measured
   and stated, not assumed.
3. The audits that earn it are wired to a gate, so the key cannot silently rot the way the outcome
   files did (`SV-CORPUS-GRAD.13e.6`).
4. Every defect this lane surfaces is ROUTED to an owning leaf with a clause cite — never reported
   and left.

## Task tree

### `.1` — the CONTRADICTION census: do two rows keyed from the same evidence disagree? (`in progress`, instrument LANDED 2026-08-24)

**The question is mechanical and nobody was asking it.** A corpus large enough to contain a
contradiction is an oracle you already own: group keyed rows by the upstream messages their key
reads, and flag any message class whose rows disagree about `must_accept` vs `must_reject`.
`.13e.3` found exactly such a pair BY HAND, and only because one of them happened to be
adjudicated for an unrelated reason.

**LANDED AND MEASURED** — `docs/tasks/artifacts/corpus_key_audit/key_contradiction_census.py`:

```text
KEY-CONTRADICTION-CENSUS: rows=499 messages=168 contradictory_classes=2
```

⛔⛔ **AND IT OVER-REPORTS BY CONSTRUCTION — SAYING SO IS HALF ITS VALUE.** A row carries ONE
verdict; its golden may carry SEVERAL messages. Attributing every message to the row's verdict
manufactures a disagreement whenever a file was pinned for a reason unrelated to most of what its
golden says — and **both** reported classes are measurably that shape: `br_gh1087b` is pinned for a
NET DATA TYPE (A.2.1.3) while its golden *also* reports a multiple-driver **elaboration** error, and
`pr1704726a/c/d` are pinned for a bare generate block (A.4.2) while their goldens *also* report a
duplicate declaration. ⇒ **the output is a candidate worklist, never a defect count** — the same
lesson `.13e.3`(b) learned when a coarse text filter was 10/11 false positives and only the parser
could tell.

- **Owed:** (a) attribute a row to its **deciding** message — the one the parser's own
  `furthest_position` lands on — instead of to all of them, and re-measure; (b) state the
  false-positive rate before/after; (c) adjudicate whatever survives, routing each to
  `SV-CORPUS-GRAD` with a clause cite; (d) decide whether a residual-free census is cheap enough to
  gate.

### `.2` — the key-PROVENANCE census: which expectations rest on a CLAUSE, and which on TOOL TESTIMONY? (`todo`)

Every basis string already names its evidence. Sweep both manifests and classify each expectation:
**clause-cited** (a pin naming an Annex A / clause line), **tool testimony** (an upstream driver key
or golden log), or **suite convention**. ⛔ Only the second class can be wrong in the way `.13e.3`
found twice, so its size IS the trust bound on the published bar — and today that number is
unpublished. Publish it beside the bar.

### `.3` — the CAMOUFLAGED-match audit: over-acceptances hide inside `match` (`todo`)

⛔ **The direction the corpus structurally cannot report.** A wrong `must_accept` on a file the
parser accepts reads `match`. `SV-0068`'s three rows sat there for the whole campaign, and
`SV-CORPUS-GRAD.13e.4` was found only because a rejected file was read line by line for another
reason. Sample `match` rows whose text carries a construct on the known non-derivable list (the
`V2005_LRM_PINNED` clause set is the seed) and measure the hit rate. If it is non-zero, the bar
under-reports and this lane says by how much.

### `.4` — generalize the parse-stage vocabulary beyond ivtest and verilator (`todo`)

`.3.24` gave verilator a parse-stage message vocabulary; `.13e.3`(b) gave ivtest one, **split by
edition** because several of its refusals are dialect gates. sv2v, Surelog and ispras have had
neither treatment. ⛔ Do NOT copy the flat shape: `.13e.3`(b) measured that an unsplit list would
have manufactured five false defects.

## Current frontier

1. `.1` (a) — deciding-message attribution, then re-measure.
2. `.2` — the provenance census (cheap, and it produces the trust bound the bar is missing).
3. `.3`, `.4`.

## Decisions

- **2026-08-24 — the lane is a TREE, not a leaf under `SV-CORPUS-GRAD`.** The technique is
  family-agnostic (every family gets an external corpus with an answer key under
  `CORPUS-GRAD-ALL`), while `SV-CORPUS-GRAD` owns one family's burn-down. Keeping it separate is
  what lets `.4` reach sv2v and Surelog without widening an SV leaf.
- **2026-08-24 — the founding number ships with its own instrument.** `.1`'s census is tracked and
  re-runnable from the first commit, because a lane premised on *"published numbers must be
  re-derivable"* cannot open with a hand-counted one.
- **2026-08-24 — over-reporting is DECLARED, not discovered later.** The census artifact states its
  own false-positive mechanism in its header, so a reader cannot mistake the worklist for a count.

## Open questions

- Is `.3` affordable? The `match` population is ~5 845 rows on the SV lane alone; the audit must be
  a sampled or construct-keyed sweep, and the sampling design is the leaf's first job.
- Should `.2`'s trust bound be published beside the bar (a co-publication like
  `SV-CORPUS-DENOMINATOR`'s tuple), or reported on demand? Co-publication is the stronger claim and
  the more expensive one.

## Blockers

None. The lane is chartered and its first instrument is landed.

## Verification log

- **2026-08-24, `.1` instrument** — `python3 docs/tasks/artifacts/corpus_key_audit/key_contradiction_census.py`
  → `rows=499 messages=168 contradictory_classes=2`, artifact written. Both reported classes
  hand-adjudicated the same day as multi-message artifacts (see `.1` above), so the honest reading
  today is **0 confirmed key contradictions and 2 candidates awaiting deciding-message
  attribution** — stated this way because the reverse reading would inflate the lane's own founding
  number, which is the failure mode this tree exists to catch.

## Commit log

- `.1` instrument landed: `PGEN-CORPUS-KEY-AUDIT-0001` (2026-08-24).

## Changelog

- **2026-08-24** — tree created by director approval; `.1` instrument landed and measured; `.2`,
  `.3`, `.4` opened.

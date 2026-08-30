# DONE-BAR-VISIBILITY — make each parser's DISTANCE to `Done` visible in the book

## Metadata

- Tree ID: `DONE-BAR-VISIBILITY`
- Status: `active`
- Roadmap lane: documentation / closure reporting (cross-cutting, all 10 families)
- Created: 2026-08-30
- Origin: **DIRECTOR, 2026-08-30, verbatim**: *"You just flag parser statuses as Mostly Done, In
  progress, … I have no idea for each parser the amount of work to reach full completion (Done). I
  have no idea or clue about (1) the amount of test corpus that are passing (2) a synthesis about
  what's left to be done to reach the Done status. … we are iterating for countless of slices, lanes,
  task-trees and from my point of view there seems to be no end in sight."*

## Goal

⭐⭐ **ONE PRODUCER, TWO RENDERINGS — director, 2026-08-30, clarifying the remark above**: *"I need a
way to track the progress, issues left for a given parser … with brief, synthesized information in
the terminal and thorough in the mdbook(s)."*

- **CHAT — brief, synthesized, reported BY ME.** ⛔⛔ **CORRECTED 2026-08-30 by the director within
  the hour, and the first reading would have built the wrong thing**: *"Not me running in the terminal
  … but something you could display in this chat for me to read."* The director is NOT the operator of
  this surface — **I am**. The deliverable is a producer I run and whose synthesized output I paste
  into my completion messages, not a CLI the director is expected to invoke. A command nobody runs is
  the `GATE-REACHABILITY` defect wearing a status report.
  ⇒ it extends what `COMMIT.md` already obliges every completion message to carry (the live-status
  snapshot) from a bare LABEL to a DISTANCE.
- **BOOK — thorough.** The same producer emits the full section into that family's book page and the
  cross-family view into the main book.
- ⛔ **The two renderings MUST share one derivation.** A terminal summary that can disagree with the
  book is worse than neither, because the reader cannot tell which is lying. This is the
  `report_grammar_certification.sh` shape — one producer, a `--check` mode that re-derives and DIFFS
  against what is published, wired as a gate.

Both renderings answer three questions **from derived data, not prose**:

1. **Where is it against the bar** — which legs of the `Done` bar are met, which are not.
2. **How much corpus passes** — the pass/fail/unadjudicated counts against a named denominator.
3. **What remains** — the ordered, leaf-owned list of work between here and `Done`, with the
   owning leaf id for each item, so "no end in sight" becomes a finite list a reader can count.

## Non-goals

- Not a new status vocabulary. The `Done` / `Provisional` / `Mostly Done` / `In Progress` set stays
  (`quality-and-closure-model.md`); this tree makes the DISTANCE behind each label visible.
- Not hand-written progress prose. A hand-maintained percentage rots — that is `LIVE-MEANS-LIVE.1a`'s
  measured finding (25 of 61 hand-maintained fields were simply wrong). Every number must be DERIVED
  and gate-held, on the `SV-CORPUS-DENOMINATOR` / `PARSE-COST-RATCHET` co-publication pattern.
- Not a re-litigation of any family's status.

## The measured gap this tree exists to close (2026-08-30)

- `done_bar_family_register_v0.json` carries per family exactly: `claimed_status`, `corpus_roots`,
  `gate_prefixes`, `language_owner`, `leg3_surface`, `standard`. **No count, no pass rate, no
  remaining-work list.**
- `docs/book/src/roadmap-and-live-status.md` renders that as, in full, for the flagship family:
  `| systemverilog | Mostly Done | external-standard | (none declared) |`.
- A grep for a distance-to-`Done` view over the main book and all ten parser books returns **no such
  surface**.
- **ONE family has published corpus evidence at all** — SystemVerilog, via `SV-CORPUS-DENOMINATOR`:
  tuple `7556/2606/6174/4366/274` (adjudicated / routed / no-verdict / dark / bar). The other **nine
  families publish no corpus pass count anywhere a reader can reach.**
- ⇒ the director's statement is not an overstatement. For 9 of 10 families the book cannot answer
  *"how far?"*, and for the 10th it answers only through a doctrine gate's stderr, not the book.

## Acceptance criteria (tree)

- [ ] Each of the 10 families' book pages carries a DERIVED status block: bar-leg state, corpus
      counts against a named denominator, and the remaining-work list with owning leaf ids.
- [ ] Every number in that block is derived by a producer and gate-held equal to its source, so it
      cannot drift (the `SV-CORPUS-DENOMINATOR` co-publication shape).
- [ ] A family with no corpus says so LOUDLY as `NOT MEASURED`, never as a blank or a zero —
      `docs/decisions/feedback_a_check_that_cannot_run_must_say_so.md`.
- [ ] The main book gains one cross-family view so the whole picture is one page, not ten.
- [ ] **A producer exists that emits the brief form for one family in a screenful, and I report it
      in completion messages** — the director reads it in chat and is never expected to run it.
- [ ] **Terminal and book cannot disagree**: `--check` re-derives and diffs the published block,
      wired as a gate, so a stale book section fails a commit rather than misleading a reader.

## Task tree

- ID: `DONE-BAR-VISIBILITY.1` — Status: `done` (`PGEN-DONE-BAR-VISIBILITY-0003`)
  Goal: DEFINE the per-family status block AND ship the terminal renderer: exactly which fields,
  which derivation for each, and which are legitimately `NOT MEASURED` today. Deliverable is a schema
  + the producer + one worked family, not ten.
  Acceptance: the schema names a producer for EVERY field; the brief form for SystemVerilog fits a
  screenful and is REPORTED IN CHAT by me; SystemVerilog rendered end to end in both surfaces from
  the one derivation.
- ID: `DONE-BAR-VISIBILITY.2` — Status: `pending`
  Goal: the REMAINING-WORK synthesis — derive each family's open items from the task trees that
  already own them, so the list cannot drift from the trees.
  Acceptance: every item carries its owning leaf id; the list is derived, not typed.
- ID: `DONE-BAR-VISIBILITY.3` — Status: `pending`
  Goal: the corpus pass counts for the nine families that publish none, or an explicit
  `NOT MEASURED` with the leaf that owes it.
- ID: `DONE-BAR-VISIBILITY.4` — Status: `pending`
  Goal: wire the co-publication gate so the block cannot go stale.
- ID: `DONE-BAR-VISIBILITY.5` — Status: `pending`
  Goal: the cross-family page in the main book.
- ID: `DONE-BAR-VISIBILITY.6` — Status: `pending`
  Goal: the HAND-WRITTEN STATUS PROSE class, found by `.1` and routed rather than logged. The
  SystemVerilog book's `welcome.md` carried three figures and all three were wrong — the AST-dump
  schema published as `10` in one paragraph and `13` in another while the contract carried `26`,
  and the parser described as *"validated each release by the … external-corpus (14/14) … gates"*
  while `audit_done_bar.sh` records the only gate reading that corpus as a TRIAGE gate, leaving
  leg 3 unmet. `.1` fixed that ONE page by replacing the prose with a pointer to the derived
  surface. The CLASS is untouched: nine other parser books may carry the same thing, and
  `PARSER-BOOK-CURRENCY` cannot see it — it holds a book's newest CHANGELOG release equal to the
  contract and says nothing about a status paragraph.
  Acceptance: every per-parser book's status prose is censused; each figure is either DERIVED by a
  producer or DELETED in favour of a pointer; no per-parser book carries a hand-typed maturity
  number afterwards.

## Current frontier

1. `DONE-BAR-VISIBILITY.2` (the remaining-work synthesis — derived from the trees that already own
   each item, never typed). It is the frontier because it is the half of the director's remark
   `.1` deliberately left as `NOT MEASURED`: `.1` answers *where against the bar* and *how much
   corpus passes*, and publishes an explicit owed-by for *what is left*.

## Leaf records

### `.1` — the per-family status block: schema, producer, SystemVerilog end to end

Shipped in `PGEN-DONE-BAR-VISIBILITY-0003`.

**What landed.**

- `rust/test_data/grammar_quality/family_status_block_v0.json` — the SCHEMA. Every field carries
  the question it answers and the PRODUCER that answers it; a field with `state: not_measured`
  carries `owed_by`, the leaf that owes the producer. It also holds the per-family corpus-census
  wiring, so `.3` adds a family by adding data rather than by editing the renderer.
- `scripts/report_family_status.sh` — the PRODUCER. `<family>` renders the brief form (a
  screenful, reported in chat by me); `--markdown` renders the book block; `--check FILE`
  re-derives and diffs the published block, exiting 1 on drift and 2 when it cannot evaluate.
- `docs/systemverilog_parser_book/src/status-and-distance.md` — the published SV block, plus the
  SUMMARY entry and a pointer from the main book's live-status chapter.
- `docs/tasks/artifacts/done_bar_visibility/run_family_status_probes.sh` — 8 arms, 1 green control
  and 7 RED, capture in `family_status_probes.txt`.

**Two design rulings worth carrying.**

1. **The book half is derived TRACKED-ONLY; the chat half adds this machine's readings.** Leg 1 and
   the green-NOW half of leg 2 rest on gate artifacts under `rust/target/`, which is untracked — the
   same commit reads `artifact STALE` where the gate ran and `no artifact on this machine` on a
   fresh clone. Publishing either into a gate-held block would make `--check` fail for every reader
   who had not run the gate first. The producer therefore drives the audit through its own
   `PGEN_DONE_BAR_TARGET_DIR` seam pointed at an empty directory — no fork of the audit, no second
   copy of its logic — and the chat form adds the ordinary run's verdicts on top.
2. **A leg is never published without a reason, and an unrecognised reason REFUSES.** The audit's
   sentences are a diagnostic surface free to improve; the block maps them onto a closed vocabulary
   and refuses when a leg's lines match nothing, so a wording change is LOUD here instead of
   silently emptying the published block.

#### Acceptance checklist (`TOOLBOX.md`)

- [x] **ROOT CAUSE (WHY + WHERE)** — measured against HEAD blobs, not the working tree, because the
      first run of this census read the working tree and reported **3 hits** for a surface that
      does not exist at HEAD (my own edits), which is the drift the check exists to catch:
      `for f in $(git ls-files 'docs/*book*/src/*.md'); do git show "HEAD:$f" | grep -qiE 'distance
      to \`?done' && echo HIT; done` returns **0 hits over 177 tracked book pages** — WHERE: no
      page in the main book or any of the ten parser books answers *how far is this family from
      `Done`*. WHY: the done-bar register holds `claimed_status` and no count, the audit that DOES
      derive the legs is an operator command whose output is published nowhere, and the corpus
      census is reachable only through a doctrine gate's stderr. The status LABEL is published and
      the DISTANCE behind it is not, so the director reads a ladder position with no scale.
      ⛔ The same census found the SECOND defect, routed to `.6`:
      `git show HEAD:docs/systemverilog_parser_book/src/welcome.md | grep -nE 'version .1[03].|14/14'`
      returns lines 11 and 31 publishing AST-dump schema `13` and `10`, while
      `git show HEAD:docs/contracts/PGEN_SYSTEMVERILOG_PARSER_INTEGRATION_CONTRACT.md` line 827
      carries `26` — two hand-written numbers disagreeing with the contract AND with each other in
      one file, beside a claim of `external-corpus (14/14)` validation that the done-bar audit
      contradicts (the only corpus gate is a TRIAGE gate ⇒ leg 3 UNMET).
- [x] **ADDRESSED (verified)** — before: `bash scripts/report_family_status.sh systemverilog` did
      not exist and no book page carried a distance. After: it prints the SV block in **3.0 s** —
      3 of 3 legs not met with the reason for each, 16 covering gate targets of which 1 is invoked
      by nothing, and 5 845 of 16 336 corpus rows passing (35.8 %) against a named denominator with
      274 known defects, 6 174 rows carrying no verdict and 4 366 of those dark. The same
      derivation is published at `docs/systemverilog_parser_book/src/status-and-distance.md` and
      `--check` on that page returns `OK (published block equals a fresh derivation, 2344 bytes)`.
      All ten families render without refusing. The stale prose in `welcome.md` is replaced by a
      pointer to the derived page rather than by a fresher hand-typed number.
- [x] **NO REGRESSION** — `bash scripts/check_doctrines.sh`: **31/31 PASS** (`BASELINE-IDENTITY`
      correctly refused the new schema file until it was registered with a
      `not-a-derived-baseline` verdict — the closed-population arm doing its job, then green).
      `make -C rust SHELL=/bin/bash mdbook_docs_gate` PASS — all 10 per-parser book gates plus the
      main book. `bash scripts/audit_done_bar.sh` exit 0 with its 13 ground-truth controls
      reproduced and its text report unchanged (this slice adds no code to it; it is driven only
      through its existing env seam). The published block is **byte-identical** to a fresh
      derivation by construction — that is what `--check` asserts, and RED-1 proves it fails when
      one number is mutated.
- [x] **RED CONTROLS** — `bash docs/tasks/artifacts/done_bar_visibility/run_family_status_probes.sh`
      → **8 passed, 0 failed**. The four arms that were mutually indistinguishable in
      `GRAMMAR-CERT-STATUS.1b` are separated here: a mutated pass count FAILS (1), a page with no
      DERIVED block REFUSES (2), a nonexistent path REFUSES (2), another family's block on the page
      FAILS (1). Plus an unknown family and a missing family name (2 each), and the two-sided
      schema arm: a field declared in the schema that nothing renders REFUSES (2).

## Decisions

- **Derived, never hand-written** — `LIVE-MEANS-LIVE.1a` measured 25 of 61 hand-maintained
  "last updated" fields simply wrong; a hand-maintained percentage is the same class.
- **SystemVerilog first**, because it is the release lane and the only family with corpus evidence
  to render — so `.1` is inside the SV lane lock rather than an escape from it.

## Open questions

- Whether `Provisional (corpus pending)` families should show a corpus block at all before a corpus
  exists, or only the leaf that owes one. Leaning: show the leaf — an empty block reads as zero.

## Blockers

None.

## Verification log

- 2026-08-30 — the gap above measured, not asserted: register schema read, book row quoted verbatim,
  distance-view grep returned nothing, SV denominator tuple read from its own gate.
- 2026-08-30 (`.1`) — the census RE-RUN against HEAD blobs after the working-tree run reported 3
  false hits: **0 of 177** tracked book pages carry a distance-to-`Done` surface. Producer, schema,
  page, SUMMARY and probe driver landed; `--check` green on the published page; 8/8 probe arms;
  31/31 doctrines; mdbook gate PASS; all ten families render.

## Commit log

- tree created — `PGEN-DONE-BAR-VISIBILITY-0001`
- `.1` scope corrected by the director — `PGEN-DONE-BAR-VISIBILITY-0002`
- `.1` DONE (schema + producer + SystemVerilog end to end) — `PGEN-DONE-BAR-VISIBILITY-0003`

## Changelog

- 2026-08-30 — created from a direct director remark; gap measured the same day.
- 2026-08-30 — `.1` DONE. One producer, two renderings, `--check` proven able to go RED four ways.
  New leaf `.6` opened for the hand-written-status-prose class the slice measured on the way past.

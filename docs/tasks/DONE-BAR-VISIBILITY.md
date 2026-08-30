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

- ID: `DONE-BAR-VISIBILITY.1` — Status: `pending`
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

## Current frontier

1. `DONE-BAR-VISIBILITY.1` (schema + SystemVerilog worked end to end)

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

## Commit log

- tree created — `PGEN-DONE-BAR-VISIBILITY-0001`

## Changelog

- 2026-08-30 — created from a direct director remark; gap measured the same day.

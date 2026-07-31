---
name: feedback-classify-referents-by-requirement
description: DISCIPLINE (ANVIL, 2026-07-31, LIVE-MEANS-LIVE.1b/.1c) — a reference COUNT is not a dependency measure. `grep -l` inflates with comments, hint strings and append-only history that must keep its references verbatim, so it is an upper bound, not a cost. Classify every referent by what it REQUIRES (reads the CONTENT / needs the PATH to exist / mentions it in prose) and only the content-readers are blockers. Measured instance — a "96 referrers must be re-pointed" cost estimate was really ONE script, and that script was already inert.
metadata:
  node_type: memory
  type: feedback
---

**The correction, verbatim (ANVIL, 2026-07-31):**

> *"Reference count is not a dependency measure. It inflates with hint text and with append-only
> history that must keep its references raw. Classify referents by what they require."*

## The founding case

`LIVE-MEANS-LIVE` set out to delete `LIVE_ACHIEVEMENT_STATUS.md`. The blocking cost was recorded in
the tree as **"96 tracked `.md` referrers must be re-pointed"**, plus *"4 scripts, 36 docs, 57 task
files, 6 decisions"* — every one of those numbers produced by `git ls-files | xargs grep -l`.

Re-measured by **what each referent REQUIRES**, over the shell surface:

| requirement | count | detail |
|---|---|---|
| ⛔ reads the file's **CONTENT** | **1** | `run_demotion_impact_probe.sh` — greps a table row and copies the file |
| needs only the **PATH to exist** | 2 | `ci_workflow_local_gate.sh` (`assert_tracked` ×5), `check_diagnostics_and_docpaths.sh` (a doc-path glob) |
| a **HINT / routing message** | 1 | `check_readme_stability.sh` — names it as an overflow destination in advice text |
| pure **COMMENT / provenance** | 5 | the 3 family-status gates, `parser_family_status_bar.sh`, `check_published_version_currency.sh` |

⇒ **the delete is blocked by ONE script, not ninety-six** — and that one was *already inert*, because
its precondition greps for a `Done` row the tracker stopped carrying at `DONE-BAR.2b`.

An off-by-96× cost estimate is not a rounding error; it is the difference between a leaf someone
schedules and a leaf nobody starts.

## Why the count inflates, structurally

The three biggest contributors are all things that **must** keep the reference verbatim:

1. **Append-only history.** `CHANGES.md`, `DEVELOPMENT_NOTES.md` and git are records *by charter*.
   A reference inside them describes what was true when written and must NOT be rewritten — so it
   can never be a blocker, yet it counts.
2. **Hint and routing text.** An enforcer that says *"status belongs in X"* names X. That is advice,
   not a read.
3. **Provenance comments.** A migration leaves *"this moved out of X"* comments behind precisely so
   the next reader understands. Every one of them counts as a "reference" and none is a dependency.

## The discipline

> **Before pricing a delete, a rename, or a migration: classify referents by what they REQUIRE, and
> count only the requirement class that actually blocks.**

1. `grep -l` produces the **candidate set**, never the cost. Treat its output as an upper bound.
2. Bucket every hit: *reads the content* · *requires the path to exist* · *mentions it in prose*.
   Only the first bucket can block; the second is satisfied by any replacement at the same path or
   by updating one assertion list; the third is documentation to correct at leisure.
3. **Re-measure the blocking bucket after each landing** — it shrinks as work lands. `.1b` removed
   the audit's content read, so the blocking count fell before `.1c` was ever started.
4. Check whether the blocker is even **live**. The one remaining content-reader here refuses on its
   own precondition; a dependency that cannot execute is not a dependency.

## Relation to the other census error in the same tree

This is the second time one tree produced a confident number from the *convenient* query rather than
the *right* one. `LIVE-MEANS-LIVE.0` ran an ID census over two files, reported **3 orphans**, and
made rescuing them a precondition; re-run across every durable layer it was **452/452, zero
orphans**. Same shape, opposite direction: one query under-counted the destinations, the other
over-counted the dependents.

⇒ **the general rule: state what a number is a count OF, and check that it is the quantity the
decision actually turns on.**

Related: [[feedback_instrument_needs_ground_truth]] (an instrument must reproduce known facts or
refuse), [[feedback_read_prior_art_before_designing]] (re-measure, never quote a stale doc),
[[feedback_flow_findings_are_routed_not_worked]] (route the re-costing, do not stall on it).

Evidence: `docs/tasks/LIVE-MEANS-LIVE.md` leaves `.1b` (the roster inversion that removed the last
content read) and `.1c` (the re-costed referrer table).

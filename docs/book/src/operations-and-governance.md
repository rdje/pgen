# Operations and Governance

PGEN relies on disciplined operational docs, not just code.

## Continuity Docs

These four files are the live continuity spine:

- `CHANGES.md`
- `DEVELOPMENT_NOTES.md`
- `docs/book/src/roadmap-and-live-status.md`
- `MEMORY.md`

## Session and Commit Workflow

Two operational docs matter especially for future sessions:

- `SESSION_BOOTSTRAP.md`
- `COMMIT.md`

They capture:

- how a fresh session should ramp up,
- which docs must be reviewed before commit,
- what must be included in post-commit reporting,
- how live-status communication stays consistent.

## Host-Resource Governance (the memory guard)

PGEN development runs on a single machine whose RAM is a shared, exhaustible
resource, and a runaway build, bench, or parse job can take the whole host down.
That risk is governed mechanically, not by discipline alone
(`docs/decisions/feedback_host_ram_budget_all_jobs.md`):

- one heavy job at a time (fat-LTO builds, full-corpus benches/profilers, the
  known heavy parse classes),
- a pre-flight free-RAM check before launching anything heavy,
- and `scripts/run_with_memory_guard.sh`, a sampling supervisor every
  heavy/background job runs under.

The guard pre-flights system free memory AND free disk space on its working
filesystem, samples the job's process-tree RSS on an interval, and kills the
whole tree — with an always-written marker file and an unconditional log line —
when the tree exceeds its RSS budget (default ≈12 GB), when system-wide free
memory drops below a floor (default 10%), when free disk drops below a floor
(default 8 GB; `--disk-floor-gb 0` disables — added after a 2026-07-18 build
died mid-compile on a 100%-full disk and corrupted its own incremental state),
or when an optional wall-clock timeout expires. Well-behaved jobs pass their own
exit code through; guard verdicts use distinct codes (96 preflight-refused —
RAM or disk, the marker's `reason=` field disambiguates — 97 rss-budget,
98 free-floor, 95 disk-floor, 99 timeout, 130 guard-interrupted) so callers can
branch mechanically. See `README.md` → Standard Commands for usage.

Two design rules from its verification history are worth teaching:

- a kill list computed from a sampled process walk is untrusted input — the
  guard voids any sample that implicates the system at large (pid 1, the guard
  itself, or its ancestors) and relies on the kernel-scoped process-group kill
  as the primary mechanism, and
- severity is never gated by verbosity: breach and warning lines always print;
  verbosity only governs informational sampling output.

## Build Integrity (what the batteries compile is not what the repo ships)

PGEN's test batteries build `--lib` and `--tests` under the *default* feature
set. That is a strict subset of what the repository actually ships, in three
independent ways:

- a `[[bin]]` target is not built by `--lib` at all,
- a binary that declares `required-features` is excluded from every
  default-feature build,
- a test file behind `#![cfg(feature = "…")]` compiles to an empty crate
  unless that feature is on.

Each exclusion is individually reasonable. Their intersection is a hole in
which a target can stop compiling without any gate turning red — the failure
is not an error, it is a *silence*, and it surfaces days later as "that tool
is gone" rather than "a migration missed a call site". Three surfaces were
found rotted in exactly this hole in July 2026: a feature-gated binary, a
maintained gate script, and a ten-grammar integration test.

The standing proof lane is:

```bash
scripts/run_with_memory_guard.sh --budget-mb 16384 --timeout-s 5400 -- \
  make -C rust SHELL=/bin/bash bin_build_integrity_gate
```

It proves two things mechanically. First **coverage**: the binary census is
derived from `cargo metadata`, never from a hand-maintained list, and every
declared binary must be included in at least one checked configuration — a new
binary whose feature requirements nothing satisfies fails the gate *by name*.
Second **compilation**: each configuration is checked with `--all-targets`, so
bins, tests, benches, and examples are all built.

It checks a *set of configurations* rather than one union build, deliberately.
A union is not expressible here (two allocator features are mutually
exclusive), and more importantly it would hide the very failure mode that
rotted: code reachable only under one feature. Each configuration in the plan
is one this repository genuinely builds — the default build, the bootstrap
path, the `focus_*` regen path, the `.ebnf` frontend path, and the documented
dual-feature toolbox binary — so each has to compile. The plan is also what
makes *execution* honest: a cfg-gated test can appear or disappear with the
feature set, so a single configuration can run 9 of 10 tests and still report
success.

This is a heavy gate (one `cargo check` per configuration, and distinct
feature sets share no build artifacts). It is a maintained gate, not a
pre-commit hook.

> **See also: [The Gate Flow — Reference](gate-flow.md).** This chapter tells the
> story of how the proof surface broke and what was learned. That one is the
> reference: the anatomy of a gate, the four layers they compose into, what flows
> in and out, the artifact hand-off protocol, who invokes what, and the contract a
> new gate must satisfy.

## Workflow Parity (proving the CI commands locally)

While hosted GitHub Actions are paused to conserve account minutes, the lane
that stands in for them is:

```bash
make -C rust SHELL=/bin/bash ci_workflow_local_gate
```

It runs in two phases against an **export directory** built from `git ls-files`
output — a deliberate simulation of what `actions/checkout` hands a fresh
runner. The first phase is 33 *audits* (allowlists, contract surfaces, emission
shapes: everything that can be decided by reading files). The second phase
*replays* the command each tracked workflow runs.

Two properties of that design are worth stating plainly, because both were
learned the expensive way.

**An audit phase at 33/33 is not "the gate completes."** The replay phase is
where files stop being read and start being executed, and for 1,371 commits it
had never run once. When it was finally exercised, 8 of the 11 replays failed —
all eight for a single reason. `generated/` is untracked (it is pipeline output,
regenerated locally), so the export directory contains no generated parsers; and
`rust/src/lib.rs` includes two of them — the return-annotation and
semantic-annotation parsers — by *literal path* with no `has_generated_*` cfg
guard. The other nine sites are cfg-guarded, so their absence merely disables a
parser. The two unguarded ones take the whole crate down with
`error: couldn't read src/../../generated/return_annotation_parser.rs`. The three
replays that pass are exactly the three that never compile the crate.

The remedy is not to copy a developer's `generated/` into the export directory —
that would make the gate green against artifacts a fresh checkout does not have,
which is the failure mode the gate exists to detect. Instead the gate replays the
repository's own cold-clone bootstrap inside the export directory, which costs
roughly four minutes (measured 236 s from a bare tracked tree), after which the
workflow phase runs.

That preparation is now **on by default**. It was deliberately opt-in when it
first shipped, and the reason is worth keeping: at that point fourteen of the
fifteen tracked hosted workflows had no regeneration step, so defaulting it on
would have produced a green local gate standing in for a hosted side that was
still broken — **false parity, which is worse than the visible red the gate was
reporting.** Once the hosted workflows were fixed, a local green and a hosted
green mean the same thing again, and the default flipped. To skip it for one run:

```bash
PGEN_CI_WORKFLOW_LOCAL_PREPARE=0 make -C rust SHELL=/bin/bash ci_workflow_local_gate
```

which is worth doing only for a narrowed run whose selected replays do not
compile the crate; the gate then warns loudly rather than preparing.

### The regeneration recipe has exactly one home

That bootstrap sequence — `regex_parser_bootstrap`, then `annotation_parsers`,
then the per-grammar `focus_*` targets — is written down in exactly one place:

```bash
make -C rust SHELL=/bin/bash regenerate_generated_parsers
```

Everything that needs it calls that target. The local parity gate's preparation
step calls it directly; the hosted workflows reach it through the composite
action `.github/actions/regenerate-parsers`, which exists so the *step* (its
name, its comment, its `uses:` reference) also has a single definition. A make
target cannot carry a workflow step, and a composite action cannot be invoked
from a shell gate, so each owns the half the other cannot.

This mattered because the same tracked-files-only shape that breaks the local
export is exactly what `actions/checkout` hands a hosted runner — and **11 of
the 15 tracked workflows need generated parsers, while only one declared a
regeneration step.** Wiring the other ten by copy-paste would have produced
twelve copies of a sequence whose drift nothing could detect.

Three workflows are *measured* not to need it and deliberately do not pay the
four minutes: `branch-protection-contract-gate` (shell and `jq`),
`fixed-point-gate` (builds the bootstrap binary without
`--features generated_parsers`) and `mdbook-docs-gate` (mdBook only). A fourth,
`memory-architecture-gate`, runs no `make -C rust` command at all.

`audit_workflow_regeneration_surface` holds that arrangement in place, and its
polarity is the point: the roster is derived from `git ls-files`, and *needing*
the step is the default. A workflow added tomorrow that runs a `make -C rust`
gate fails the audit until someone either wires the step or measures it into the
exemption list — the safe direction. Exemption is checked in both directions too,
so an exempt workflow cannot quietly acquire a four-minute step it does not need.
Any job carrying the step must also budget at least 30 minutes; pricing that
floor is what revealed that `sota-exit-gate` had been declaring a 60-minute
timeout for a job measured at 2 h 23 m.

**A filter that matches nothing must not report success.** The gate accepts
`PGEN_CI_WORKFLOW_LOCAL_FILTER` to narrow the replay set. A mistyped name used to
skip every replay and then print `✅ Local GitHub workflow parity gate passed`
with exit 0 — and because the 32 audits *had* run, the green looked earned. An
unknown filter entry, or any run that ends up replaying zero workflows, is now
refused with the list of known names, and the closing line reports how many
workflows were actually replayed.

Both fixes are the same principle this repository keeps re-deriving, applied to
a shell gate rather than a parser: **a check that cannot run must say so, not
return green** — and its corollary, that when a check cannot run it must name
*its own* obstacle rather than failing as though the thing under test were
broken.

## A check must not require a defect in order to pass

The first end-to-end run of the workflow-replay phase turned up a fourth
variation on that theme, and it is the converse of the others. Running
`make -C rust SHELL=/bin/bash sota_exit_gate` — the repository's flagship
aggregate and a `README` standard command — cleared nineteen required sub-gates
and then died in `sv_failure_context_contract_gate` on

```
error: expected at least one generation failure-context excerpt
```

The counterexample pipeline was not broken. It was correctly reporting zero. The
contract driving that surface describes itself as *one-profile, one-sample* and
sets `"sample_count": 1`; the surface's own report read `requested_total: 1`,
`accepted_total: 1`, `parser_rejections_total: 0`. One sample was requested and
the SystemVerilog parser accepted it on the first attempt, so there was genuinely
no failure to excerpt — and the gate demanded one. **It could only pass if the
parser rejected its own generated sample: it passed when the system was broken
and failed when it worked.** The assertion and the one-sample contract had
shipped in the same commit, so the coupling was original rather than drift.

Deleting the assertion was the wrong fix, because its intent was real: if the
triage silently stopped producing excerpts, the whole failure-context surface
would be dead and nothing else would notice — the gate publishes an excerpt as
its headline evidence. So the intent is kept and only the dependence on a defect
is dropped. A zero is now accepted only when it is **earned**:

1. the surface was actually **exercised** (`attempts_total >= 1`) — a zero from
   zero attempts is the vacuous green, and the old form could not express it;
2. the zero is **consistent** — no counterexamples is acceptable only when the
   surface recorded no rejections and no generation errors. Rejections with no
   excerpt means the capture path itself is broken;
3. a present excerpt is **well-formed** — counterexamples imply at least one
   distinct context excerpt and a non-empty preview.

This is a correction, not a relaxation, and the distinction is worth stating
precisely: a healthy run, a run that attempted nothing, and a run whose capture
path is broken all produce the same excerpt count of zero. The old assertion read
only that number, so it failed all three identically and could distinguish none
of them. The new one separates them — it stops failing in the single case where
the old form was wrong, and starts failing in two cases the old form could not
see at all.

The general rule, alongside the two above: **a check must not depend on the thing
it watches being broken.**

## A check that nothing invokes does not exist

The third variation, and the one that took longest to admit. Over three sessions
this repository found three maintained gates that nothing ran — a raw-AST dump
contract that had been red for four sessions, a strict lint stage whose enabling
variable was set by no gate and no workflow, and the local workflow-parity gate
itself. All three were found **by accident**. Finding them by accident is the
defect; no artifact answered the question *"which tracked gates are reachable
from something that actually runs?"*

That artifact now exists, and it is enforced:

```bash
bash scripts/check_gate_reachability.sh --report
```

It derives the whole picture from the repository on every run — the universe of
gate targets from `rust/Makefile`, the invocation edges from the Makefile, the
gate scripts, the tracked workflows, the git hooks and `COMMIT.md`, under a
command-position rule so that *reading* a gate script (which the parity gate's
surface audits do) is never mistaken for *running* it. Invocation by prose is
tracked as its own class: a gate whose only invoker is an instruction in
`COMMIT.md` is exactly the case worth seeing, because nothing fails if it is
skipped.

At adoption: 123 targets, 92 reachable, 30 orphaned, 1 policy-only. The first
thing it surfaced was that **all ten per-parser book gates were orphaned** while
the project carries a standing directive that every parser ships a live mdBook —
so they were wired onto `mdbook_docs_gate`, which everything already runs, for
about three seconds.

The remaining thirty-one carry a recorded disposition in a tracked register, and
this is a **ratchet rather than a report**: the orphan set is re-derived every
run, an untriaged orphan fails the check, and a register entry that no longer
names an orphan fails too, so the exemption list can neither be bypassed nor
quietly accumulate. Twenty-eight of those dispositions are honest accepted risk —
real proof lanes deliberately left outside the aggregates because of their cost —
and the register says so in its own text rather than implying they are covered.

Two things are worth carrying away from building it. First, the scan produced
**six different confident answers** before it was right, each from a real defect
(a mention counted as an invocation, a workflow's `run:` prefix, make's `@`,
backslash continuations, a nested `make` inside a runner's arguments, a
prerequisite list held in a make variable) — and not one was caught by reading
the code. Every one was caught by requiring the output to reproduce facts the
project had already measured. Those facts are now assertions inside the check: if
it cannot reproduce them it reports **MISCALIBRATED** and refuses, because a
wrong reachability number would certify the very rot it exists to find.

Second, it asks `make` what a derived prerequisite list contains rather than
re-implementing `$(wildcard)` and `$(patsubst)`. A second implementation of a
rule is a second thing that can drift from it.

## A live document must be currently TRUE, not merely bounded

`README.md` is capped on two axes — a line cap *and* a byte cap. That is half a fix. The other
half is checking **where the overflow lands**, because a cap that redirects content has not removed
the pressure, it has moved it to whichever neighbouring surface has no instrument.

That is not hypothetical here. The README's own overflow rule named a status file as the
destination for family-status content. The README stayed inside both caps; the destination reached
**1 547 057 bytes, of which 94.7 % was a dated changelog**, and nothing watched it at all.

The `LIVE-DOC-CURRENCY` doctrine (`scripts/check_live_document_currency.sh`, run by the doctrine
enforcer on every commit) closes that with two instruments and a closure rule.

### Why not simply cap every document?

Because *big* and *rotted* are different questions, and a byte cap answers the wrong one.
`gate-flow.md` is the **largest** of the healthy overflow destinations and also the healthiest. A
byte cap would rank it worst. The question worth asking is *has this surface stopped being a status
document*, and size does not answer it.

### Instrument A — distinct dates, paired with a declared charter

Count the distinct `YYYY-MM-DD` dates a surface carries. A status view carries one or two; a log
carries hundreds. But the count alone is a false-positive machine: `CHANGES.md` scores 175 and is
**correct**, because being a dated history is its charter.

So each watched surface declares its charter in
`rust/test_data/grammar_quality/live_document_currency_register_v0.json`:

| charter | meaning | subject to the ceiling? |
|---|---|---|
| `status` | a current-state view | **yes** |
| `log` | a dated history by charter (`CHANGES.md`, the bug ledger) | no — accumulating dates is the job |
| `index` | one row per record; the count scales with entry count | no |

**The instrument classifies; the charter says which classification is permitted.** A `log` or
`index` charter needs a written `_why` — an exemption nobody justified is an exemption nobody
reviewed — and it exempts the surface from *this* instrument only. It is never a clean bill of
health on every axis.

A `status` surface above the ceiling must be declared as **owned debt** naming a task leaf that
exists. That entry is a two-sided ratchet: a new breach fails, and a breach that has been *repaired*
while the entry survives also fails, with *"the debt is paid, remove the entry"*. Debt cannot
silently become permanent, and the ceiling is never raised to make a surface green.

### Instrument B — a document that refutes itself

Many documents declare `Last updated: <date>`. Compare that declaration against the newest date in
the document's own body. If the body is newer, the file's two halves contradict each other.

This needs **no baseline and no threshold at all** — no history, no reference snapshot, no chosen
number. The evidence is entirely inside the file.

It is also the instrument that proves *bounded* and *current* are independent properties: the files
it catches are all comfortably inside every size bound and correctly routed.

### Why the check refuses instead of listing spellings

This instrument measured the same population three times and got three answers — 10, then 16, then
18 — because the repository writes the declaration four different ways, and each pass knew only some
of them:

| shape | example | where |
|---|---|---|
| `bare` | `Last updated: 2026-05-14` | root docs, `docs/reference/` |
| `backtick` | `` - Last updated: `2026-05-31` `` | every `docs/tasks/` tree |
| `continuation` | `- Last updated:` with the date on the **next** line | `docs/contracts/` |
| `template` | `` - Last updated: `YYYY-MM-DD` `` | the task-tree template — declares nothing |

Every miss failed **silently in the passing direction**: an unmatched file is not a reported miss,
it is an absent row, so the instrument under-reports and looks clean doing it. Each correction was
possible only because a *prior published number* existed to disagree with.

So the check does not enumerate spellings. It requires **total classification** and **refuses**
(exit 2) on any anchored `Last updated` line it cannot classify. A fifth spelling appearing tomorrow
is a loud refusal, not an absent row — the one property a longer list can never have.

Two exclusions are pinned by construction, both measured rather than imagined: a declaration inside
a fenced code block is sample output, and an *indented* one is quoted material. The second was found
when the check refused on its own task leaf, where a wrapped quotation of its output line put
`Last updated:` at the start of a continuation line.

### Route closure — a guard that names a destination is defining a route

The edge that carried the pressure out of `README.md` was a **hint string inside an error message**.
No hand-authored route registry would ever have listed it. So the destinations are **derived** from
the enforcers' own output text — the routing hint `check_readme_stability.sh` prints on a cap
breach, and `COMMIT.md`'s own *Files Involved* list — and every derived `.md` destination must be a
watched surface.

### Ground truth — the check refuses rather than guesses

An instrument with no ground truth is a confident guess, and this one has been wrong three times.
Every run first executes nine in-memory fixtures with known answers — a positive, a negative, one
per pinned shape, the two exclusions, and one proving the refusal path itself is live — through the
**same** extractor the real scan uses. Any miss aborts before a number is published.

```bash
bash scripts/check_live_document_currency.sh     # or via: bash scripts/check_doctrines.sh
```

Exit `0` holds; `1` is a breach; `2` is a refusal — the check could not judge, and says so instead
of returning green.

## Documentation Governance

The intended split is:

- the book is the primary public documentation surface,
- the continuity docs are internal continuity and crash-recovery surfaces,
- the contracts and reference docs are the deep authoritative details behind the book.

That split keeps the repository teachable without losing high-signal live state.

See `Documentation Model` for the fuller public explanation of that split.

## Book Maintenance Doctrine

The book is not a one-time scaffold. It is part of the maintained repo surface.

That means:

- if a change affects a user-facing or developer-facing surface already covered by the book, update the relevant chapter in the same wave,
- if a new important surface appears often enough to matter, add a new section or chapter,
- use the book to curate and teach, not to mirror every raw implementation note,
- but treat the book as the place where the world should be able to understand what PGEN does, how it works, and why it is designed that way.

The maintained proof lane for this doctrine is:

```bash
make -C rust SHELL=/bin/bash mdbook_docs_gate
```

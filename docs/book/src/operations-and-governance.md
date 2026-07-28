# Operations and Governance

PGEN relies on disciplined operational docs, not just code.

## Continuity Docs

These four files are the live continuity spine:

- `CHANGES.md`
- `DEVELOPMENT_NOTES.md`
- `LIVE_ACHIEVEMENT_STATUS.md`
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

## Workflow Parity (proving the CI commands locally)

While hosted GitHub Actions are paused to conserve account minutes, the lane
that stands in for them is:

```bash
make -C rust SHELL=/bin/bash ci_workflow_local_gate
```

It runs in two phases against an **export directory** built from `git ls-files`
output — a deliberate simulation of what `actions/checkout` hands a fresh
runner. The first phase is 32 *audits* (allowlists, contract surfaces, emission
shapes: everything that can be decided by reading files). The second phase
*replays* the command each tracked workflow runs.

Two properties of that design are worth stating plainly, because both were
learned the expensive way.

**An audit phase at 32/32 is not "the gate completes."** The replay phase is
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
which is the failure mode the gate exists to detect. Instead:

```bash
PGEN_CI_WORKFLOW_LOCAL_PREPARE=1 make -C rust SHELL=/bin/bash ci_workflow_local_gate
```

replays the repository's own cold-clone bootstrap inside the export directory —
`regex_parser_bootstrap`, then `annotation_parsers`, then the per-grammar
`focus_*` targets — the same sequence the hosted generated-clippy workflow
already uses. It costs roughly four minutes, after which the workflow phase runs.

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

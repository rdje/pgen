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

The guard pre-flights system free memory, samples the job's process-tree RSS on
an interval, and kills the whole tree — with an always-written marker file and
an unconditional log line — when the tree exceeds its RSS budget (default
≈12 GB), when system-wide free memory drops below a floor (default 10%), or when
an optional wall-clock timeout expires. Well-behaved jobs pass their own exit
code through; guard verdicts use distinct codes (96 preflight-refused,
97 rss-budget, 98 free-floor, 99 timeout, 130 guard-interrupted) so callers can
branch mechanically. See `README.md` → Standard Commands for usage.

Two design rules from its verification history are worth teaching:

- a kill list computed from a sampled process walk is untrusted input — the
  guard voids any sample that implicates the system at large (pid 1, the guard
  itself, or its ancestors) and relies on the kernel-scoped process-group kill
  as the primary mechanism, and
- severity is never gated by verbosity: breach and warning lines always print;
  verbosity only governs informational sampling output.

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

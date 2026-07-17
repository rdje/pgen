# ⛔ Disk hygiene is PROACTIVE — never reminded, never reactive (director, 2026-07-18)

- **Category:** feedback (standing discipline)
- **Date:** 2026-07-18
- **Status:** standing
- **Related:** [feedback_host_ram_budget_all_jobs.md](feedback_host_ram_budget_all_jobs.md) ·
  [feedback_cargo_sweep_cadence.md](feedback_cargo_sweep_cadence.md) ·
  `docs/tasks/OPS-MEMSAFE.md` leaf `.2`

## Context

2026-07-18, session #146: the RGX-0078 `.5.i.15` heavy probe build died ~5 minutes in on
`No space left on device` — the host disk was at **100% (1.1 GB free of 461 GB)** and the
failure tore the build's own incremental state. `rust/target/debug/incremental` alone held
**66 GB**; two forgotten 6.4 GB regen-train logs and ~15 GB of stale gate build caches sat
alongside. The session-startup directive (§8 artifact cleanup, every 12–24 h) existed but had
not been executed proactively; the RAM guard pre-flighted memory but NOTHING pre-flighted
disk. The director had to interrupt with an emergency stop-and-clean order and stated the
standing expectation verbatim: **"I shouldn't have to tell you to clean your work regularly.
I shouldn't have to remind you that."** Cleanup recovered ≈111 GB (1.1 GB → 112 GB free)
with zero loss of load-bearing artifacts (preserved floor probes, raw profile evidence, and
instruments all verified intact afterwards).

## Decision

1. **Proactive cadence, self-initiated.** At session start (and roughly every 12–24 h in
   long sessions), CHECK free disk (`df -h /`) and run the §8 artifact cleanup when free
   space is low or the last cleanup is >24 h old. The director must NEVER have to ask.
2. **Mechanical backstop.** `scripts/run_with_memory_guard.sh` now enforces a DISK floor
   alongside the RAM floor (`OPS-MEMSAFE.2`): pre-flight refusal (exit 96,
   `reason=disk-floor`) and in-flight breach kill (exit 95) below `--disk-floor-gb`
   (default 8; 0 disables). Every heavy job already runs under the guard, so a heavy job
   can no longer launch into — or silently fill — a nearly-full disk.
3. **The 100%-safe deletion inventory** (proven by this incident's cleanup):
   `rust/target/debug/incremental/` (pure rebuildable cache; the largest single consumer),
   `rust/target/release/incremental/`, gate-local cargo target caches
   (`rust/target/parse_harness_*`, `rust/target/ebnf_frontend_build`), `*.log`/`*.bin`
   under `rust/target/**`, bulk build logs under `rust/target/generated_logs/` whose
   durable extracts are already in-repo, and `cargo sweep --time 1` for the remainder.
4. **The KEEP list (never bulk-delete):** preserved bench probe binaries + `.sha` files
   (the validated floor references, e.g. `rust/target/generated_logs/spine_bench/`,
   `termlit_emission/bench/`), raw profiler sample data backing tracked categorizer
   regression proofs (`reprofile*/sample*_raw.txt`), artifact-vintage snapshots
   (`regex_parser.<hash>.rs`), and `~/.cargo` (offline-cached crates are load-bearing —
   the `-0118` smallvec decision depended on one).
5. **Sequencing hazard recorded:** `cargo sweep --time 1` and a checkpoint-warm profile
   build cache are in tension — sweep prunes by mtime and the warm cache lives under
   `rust/target/`; sweep AFTER a pending warm-cache build, not before.

## Consequences

- A full disk can no longer kill a guarded heavy job mid-write: the guard refuses at
  launch or kills cleanly at the floor with an observable marker, before ENOSPC corrupts
  incremental state.
- Session workflow gains a standing self-initiated hygiene step; being reminded by the
  director is treated as a discipline failure, not a normal trigger.
- The safe-delete/keep inventory above turns future cleanups into a mechanical sweep
  instead of a risk assessment under pressure.

# feedback_host_ram_budget_all_jobs

- **Category:** feedback (standing director directive)
- **Date:** 2026-07-14
- **Status:** binding until RGX-0078 closure (then re-confirm)

## Context

On 2026-07-14 the host (24 GB Mac mini) crashed from memory exhaustion while three
AI sessions ran on three different projects. Jetsam evidence
(`/Library/Logs/DiagnosticReports/JetsamEvent-2026-07-14-081536.ips`) names the
culprit: `nexsim_core` at ≈52 GB resident — a co-resident NON-PGEN process; no
pgen/cargo process appeared among the top consumers (pgen exonerated). The crash
still cost this project: the reboot wiped `/private/tmp`, destroying the
#114–#116 session scratchpads (bench logs, probe binaries, and the
`p4i_dumps`/`p4i_ast` byte-identity reference dumps — see `PGEN-RGX-0078-0069`).

## Decision (director, 2026-07-14)

The machine runs ONLY this project until the goal is met, and the agent is
accountable that **no spawned job can exhaust host RAM**. Be very cautious.

## Consequences (how to apply — binding)

1. **ONE heavy job at a time.** Heavy = fat-LTO builds (the SV parser source is
   ≈123 MB), full-corpus benches/sweeps, profilers, and SV/UVM parse jobs (the
   known ≈14 GB-residual class, `project_uvm_memory_not_the_memo.md`). Never run
   two heavy jobs concurrently; never overlap a bench with a build.
2. **Pre-flight check.** Before launching any heavy job, check free RAM
   (`memory_pressure -Q`) and do not launch into a pressured system.
3. **Memory-guard wrapper (mechanical enforcement) — ✅ DELIVERED 2026-07-14
   (`OPS-MEMSAFE.1`, `PGEN-OPS-MEMSAFE-0002`).** Heavy/background jobs run
   under `scripts/run_with_memory_guard.sh`: samples the job's process-tree RSS
   on an interval, kills the tree (with a breach marker + log line) when it
   exceeds a budget (default ≈12 GB = half RAM) OR when system-free drops below
   a floor (default ≈10%). Verified by battery T1–T9 (23/23). Incident note:
   the FIRST build's own T2 test killed the host's user session (awk
   auto-vivification made the tree walk system-wide; 2026-07-14 13:04) —
   root-caused, fixed, and pinned by regression test T8 plus three kill-path
   fail-safes; full record in `docs/tasks/OPS-MEMSAFE.md`.
4. Composes with `feedback_background_job_observability.md` (completion marker +
   bounded timeout + liveness probe; no self-matching `pgrep`) and
   `feedback_dont_run_jobs_that_hit_known_pathological_inputs.md`.

## Budget calibration (amended 2026-07-15, `PGEN-RGX-0078-0087`)

The D2-A fused cascade emission grew every generated artifact (regex ×1.11, SV +5.5%,
fused graphs in all 11 parsers), which pushed the two heavy compile classes past the
12288 MB default budget on this 24 GB host:

- the dual-feature test-binary compile (`cargo test --features "generated_parsers
  ebnf_dual_run" --lib`) peaked **12403 MB** — the guard correctly killed it (exit 97);
- the fat-LTO release builds now peak **≈12.5 GB** (previously ≈12.1–12.2 GB).

Calibrated posture: run BOTH classes under the guard at an explicit
`--budget-mb 16384`, keeping the 10% system-free floor (the actual host-exhaustion
backstop) unchanged. Measured green peaks at that budget: 12076 MB (cold test compile),
7389 MB (warm test run), 12531/12360 MB (fat-LTO A/B builds). The default in the script
stays 12288 so casual jobs keep the tighter bound; the 16384 figure is the recorded
per-job choice for these two named classes.

## Host pressure is not "free %", and a guard kills only what it owns (amended 2026-10-20, `PGEN-OPS-MEMSAFE-0022`)

On 2026-10-20 a root-owned `fseventsd` (PPID 1, started at boot, never inside any job tree)
grew to 9–10 GB RSS at 99 % CPU on an 11-day FSEvents journal of the build volume (116,998
files, 4.0 GB, under Spotlight indexing of `rust/target`); swap reached 22.06 of 23.55 GB, and
the host was unusable for hours while two guarded push-mirror runs tripped no axis: the guard's
free-memory floor read **51 % free** throughout, because `memory_pressure -Q` counts purgeable
and cached pages as free. The operator remediated the host outside every repository (`no_log`
on the volume, Spotlight off, the journal pruned, the daemon restarted at 4 MB). Measured and
recorded in `docs/tasks/OPS-MEMSAFE.md` leaf `.6`; the figure that did move was `vm_stat`'s
swap-out counter.

Consequences (binding, additive to 1–4 above):

5. **A guard kills only what it owns.** Host pressure caused by something outside the job's
   tree is OBSERVED and RECORDED (the swap-out rate, the largest unowned process, the tree's RSS
   at the pressure peak — eight marker fields), never killed on: a healthy seven-hour run killed
   for a daemon's leak is a destructive non-fix, and the 2026-07-14 session kill is the
   precedent. Exit 94 is reserved for a swap-out breach kill and stays unimplemented until the
   recorded markers show what rule would have been right.
6. **Refuse to launch into a host hazard, by name.** The guard's pre-flight refuses (exit 96)
   when `fseventsd` holds more than 512 MB (`reason=host-hazard`) or the swap-out rate across a
   3 s window is at or over 1,000 pages/s (`reason=host-pressure`), and names the cause instead
   of grinding for hours. Where a reading cannot run (no daemon, no `vm_stat`: a Linux runner)
   it says NOT EVALUATED and launches. Host configuration (the `no_log` marker, Spotlight) stays
   outside the guard's remit.
7. **The budget is derived, never chosen at invocation.** `budget = min(max measured peak × 1.2,
   half of physical RAM)`; when the first term exceeds the second, reduce the peak
   (`OPS-MEMSAFE.4` question 3) — do not raise the budget. The guard warns on any budget over
   half of RAM. For the push mirror the guard's own markers give the table below, so its one
   sanctioned launch is `make -C rust SHELL=/bin/bash ci_workflow_local_gate_guarded` at
   **12,288 MB** (`min(10,943 × 1.2, 24,576 / 2)`); four budgets had been on disk for the same
   job (`8192`, `12288`, `14336`, `16384`), chosen in-session.

| budget_mb | full mirror runs' `peak_rss_mb` (2026-09 → 2026-10-20) | status |
| --- | --- | --- |
| 12288 | 9,564 · 9,639 | completed |
| 14336 | 5,967 · 8,135 · 8,878 · 9,620 · 9,873 · 10,014 · 10,419 | six completed; 10,419 `guard-interrupted`, not a breach |
| 16384 | 5,010 · 5,090 · 10,943 | completed |

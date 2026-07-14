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

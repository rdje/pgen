---
id: a-guard-kills-only-what-it-owns-and-a-script-parses-itself-before-it-runs
title: A guard kills only what it owns, "free %" is not pressure, the budget is derived from measured peaks, and a script that must survive its own run parses itself whole before it launches anything
answers:
  - "why does the memory guard not kill the job when the host is swapping"
  - "why did two guarded push-mirror runs trip nothing while fseventsd held 9 GB and swap was exhausted"
  - "is memory_pressure -Q's free percentage a pressure reading"
  - "which metric moves when the host thrashes swap"
  - "what does exit 96 reason=host-hazard or reason=host-pressure mean"
  - "what is exit 94 reserved for in run_with_memory_guard.sh"
  - "how is the push mirror's memory budget derived and where is it pinned"
  - "why is run_with_memory_guard.sh one function called on its last line"
  - "why did push-mirror run 10's guard die with a syntax error after the mirror passed"
  - "what happens when a running bash script is rewritten in place"
tags: [ops, memory-guard, host-pressure, swap, fseventsd, bash, parse-before-run, budget, push-mirror]
date: 2026-10-20
status: current
evidence: "PGEN-OPS-MEMSAFE-0022 (OPS-MEMSAFE.6): on 2026-10-20 a root-owned fseventsd held 9–10 GB for hours and swap stood at 22.06 of 23.55 GB while the guard read last_free_pct=51 (memory_pressure -Q counts purgeable and cached pages as free); vm_stat's swap-out counter was the metric that moved (MemWatch logged 1,141–1,327 pages/s). The job's tree stood at 35–76 % of its budget, so a kill rule would have killed a healthy run for a daemon's leak. The same day mirror run 10 finished green and its guard died at the loop's end on its own script, rewritten in place under it by a hand outside the session: bash reads a running script by file offset. Battery T12–T21 + T20c 15 / 15 (rust/target/svpp9/guard6/battery.sh)."
reverify: "bash -n scripts/run_with_memory_guard.sh && tail -n 1 scripts/run_with_memory_guard.sh | grep -qxF 'main \"$@\"' && scripts/run_with_memory_guard.sh --help | grep -qF 'host-hazard | host-pressure' && scripts/run_with_memory_guard.sh --help | grep -qF '94  (reserved)' && grep -qF 'ci_workflow_local_gate_guarded:' rust/Makefile && echo 'memory-guard: parse-before-run, host axes, exit 94 reserved, the mirror budget pinned — OK'"
---

**Four rules, each measured on 2026-10-20.**

## The rules

1. **A guard kills only what it owns.** The guard's kill axes (RSS budget, free floor, disk floor,
   timeout) are scoped to the job's process tree by design. Host pressure caused by a process
   outside that tree is a reason to REFUSE a launch (exit 96, `reason=host-hazard` when
   `fseventsd` holds more than `--fseventsd-limit-mb`, `reason=host-pressure` when the swap-out rate
   across a 3 s window is at or over `--swapout-floor`) and a reason to RECORD while the job runs
   (one warning per episode naming the largest process outside the tree; marker fields
   `peak_swapout_ps`, `pressure_samples`, `tree_mb_at_peak_swapout`, `unowned_top`), never a reason
   to kill the job. Exit 94 is reserved for a swap-out breach kill and is not implemented: on the
   day the rule would have been written, the tree stood at 35–76 % of its budget while a daemon was
   the cause, so a "kill when the tree is over 70 % under pressure" rule would have killed a healthy
   seven-hour run. The kill rule waits on the data the marker now records.
2. **"Free %" is not pressure.** `memory_pressure -Q` counts purgeable and cached pages as free; it
   read 51 % while swap was exhausted and the host was unusable. The figure that moves is `vm_stat`'s
   cumulative `Swapouts:` counter; its rate (pages per second, page size 16 384) is the axis.
3. **Derive the budget, never choose it at launch.** The markers already hold every peak; the
   budget is `min(max measured peak × 1.2, half of physical RAM)` and is pinned as a literal with its
   derivation beside it (`ci_workflow_local_gate_guarded`, 12 288 MB). A budget over half of RAM
   prints a warning; the lever is the peak, not the number. A runtime derivation from an untracked
   marker directory would make a launch depend on what a cleanup left behind.
4. **A script that must survive its own run parses itself whole before it launches anything.** bash
   reads a script incrementally, by file offset, as it executes: a compound command is parsed in
   full, but the first command after it is read from the file when the compound command ends. Run
   10's guard ran its monitor loop for 6 h 48 min, then read the bytes after the loop from a file
   that had been rewritten in place under it (and discarded through git, by a hand outside the
   session): `line 347: syntax error near unexpected token`, exit 2, the marker lost. The rule "never
   edit a running script in place" is necessary and was not sufficient, because the hand was not the
   session's. The guard's whole body is now one `main()` called on the last line, every path out of
   it an `exit`, so bash has parsed every byte before the job starts and never reads the file again.
   T20 (a copy rewritten in place, same inode, 1 s into a 4 s job ⇒ exit 7 through, marker written)
   and T20c (the previous guard under the same rewrite ⇒ rc 2, no marker) prove it in five seconds.

## Where it lives

- `scripts/run_with_memory_guard.sh` — the header documents the axes, the exit codes and
  PARSE-BEFORE-RUN; `--help` prints it.
- `docs/decisions/feedback_host_ram_budget_all_jobs.md` — the directive and its consequences 5–7
  (amended 2026-10-20).
- `docs/tasks/OPS-MEMSAFE.md` leaf `.6` — the incident, the evidence, the battery, what is owed.
- `docs/book/src/operations-and-governance.md` — "What the guard cannot see, and what it does
  about it".

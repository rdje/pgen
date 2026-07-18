# OPS-MEMSAFE: Host-RAM budget mechanical enforcement (the memory-guard wrapper)

## Metadata

- Tree ID: `OPS-MEMSAFE`
- Status: `complete` (2026-07-18 — REOPENED same day for leaf `.2` disk-floor guard after the RGX-0078 `.5.i.15` probe build died on host-disk exhaustion, and CLOSED with T10/T11 + regression green; previously `complete` 2026-07-14 with battery T1–T9; the DIRECTIVES stay binding: heavy jobs run UNDER the guard, disk hygiene is proactive)
- Roadmap lane: operational continuity / host-resource governance (standing director directive `docs/decisions/feedback_host_ram_budget_all_jobs.md` + the 2026-07-18 disk-hygiene directive `docs/decisions/feedback_disk_hygiene_proactive.md`)
- Created: `2026-07-14`
- Last updated: `2026-07-18`
- Owner: repo-local workflow

## Goal

Mechanically enforce the HOST-RAM BUDGET DIRECTIVE (director, 2026-07-14, post-crash):
**no spawned job may exhaust host RAM** on the 24 GB single-project machine. Deliver
`scripts/run_with_memory_guard.sh` — a wrapper every heavy/background job runs under —
that (a) pre-flights system free RAM before launch, (b) samples the job's
process-tree RSS on an interval, and (c) kills the whole tree (with an observable
breach marker + always-on log line) when the tree exceeds its RSS budget
(default ≈12 GB = half RAM), when system-wide free memory drops below a floor
(default 10%), or when an optional wall-clock timeout expires. Building this wrapper
was recorded as the tracked FIRST work item of this session (`PGEN-OPS-MEMSAFE-0001`)
and gates any heavy job.

## Non-Goals

- Not a per-process ulimit/cgroup substitute (macOS has no cgroups; the guard is a
  sampling supervisor, not a kernel limit).
- Not a scheduler: serializing heavy jobs (ONE at a time) remains workflow discipline
  (directive consequence #1); the guard makes a single job's blast radius bounded.
- No engine/parser/codegen surface is touched.

## Acceptance Criteria

- `scripts/run_with_memory_guard.sh` exists, is executable, and implements:
  pre-flight free-RAM check (refuses to launch into a pressured system),
  process-tree RSS sampling (process group + descendant walk), RSS-budget kill,
  system-free-floor kill, optional timeout kill, TERM→grace→KILL escalation,
  always-written completion/breach marker (composes with
  `feedback_background_job_observability`), exit-code transparency for
  well-behaved jobs, and `PGEN_TRACE_VERBOSITY`-aligned verbosity where
  breach/warning severity is NEVER gated by verbosity
  (`feedback_severity_never_gated_by_verbosity`).
- A deterministic test battery proves each kill path and the no-interference path.
- Live docs + book lockstep: README (Standard Commands), the book's
  Operations and Governance chapter, the decision record's status, CHANGES,
  DEVELOPMENT_NOTES, MEMORY.md.
- Committed per `COMMIT.md` with this tree's leaf ID in the subject.

## Task Tree

- ID: `OPS-MEMSAFE`
  Status: `complete`
  Goal: mechanical host-resource (RAM + disk) budget enforcement for spawned jobs
  Children: `OPS-MEMSAFE.1`, `OPS-MEMSAFE.2`

- ID: `OPS-MEMSAFE.1`
  Status: `done`
  Goal: implement + verify `scripts/run_with_memory_guard.sh` (budget/floor/timeout kill paths, tree-wide accounting + kill, markers, pre-flight)
  Acceptance: test battery T1–T9 below all PASS; docs lockstep complete
  Verification: see Verification Log
  Commit: `PGEN-OPS-MEMSAFE-0002`

- ID: `OPS-MEMSAFE.2`
  Status: `done`
  Goal: DISK-floor enforcement in the same guard — pre-flight refusal + in-flight breach kill when free disk on the guard's filesystem drops below `--disk-floor-gb` (default 8; 0=off), with its own exit code 95, marker fields, and a deterministic test seam (`PGEN_MEMORY_GUARD_FAKE_DISK_FREE_GB_FILE`), mirroring the RAM floor's proven design
  Why: 2026-07-18 incident — the RGX-0078 `.5.i.15` probe build died mid-compile on `No space left on device` (host disk 100%, 1.1 GB free; `rust/target/debug/incremental` alone was 66 GB) and tore its own incremental state; the RAM guard pre-flighted RAM but NOTHING pre-flighted disk. Director directive same day: artifact hygiene must be proactive, not reminded (`docs/decisions/feedback_disk_hygiene_proactive.md`)
  Acceptance: T10 (pre-flight disk refusal: fake 2 GB < floor 8 ⇒ exit 96, marker `reason=disk-floor`, command NEVER ran) + T11 (in-flight breach: fake 50→2 GB mid-run ⇒ tree killed, exit 95, marker `reason=disk-floor`) + regression (RAM pre-flight T5 replay still exit 96 `reason=free-floor`; normal completion still transparent; `bash -n` + doctrines green); docs lockstep (README exit codes, book operations-and-governance, host-RAM decision record cross-ref)
  Verification: see Verification Log
  Commit: `PGEN-OPS-MEMSAFE-0003`

## Current Frontier

| Order | Leaf | Status | Why next |
| --- | --- | --- | --- |
| — | `OPS-MEMSAFE.2` | `done` | Disk-floor axis delivered (T10/T11 + regression green); tree complete again. |
| — | `OPS-MEMSAFE.1` | `done` | Wrapper delivered + fratricide defect fixed; heavy jobs are unblocked (run them UNDER the guard). |

## Acceptance Checklist (enforced)

- [x] **REPRODUCE / ISSUE** — the first wrapper build KILLED THE HOST'S USER SESSION when its own T2 test ran (2026-07-14 ~13:04): the guard's tree walk claimed 645 pids / 12,597 MB for a 300 MB perl balloon, then TERM/KILL'd every user-owned process on the machine (the AI session's node, Terminal children, the battery bash, and the guard itself — whose INT/TERM trap then re-fired in a loop). Evidence: the dead session's surviving `t2.log`/`t2.marker` (`BREACH (rss-budget): tree RSS 12597MB > budget 100MB (pids=645)`, marker `reason=guard-interrupted exit=130 peak_rss_mb=12597 ended_at=13:04:32`), the 13:04:32 bash crash report (`~/Library/Logs/DiagnosticReports/bash-2026-07-14-130440.ips`), and a LIVE repro of the walk against a lone `sleep`: 602 pids / 12,215 MB marked.
- [x] **ROOT CAUSE (WHY + WHERE)** — awk ARRAY AUTO-VIVIFICATION in the tree-membership closure (both `sample_tree_rss_kb` and `list_tree_pids`): `if (!mark[p] && (ppid[p] in mark))` — merely READING `mark[p]` creates key `p`, and `in` tests key EXISTENCE, not truthiness; after one sweep every pid on the system is a `mark` key, so the closure marks the entire process table. WHERE: `scripts/run_with_memory_guard.sh` (pre-fix lines 150 and 167).
- [x] **FIX** — (a) existence test `!(p in mark)` replaces the vivifying `!mark[p]` (single shared `TREE_WALK_AWK` for both walk functions); (b) FAIL-SAFE: the walk emits NOTHING (= failed sample, never drives a kill) if it marks pid 1, the guard, or any guard ancestor; (c) `signal_tree` never signals pid ≤ 1, the guard, or the root outside the group-kill; the kernel-scoped `kill -- -pgid` stays the primary kill; (d) INT/TERM trap is reset on entry to `breach()`/`on_guard_signal()` — no re-entry loop, no marker overwrite.
- [x] **ADDRESSED (verified)** — battery T1–T9 = 23/23 PASS in-session (the session SURVIVED T2/T3 this time); T2 breach line now `tree RSS 603MB > budget 100MB (pids=1)`, T3 `621MB (pids=4)` (bash -c + perl + 2 sleeps — exact membership); T8 regression test pins the scoping (max sampled pids=1 for a lone perl; pre-fix identical setup marked 602).
- [x] **NO REGRESSION** — wrapper is NEW (no consumer yet); `bash -n` clean; `scripts/check_doctrines.sh` ALL 6 PASS; no engine/parser/codegen surface touched.
- [x] **LOCKSTEP** — this tree + `docs/TASK_TREE.md` index row + decision record status (`docs/decisions/feedback_host_ram_budget_all_jobs.md`) + README Standard Commands + book `operations-and-governance.md` + `CHANGES.md` + `DEVELOPMENT_NOTES.md` + `MEMORY.md`, same commit.

## Verification Battery (leaf `.1`) — ALL PASS 2026-07-14

- T1 exit-code transparency: `--budget-mb 1024 -- bash -c 'exit 7'` → wrapper exits 7, marker `status=completed exit=7`. **PASS**
- T2 RSS-budget breach: a ~300 MB balloon under `--budget-mb 100 --interval-s 1` → tree killed, exit 97, marker `reason=rss-budget`, balloon PID gone. **PASS** (breach line: `tree RSS 603MB … (pids=1)`)
- T3 tree-wide accounting + kill: the balloon as a GRANDchild among sibling `sleep`s → breach still detected (tree RSS, not direct-child RSS); ALL descendants gone after the kill. **PASS** (`621MB (pids=4)`)
- T4 in-flight floor breach: fake-free-pct test seam file 90→5 with `--floor-pct 10` → tree killed, exit 98, marker `reason=free-floor`. **PASS**
- T5 pre-flight refusal: fake free pct 5 with `--floor-pct 10` → exit 96, marker `status=preflight-refused`, the command NEVER ran (side-effect file absent). **PASS**
- T6 timeout: `--timeout-s 2 -- sleep 600` → tree killed, exit 99, marker `reason=timeout`. **PASS**
- T7 static sanity: `bash -n` clean; `scripts/check_doctrines.sh` green (ALL 6). **PASS**
- T8 tree-walk scoping regression (NEW — pins the 2026-07-14 fratricide defect): huge budget + `PGEN_TRACE_VERBOSITY=medium` over a lone perl → max sampled `pids` ≤ 3 and plausible peak RSS. **PASS** (max pids=1, peak 3 MB; pre-fix: 602 pids / 12,215 MB)
- T9 guard-interrupted (NEW): TERM to the guard mid-run → exit 130, marker `reason=guard-interrupted`, trap fires EXACTLY once, child gone. **PASS**

## Acceptance Checklist (enforced) — leaf `.2` disk-floor guard

- [x] **REPRODUCE / ISSUE** — 2026-07-18 00:29: the RGX-0078 `.5.i.15` guarded probe build died mid-compile with `error: failed to write file …/incremental/…/dep-graph.part.bin: No space left on device (os error 28)` (`rust/target/generated_logs/reprofile16/build16.log`); `df -h /` read **461G disk, 460G used, 1.1G free, 100%** — the guard pre-flighted RAM (`free=78%`, healthy) and launched anyway, because no disk axis existed.
- [x] **ROOT CAUSE (WHY + WHERE)** — `scripts/run_with_memory_guard.sh` guarded exactly three resource axes (tree RSS, system free RAM %, wall clock) and ZERO disk axes: the pre-flight block read only `free_pct()` (RAM) and the monitor loop sampled only RSS + RAM + timeout — a heavy compile writing tens of GB of incremental state had no launch gate and no in-flight kill before hard ENOSPC. WHERE: the pre-flight section and monitor loop (pre-fix — no `df` read anywhere in the script, verified by grep).
- [x] **FIX** — tier: engine/ops script (the guard is workflow infrastructure, no parser surface). Mirror the RAM floor's proven design on the disk axis: `free_disk_gb()` via POSIX `df -Pk .` (line 2 col 4 = available KB; guard launched from the job's build filesystem by repo convention) with deterministic seam `PGEN_MEMORY_GUARD_FAKE_DISK_FREE_GB_FILE`; `--disk-floor-gb N` (default 8, `0` disables); pre-flight refusal exit 96 marker `reason=disk-floor` (status `preflight-refused` shared with the RAM axis, reason disambiguates); in-flight breach exit **95** through the EXISTING battle-tested `breach()`/`terminate_tree` kill path; marker gains `disk_floor_gb=`/`last_disk_free_gb=`; header/usage/started-line updated.
- [x] **ADDRESSED (verified)** — **T10** pre-flight refusal (fake 2 GB < floor 8): exit 96, `status=preflight-refused reason=disk-floor last_disk_free_gb=2`, side-effect file ABSENT (command never ran). **T11** in-flight breach (fake 50→2 GB mid-run, interval 2 s): `BREACH (disk-floor): free disk 2GB < floor 8GB`, exit **95**, `status=killed reason=disk-floor`, tree gone (`pgrep` empty). Live validation: the relaunched `.5.i.15` probe build runs under the hardened guard — started line `budget=16384MB floor=10% disk_floor=8GB … free=77% disk=109GB` (`build16b.log`).
- [x] **NO REGRESSION** — T5-replay RAM pre-flight (fake RAM 5% / disk healthy): exit 96 `reason=free-floor`, side-effect absent — UNCHANGED; transparency (no seams, real `df`): child `exit 7` passes through, marker `status=completed` with `last_free_pct=77 last_disk_free_gb=110` (matches actual host free disk); `--disk-floor-gb 0` disables the axis (fake 2 GB, child exit 0 passes); `bash -n` clean; `scripts/check_doctrines.sh` ALL 7 PASS (pre-commit run); RSS/timeout/guard-interrupt paths untouched (no edit inside their logic — the disk check is an additive block in the loop + pre-flight).
- [x] **LOCKSTEP** — this tree (metadata/task-tree/frontier/checklist/logs) + README Standard Commands (option + exit codes incl. 95) + book `operations-and-governance.md` + NEW decision record `docs/decisions/feedback_disk_hygiene_proactive.md` + `INDEX.md` row + `docs/TASK_TREE.md` index row + `CHANGES.md` + `DEVELOPMENT_NOTES.md` + `MEMORY.md`, same commit; `mdbook_docs_gate` green (book touched).

## Verification Battery (leaf `.2`) — ALL PASS 2026-07-18

- T10 pre-flight disk refusal: fake free-disk seam 2 GB with `--disk-floor-gb 8` → exit 96, marker `status=preflight-refused reason=disk-floor`, the command NEVER ran (side-effect file absent). **PASS**
- T11 in-flight disk breach: fake seam 50→2 GB mid-run → tree killed, exit 95, marker `reason=disk-floor`, `last_disk_free_gb=2`, no survivors. **PASS**
- T5-replay (RAM axis regression): fake RAM 5% < floor 10 → exit 96 `reason=free-floor`, command never ran — byte-for-byte the `.1` behavior. **PASS**
- Transparency regression: healthy system, real `df` — child `exit 7` passes through; marker `completed`/`none` with both floors' last-readings populated. **PASS**
- Disable switch: `--disk-floor-gb 0` with fake 2 GB free → child runs and exits 0. **PASS**
- Static sanity: `bash -n` clean; doctrine driver ALL 7 PASS at commit. **PASS**

## Leaf `.3` — destructive-target guard + probe custody relocation (opened 2026-07-19, `PGEN-OPS-MEMSAFE-0004`, status: `done`)

**Trigger (director directive, emphatic, 2026-07-19):** *"Please, please, please, make this
sort of incident does not happen again"* — in response to the 2026-07-18 incident report:
`make -C rust annotation_parsers` (name promises "regenerate the annotation pair") aliased
`return_semantic_parsers`, whose `clean` dep deleted every generated artifact AND ran
`cargo clean` — 102.8 GiB destroyed, including every preserved perf-probe binary
(`d9d3d611`/`ee10968c`/`7fd0a31b`/`4bbfb4e9`/`d12513c0`), forcing a full cold bootstrap +
two cold fat-LTO probe rebuilds (~2 h). The alias itself was de-fanged same-day
(`PGEN-RGX-0078-0141`); this leaf generalizes the protection so the CLASS is closed.

**Delivered (three legs, mechanical-first):**
1. **Refuse-by-default destruction** — the `clean` recipe exits **96** with a loud,
   self-explaining refusal unless `PGEN_CONFIRM_CLEAN=1`; the whole dependent family
   (`clean-all`, `rebuild`, `return_semantic_parsers`, `bootstrap-test`) is guarded
   TRANSITIVELY (make stops on the failed dep). Legitimate cleans remain one explicit
   command: `PGEN_CONFIRM_CLEAN=1 make -C rust clean`.
2. **Doctrine enforcement** — `scripts/check_destructive_target_guard.sh` (STRUCTURAL
   archetype): asserts the guard exists in `clean:` BEFORE any destructive line, that
   `annotation_parsers` does not route into the destructive family, and that the set of
   targets depending on `clean`/`clean-all` equals the explicit allowlist. Registered in
   `scripts/check_doctrines.sh` (pre-commit E3 + CI E4). **First-run catch:** the check
   immediately surfaced a SECOND latent family member (`bootstrap-test: clean-all`) —
   adjudicated onto the allowlist (transitively guarded).
3. **Probe custody out of the blast radius** — new top-level gitignored
   `preserved_probes/` (naming: `<bin>_<slice-tag>_<sha8>`); no `cargo clean` touches it.
   Current residents: `regex_perf_probe_pre_k3a_15c218d7` + `regex_perf_probe_k3a_1ada62fe`.
   `rust/target/generated_logs/` is henceforth SCRATCH-ONLY. Durable custody remains the
   banked numbers in `docs/tasks/artifacts/` (git-tracked) + rebuild-with-floor-validation
   (exercised successfully during the incident recovery).

Decision record: `docs/decisions/feedback_no_unguarded_destructive_targets.md` (+ INDEX).
Process rule recorded in MEMORY: `make -n <target>` before any first-time invocation.

## Acceptance Checklist (enforced) — leaf `.3` destructive-target guard
- [x] **REPRODUCE / ISSUE** — the incident is fully recorded with evidence (`PGEN-RGX-0078-0141` leaf + `docs/tasks/artifacts/k3_constants/`; the destroyed-state inventory and recovery cost named).
- [x] **ROOT CAUSE (WHY + WHERE)** — WHY: destructive recipes were reachable through innocuously-named aliases/deps with zero friction. WHERE: `rust/Makefile` `clean:` (rm + `cargo clean`), reached via `annotation_parsers → return_semantic_parsers → clean` (and latently `bootstrap-test → clean-all → clean`).
- [x] **FIX** — refuse-by-default guard on `clean` (exit 96, `PGEN_CONFIRM_CLEAN=1` opt-in) + doctrine check + registry entry + probe relocation + decision record.
- [x] **ADDRESSED (verified)** — negative test: `make -C rust clean` REFUSES exit 96, nothing deleted; check script PASS on the real Makefile; mutation probe (guard stripped from a Makefile copy) shows the check's guard-detection fails as designed; the check's first real run caught `bootstrap-test`.
- [x] **NO REGRESSION** — the guard touches ONLY the `clean:` recipe (non-destructive targets never evaluate it); Makefile parse integrity smoke-tested (`make -n focus_regex` OK); `bash scripts/check_doctrines.sh` = ALL 8 enforced doctrines PASS including the new check; the in-flight `-0142` gate chain runs make targets through the guarded Makefile.
- [x] **LOCKSTEP** — decision record + INDEX + MEMORY (repo) + session memory + this leaf + `docs/TASK_TREE.md` row same-commit.

## Decisions

- `2026-07-14`: Sampling supervisor design (ps snapshot: process-group members ∪ ppid-descendant closure; macOS `memory_pressure -Q` for the system-free floor; `PGEN_MEMORY_GUARD_FAKE_FREE_PCT_FILE` as the deterministic test seam for the floor paths). Distinct exit codes per outcome (96 preflight-refused / 97 rss-budget / 98 free-floor / 99 timeout / 130 guard-interrupted; otherwise the child's own exit code) so callers can branch on the guard verdict mechanically.
- `2026-07-14`: Honest limit recorded: a descendant that re-parents AND changes its own process group escapes the walk; jobs PGEN spawns (make/cargo/bench/parse) do neither. The guard is bounded-blast-radius enforcement, not a kernel limit.
- `2026-07-14` (post-fratricide hardening): a kill list computed from a SAMPLED walk is UNTRUSTED input to the kill path — three independent rails now hold even if the walk is wrong again: (1) INSANE-SAMPLE VOIDING (a walk that marks pid 1 / the guard / a guard ancestor yields an empty result — a warned no-op, never a kill), (2) the kernel-scoped process-group kill (`kill -- -pgid`) is the primary kill and needs no walk at all, (3) `signal_tree` refuses pid ≤ 1 and the guard's own pid outright. General lesson (portable beyond this script): in awk, `!arr[k]` is a MEMBERSHIP BUG — reading `arr[k]` auto-vivifies the key and corrupts every later `in` test; membership is `!(k in arr)`.

## Open Questions

- None.

## Blockers

- None.

## Verification Log

| Date | Leaf | Checks | Result |
| --- | --- | --- | --- |
| `2026-07-14` | `OPS-MEMSAFE.1` | first build, battery run in-session | **FATAL DEFECT CAUGHT BY ITS OWN T2**: tree walk marked the whole process table (645 pids / 12,597 MB) → guard killed every user process incl. the running AI session (session death 13:04:32) |
| `2026-07-14` | `OPS-MEMSAFE.1` | live walk repro (pre-fix logic, lone `sleep` root) | 602 pids / 12,215 MB marked ⇒ defect reproduced deterministically, independent of the balloon |
| `2026-07-14` | `OPS-MEMSAFE.1` | T1–T9 after the fix (`bash -n` + doctrines = T7) | **23/23 PASS**; T2 breach `603MB (pids=1)`, T3 `621MB (pids=4)`, T8 max pids=1, T9 trap fires once; session survived T2/T3 |
| `2026-07-18` | `OPS-MEMSAFE.2` | T10/T11 + T5-replay + transparency + disable-switch + `bash -n` + doctrines | **ALL PASS**: T10 exit 96 `reason=disk-floor` (never launched), T11 exit 95 tree killed, RAM axis + exit-code transparency byte-unchanged; live `.5.i.15` relaunch shows `disk_floor=8GB disk=109GB` |

## Commit Log

| Leaf | Commit subject or reference | Notes |
| --- | --- | --- |
| `OPS-MEMSAFE.1` | `PGEN-OPS-MEMSAFE-0002 (leaf OPS-MEMSAFE.1)` | wrapper + fratricide root-cause fix + T1–T9 battery green + docs lockstep |
| `OPS-MEMSAFE.2` | `PGEN-OPS-MEMSAFE-0003 (leaf OPS-MEMSAFE.2)` | disk-floor axis (pre-flight + in-flight exit 95) + T10/T11 + regression green + disk-hygiene decision record + docs lockstep |
| `OPS-MEMSAFE.3` | `PGEN-OPS-MEMSAFE-0004 (leaf OPS-MEMSAFE.3)` | destructive-target guard (refuse-by-default `clean` family, exit 96) + doctrine check registered + `preserved_probes/` relocation + decision record + docs lockstep |

## Changelog

- `2026-07-14`: Created task tree; leaf `.1` opened as the session's directive-mandated first item.
- `2026-07-14`: First build's own T2 test killed the host's user session (awk auto-vivification made the tree walk system-wide; the kill list followed). Root-caused with the surviving guard logs + crash reports + a live repro; fixed (`!(p in mark)`) + three kill-path fail-safes; battery extended with T8 (scoping regression) + T9 (guard-interrupted); 23/23 PASS. Leaf `.1` DONE — heavy jobs now run UNDER the guard.
- `2026-07-18`: Tree REOPENED for leaf `.2` after the RGX-0078 `.5.i.15` probe build died on host-disk exhaustion (100% disk, 1.1 GB free — the guard had no disk axis). Disk floor delivered mirroring the RAM floor's design (pre-flight refusal exit 96 `reason=disk-floor`; in-flight breach exit 95 via the existing kill path; `--disk-floor-gb` default 8, 0 disables; `df -Pk .` + deterministic seam); T10/T11 + full regression PASS; standing disk-hygiene directive recorded (`docs/decisions/feedback_disk_hygiene_proactive.md`). Leaf `.2` DONE — tree `complete` again.
- `2026-07-19`: Tree REOPENED for leaf `.3` after the destructive-clean incident (director directive, emphatic): `make annotation_parsers` → alias → `clean` destroyed 102.8 GiB incl. every preserved perf probe. Delivered refuse-by-default guard on the `clean` family (exit 96, `PGEN_CONFIRM_CLEAN=1` opt-in), the registered doctrine check `check_destructive_target_guard.sh` (first run caught the latent `bootstrap-test` member), and the `preserved_probes/` relocation outside the `cargo clean` blast radius. Leaf `.3` DONE — tree `complete` again.

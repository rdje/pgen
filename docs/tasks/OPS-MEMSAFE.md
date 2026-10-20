# OPS-MEMSAFE: Host-RAM budget mechanical enforcement (the memory-guard wrapper)

## Metadata

- Tree ID: `OPS-MEMSAFE`
- Status: `active` (**`.6` DONE 2026-10-20, `PGEN-OPS-MEMSAFE-0022`** — the guard refuses to launch into a host hazard by name and observes host pressure in flight, never a kill; the mirror's budget derived and pinned; the guard parses its whole body before it launches anything, after run 10's guard died under an in-place rewrite from outside the session; `.4` stays open for spike visibility and peak reduction. **REOPENED 2026-07-27 session #214 for leaf `.4`** — the guard cannot see a
  spike between samples, and on this 24 GB host the RSS budget is not the binding constraint;
  opened by `GENERATED-LINT-CORRECTNESS.2` after it drew the WRONG conclusion from two OS
  kills. Previously `complete` 2026-07-18 — REOPENED same day for leaf `.2` disk-floor guard after the RGX-0078 `.5.i.15` probe build died on host-disk exhaustion, and CLOSED with T10/T11 + regression green; previously `complete` 2026-07-14 with battery T1–T9; the DIRECTIVES stay binding: heavy jobs run UNDER the guard, disk hygiene is proactive)
- Roadmap lane: operational continuity / host-resource governance (standing director directive `docs/decisions/feedback_host_ram_budget_all_jobs.md` + the 2026-07-18 disk-hygiene directive `docs/decisions/feedback_disk_hygiene_proactive.md`)
- Created: `2026-07-14`
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
  Children: `OPS-MEMSAFE.1`, `OPS-MEMSAFE.2`, `OPS-MEMSAFE.3`, `OPS-MEMSAFE.4`, `OPS-MEMSAFE.5`, `OPS-MEMSAFE.6`

- ID: `OPS-MEMSAFE.6`
  Status: `done`
  Goal: the guard refuses to launch into a host hazard it cannot own (`fseventsd` over `--fseventsd-limit-mb`, the swap-out rate at or over `--swapout-floor`; exit 96, `reason=host-hazard` / `host-pressure`), observes host pressure in flight (one warning per episode naming the largest process outside the job's tree; eight marker fields; never a kill, exit 94 reserved), warns on a budget over half of physical RAM, and gives the push mirror one sanctioned launch at a budget DERIVED from its own markers (`ci_workflow_local_gate_guarded`, 12,288); and parses its whole body before launching anything (one function, every exit an `exit`), so a rewrite of the script during a run cannot reach it — run 10's guard died that way at its loop's end
  Why: the 2026-10-20 incident — a root-owned `fseventsd` at 9–10 GB for hours, swap 22.06 of 23.55 GB, two guarded mirror runs tripping nothing because `memory_pressure -Q`'s free % (51 %) is not pressure; the report an external agent wrote for the director, every claim re-measured here
  Acceptance: battery T12–T21 + the T20c control + replays T1 / T2 / T5 / T9 all PASS (15 / 15) on the scratch copy and in place; the first live launch of the installed guard recorded; the host re-measured; docs lockstep (card, book, Makefile, indexes)
  Verification: see Verification Log
  Commit: `PGEN-OPS-MEMSAFE-0022`

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
| — | `OPS-MEMSAFE.6` | `done` 2026-10-20 (`PGEN-OPS-MEMSAFE-0022`) | The 2026-10-20 host incident: host-hazard + host-pressure pre-flight, in-flight observation (never a kill), the over-half-RAM warning, the mirror's derived budget pinned, parse-before-run after run 10's guard died under an outside in-place rewrite. Owed: the kill rule (exit 94) once the markers' `tree_mb_at_peak_swapout` say what rule is right. |
| — | `OPS-MEMSAFE.5` | `done` (recurring — re-run when `docs/ARTIFACT_CLEANUP.md` is older than a day) | The §8 artifact cleanup of 2026-09-29: 54.0 GB freed under the decision record's safe inventory; `debug/incremental` kept as the active lane's hot cache. |
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

- `2026-10-20` (`.6`): the guard's mandate stays JOB-scoped for kills and widens to the HOST for refusal at launch and for evidence in flight — a guard kills only what it owns (the 2026-07-14 session kill is the precedent; on 2026-10-20 the tree stood at 35–76 % of its budget while a daemon was the cause, so a "kill when the tree is over 70 % under pressure" rule would have killed a healthy run). Host pressure is read from `vm_stat`'s swap-out counter (its rate), never from `memory_pressure -Q`'s free % (purgeable and cached pages count as free: 51 % at 22 of 23.5 GB of swap). Exit 94 reserved for a swap-out breach kill, unimplemented until the markers show the rule. The budget is DERIVED, `min(max measured peak × 1.2, half of physical RAM)`, pinned as a literal beside its derivation — never chosen at launch, never derived at runtime from an untracked marker directory. Host configuration (`no_log`, Spotlight) is outside the guard's remit.

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
| `2026-09-29` | `OPS-MEMSAFE.5` | `git ls-files rust/target` (0 tracked) · population census by directory before deleting · `du -sk rust/target` before/after · KEEP paths excluded by `-not -path` and re-listed after | **PASS**: 265.9 → 211.9 GB (54.0 GB freed); 91 282 stale `*.log`/`*.bin` + 4 gate caches + `release/incremental` removed; 2 681 recent logs kept; `debug/incremental` (50.9 GB) kept by decision; debug/release probe binaries intact; disk 21 % |
| `2026-09-30` | `OPS-MEMSAFE.5` (recurring) | `git ls-files rust/target` (0 tracked) · run-dir census (workflow logs present / absent) before deleting · `du -sk rust/target` before/after · every cited `logs/` dir re-listed after | **PASS**: 283 → 223 GB (59 GB freed); 108 log-less CI-mirror run dirs + the `export/` trees of the 3 log-bearing runs + the `.117` cold export removed; 3 `logs/` dirs (11 files) kept; the `-0189` generator kept as the `.98` correction probe's BEFORE arm (`rust/target/eus98_corr/ast_pipeline.before`); disk 2.9 TB free of 3.7 TB (`docs/ARTIFACT_CLEANUP.md` overwritten, `PGEN-OPS-MEMSAFE-0011`) |
| `2026-10-02` | `OPS-MEMSAFE.5` (recurring) | `git ls-files rust/target` (0 tracked) · population census (stale logs by directory, the export tree's size, deps older than a day) before deleting · `du -sh rust/target` before/after · the three preserved generator binaries and the mirror run's `logs/` re-listed after · the debug and release probes run after | **PASS**: 272 → 184 GB (88 GB freed); the `run.ehxwr3` export tree (27 GB, 13 logs kept), 511 stale `*.log`/`*.bin` > 7 d, `cargo sweep --time 1` (`debug/deps/` 151 → 95 GB); `debug/incremental/` (47 GB), the gate-local caches and every BEFORE-arm binary kept; disk 23 % used |
| `2026-10-03` | `OPS-MEMSAFE.5` (recurring) | `git ls-files rust/target` (0 tracked) · stale-file census by directory before deleting (KEEP paths excluded by `-not -path`) · `du -sm rust/target` before/after · the debug and release probes listed after · `check_no_background_jobs.sh` | **PASS**: 206.1 → 203.3 GB (2.7 GB freed); 366 stale `*.log`/`*.bin` > 7 d (177 MB), `cargo sweep --time 1` (its line `5.86 GiB`, over-counted as before); no export tree this time; `debug/incremental/` (48 GB), the gate-local caches, the preserved BEFORE-arm generators and the last two days' release evidence kept; disk 26 % used (`docs/ARTIFACT_CLEANUP.md` overwritten) |
| `2026-10-05` | `OPS-MEMSAFE.5` (recurring) | `git ls-files rust/target` (0 tracked) · stale-file census before deleting (KEEP paths excluded by `-not -path`; the age distribution of the tree's logs read to confirm the count) · `du -sm rust/target` before/after · the debug and release probes listed and the release probe run after · `check_no_background_jobs.sh` | **PASS**: 204.3 → 166.1 GB (38.3 GB freed), all by `cargo sweep --time 1` (its line `84.75 GiB`, over-counted as before; `debug/deps/` 102.7 → 67.2 GB); 0 stale `*.log`/`*.bin` > 7 d (oldest log 2026-09-28); `debug/incremental/` (51.4 GB), the gate-local caches, the preserved BEFORE-arm generators, `ci_parity_gate_rot_probe/export` (read by tracked probes) and the last week's evidence kept; run after the day's builds; disk 27 % used (`docs/ARTIFACT_CLEANUP.md` overwritten) |
| `2026-10-06` | `OPS-MEMSAFE.5` (recurring) | `git ls-files rust/target` (0 tracked) · stale-file census before deleting (12 `*.log` older than seven days, all `rust/target/s385/` chain logs, 41 MB; the tracked-referenced `prereg.md` kept) · `cargo sweep --time 1` from `rust/` under the guard (`Cleaned 316.02 MiB`) · `du -sm rust/target` 175,032 → 174,672 MB · probes present, release probe run (`a5b5b9dc5c8ee247`) · `check_no_background_jobs.sh` rc 0 | **0.36 GB freed**; no build since 02:08 so the hot cache was not in tension with the sweep; disk 27 % used; `docs/ARTIFACT_CLEANUP.md` overwritten |
| `2026-10-08` | `OPS-MEMSAFE.5` (recurring) | `git ls-files rust/target` (0 tracked) · stale-file census before deleting (610 `*.log`/`*.bin` older than seven days outside the KEEP paths, by directory; the 8 cited by a live tracked document found by `git grep` and kept) · every path checked under `rust/target/` before the `rm` · `cargo sweep --time 1` from `rust/` under the guard (`Cleaned 1.78 GiB`) · `du -sm rust/target` 180,430 → 177,451 MB · probes present, release probe run (`--parser-fingerprint`, systemverilog `1781b11d85172ede…`) · `check_no_background_jobs.sh` rc 0 | **2.98 GB freed**; the last build (2026-10-07 20:18) four hours before the sweep, nothing pending; disk 29 % used; `docs/ARTIFACT_CLEANUP.md` overwritten |
| `2026-10-09` | `OPS-MEMSAFE.5` (recurring) | `git ls-files rust/target` (0 tracked) · stale-file census before deleting (463 `*.log`/`*.bin` older than seven days outside the KEEP paths, by directory; the 14 cited by a live tracked document found by `git grep -F` over every stale path and kept — six more than last time, logs that aged past seven days) · every path checked under `rust/target/` and not a symlink before the `rm` · `cargo sweep --time 1` from `rust/` under the guard (`Cleaned 3.26 GiB`, 20 s) · `du -sm rust/target` 194,358 → 189,831 MB · probes present, release probe run (`--parser-fingerprint`, systemverilog `2c074083736e6cb1…`) · the three preserved BEFORE-arm binaries re-listed · `check_no_background_jobs.sh` rc 0 | **4.53 GB freed** (449 files / 1,535 MB + the sweep); the last build (2026-10-09 17:53, the `.48` ceremony's debug probe) inside the sweep's one-day window, nothing pending; `debug/incremental/` 51.6 GB kept; disk 30 % used; `docs/ARTIFACT_CLEANUP.md` overwritten |
| `2026-10-16` | `OPS-MEMSAFE.5` (recurring) | `git ls-files rust/target` (0 tracked) · stale-file census before deleting (13,975 `*.log`/`*.bin` older than seven days outside the KEEP paths; the 12,657 under `sota_exit_gate/` KEPT whole — the flagship state the telemetry gates read; of the other 1,318 the 15 cited by a live tracked document found by `git grep -o -F -f` and kept) · every path checked under `rust/target/` and not a symlink before the `rm` · `cargo sweep --time 7` from `rust/` under the guard (`Cleaned nothing`, 25 s — the week window chosen because the last build, the `1.0.253` ceremony's of 2026-10-10, is six days old and a one-day window would have evicted the release's warm cache) · `du -sm rust/target` 198,792 → 196,142 MB · probes present, release probe run (`--parser-fingerprint`, systemverilog `aa7972eda8377806`) · `sv_ceremony_binaries.sh verify rust/target/sv50` OK (one note: the debug `ast_pipeline` the doctrine runs re-uplift, named by no tracked row → `SV-CORPUS-GRAD.56`) · the three preserved BEFORE-arm binaries re-listed · `check_no_background_jobs.sh` rc 0 | **2.65 GB freed** (1,303 files / 2,714 MB; the sweep nothing); the session had paused six days with the ceremony measured; disk 33 % used; `docs/ARTIFACT_CLEANUP.md` overwritten; riding: the register's `1.0.253` row back-filled `PENDING` → `9c0b06a9`, the grammar-certification attestation re-stamped (`PGEN-OPS-MEMSAFE-0018`) |
| `2026-10-18` | `OPS-MEMSAFE.5` (recurring) | `git ls-files rust/target` (0 tracked) · stale-file census before deleting (159 `*.log`/`*.bin` older than seven days outside the KEEP paths; `sota_exit_gate/` KEPT whole; the 19 cited by a live tracked document found by `git grep -h -o -F -f` and kept) · every path checked a plain file under `rust/target/` before the `rm`, 0 of 140 present after · `cargo sweep --time 1` under the guard · `du -sm rust/target` 197,944 → 194,520 · the release probe RUN (`systemverilog` `aa7972eda8377806`, `systemverilog_preprocessor` `e066baa67699919c`) · `sv_ceremony_binaries.sh verify rust/target/svpp8` notes=1 → `restore` → notes=0 · `check_no_background_jobs.sh` rc 0 | ✅ `PGEN-OPS-MEMSAFE-0019` — 3.4 GB, record overwritten |
| `2026-10-19` | `OPS-MEMSAFE.5` (recurring) | `git ls-files rust/target` (0 tracked) · the LIVE push-mirror run `run.AzKXa6` excluded from every step by path · stale-file census before deleting (5,103 `*.log`/`*.bin` older than seven days outside the KEEP paths; `sota_exit_gate/` KEPT whole; the 21 cited by a live tracked document found by `git grep -h -o -F -f` and kept) · every path checked a plain file under `rust/target/` before the `rm`, 5,082 removed (58 MB), 0 present after · the two FINISHED mirror exports `run.okkavv/export` (28,940 MB) and `run.kRR6e8/export` (21,779 MB) removed, `logs/` kept; `run.HOoExd` kept whole (a tracked doc cites a log inside its export) · `cargo sweep --time 1` under the guard (`Cleaned 166.09 MiB`) · `du -sm rust/target` 327,601 → 260,705 · the release probe RUN (`systemverilog` `aa7972eda8377806`, `systemverilog_preprocessor` `0aca5ffe7813dd1c`) · `sv_ceremony_binaries.sh verify rust/target/svpp9` notes=0, no restore · disk 37 % used | ✅ `PGEN-OPS-MEMSAFE-0020` — 66.9 GB, record overwritten |
| `2026-10-20` | `OPS-MEMSAFE.5` (recurring) | `git ls-files rust/target` (0 tracked) · no mirror run live (run 9 finished 06:29; run 10 started after this commit) · stale-file census before deleting (20 `*.log`/`*.bin` older than seven days outside the KEEP paths, all 20 cited by a live tracked document and kept, 0 removed) · the four FINISHED mirror runs' export build directories removed (`export/rust/target/{debug,release}` of `run.5tEW2W` / `run.AzKXa6` / `run.D9jLgP` / `run.HOoExd`, 119,480 MB; no tracked citation names them, every cited path kept, `summary.csv` and `logs/` intact) · the `-0126` CI-shaped worktree `rust/target/ci_shape` removed · `cargo sweep --time 1` under the guard (its own line `Cleaned 117.69 GiB` by file size; `du` attributes ≈30 GB — the one-day window now past the 2026-10-18 ceremony's builds, the next local debug build is cold) · `du -sm rust/target` 349,036 → 199,407 · `sv_ceremony_binaries.sh verify rust/target/svpp9` OK notes=0, no restore (release `cce7d537…`, debug `d2cdfaaf…` in place) · disk 36 % used, 2.45 TB free | ✅ `PGEN-OPS-MEMSAFE-0021` — 149.6 GB, record overwritten |
| `2026-10-20` | `OPS-MEMSAFE.6` | battery T12–T21 + the T20c control + replays T1 / T2 / T5 / T9 (`rust/target/svpp9/guard6/battery.sh`; `battery_run2.log` 12 / 12 before T20 existed, `battery_run4.log` 15 / 15 on the scratch copy, `battery_inplace2.log` 15 / 15 in place after the rename install) · the first live launch of the installed guard: the reproducibility rebaseline under `--budget-mb 12288` (`rust/target/svpp9/guard6/rebaseline.log`, marker `guard.85535.marker`: `peak_rss_mb=7622 elapsed_s=201 host_pressure_axis=on last_swapout_ps=0 pressure_samples=0 fseventsd_rss_mb=10`) · `bash -n` · `--help` prints the whole header · the host re-measured read-only (`fseventsd` 6 MB after its 11:02 restart, swap 12,100 of 13,312 MB cold, swap-out 0 pages/s over 5 s, 71 % free, `no_log` present, Spotlight off) · the 22 mirror markers' peaks reproduced · `launchctl list` for the supervisors | **15 / 15 PASS**: T12 exit 96 `reason=host-hazard`, never launched; T14 exit 96 `reason=host-pressure` at 1,666 pages/s naming `rustc:3462` (a rustc of the live mirror run, outside the test's tree); T15 the job SURVIVED six pressure samples at 2,000 pages/s with ONE warning, `tree_mb_at_peak_swapout=1`; T16 12,289 warns once, 12,288 silent, exit 7 through both; T17 NOT EVALUATED said, the job ran; T2 / T5 / T9 replays byte-for-byte verdicts; T20 the guard's own file rewritten in place (same inode) 1 s into a 4 s job ⇒ exit 7 through, marker written, no syntax error; T20c the HEAD guard under the same rewrite ⇒ rc 2, syntax error, no marker (run 10 reproduced in 5 s); T21 a static counter ⇒ `peak_swapout_ps=0`, a measured zero, not a dash (the first live marker had printed `-` for a 0-pages/s run; fixed, re-installed by rename, the battery re-run) |
| `2026-10-20` | `OPS-MEMSAFE.6` (first full-run marker, `.6`(e)) | mirror run 11 launched through `ci_workflow_local_gate_guarded` on `2abfaf026` after a foreign-job census (`pgrep -x cargo`, `pgrep -x rustc`, the mirror and guard script paths: none live) · `rust/target/svpp9/ci_gate11.log` read to its end · `guard.11929.marker` read field by field · `git log --oneline origin/main..HEAD` before and after the push | **PASS**: 12 replayed jobs `ok`, run dir removed, `CI-GATE rc=0`; guard `completed exit=0 peak_tree_rss=9999MB elapsed=23031s peak_swapout=14264/s pressure_samples=6`, marker `tree_mb_at_peak_swapout=2870 unowned_top=- fseventsd_rss_mb=7` (burst-shaped pressure: no two consecutive samples, no warning, no attribution — `.6`(f) owed); pushed `a0bec1ab6..2abfaf026` at 21:16:48 |

## Commit Log

| Leaf | Commit subject or reference | Notes |
| --- | --- | --- |
| `OPS-MEMSAFE.1` | `PGEN-OPS-MEMSAFE-0002 (leaf OPS-MEMSAFE.1)` | wrapper + fratricide root-cause fix + T1–T9 battery green + docs lockstep |
| `OPS-MEMSAFE.2` | `PGEN-OPS-MEMSAFE-0003 (leaf OPS-MEMSAFE.2)` | disk-floor axis (pre-flight + in-flight exit 95) + T10/T11 + regression green + disk-hygiene decision record + docs lockstep |
| `OPS-MEMSAFE.3` | `PGEN-OPS-MEMSAFE-0004 (leaf OPS-MEMSAFE.3)` | destructive-target guard (refuse-by-default `clean` family, exit 96) + doctrine check registered + `preserved_probes/` relocation + decision record + docs lockstep |
| `OPS-MEMSAFE.5` | `PGEN-OPS-MEMSAFE-0009 (leaf OPS-MEMSAFE.5)` | the §8 recurring artifact cleanup of 2026-09-29 + the new `docs/ARTIFACT_CLEANUP.md` record (latest run only) |
| `OPS-MEMSAFE.6` | `PGEN-OPS-MEMSAFE-0022 (leaf OPS-MEMSAFE.6)` | host-hazard + host-pressure pre-flight (exit 96, `reason=host-hazard` / `host-pressure`), in-flight pressure observation recorded in eight marker fields (never a kill; exit 94 reserved), the over-half-RAM budget warning, `ci_workflow_local_gate_guarded` pinned at the derived 12,288, parse-before-run (the whole body one function) after run 10's guard died under an outside in-place rewrite, battery T12–T21 + T20c + replays 15 / 15, the decision card amended, book + Makefile lockstep |

## Changelog

- `2026-10-20` (21:16): the first full `_guarded` mirror run (run 11, GREEN, pushed) wrote the first marker with the eight `.6` fields: peak 9,999 MB, six burst-shaped pressure samples peaking at 14,264 pages/s with the tree at 23 % of its budget, no attribution captured (the episode rule needs two consecutive samples) — `.6`(e) supplied, `.6`(f) owed.
- `2026-10-20`: Leaf `.6` opened and closed in one slice on the host incident of the same day (a root-owned `fseventsd` at 9–10 GB, swap 22 of 23.5 GB, the guard reading 51 % free through it): the guard refuses to launch into a host hazard by name, observes host pressure in flight and records it (never a kill — a guard kills only what it owns), warns on a budget over half of RAM; the push mirror's budget derived from its own markers (12,288) and pinned in `ci_workflow_local_gate_guarded`. The external report's figures re-measured, two corrected. `.4` keeps spike visibility and peak reduction. Found while the slice was open: run 10's guard died at its loop's end on its own script, rewritten in place under it from outside the session; the guard now parses its whole body before launching anything (FIX (7)).
- `2026-09-29`: Leaf `.5` opened and closed in one slice — the session directive's §8 cleanup record did not exist, so a cleanup was due; 54.0 GB freed from `rust/target/` under the decision record's inventory, recorded in `docs/ARTIFACT_CLEANUP.md`.
- `2026-07-14`: Created task tree; leaf `.1` opened as the session's directive-mandated first item.
- `2026-07-14`: First build's own T2 test killed the host's user session (awk auto-vivification made the tree walk system-wide; the kill list followed). Root-caused with the surviving guard logs + crash reports + a live repro; fixed (`!(p in mark)`) + three kill-path fail-safes; battery extended with T8 (scoping regression) + T9 (guard-interrupted); 23/23 PASS. Leaf `.1` DONE — heavy jobs now run UNDER the guard.
- `2026-07-18`: Tree REOPENED for leaf `.2` after the RGX-0078 `.5.i.15` probe build died on host-disk exhaustion (100% disk, 1.1 GB free — the guard had no disk axis). Disk floor delivered mirroring the RAM floor's design (pre-flight refusal exit 96 `reason=disk-floor`; in-flight breach exit 95 via the existing kill path; `--disk-floor-gb` default 8, 0 disables; `df -Pk .` + deterministic seam); T10/T11 + full regression PASS; standing disk-hygiene directive recorded (`docs/decisions/feedback_disk_hygiene_proactive.md`). Leaf `.2` DONE — tree `complete` again.
- `2026-07-19`: Tree REOPENED for leaf `.3` after the destructive-clean incident (director directive, emphatic): `make annotation_parsers` → alias → `clean` destroyed 102.8 GiB incl. every preserved perf probe. Delivered refuse-by-default guard on the `clean` family (exit 96, `PGEN_CONFIRM_CLEAN=1` opt-in), the registered doctrine check `check_destructive_target_guard.sh` (first run caught the latent `bootstrap-test` member), and the `preserved_probes/` relocation outside the `cargo clean` blast radius. Leaf `.3` DONE — tree `complete` again.

## Leaf `.4` — the guard cannot SEE a spike, and on this host the RSS budget is not the binding constraint (`todo`)

- **Status: `todo`** — REOPENS the tree (2026-07-27, session #214). Opened by
  `GENERATED-LINT-CORRECTNESS.2`, which hit the failure twice and **first drew the wrong
  conclusion from it**; the correction is what created this leaf.
- ⛔ **The wrong conclusion, recorded so it is not repeated:** *"the banked
  `--budget-mb 16384` is no longer enough — use 18432."* It is refuted by the guard's own
  markers, which were already on disk when the claim was written:

| marker | command | budget_mb | peak_rss_mb | reason | exit | last_free_pct |
|---|---|---|---|---|---|---|
| `guard.24252` | `make -C rust focus_*` | 16384 | 15,079 | `none` | 2 | 83 |
| `guard.54226` | `make -C rust parse_harness_combinator_gate` | **18432** | 15,253 | `none` | 2 | 82 |

  **The second kill happened with the proposed fix already in effect.** Neither was a guard
  kill: `reason=none` in both, both peaks BELOW their budget, and system-free 82–83%, so
  the RSS budget AND the `--floor-pct` system-free floor both saw nothing. The child rustc
  received `signal: 15` from outside the guard.

- **Measured host state (2026-07-27):** `hw.memsize` 24.0 GB; `vm.swapusage` total
  4096.00M / **used 2509.75M** / free 1586.25M. ⇒ the binding constraint is RAM plus
  nearly-exhausted swap headroom, not the guard's RSS budget. ⛔ **Raising the budget makes
  the guard LESS protective** — it licenses 18.4 GB on a machine the OS starts killing near
  15, i.e. it guarantees the OS wins the race.

- ⚠️ **This was already known.** `DEVELOPMENT_NOTES.md` records the same mechanism from the
  `.3.x` era: *"the guard reported `reason=none`, `last_free_pct=84`, because it samples
  periodically and cannot see a spike between samples. Contributing factor: swap is capped
  at 3,072 MB with ~1,989 MB already in use."* (Swap has since grown to 4,096 MB and is
  still ~61% consumed.) Re-discovering it and attaching a wrong remedy is a RE-MEASURE
  failure of `docs/decisions/feedback_read_prior_art_before_designing.md`.

- 📌 **ROUTED IN 2026-08-01 (`SV-EXH-PROOF.7.4.6.10`) — three fresh markers, and they support
  question 3 (*reduce the peak*) over any budget change.** Recorded here rather than worked; the
  SV leaf needed a probe to run, not a guard fix.

| marker | command | budget_mb | peak_rss_mb | util | reason | exit |
|---|---|---|---|---|---|---|
| `guard.45571` | ratchet probe ×6, **default cargo jobs** | 12288 | 12,415 | 101% | `rss-budget` | 97 |
| `guard.36810` | ratchet probe ×6, **`CARGO_BUILD_JOBS=2`** | 16384 | 11,921 | 73% | `completed` | 0 |
| `guard.7211` | canonical `make sv_stimuli_quality_gate` | 12288 | **11,330** | **92%** | `completed` | 0 |

  - **The breaching stage is named, and it is the BUILD**: the killed case log ends at
    `build_ast_pipeline_for_sv_generation`, not at generation or the closed loop. Same conclusion
    as this leaf's own §3 (*"the drivers are known: the `--test` lib link and the
    `target/debug/ast_pipeline` rustc invocation"*) — re-measured on a different workload.
  - ⭐ **Capping cargo parallelism moved the peak 12,415 → 11,921 MB on the same work.** The gate
    already exposes the lever (`PGEN_SV_STIMULI_CARGO_BUILD_JOBS`,
    `sv_stimuli_quality_gate.sh:25`/`:804`); it just was not being used. That is a *cheap,
    existing* peak-reduction knob for §3 to price against the harder options.
  - ⚠️ **The budget raise to 16384 in that probe was BELT-AND-BRACES, not the remedy**, and this
    leaf's own warning applies: raising the budget makes the guard less protective. The measurement
    says so too — with `jobs=2` the run peaked at 11,921 MB, so 12288 would also have held, at only
    3% headroom. The peak-reduction lever is what did the work.
  - ⚠️ **The documented 12288 HOLDS for the canonical SV gate — twice — but the margin is thin.**
    10,401 MB (`SV-EXH-PROOF.7.4.6.11`) and now 11,330 MB, i.e. **92% of budget**, for ONE stage of
    the ~32-stage `sota_exit_gate` that `README.md` prescribes 12288 for. No defect to route: the
    documented budget did not breach on either sample. But 8% headroom on the SV stage is a number
    §2 (*the budget doctrine on this host*) should decide against deliberately rather than inherit.

### The real questions this leaf owns

1. **Make the guard SEE spikes, or say it cannot.** `--interval-s` defaults to 5 s and a
   fat-LTO / large-crate link can add several GB inside one interval. Options to price:
   a much shorter interval for the heavy classes; sampling *system* pressure rather than
   only tree RSS; or — the honest minimum — emitting a distinguishable marker when the
   child dies of an external signal, so `reason=none exit=2` stops reading like a normal
   child failure. ⭐ This is the same principle the `GENERATED-LINT-CORRECTNESS` tree is
   named for: **a check that cannot see must say so, not return green.**
2. **Decide the budget doctrine on THIS host.** A budget *below* the OS-kill point converts
   a random external SIGTERM into a deterministic, branchable guard kill (exit 97 with a
   `reason=rss-budget` marker) — worse throughput, far better diagnosability. A budget
   above it is decoration. Neither makes the job complete; that needs (3).
3. **Reduce the peak.** The drivers are known: the `--test` lib link and the
   `target/debug/ast_pipeline` rustc invocation, both dominated by the 130 MB SV parser.
   Candidates: `-C debuginfo=0` for regeneration-only builds, splitting the `focus_*`
   fan-out, or serializing so two heavy jobs never overlap.
4. ⚠️ **A mixed-vintage hazard, already paid for once.** The first kill died partway
   through a `focus_*` fan-out and left 2 of 11 generated parsers on the new codegen and 9
   on the old. Nothing detected that; it was caught only because the operator was watching
   timestamps. A partially-completed regeneration is silently unattributable evidence —
   worth an explicit all-or-nothing marker.

### Interim guidance (until this leaf lands)

- ⛔ Do **not** treat `--budget-mb 18432` as the remedy; it is measured not to be one.
- A `reason=none exit=2` marker on a heavy job means **an external kill**, not a guard
  breach — re-run it, and if it recurs, reduce the peak rather than raise the budget.
- After any interrupted `focus_*` fan-out, **redo the whole regeneration**; never resume it.
- 📌 **2026-10-20 (`.6`, `PGEN-OPS-MEMSAFE-0022`):** question 1's "sample *system* pressure rather than only tree RSS" is now in the guard as an OBSERVATION axis (the swap-out rate, recorded in every marker with the tree's RSS at the pressure peak and the largest unowned process; never a kill), and question 2 is answered for the push mirror by a DERIVED budget pinned in one target (`min(max peak × 1.2, half of RAM)` = 12,288). Spike visibility between samples (1) and peak reduction (3) remain this leaf's.

## Leaf `.5` — the recurring §8 artifact cleanup and its record (opened + closed 2026-09-29, `PGEN-OPS-MEMSAFE-0009`, status: `done`, recurring)

- **Goal:** own the session directive's §8 cleanup as tracked work — a bounded sweep of `rust/target/` under the
  100 %-safe inventory of `docs/decisions/feedback_disk_hygiene_proactive.md`, and a record a fresh session can
  read to decide whether one is due (`docs/ARTIFACT_CLEANUP.md`, latest run only, overwrite-only).
- **Why:** the record file did not exist, so by the directive a cleanup was due this session; `rust/target/`
  measured **265.9 GB**, of which 42.1 GB were 91 282 `*.log` / `*.bin` files older than seven days (65 552 the
  per-file preprocessor logs of `svpp_all_population{,_k}/before/`, 12 596 under `sota_exit_gate/`, 8 314 under
  `ci_workflow_local_gate/`) and 15.2 GB the four gate-local cargo caches.
- **Done (measured):** `git ls-files rust/target` → 0 before anything was deleted; the population listed by
  directory and sampled; KEEP paths excluded by `-not -path` (none of the stale files sat under them — 0);
  deleted the 91 282 stale logs / bins, `parse_harness_{combinator,it,semantic}/`, `ebnf_frontend_build/`,
  `release/incremental/`; **265.9 → 211.9 GB, 54.0 GB freed**; 2 681 logs younger than seven days kept (this
  session's `s34/` evidence, the `.35`-cited `s388/`); the debug `ast_pipeline` / `parseability_probe` and the
  release probe intact afterwards; disk 21 % used (3.0 TB free).
- **Kept by decision, stated:** `rust/target/debug/incremental/` (50.9 GB) — the inventory allows deleting it, but it
  is the hot cache of the active SV lane (a cold rebuild of the crate with its generated parsers costs the next
  slice tens of minutes) and at 21 % disk use nothing is bought by paying that; the sequencing hazard the record
  names (sweep AFTER a pending warm-cache build) also argues for leaving it while builds are in flight daily.
- **Recurring:** re-run when `docs/ARTIFACT_CLEANUP.md` is older than a day; overwrite that file, append a row to
  this tree's Verification Log. Not a code change — no acceptance checklist is owed (nothing under the enforced
  code globs moved); the evidence is the before/after measurement above.

### `.5` correction (2026-09-30, `PGEN-OPS-MEMSAFE-0010`) — the record `.5` created was never admitted to the CI mirror's top-level docs allowlist, so the full CI mirror went RED on its second audit the first time the push lane ran it

- **HOW IT WAS FOUND.** The unpushed count reached the 300-commit push cadence at `b69eceb39`; directive 16 runs the full CI mirror before a push, and `make -C rust SHELL=/bin/bash ci_workflow_local_gate` (under the memory guard) refused at its second surface audit: `auditing top-level docs allowlist` → `error: top-level docs allowlist drift detected; prune stale docs or update the tracked policy deliberately`, the actual surface carrying `docs/ARTIFACT_CLEANUP.md` and the expected one not.
- **WHY + WHERE.** `rust/scripts/ci_workflow_local_gate.sh::audit_top_level_docs_surface` compares the tracked `docs/*.md` set (`git ls-files -z | perl … | sort`) VERBATIM against a hand-kept `expected_top_level_docs` roster; `.5` (`707d276a2`, 2026-09-29) created `docs/ARTIFACT_CLEANUP.md` — the record the session directive's §8 mandates by that name — without telling the roster, the same class `CI-PARITY-GATE-ROT.1` (four docs, 2026-07-27) and `.46` (two docs, 2026-08-25) admitted before. The audit was RIGHT: no deliberate policy update had happened.
- **FIX — the deliberate policy update.** `"docs/ARTIFACT_CLEANUP.md"` admitted as the roster's first entry (the audit's `sort` places it before `docs/AST_GENERATOR_ARCHITECTURE.md`), with the admission's evidence in the comment: one commit in its history, referenced by the directive that mandates it — not a stray.
- **VERIFIED.** The audit's own comparison replayed on the edited script (the array extracted verbatim, the actual surface from the same `git ls-files … | sort`): `TOP-LEVEL-DOCS-ALLOWLIST: OK (14 docs, expected == actual, verbatim order)` — was `MISMATCH` on `docs/ARTIFACT_CLEANUP.md`; `bash -n` clean. The whole mirror is re-run under the guard for the push (its result belongs to the push lane, not to this correction).

#### Acceptance Checklist (enforced) — `.5` correction (`PGEN-OPS-MEMSAFE-0010`)
- [x] **REPRODUCE / ISSUE** — `make -C rust SHELL=/bin/bash ci_workflow_local_gate` on `b69eceb39` → `auditing top-level docs allowlist` … `error: top-level docs allowlist drift detected; prune stale docs or update the tracked policy deliberately` (run dir `rust/target/ci_workflow_local_gate/run.zk13Tg`, retained as failed); the two printed surfaces differ in exactly `docs/ARTIFACT_CLEANUP.md`.
- [x] **ROOT CAUSE (WHY + WHERE)** — WHERE: `rust/scripts/ci_workflow_local_gate.sh`, `audit_top_level_docs_surface`'s `expected_top_level_docs` roster. WHY: `git ls-files docs/ARTIFACT_CLEANUP.md` → tracked since `707d276a2` (`git log -- docs/ARTIFACT_CLEANUP.md` → one commit, 2026-09-29, `.5`), and the roster — a hand-kept allowlist the audit compares verbatim — was not updated in that commit; the audit's expected/actual dump is the instrument that named the one file.
- [x] **FIX** — ops / build-flow tier: one roster entry with its admission comment; no lower tier (the record is the director's, by name).
- [x] **ADDRESSED (verified)** — the audit's comparison replayed on the edited script: `TOP-LEVEL-DOCS-ALLOWLIST: OK (14 docs, expected == actual, verbatim order)` (was `MISMATCH`); `bash -n rust/scripts/ci_workflow_local_gate.sh` clean.
- [x] **NO REGRESSION** — the roster's other thirteen entries untouched (the replay's expected list equals the tracked `docs/*.md` set entry for entry); ZERO Rust / grammar / generated bytes — `GENERATED-REPRODUCIBILITY` tier 1 reads every generated artifact byte-identical with HEAD's source, and no clippy run is owed; `bash scripts/check_doctrines.sh` on the staged set → `doctrines: ALL 45 enforced doctrines PASS.`
- [x] **LOCKSTEP** — this leaf, `docs/TASK_TREE.md`, `CHANGES.md`, `DEVELOPMENT_NOTES.md`; `MEMORY.md`'s push-lane note. No book page enumerates the roster.
- [x] **NEUTRALITY TIER** — N/A: a CI audit roster, no language named.

## Leaf `.6` — the guard was blind to a host hazard it did not own (the 2026-10-20 `fseventsd` incident): host-hazard pre-flight, host-pressure observation, the mirror's derived budget (opened + closed 2026-10-20, `PGEN-OPS-MEMSAFE-0022`, status: `done`)

- **Status: `done`** — opened on an incident report an external agent wrote for the director
  (`~/Desktop/PGEN-feedback-host-memory-2026-10-20.md`, outside every repository, forwarded
  2026-10-20 with *"read this whenever you have some spare time"*). It asked three questions —
  widen the guard's mandate?, which axes, and do they kill?, pin the budget? — and shipped a
  patch for the first. Nothing in it was taken on trust: every figure below was re-measured here
  before anything was built, and two of its claims were corrected (*Verified*, below).
- **Found.** The host was unusable for hours on 2026-10-20 while a root-owned `fseventsd`
  (PID 377, PPID 1, started two minutes after boot, never inside any job tree) grew to 9–10 GB
  RSS at 99 % CPU on an 11-day FSEvents journal of the build volume (116,998 files, 4.0 GB,
  under Spotlight indexing with a 48 GB index that covered `rust/target`); swap reached 22.06 of
  23.55 GB with 103 MB of physical RAM unused. Two guarded mirror runs were live in that window
  and neither tripped an axis: the guard read `last_free_pct=51` throughout. The operator
  remediated the host outside the repository (`no_log` on the volume, Spotlight off and the
  index erased, the journal pruned, the daemon restarted at 4 MB); an alert-only menu-bar app
  now watches the host. ⛔ PGEN did not cause the leak and must not mutate host configuration
  from a gate; PGEN's push mirror is the largest churn source on that volume (103,639 files
  exported per run, 22,877 files touched under `rust/target` in two hours).
- **Verified here (2026-10-20 12:43–12:57 CEST, read-only).** `fseventsd` PID 96988 restarted
  11:02:09, 6 MB, 0 % CPU · `vm.swapusage` 13,312 MB total / 12,100 used (cold) · swap-out
  **0 pages/s** over 5 s · `memory_pressure -Q` 71 % free · `/Volumes/SSD/.fseventsd` holds two
  entries (`fseventsd-uuid`, `no_log`), 4 KB · `mdutil -s` "Indexing disabled", the index
  directory 17 GB · 5,254 guard markers on disk · the 22 markers naming `ci_workflow_local_gate`
  reproduce the report's peak table (below) · the one `killed` marker (10,419 MB, 2026-10-19
  13:43) was `guard-interrupted`, exit 130, not a budget breach. ⚠️ Two corrections: (1) *"14336
  appears nowhere in the repo"* is false by one citation
  (`docs/tasks/GENERATED-LINT-CORRECTNESS.md:641`) and true for every launch site — the figure
  was chosen in-session; (2) the book's "other supervisor" (`~/.memwatch/memwatch.sh`, kills any
  process over 14,336 MB) is **no longer loaded**: `launchctl list` carries only
  `local.tools.memwatch` (the alert-only app), and the shell watchdog's log was last written
  2026-10-09 09:51 — the book paragraph is re-dated in this slice. ⭐ One finding the report
  could not have: `~/Library/Logs/MemWatch.log` records swap thrashing at **1,141 and 1,327
  pages/s at 12:30–12:32 CEST today**, after the remediation, with mirror run 10 in its SV
  stages and nothing else heavy — the mirror itself can press the host, which is exactly why a
  kill rule must wait for data (FIX (3)).

| budget_mb | full mirror runs' `peak_rss_mb` (markers under 2 GB excluded: audits-only and short runs) | status |
| --- | --- | --- |
| 12288 | 9,564 · 9,639 | completed |
| 14336 | 5,967 · 8,135 · 8,878 · 9,620 · 9,873 · 10,014 · 10,419 | six completed; 10,419 `guard-interrupted` (exit 130, 2026-10-19 13:43), not a breach |
| 16384 | 5,010 · 5,090 · 10,943 | completed |

- ⭐ **Found while the slice was open (run 10's end, 14:13:03 CEST).** Mirror run 10 (`run.R9hchX`,
  started 07:25:07 on `a0bec1ab6` under the tracked guard at 14,336) finished GREEN — 12 replayed
  jobs `ok`, the run dir removed on success, the Makefile's final `✅` echo executed, so the script
  and `make` exited 0 — and then the guard itself printed `scripts/run_with_memory_guard.sh: line
  347: syntax error near unexpected token `('` and exited 2: no `completed` marker, run 10's peak
  unrecorded. Line 347 of the tracked file is `wait "$CHILD"`; the line bash quoted is the
  RSS-budget breach line of a different image of the file. Evidence, read-only: the path was
  RECREATED at 12:33:03 (birth time = mtime, a new inode; `.git/index` rewritten 12:33:11; no other
  tracked file reborn that minute; content equal to HEAD), so the inode the guard held open had
  been rewritten IN PLACE before that and the rewrite discarded through git. The three Claude
  Code transcripts on this host for the day (this session, `specforge`, `archogen`) carry no tool
  call writing that path, and this session made no tool call at all between 12:20 and 12:40 —
  the hand was outside the session, inside the operator's host-remediation window. bash reads a
  script incrementally, by file offset, as it executes: the monitor loop was parsed at launch and
  ran to its end; the first read after it, 6 h 48 min later, fetched the rewritten bytes. The
  mirror's verdict is unaffected (the child had exited before the guard read anything);
  `a0bec1ab6` was pushed at 14:19 on that verdict (`CI-PARITY-GATE-ROT.121`).
- [x] **REPRODUCE / ISSUE** — the battery's T14 / T15 (`rust/target/svpp9/guard6/battery.sh`):
  a swap-out counter advancing 2,000 pages/s with the free-% seam untouched. Before this slice
  the guard had no reader for any of it (`grep -c vm_stat scripts/run_with_memory_guard.sh` = 0;
  no `pgrep`, no `hw.memsize`), so the same conditions produced `started … free=…%` and a green
  `completed` marker — run 9's own marker (`guard.70109.marker`: `peak_rss_mb=5967
  last_free_pct=74 reason=none`, 27,557 s inside the incident window) is the evidence.
- [x] **ROOT CAUSE (WHY + WHERE)** — WHY: the guard's three kill axes are scoped to the job's
  process tree by design (a guard kills only what it owns), and its one host-wide reading is not
  a pressure reading: `memory_pressure -Q`'s free % counts purgeable and cached pages as free, so
  it reads 51 % while the machine swaps itself to a halt. A root-owned daemon outside every tree
  was therefore invisible to all four readings. A second cause, independent of the first: the
  budget was chosen at invocation — `8192`, `12288`, `14336`, `16384` all occur in the markers
  for the same job — not derived from what the runs measure. WHERE:
  `scripts/run_with_memory_guard.sh` pre-flight (pre-fix lines 237–259: `free_pct` and
  `free_disk_gb` only) and monitor loop (pre-fix lines 295–345: RSS, free %, disk, timeout — no
  swap, no unowned process); the mirror's launch line composed by hand in each session. HOW READ:
  run 9's `guard.70109.marker` (`reason=none`, `peak_rss_mb=5967`, `last_free_pct=74`, written
  inside the incident window) against `vm.swapusage` and `vm_stat`'s swap-out counter; the 22
  `ci_workflow_local_gate` markers for the budget table; for the run-10 death, `bash -n
  scripts/run_with_memory_guard.sh` clean on both images, `git ls-files -z | xargs stat -f '%SB'`
  for the one path reborn at 12:33:03, and `git reflog` / `git stash list` empty of any
  checkout of this session's.
- [x] **FIX** — tier: ops script + Makefile + docs; no engine / parser / grammar byte.
  (1) **Host-hazard pre-flight** (`--fseventsd-limit-mb`, default 512, 0 off): the daemon's RSS
  over the limit ⇒ exit 96, `reason=host-hazard`, the cause named, the job never launched (the
  report's P1, re-expressed in the guard's own option / seam / marker conventions).
  (2) **Host-pressure pre-flight** (`--swapout-floor`, default 1,000 pages/s, 0 off): `vm_stat`'s
  swap-out counter sampled across a 3 s window; at or over the floor ⇒ exit 96,
  `reason=host-pressure`, the largest process on the host named.
  (3) **In-flight observation, never a kill**: every interval samples the swap-out rate; two
  consecutive samples at or over the floor print ONE warning per episode naming the largest
  process outside the job's tree, and the marker records `host_pressure_axis`,
  `swapout_floor_ps`, `last_swapout_ps`, `peak_swapout_ps`, `pressure_samples`,
  `tree_mb_at_peak_swapout`, `unowned_top`, `fseventsd_rss_mb`. Exit **94** is reserved for a
  swap-out breach kill and NOT implemented (the report's P2 kill branch): on 2026-10-20 the tree
  stood at 35–76 % of its budget while the cause was a daemon, so the proposed "kill when the
  tree is over 70 %" rule would have killed a healthy run; the marker now records what a right
  rule needs.
  (4) **The over-half-RAM warning**: `--budget-mb` above `hw.memsize / 2` prints one warning at
  launch (non-fatal; the four documented 16,384 call sites keep working and now say what they
  cost).
  (5) **The mirror's one sanctioned launch**: `make -C rust SHELL=/bin/bash
  ci_workflow_local_gate_guarded` at the DERIVED budget `min(10,943 × 1.2, 24,576 / 2)` =
  **12,288** and a 12 h ceiling; callers stop composing their own guard line.
  (6) Every new reading degrades to NOT EVALUATED where it cannot run (no daemon, no `vm_stat`,
  no `sysctl`: a Linux runner) and says so — never a verdict
  (`docs/decisions/feedback_a_check_that_cannot_run_must_say_so.md`); three deterministic seams
  (`PGEN_MEMORY_GUARD_FAKE_{SWAPOUTS,FSEVENTSD_RSS_MB,MEMSIZE_MB}_FILE`) make every path testable.
  (7) **Parse-before-run** (the finding above): the whole body is one `main()` called on the last
  line, every path out of it an `exit` (no top-level `return`, verified by grep); bash therefore
  parses the entire file before the job is launched and never reads it again. The rule *never
  edit a running script in place* stands — and was not enough, because the hand was not this
  session's; the script now protects itself.
  (8) A run whose every swap-out sample is zero records `peak_swapout_ps=0` — the installed
  guard's first live marker (the reproducibility rebaseline, 201 s) printed `-`, which reads as
  "not sampled" when the counter had in fact been read 40 times (T21).
- **Decisions (the report's §7, answered).** (1) The mandate stays job-scoped for KILLS and
  widens to the host for REFUSAL at launch and for EVIDENCE in flight. (2) Exit 94 reserved;
  marker fields as above. (3) P1 on by default; thresholds are options. (4) The budget is a
  literal in the Makefile with its derivation beside it — a runtime derivation from an untracked
  marker directory would make a launch depend on what a cleanup left behind. (5) The `no_log`
  marker and Spotlight are host configuration, outside the guard's remit: the guard observes the
  host and refuses to launch into a hazard; it does not require a host configuration.
  (6) Building on the external volume is the standing rule (project data lives on `/Volumes/SSD`),
  so the churn is a permanent property; the report's P6 (date-sharded marker directories, a
  hardlinked export) is parked below on its merits, not on this incident.
- [x] **ADDRESSED (verified)** — battery **15 / 15 PASS** on the scratch copy
  (`rust/target/svpp9/guard6/battery_run4.log`; `battery_run2.log` was 12 / 12 before T20 existed)
  and again in place after the rename install (`battery_inplace2.log`, 15 / 15): T12 fake
  daemon 900 MB ⇒ exit 96 `reason=host-hazard`, never launched, `fseventsd_rss_mb=900`; T13
  `--fseventsd-limit-mb 0` launches and still records the 900; T14 a counter at 2,000 pages/s ⇒
  exit 96 `reason=host-pressure` at 1,666 pages/s, never launched, `unowned_top=rustc:3462` (a
  rustc of the live mirror run — outside the test's tree, named correctly); T15 static during
  pre-flight then 2,000 pages/s for 12 s: the job SURVIVED (exit 0), exactly one warning,
  `pressure_samples=6 peak_swapout_ps=2000 tree_mb_at_peak_swapout=1 unowned_top=rustc:2594`;
  T16 budget 12,289 on a 24,576 MB host warns once, 12,288 is silent, exit 7 passes through both;
  T17 an unreadable counter ⇒ `host_pressure_axis=not-evaluated`, said, the job ran; T18
  `--swapout-floor 0` ⇒ axis off, launch in 1 s; T19 a non-integer option ⇒ exit 2. Real host, no
  seams (T1-replay): exit 7 through, `fseventsd_rss_mb=3`, `host_pressure_axis=on`, no warning. T20: a
  copy of the guard rewritten in place (same inode, verified) 1 s into a 4 s job ⇒ exit 7 through,
  `status=completed exit=7`, no syntax error; T20c, the control: the HEAD guard under the same
  rewrite ⇒ rc 2, `syntax error near unexpected token`, no marker — run 10 reproduced in 5 s. T21: a
  static counter for 3 s ⇒ `peak_swapout_ps=0 last_swapout_ps=0 pressure_samples=0`. First live
  launch of the installed guard: the reproducibility rebaseline at the derived 12,288 —
  `completed exit=0 peak_tree_rss=7622MB elapsed=201s`, `host_pressure_axis=on`, swap-out 0 pages/s
  throughout, `fseventsd_rss_mb=10`; the eight new fields present in a real marker.
  `bash -n` clean; `--help` prints the whole header (the usage range now ends at the
  `set -uo pipefail` line, not at a line number).
- [x] **NO REGRESSION** — T2-replay (300 MB balloon under 100 MB ⇒ exit 97 `reason=rss-budget`,
  balloon gone), T5-replay (fake free 5 % ⇒ exit 96 `reason=free-floor`, first in order, never
  launched), T9-replay (TERM ⇒ exit 130 `reason=guard-interrupted` through the extended writer):
  byte-for-byte verdicts. The thirteen existing marker fields keep their names and order (the
  eight new ones follow `last_disk_free_gb`); every existing caller is unchanged (defaults on,
  3 s added to a launch while the axis is on, nothing when `--swapout-floor 0`); the RSS /
  free-floor / disk / timeout logic untouched. The bare `ci_workflow_local_gate` target is
  untouched; `_guarded` wraps it. The function wrap changes no behaviour: `set -m`, `trap`, `$!`,
  `wait` and the globals behave identically inside a function; T12–T19 and the four replays pass
  byte-for-byte after it, and `--help` still prints the whole header. The generated surface:
  the reproducibility re-derivation under the installed guard reports every generator stage
  `re-derives byte-identically (0 sites)` (`rust/target/svpp9/guard6/rebaseline.log`;
  `generated_reproducibility_v0.json` re-keyed to `a0bec1ab6` because `rust/Makefile` is inside
  `emission_sha`); zero engine, grammar or generated bytes in the slice.
- [x] **LOCKSTEP** — this tree (metadata, task tree, frontier, decisions, logs, changelog, the
  `.4` pointer) + `docs/TASK_TREE.md` + the decision card
  `docs/decisions/feedback_host_ram_budget_all_jobs.md` (consequences 5–7 + the peak table) +
  `docs/decisions/INDEX.md` + the book `operations-and-governance.md` (the guard section, the
  re-dated supervisor paragraph, the Workflow Parity launch) + `rust/Makefile` + `CHANGES.md` +
  `DEVELOPMENT_NOTES.md` + `MEMORY.md`, same commit; the reproducibility baseline re-recorded
  (`rust/Makefile` is inside `emission_sha`).
- **Owed / parked.** (a) The kill rule (exit 94): read `tree_mb_at_peak_swapout` and
  `unowned_top` across the next mirror runs' markers before writing one. (b) The daemon is read at
  pre-flight only; an in-flight re-read is cheap once the first markers show whether it matters.
  (c) P6 churn hygiene on its merits: 5,254 markers in one flat directory; a 103,639-file export
  per mirror run. (d) `.4` keeps questions 1 (spike visibility between samples) and 3 (peak
  reduction); its question 2 is answered for the mirror by FIX (5). (e) Run 10's peak is
  unrecorded (its guard wrote no marker); the first `_guarded` mirror run supplies the first
  marker with the eight new fields and the first swap-out reading of a full run.
- **First full-run marker (read 2026-10-20, 21:16 — `.6`(e) supplied, `.6`(a)'s first data point).**
  Mirror run 11 (`run.CAr8Ca`, 14:52:02–21:15:56, the first launch through
  `ci_workflow_local_gate_guarded`, `rust/target/svpp9/ci_gate11.log`) finished GREEN (12 replayed
  jobs, `CI-GATE rc=0`) and its guard wrote `rust/target/generated_logs/memory_guard/guard.11929.marker`:
  `status=completed exit=0 peak_rss_mb=9999 elapsed_s=23031 host_pressure_axis=on peak_swapout_ps=14264
  pressure_samples=6 tree_mb_at_peak_swapout=2870 unowned_top=- last_swapout_ps=0 last_free_pct=66
  fseventsd_rss_mb=7`. Read: the tree peaked at 9,999 MB (81 % of 12,288, under the 10,943 the budget
  was derived from, so the budget stands); the host swapped out at up to 14,264 pages/s (223 MB/s,
  fourteen times the floor) while the tree held 2,870 MB — 23 % of its budget — so the first full-run
  data point says what the incident's logs said: the pressure was not the job's, and a kill rule keyed
  on the tree's share of its budget would have had no case. Six samples at or over the floor and no two
  in a row: the pressure on this host is BURST-shaped (one 5 s sample at a time), the *two consecutive
  samples* episode rule printed no warning, and `unowned_top` is `-` — the attribution half of the
  evidence is missing for exactly the shape the run saw. **(f) Owed, parked behind the feature lanes
  (no verdict depends on it):** capture `unowned_top` at the peak sample (one `ps` per new peak) and
  stamp `peak_swapout_at=` in the marker so a burst can be placed against the mirror's job log; a leaf
  of its own with the battery (T15's expectation moves from the second sample to the first).

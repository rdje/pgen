# feedback — delete reclaimable artifacts on a regular basis

**Category:** `feedback` (standing discipline)
**Stated:** director, 2026-07-30, session #226 — verbatim: *"delete reclaimable files on a regular
basis."*
**Context:** given in direct answer to a measured finding — the aggregate's scratch tree had grown to
**198 GB**, of which ~158 GB (and 228 GB across the whole `rust/target/` tree) was a single class of
unread trace log. See `DONE-BAR.5f`.

## The directive

Reclaimable generated artifacts are to be deleted **routinely, not on request**. This upgrades
`CLAUDE.md` §8 (*"roughly every 24 hours, look for generated artifacts that are no longer needed and
safely delete them"*) from a periodic chore to a **standing authorization**: the engineer does not
need to ask before sweeping artifacts that are provably reclaimable.

## What the authorization does NOT relax

⛔ **`CLAUDE.md` §8's own bar is unchanged: *"only delete when it is 100% safe to do so."*** The
authorization removes the need to ASK; it does not remove the need to PROVE. The discipline that makes
a sweep safe is unchanged and is the expensive half:

1. **Prove nothing reads it.** For the 2026-07-30 sweep this meant
   `grep -rn '<artifact stem>' rust/scripts/*.sh scripts/*.sh | grep -iE '\.log|logs/'` → **zero
   hits**, plus confirming the stage's genuinely-consumed artifact is a different file (the structured
   `--parseability-report-json`, `require_nonempty_file`d by the gate). ⭐ This check is mandatory
   because `CI-PARITY-GATE-ROT.11` measured **14 gates that scrape prose log lines for values** — in
   this repository a log CAN be an input.
2. **Prove no tracked oracle depends on it.** `docs/tasks/artifacts/**` probe drivers read gate state
   dirs by literal path; deleting one degrades a committed oracle rather than freeing space
   (`DONE-BAR.5f`).
3. **Delete the narrowest class you verified — never the enclosing tree.** The 2026-07-30 sweep took
   `profile_*_closed_loop_replay_parseability_shadow.log` and left every `summary.*`, every structured
   report and every smaller log untouched (91 summary pairs and 33 shadow reports verified intact
   after).
4. **Defer what you did not verify.** The same census found ~40 GB of `regen_*.log` whose readers were
   NOT checked; they were deliberately left in place. *An unverified file is not a reclaimable file.*
5. **Never sweep an ambiguous shared cache** — the pre-existing project-data policy (`CLAUDE.md` §13)
   still governs: populate a project-local cache and remove only what the project provably owns.

## Cost-bearing exceptions (reclaimable, but not free)

`rust/target/debug/{deps,incremental}` (~64 GB measured) and `rust/target/release` are reclaimable in
the safety sense but **force a rebuild**, and the debug binaries
`rust/target/debug/{ast_pipeline,parseability_probe}` are hard preconditions of the toolbox and of
`silent_success_sentinel_gate`. Sweeping them ahead of a long acceptance run trades hours of rebuild
for disk that is not scarce. **Price the rebuild before sweeping a build cache**; the directive is
about artifacts nobody needs, not about artifacts that are merely regenerable.

## Measured outcome of the first sweep under this directive (2026-07-30)

| | before | after |
|---|---|---|
| `/Volumes/SSD` used | 383 G | **156 G** |
| `rust/target/sota_exit_gate/work` | 198 G | **2.8 G** |
| files removed | — | 18 (one verified class) |
| reclaimed | — | **228 G** |
| `summary.*` pairs / shadow reports | 91 / 33 | **91 / 33** (intact) |

## Why a habit is not the end state

⭐ This repository's own doctrine says *a check nothing invokes is indistinguishable from a check that
does not exist* (`GATE-REACHABILITY`). The same applies to cleanup: **a sweep that depends on someone
remembering is not a mechanism.** The signoff-grade form is a tracked **reclaim register** of
patterns proven unread (evidence per entry) plus a script that deletes only registered patterns,
reports bytes reclaimed, and **refuses** on an unregistered large artifact — the ratchet shape already
used by `gate_reachability_register_v0.json`. Owned by `DONE-BAR.5f`; until it exists, this record is
the durable instruction.

⛔ **And the real fix is upstream of cleanup:** the 228 GB existed because a promotion gate leaves
tracing ON by default (`rust/scripts/sv_stimuli_quality_gate.sh:49`, `REPLAY_TRACE_VERBOSITY` default
`low` = the backtracks level), with 73% of each line a twice-repeated absolute path. Sweeping the
output regularly treats the symptom; `DONE-BAR.5f` owns not generating it.

## Related

- [[feedback_host_ram_budget_all_jobs]] — the sibling resource directive (RAM).
- `keep-project-data-on-project-volume` (auto-memory) + `CLAUDE.md` §13 — data locality.
- `DONE-BAR.5f` — the measurement, the upstream fix, and the register mechanism.
- `CI-PARITY-GATE-ROT.11` — why "nothing reads this log" must be checked, not assumed.

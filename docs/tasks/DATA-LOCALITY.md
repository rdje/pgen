# DATA-LOCALITY: project-owned data stays on the repository volume

## Metadata

- Tree ID: `DATA-LOCALITY`
- Status: **`active`** — created 2026-09-15 by a DIRECT DIRECTOR REMARK. Answering a challenge about
  whether this session's findings still held, I published one that I had NOT checked outside the one
  module it was found in, and closed it with *"say the word and it becomes a leaf"*. The director's
  reply: ***"Raising findings without owning them is just complaint."*** This tree is that ownership.
- Family / slice-id prefix: `PGEN-DATA-LOCALITY-<NNNN>`
- Roadmap lane: operational continuity / host governance — the standing policy in the project
  instructions §13 and [[project_north_star]] (*"data stays on the repo volume"*), sibling to
  `OPS-MEMSAFE` (host RAM) and the disk-hygiene directive
- Created: `2026-09-15`
- Owner: repo-local workflow

## Goal

**Every tracked, LIVE code path that creates project-owned scratch data derives its path from the
repository root at run time** — no `$TMPDIR`, no `/tmp`, no user-home cache — and it is
MECHANICALLY held there, so the class cannot come back one convenience call at a time.

## The frame — how this was found, and what the finding actually was

`SVPP-EXPANSION.2` (2026-09-09) found the SystemVerilog preprocessor's unit tests writing scratch
files to `std::env::temp_dir()` and routed the repair to `.5`. `.5`(e) did the work. **The routed
question then stayed open for eighteen commits while the code was already correct**, and was
corrected on 2026-09-15 (`PGEN-SVPP-EXPANSION-0024`) — ⇒ *a routed item is not closed by the fix; it
is closed by someone re-reading the route.*

⛔ **But the finding underneath was never the 26 tests.** It was measured in ONE module and published
as if that were its extent. The repo-wide census below is what the finding always was.

## The founding measurement (2026-09-15, `scripts/data_locality_census.py`)

```text
DATA-LOCALITY: live=19 sites in 11 file(s)
  [py_tempfile_no_dir=1 rs_env_temp_dir=7 rs_tempfile_default=7 sh_mktemp_off_volume=4];
  frozen=67 sites in 46 probe artifact(s) under docs/tasks/artifacts/**
  [literal_tmp_path=34 sh_mktemp_off_volume=33] — counted, never edited
```

⭐ **TWO POPULATIONS, AND THE SECOND IS NOT A BACKLOG.** `docs/tasks/artifacts/**` holds one-shot
PROBE scripts kept as the evidence of a measurement already taken; rewriting them would falsify the
record of what was actually run. They are COUNTED and CLASSIFIED on every run so the exemption is a
number rather than a silence.

⛔⛔ **THE CLASSIFIER WAS WRONG TWICE BEFORE IT WAS WRITTEN DOWN, AND BOTH CUTS ARE PINNED AS
SELF-TEST ARMS.** `mktemp` has four spellings and only the ARGUMENT decides:

| spelling | verdict |
|---|---|
| `mktemp` / `mktemp -d` | OFF-volume (no template: `$TMPDIR`) |
| `mktemp "${TMPDIR:-/tmp}/x.XXXXXX"` | OFF-volume (an explicit off-volume template) |
| `mktemp "$ROOT/rust/target/x.XXXXXX"` | **ON-volume** — cut 1 counted this as a breach |
| `mktemp -d "$ROOT/rust/target/y.XXXXXX"` | **ON-volume** — cut 2 read the opening quote as the end of the call |

Cut 1 reported **52** shell breaches where the true count is **4**; cut 2 still read a repo-rooted
template as bare. ⇒ the rule is not *"does it mention `mktemp`"* but *"where does the path it returns
RESOLVE"*. A text census is only as good as its ability to tell one spelling from another, and this
instrument's `--self-test` now refuses both mistakes by name.

## Task tree

| Leaf | What | Status |
|---|---|---|
| `.1` | the census instrument + the repo-wide measurement (this section) | ✅ `done` 2026-09-15 |
| `.2` | take the LIVE surface to **0** — 4 shell, 1 python, 13 Rust test, 1 Rust PRODUCTION site | ✅ `done` 2026-09-15 |
| `.3` | the `DATA-LOCALITY` doctrine: the live surface is held at 0, the frozen cohort counted | ✅ `done` 2026-09-15 |
| `.4` | the FROZEN cohort's disposition — 67 sites in 46 probe artifacts, counted and left as the record | ✅ `done` 2026-09-15 (ruled, not edited) |

## Acceptance criteria (tree)

1. A tracked instrument derives the population and can prove it can FAIL (`--self-test`).
2. The LIVE population is **0** and a registered doctrine keeps it there.
3. The FROZEN population is classified and counted on every run, never silently excluded.
4. Every gate, enforcer and runner touched still passes — the enforcers ARE part of the population.

## `.1` the census instrument — ✅ done 2026-09-15

`scripts/data_locality_census.py` (census / `--detail` / `--check` / `--self-test`). The founding
measurement is above. ⛔⛔ **Its classifier was wrong THREE times before it was trusted, and all three
cuts are pinned as self-test arms** — see the table in *The founding measurement*. The third cut is
where it stops being a spelling list: a shell variable in a template is **RESOLVED against its own
assignments in the same file** (four levels) before the verdict, because the question is *where the
path resolves*, never *what the call looks like*. A REACH arm fails if the frozen population ever
reads zero, because a census that gets clean by looking less is the failure mode that looks most like
success.

## `.2` the LIVE surface taken to 0 — ✅ done 2026-09-15

| what | sites | how |
|---|---:|---|
| `rust/src/parse_harness.rs` — **PRODUCTION** | 1 | `fresh_temp_dir` now returns `repo_scratch_dir("pgen_parse_harness")`. ⛔ Not incidental scratch: this directory holds the **generated parser, the scratch crate and the compiled binary** the harness builds for a grammar — project-owned build output by any reading of §13. Uniqueness unchanged (pid + monotonic counter; no clock, no randomness); only the root moved. |
| `rust/src/ast_pipeline/fusibility_census.rs` | 4 | `repo_scratch_dir("fusibility_census_test")` |
| `rust/src/ast_pipeline/library.rs` | 6 | `tempfile::Builder::new().prefix(…).tempdir_in(scratch_root())` |
| `rust/src/ast_shape_contract.rs` | 1 | same |
| `rust/src/bin/parseability_probe.rs`, `rust/src/main.rs` | 2 | `repo_scratch_dir(…)` |
| `knowledge-map/scripts/check_knowledge_map.sh` | 2 | a `$ROOT/rust/target/knowledge_map_gate` template |
| `scripts/check_scratch_slot_header.sh` | 1 | `${TMPDIR:-/tmp}` → `$ROOT/rust/target` |
| `scripts/require_ast_pipeline_features.sh` | 1 | bare `mktemp -d` → a template under the root derived from `${BASH_SOURCE[0]}` |
| `rust/scripts/sv_preprocessor_reference_runner.sh` | 1 | `${TMPDIR:-/tmp}` → a root derived from `${BASH_SOURCE[0]}` |
| `stimuli/sv/adjudicate_external_corpus.py` | 1 | `TemporaryDirectory(dir=REPO_ROOT / "rust/target/adjudicator_refusals")` |

⭐ **One helper, not nineteen decisions**: `rust/src/repo_scratch.rs` (`scratch_root`,
`repo_scratch_dir`) is the single place that answers *where does project-owned scratch go*, so the
answer cannot drift one convenience call at a time. Its own test asserts the property the policy is
about — the path is under the repository and **not** under `std::env::temp_dir()` — and that two
calls differ, because a dozen parallel `cargo test` threads inside one clock tick would otherwise
share a directory (observed once in `sv_preprocessor`'s helper before it grew a counter).

## `.3` the doctrine — ✅ done 2026-09-15

`DATA-LOCALITY` (`scripts/check_data_locality.sh`, registry **41 → 42**). Blocking at zero with no env
escape, which is free on day one by the `SV-CORPUS-GRAD.13e.11` rule: a guard is safe to make blocking
over its whole population exactly when the fix precedes it — all 19 were repaired first.

## `.4` the FROZEN cohort — ✅ ruled 2026-09-15, deliberately NOT edited

**67 sites in 46 probe artifacts under `docs/tasks/artifacts/**`.** These are one-shot scripts kept as
the EVIDENCE of a measurement already taken; a probe that says `/tmp/x` is the record of where that
probe actually wrote. Rewriting them would make the record disagree with the run it documents — the
`GENERATED-REPRODUCIBILITY` defect (an artifact carrying a line its producer cannot emit) in another
costume. They are COUNTED and CLASSIFIED on every run, so the exemption is a **number** rather than a
silence, and the REACH arm refuses a run in which that number falls to zero. ⚠️ Honest bound, stated
rather than discovered: this rules the cohort out of the FIX, not out of the POLICY — a probe script
re-run today would still write off-volume, and the rule for new ones is the live surface's rule.

## Acceptance Checklist (enforced)

- [x] **REPRODUCE / ISSUE** — `python3 scripts/data_locality_census.py --detail` on the pre-change tree:
      `DATA-LOCALITY: live=19 sites in 11 file(s) [py_tempfile_no_dir=1 rs_env_temp_dir=7
      rs_tempfile_default=7 sh_mktemp_off_volume=4]`, one of them `rust/src/parse_harness.rs:605`
      `let dir = std::env::temp_dir().join(format!("pgen_parse_harness_{}_{}", …))` — production.
- [x] **ROOT CAUSE (WHY + WHERE)** — WHY: `std::env::temp_dir()`, `tempfile::tempdir()`, a bare
      `mktemp` and `tempfile.TemporaryDirectory()` all default to `$TMPDIR`, which on this host is a
      different filesystem volume from the repository, against the standing policy (§13). WHERE: the
      eleven files in the `.2` table, each line printed by the census with its byte offset and its
      classified kind (`git ls-files` is the population; the instrument reads it on every run).
      ⛔ The deeper WHERE is procedural and is recorded because it is the reason the class survived:
      the finding was published as a 26-test housekeeping item **in one module**, routed, fixed there,
      and never re-read — so the CLASS was never counted.
- [x] **FIX** — declarative-first is not available (this is a path decision in code), so: one helper
      (`pgen::repo_scratch`) plus nineteen call-site conversions; no behaviour beyond the root.
- [x] **ADDRESSED (verified)** — `python3 scripts/data_locality_census.py` reads
      `live=0 sites in 0 file(s) [none]` against a before of 19; `--self-test` PASS with 10 classifier
      arms + 1 reach arm, including the three cuts that were wrong.
- [x] **NO REGRESSION** — `bash scripts/check_doctrines.sh` **ALL 42 PASS** (the three edited files are
      themselves enforcers: `check_knowledge_map.sh` → `knowledge-map: OK`,
      `check_scratch_slot_header.sh --self-test` 8/8 with its GREEN restore control,
      `require_ast_pipeline_features.sh --self-test` 4/4); `cargo test --lib` over the touched modules
      66/66; `check_generated_reproducibility.sh --rebaseline` **11/11 re-derive byte-identically**
      (the emission-source digest moved because `ast_pipeline/` did, and the artifacts did not);
      `make parse_harness_equivalence_gate` **4/4 green** — the gate that exercises the one
      PRODUCTION site end to end, since the harness builds its scratch crate in exactly the
      directory that moved — and `make clippy_on_rust_change` ✅ (strict source lint, generated-parser
      stage pass, the 68-lint correctness roster intact).
- [x] **LOCKSTEP** — `DOCTRINE_ENFORCEMENT.md` §10, `docs/book/src/the-enforced-doctrines.md`
      (re-derived), `docs/book/src/gate-flow.md` (41 → 42), `docs/TASK_TREE.md`, `MEMORY.md`,
      `CHANGES.md`, `DEVELOPMENT_NOTES.md`.

## Blockers

- None. The SV lane lock does not bind: this is a standing DIRECTOR POLICY (§13) raised by the
  director in direct response to a finding, the same authorisation shape as the 2026-09-09 steer.

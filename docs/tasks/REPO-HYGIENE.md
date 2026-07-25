# Task Tree: REPO-HYGIENE

Repo-hygiene lane — remove vestigial/orphaned tracked artifacts that no build,
gate, or doc depends on, so the repository stays clean and tooling is not
misled. Each removal is evidence-backed (tool-proven orphaned) and lands as its
own leaf under the code-change doctrine.

- **Status:** `active`
- **Roadmap lane:** cross-cutting repo integrity / developer-experience hygiene
- **Owner discipline:** every leaf proves the artifact is unreferenced (no CI /
  script / hook / workspace / doc dependency) BEFORE removal, then verifies the
  real build/gates are unaffected.

## Leaves

### `REPO-HYGIENE.1` — remove the vestigial root `test` package (`Cargo.toml` + empty `src/lib.rs`)

- **Status:** `done` (`PGEN-REPO-HYGIENE-0001`, session #52)
- **Goal:** delete the repo-root `Cargo.toml` (`name = "test"`) and the empty
  `src/lib.rs` — an orphaned scaffold package present since the initial commit,
  built by nothing, whose only live effect is trapping `cargo` invoked from the
  repo root into building the wrong crate.
- **Context / discovery:** surfaced during `GRAMMAR-WELLFORMED.A2.4` (session
  #52) when `cargo test --lib` run from the repo root silently resolved to this
  empty `test` package (`running 0 tests`) instead of the real `pgen` crate
  under `rust/`. Director authorized the cleanup ("you decide … clean the flow
  there").

#### Acceptance Checklist (enforced)
- [x] **REPRODUCE / ISSUE** — `cargo test --lib` from the repo ROOT resolves to
  the `test` package and reports `running 0 tests` / `0 filtered out` (the empty
  root stub), NOT the `pgen` crate's ~695 lib tests; `cargo metadata --no-deps`
  from root reports `"name":"test"`. The stub misdirects any root-level `cargo`.
- [x] **ROOT CAUSE (WHY + WHERE)** — the repo root carries a tracked standalone
  `Cargo.toml` (`name = "test"`, version `0.1.0`, deps `regex`/`lazy_static`) +
  a **0-byte** `src/lib.rs`, both present since `b579dc8a` (initial commit). It
  is NOT a Cargo workspace root (no `[workspace]` stanza) and `rust/Cargo.toml`
  does not reference it, so the real crates (`rust/`, `rtl_frontend/`,
  `rtl_const_expr/`) are wholly independent of it. It is a dead scaffold.
- [x] **FIX** — `git rm` the two files (`Cargo.toml`, `src/lib.rs`). Fix tier =
  repo hygiene (delete orphaned files; no source/codegen/generated/grammar
  behavior touched).
- [x] **ADDRESSED (verified)** — after removal, `cargo metadata` / `cargo test`
  from the repo root no longer resolve a stray `test` package (root is no longer
  a Cargo package); the real `pgen` lib tests still run via `rust/`
  (`cargo test --lib` from `rust/` → 695/695), and all gates run through
  `make -C rust …` / `--manifest-path` unchanged.
- [x] **NO REGRESSION** — tool-proven the stub is unreferenced BEFORE removal:
  no `.github/` workflow builds from repo root (only `cargo install mdbook`);
  every script/gate/hook cargo call uses `-C rust` or `--manifest-path` to
  `rust/`/`rtl_*/`; not in any `.gitignore`; no root `Cargo.lock` or tracked
  root `target/`; no live doc/config references the `test` package (all
  `src/lib.rs` doc hits are cargo's `Running unittests src/lib.rs` output
  strings or historical `rust/src/lib.rs` debug logs; no `core/` crate exists).
  After removal: `rust/` lib build + `cargo test --lib` GREEN (695/695);
  `check_doctrines.sh` all 6 PASS.
- [x] **LOCKSTEP** — N/A (no user-facing surface, book chapter, contract, or
  release affected — an internal dead-scaffold removal). Tracker + this leaf +
  `docs/TASK_TREE.md` row updated.

### `REPO-HYGIENE.2` — §8 artifact sweep of UNTRACKED stray artifacts (director-ordered, session #203)

- **Status:** `done` (`PGEN-REPO-HYGIENE-0002`, session #203, 2026-07-25)
- **Goal:** execute the standing §8 artifact-cleanup mandate
  (`docs/decisions/feedback_disk_hygiene_proactive.md`) on the director's
  explicit end-of-ramp-up order ("delete all the artifacts"), deleting only
  artifacts proven **untracked/ignored, PGEN-owned, and unreferenced**.
- **Charter note:** leaf `.1` removed *tracked* vestigial files; this leaf
  extends the same evidence-first discipline to *untracked* stray artifacts, so
  a routine sweep is tree-owned rather than ad hoc.

#### Acceptance Checklist (enforced)
- [x] **REPRODUCE / ISSUE** — repo at 3.2 GB; sweep found 12 editor swap files
  (`.swp`/`.swo`) in PGEN-owned dirs (`tools/generators/`, `fx/perl/`,
  `perl/AST/`; largest `fx/perl/.mipicsi2_laned_clog.log.swp` 4.8 MB), 3
  `.DS_Store`, 4 stale `test_logs/**` logs (2025-10-03), and 6 legacy generated
  `stability_test_results/*_parser.pm` outputs (2025-08-30).
- [x] **ROOT CAUSE (WHY + WHERE)** — accumulated editor/session residue and
  legacy pre-Rust-era run outputs; all confirmed **untracked** via
  `git ls-files --error-unmatch` per file (the `stability_test_results/*.ebnf`
  inputs and `stability_test_report.txt` ARE tracked and were left untouched),
  and unreferenced by any gate/script/doc.
- [x] **ADDRESSED (verified)** — `du -sk .` before→after **3,326,404 → 3,321,120
  KB = 5,284 KB (5.28 MB) reclaimed**; post-sweep `find` for `*.swp`/`*.swo`/
  `.DS_Store` outside `.git/` and submodules returns **empty**; `df -h /` 44 %
  used / 259 GB free (no disk pressure — the large reclaim already happened in
  session #194).
- [x] **NO REGRESSION** — `git status --porcelain` = **0 lines before AND after**
  (nothing tracked was touched; the tree stayed handoff-ready throughout);
  `git submodule status` unchanged — the 835 `*.log` + 1 `*.bin` under
  `stimuli/**/subs/**` are **upstream-tracked submodule content** (verified
  `git ls-files --error-unmatch` inside `stimuli/sv/subs/Surelog`) and were
  DELIBERATELY not touched, as were `docs/tasks/artifacts/**/*.log` (task-leaf
  evidence) and `regex_corpus_bundle/.cache/downloads` (24 MB pinned upstream
  snapshots — deletion would force a network re-fetch, failing the "100 % safe"
  criterion).
- [x] **DELIBERATELY RETAINED (with cause)** — `preserved_probes/` (185 MB, 18
  probes) = the PERMANENT perf-guardrail re-proof custody per the standing
  director law; `generated/` (244 MB, 33 artifacts = all 11 parsers at HEAD
  vintage) = active working state whose deletion would force a multi-hour cold
  bootstrap, i.e. NOT "no longer needed"; `conversation.txt` (58 MB,
  user-created Feb 2026 capture) = re-affirms the session-#194 ruling recorded
  in `CHANGES.md`/`DEVELOPMENT_NOTES.md` — user-created content is
  director-owned, surfaced for an explicit call rather than swept.
- [x] **LOCKSTEP** — ⭐ the sweep surfaced a live **continuity drift**:
  `rust/target/` is **entirely absent** (no build tree at all), so `MEMORY.md`'s
  ON-DISK block still claiming a built `target/release/regex_perf_probe`
  `8d392176` + debug `parseability_probe` + dual-feature `ast_pipeline` was
  STALE and would have misled the next session's first toolbox command. Layer A
  corrected same-commit (rebuild-required note + the intact
  `preserved_probes/`/`generated/` facts, so a rebuild is a compile, not a cold
  bootstrap). `CHANGES.md` + `DEVELOPMENT_NOTES.md` + `docs/TASK_TREE.md` row
  updated; no code, book, contract, or release surface touched.

## Acceptance Criteria (tree)
- Each leaf removes only artifacts proven unreferenced by CI, scripts, hooks,
  workspace config, and live docs; the real build + gates are verified
  unaffected before and after.

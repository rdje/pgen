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

## Acceptance Criteria (tree)
- Each leaf removes only artifacts proven unreferenced by CI, scripts, hooks,
  workspace config, and live docs; the real build + gates are verified
  unaffected before and after.

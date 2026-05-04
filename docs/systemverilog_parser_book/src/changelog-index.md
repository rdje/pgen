# Changelog Index

This chapter is an index — pointers into other docs that carry the full changelog detail. Use it to find what changed in a given release.

## Where the canonical changelogs live

| Source | Granularity | Purpose |
|---|---|---|
| `docs/contracts/PGEN_SYSTEMVERILOG_PARSER_INTEGRATION_CONTRACT.md` | Per-release shape change | The authoritative contract. Each release's section lists the AST shape changes consumers care about. |
| `docs/contracts/PGEN_RELEASED_PARSER_BUG_LEDGER.md` | Per-bug | When a bug is fixed in a release, the ledger entry records the input/output shape change. |
| `CHANGES.md` (root) | Per-release | Human-readable summary of all changes. |
| Git tags + commit log | Commit-by-commit | The most granular source. |

When investigating "what changed and why," start with the contract document, drop down to the ledger for specific bugs, fall back to git for diffs.

## Releases relevant to this book

This book is **live** and tracks current main HEAD. Versioning summary:

- The most recent **published** parser-release section in the contract is **1.0.0 / Contract 1.0.0** (foundation baseline).

### 1.0.0 / Contract 1.0.0 — Foundation baseline (mdbook + contract Highlights structure)

**What changed:** Initial systemverilog mdBook scaffolded at `docs/systemverilog_parser_book/`. The integration contract `docs/contracts/PGEN_SYSTEMVERILOG_PARSER_INTEGRATION_CONTRACT.md` was upgraded from a thin "stable surface" pointer to the same release-tracked Highlights structure used by the regex parser contract.

**Mdbook chapters landed:** welcome, quickstart, build-recipe, public-api, ast-envelope, parse-content-variants, json-carrier, walking-the-ast, rules-top-level, examples-minimal-module, schema-versioning, glossary, changelog-index. Per-rule and per-feature example chapters land as the annotation campaign progresses.

**Contract section:** [`docs/contracts/PGEN_SYSTEMVERILOG_PARSER_INTEGRATION_CONTRACT.md`](../../contracts/PGEN_SYSTEMVERILOG_PARSER_INTEGRATION_CONTRACT.md) → "Release 1.0.0 / Contract 1.0.0 Highlights".

**Build status:** Generated SV parser is **NOT in default `cargo test` build** — produced on-demand by `sv_stimuli_quality_gate`. See [Build Recipe](build-recipe.md).

**Annotation campaign:** Not yet started. `grammars/systemverilog.ebnf` is un-annotated except for one commented-out trial annotation at line 200. First slice will land in a follow-up commit.

**Schema baseline:** `1` (corresponds to `version: 1` in `rust/test_data/ast_shape_contract/systemverilog_v1.json`).

**Public API surface:** Unchanged. See [Public API Surface](public-api.md).

**Bug ledger:** No SV-NNNN entries blocking the baseline.

## How to track per-slice changes

Each annotation slice gets:

1. A grammar change in `grammars/systemverilog.ebnf` (the `-> ...` annotation).
2. A manifest update in `rust/test_data/ast_shape_contract/systemverilog_v1.json`.
3. A parser-release / contract-version bump in `docs/contracts/PGEN_SYSTEMVERILOG_PARSER_INTEGRATION_CONTRACT.md`.
4. A row in [Schema Versioning](schema-versioning.md) tagging the new schema version.
5. An entry in this changelog index summarising the slice.
6. A regression-lock test in `rust/src/embedding_api.rs` (or related test module) pinning the typed shape.

Per-slice commits should bundle all six (the live-book policy). See `docs/contracts/PGEN_REGEX_PARSER_INTEGRATION_CONTRACT.md` for an example of a mature contract with 50+ Highlights sections to mirror.

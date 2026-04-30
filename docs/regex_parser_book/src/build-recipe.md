# Build Recipe

This chapter is the long form of the [Quickstart](quickstart.md). Read this when:

- The Quickstart didn't work and you need to debug.
- You want to understand what the bootstrap target does internally.
- You need to script the build into a CI pipeline.
- You hit a chicken-and-egg state and need a manual recovery path.

## The Single-Command Path

```bash
make -C subs/pgen/rust SHELL=/bin/bash regex_parser_bootstrap
```

That's the whole recipe. The target is **idempotent** — safe to run unconditionally on every build.

For a guaranteed-fresh wipe-and-rebuild from absolute zero:

```bash
make -C subs/pgen/rust SHELL=/bin/bash regex_parser_fresh
```

## What the bootstrap target does

The Make dependency graph chains the entire build. From a clean checkout, `regex_parser_bootstrap` transitively pulls:

| Stage | Target | Output |
|---|---|---|
| 1 | Build `ast_pipeline_bootstrap` (--no-default-features --features bootstrap) | `rust/target/debug/ast_pipeline_bootstrap` |
| 2 | Bootstrap-mode regen of return_annotation_parser | `generated/return_annotation_parser.rs` |
| 3 | Bootstrap-mode regen of semantic_annotation_parser | `generated/semantic_annotation_parser.rs` |
| 4 | Build `ast_pipeline` (--features ebnf_dual_run) | `rust/target/debug/ast_pipeline` |
| 5 | If `generated/ebnf.rs` is missing: Rust EBNF frontend → `generated/ebnf.json` → `generated/ebnf.rs` | `generated/ebnf.json`, `generated/ebnf.rs` |
| 6 | Rebuild `ast_pipeline` (now with `has_generated_ebnf_parser` cfg active) | `rust/target/debug/ast_pipeline` |
| 7 | EBNF frontend: `grammars/regex.ebnf` → `generated/regex.json` | `generated/regex.json` |
| 8 | Codegen: JSON → `generated/regex_parser.rs` | `generated/regex_parser.rs` |

The chicken-and-egg between steps 4 and 5 was the historic stumbling block: `ast_pipeline`'s `ebnf_dual_run` feature used to require `generated/ebnf.rs` at compile time. Resolved at PGEN parser release 1.1.33 / contract 1.1.35 by gating the include site on `has_generated_ebnf_parser` — set by `rust/build.rs` only when the file exists. The build now compiles cleanly without `generated/ebnf.rs`, lets the Rust frontend produce it, then rebuilds with the cross-check active.

## Manual decomposition

If the single-command target fails midway and you need to debug step by step:

```bash
# Step A — bootstrap binary, no generated/* deps
( cd subs/pgen/rust && cargo build --bin ast_pipeline_bootstrap --no-default-features --features bootstrap )

# Step B — regenerate the bootstrap-safe annotation parsers
make -C subs/pgen/rust SHELL=/bin/bash return_annotation_parser
make -C subs/pgen/rust SHELL=/bin/bash semantic_annotation_parser

# Step C — full ast_pipeline binary (compiles with or without generated/ebnf.rs)
( cd subs/pgen/rust && cargo build --features ebnf_dual_run --bin ast_pipeline )

# Step D — if generated/ebnf.rs is missing, seed it via the Rust frontend
if [ ! -f subs/pgen/generated/ebnf.rs ]; then
    subs/pgen/rust/target/debug/ast_pipeline \
        subs/pgen/grammars/ebnf.ebnf \
        --emit-raw-ast-json subs/pgen/generated/ebnf.json
    subs/pgen/rust/target/debug/ast_pipeline \
        --generate-parser --debug --eliminate-left-recursion \
        subs/pgen/generated/ebnf.json -o subs/pgen/generated/ebnf.rs
    # Rebuild ast_pipeline now that has_generated_ebnf_parser is set
    ( cd subs/pgen/rust && cargo build --features ebnf_dual_run --bin ast_pipeline )
fi

# Step E — finally produce generated/regex_parser.rs
make -C subs/pgen/rust SHELL=/bin/bash regex_parser
```

## Cleaning state

Three increasingly aggressive cleans, depending on what you need to wipe:

```bash
# Drops generated/*.{pl,json,rs,placeholder} files AND `cargo clean`s rust/target
make -C subs/pgen/rust SHELL=/bin/bash clean

# Above + removes the entire generated/ directory
make -C subs/pgen/rust SHELL=/bin/bash clean-all

# `clean-all` + immediate full rebuild via regex_parser_bootstrap.
# Everything from absolute zero. Typical wall time: 4-6 minutes
# (the cargo full rebuild is the long pole).
make -C subs/pgen/rust SHELL=/bin/bash regex_parser_fresh
```

For total-isolation (e.g. a corrupted Make state), manual `rm -rf`:

```bash
rm -rf subs/pgen/generated/
rm -rf subs/pgen/rust/target/
make -C subs/pgen/rust SHELL=/bin/bash regex_parser_bootstrap
```

If you also need to throw away local git changes inside `subs/pgen` (use with care — destructive):

```bash
git -C subs/pgen reset --hard HEAD
git -C subs/pgen clean -fdx generated/ rust/target/
```

## Determinism

Per the slice-5 (2026-04-29) policy in `LIVE_ACHIEVEMENT_STATUS.md`: regen is deterministic given the same input. Two consecutive `make regex_parser_bootstrap` runs against the same `grammars/regex.ebnf` and the same PGEN source must produce **byte-identical** `generated/regex_parser.rs`. To check:

```bash
make -C subs/pgen/rust SHELL=/bin/bash regex_parser_bootstrap \
    && shasum -a 256 subs/pgen/generated/regex_parser.rs > /tmp/sha1
make -C subs/pgen/rust SHELL=/bin/bash regex_parser_bootstrap \
    && shasum -a 256 subs/pgen/generated/regex_parser.rs > /tmp/sha2
diff /tmp/sha1 /tmp/sha2   # must be empty
```

If two consecutive runs produce different SHAs, **that's a non-determinism bug** — file it; do NOT ship the unstable parser.

The SHA itself shifts whenever the grammar source or the PGEN pipeline source legitimately changes. The contract is determinism, not a fixed SHA.

## Optional: typed-entry-point fast path

To get the opt-in `parse_regex_typed()` typed entry point (returns `serde_json::Value` directly, bypassing `ParseNode` allocation):

```bash
# Replace the regex-parser regen step with --enable-parser-hooks:
subs/pgen/rust/target/debug/ast_pipeline \
    --generate-parser --debug --trace --eliminate-left-recursion \
    --enable-parser-hooks \
    subs/pgen/generated/regex.json -o subs/pgen/generated/regex_parser.rs
```

Default `make regex_parser` does NOT register the hook → the default emit doesn't carry the typed methods. The legacy `parse_regex()` API is unchanged either way.

## What can go wrong

| Symptom | Cause | Fix |
|---|---|---|
| `cargo build --features generated_parsers` errors `file not found: generated/regex_parser.rs` | bootstrap target not run yet on a fresh clone | `make regex_parser_bootstrap` |
| `make regex_parser` errors at the EBNF→JSON step (`ast_pipeline: not found`) | the bin hasn't been built yet | use `make regex_parser_bootstrap` instead — it builds `ast_pipeline` first |
| `regex_parser_bootstrap` fails at the seed step with a Rust compile error mentioning `ebnf_generated_parser` or `EbnfParser` | outdated PGEN checkout where the cfg gating wasn't in place yet | pull PGEN to a commit at or after the contract `1.1.35` cold-clone fix |
| Two consecutive `make regex_parser_bootstrap` produce different `generated/regex_parser.rs` SHAs | non-determinism bug | report; do NOT ship the unstable parser |
| Grammar parse error during EBNF→JSON | a stale `grammars/regex.ebnf` from a partial pull, or hand-edit drift | `git -C subs/pgen checkout grammars/regex.ebnf`; rerun bootstrap |
| `make: *** No rule to make target ...` | called from the wrong directory | use `make -C subs/pgen/rust ...` (or `cd subs/pgen && make -C rust ...`) |
| Build succeeds but runtime `parser_embedding_api_contract().supports_regex_generated_backend == false` | downstream's own build wasn't rebuilt with `--features generated_parsers` after the bootstrap | rebuild downstream with the feature |
| `cargo build` errors on `generated_parsers` feature complaining about missing types in `regex_parser.rs` | the regen produced a corrupt or partial `regex_parser.rs` (rare; usually a Make interruption mid-write) | run `make regex_parser_fresh` to wipe and re-bootstrap |

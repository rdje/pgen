# Quickstart for Downstream Consumers

If you've cloned PGEN (or pulled it as a submodule under your own project, e.g. RGX's `subs/pgen`) and need a working regex parser **right now**, the entire build is one command:

```bash
make -C subs/pgen/rust SHELL=/bin/bash regex_parser_bootstrap
```

This is **idempotent** — safe to run unconditionally on every build. After it completes, `subs/pgen/generated/regex_parser.rs` is on disk and your downstream Rust code can build against it via `--features generated_parsers`.

For a guaranteed-fresh rebuild from absolute zero (wipes `generated/` and `cargo clean`s `rust/target/`):

```bash
make -C subs/pgen/rust SHELL=/bin/bash regex_parser_fresh
```

## Prerequisites

| Tool | Version |
|---|---|
| Rust toolchain | `1.95` or newer (project MSRV) |
| `cargo` | ships with rustup |
| GNU `make` | any recent |
| `bash` | any recent |

No Perl is required. Earlier versions of PGEN used `tools/ebnf_to_json.pl` as a bootstrap fallback; the current cold-clone path uses the Rust EBNF frontend exclusively.

## Verifying the build

```bash
echo -n 'a*' > /tmp/sample.txt
cargo run --manifest-path subs/pgen/rust/Cargo.toml \
    --features generated_parsers \
    --bin parseability_probe -- \
    --parse regex /tmp/sample.txt --profile regex_default
```

If you see `parse_full passed for grammar 'regex'`, the parser is alive.

To dump the parsed AST for inspection:

```bash
cargo run --manifest-path subs/pgen/rust/Cargo.toml \
    --features generated_parsers \
    --bin parseability_probe -- \
    --parse-dump-ast-pretty regex /tmp/sample.txt /tmp/sample_ast.json --profile regex_default
cat /tmp/sample_ast.json
```

## What you've just built

`generated/regex_parser.rs` is a single ~9 MB Rust source file that compiles into your library or binary as a module. It exposes `RegexParser::new(input, logger).parse_full_regex()`, which returns a `ParseResult<ParseNode<'input>>`. That `ParseNode` is the **AST envelope** documented in the next chapter.

For the typed-Json fast path (opt-in, regenerates with `--enable-parser-hooks`), see [Public API Surface](public-api.md).

## Pulling new PGEN

When you bump the PGEN submodule pin to a newer commit, just rerun:

```bash
make -C subs/pgen/rust SHELL=/bin/bash regex_parser_bootstrap
```

Make's incremental dep graph rebuilds only what changed. If anything has gone weird, fall back to `regex_parser_fresh` for a guaranteed-fresh wipe.

## Next steps

- If the AST shape changed across the bump, see [Migration from the Recursive Envelope](migration-from-recursive-envelope.md) and the [Changelog Index](changelog-index.md).
- For the structural overview of what comes back from the parser, read [AST Envelope Structure](ast-envelope.md).

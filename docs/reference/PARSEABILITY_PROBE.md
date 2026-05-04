# `parseability_probe` — CLI Reference

`parseability_probe` is a small command-line wrapper around PGEN's `pgen::parser_registry` and `pgen::embedding_api` host surface. It is the canonical terminal-access tool for:

- **CI / verification scripts** that need a yes/no parse result on a single input.
- **Bug-report repro flows** (per `docs/contracts/PGEN_PARSER_ISSUE_REPORTING_PROTOCOL.md`) — the contract requires bug reports to attach a `parseability_probe` invocation that reproduces the issue.
- **AST shape inspection** for downstream consumers walking the typed JSON tree.
- **Capability probing** in CI to discover which generated grammar adapters are present in a given build.

This page is the canonical reference for the tool. Each PGEN parser book (regex, systemverilog, vhdl, rtl_*) links here from its quickstart / build-recipe chapters instead of duplicating the description.

## Source

`rust/src/bin/parseability_probe.rs`. Built via cargo with the `generated_parsers` feature and (typically) `ebnf_dual_run`:

```bash
# In the rust/ directory:
cargo build --release --features generated_parsers --features ebnf_dual_run \
    --bin parseability_probe
# Binary lands at rust/target/release/parseability_probe
```

The tool requires the `generated_parsers` cargo feature; without it the binary builds but every sub-command fails with "parseability_probe requires building with --features generated_parsers".

## Command grammar

```text
parseability_probe --supports <grammar_name>                                    [global flags]
parseability_probe --parse <grammar_name> <input_file>                          [global flags]
parseability_probe --parse-dump-ast <grammar_name> <input_file> [output_file]   [global flags]
parseability_probe --parse-dump-ast-pretty <grammar_name> <input_file> [output_file] [global flags]
```

Global flags (any sub-command):

| Flag | Argument | Meaning |
|---|---|---|
| `--profile` | `<profile_name>` | Select a host profile within the grammar family. Required for grammars that have multiple profiles (e.g. `regex` → `regex_default`; `systemverilog` → `sv_2017` / `sv_2023`). May be specified at most once. |
| `--trace` | _(none)_ | Enable runtime tracing on the parser. Output goes to stderr unless `--trace-log-file` is set. Trace verbosity defaults to `low` and can be raised via the `PGEN_TRACE_VERBOSITY` env var. |
| `--trace-log-file` | `[FILE]` | Redirect runtime trace output to `FILE`. If `FILE` is omitted, a timestamped file under the working directory is used. May be specified at most once. |
| `--max-bytes` | `N` (positive integer) | Truncate the AST dump output if encoded JSON exceeds `N` bytes. Only meaningful for `--parse-dump-ast` / `--parse-dump-ast-pretty`. May be specified at most once. |

Two environment variables are honored as fallback defaults:

| Env var | Effect |
|---|---|
| `PGEN_PARSE_DUMP_AST_MAX_BYTES` | Default `--max-bytes` if not specified on the command line. |
| `PGEN_TRACE_VERBOSITY` | Default trace verbosity (e.g. `off`, `low`, `medium`, `high`) when `--trace` is set. |

## Sub-commands

### `--supports <grammar_name>` — capability check

Returns `0` if a generated parser adapter for `<grammar_name>` is registered in the current build, `2` otherwise. Useful in CI to skip grammar-specific validation when the corresponding parser was not built into this binary.

```bash
parseability_probe --supports regex
# → "generated parseability adapter available for grammar 'regex'"
# Exit 0.

parseability_probe --supports somefakelang
# → "Error: parseability adapter unavailable for grammar 'somefakelang'.
#       Supported grammars: return_annotation, semantic_annotation, ..."
# Exit 1 (or 2 depending on cargo error chaining).
```

### `--parse <grammar_name> <input_file>` — go/no-go parse

Parses `<input_file>` against `<grammar_name>`'s generated parser and prints either:

```text
parse_full passed for grammar 'regex' on '/path/to/input.txt'
```

(exit `0`) or:

```text
Error: parse_full rejected sample for grammar 'regex' on '/path/to/input.txt': <diagnostic>
```

(exit `1`). No AST is written; this is the cheapest way to verify acceptance.

### `--parse-dump-ast <grammar_name> <input_file> [output_file]` — compact AST dump

Parses + writes the JSON AST dump to `output_file`. If `output_file` is omitted, the default is `<grammar_name>_ast.json` in the current working directory.

The dump is the same JSON structure consumers receive from `pgen::embedding_api::parse_grammar_profile_ast_dump_named`. Output is **compact JSON** (no whitespace).

If `--max-bytes N` is specified and the encoded dump exceeds `N` bytes, the dump is truncated at a JSON node boundary and the success line includes truncation diagnostics:

```text
parse_full passed for grammar 'regex' on 'input.txt' (AST truncation diagnostics: <kind>, full_bytes=N1, max_bytes=N2, written_bytes=N3)
```

Exit codes match `--parse`.

### `--parse-dump-ast-pretty <grammar_name> <input_file> [output_file]` — pretty-printed AST dump

Identical to `--parse-dump-ast` but emits **pretty-printed JSON** (2-space indented). Use this for human inspection; use the compact form for tooling that re-parses the JSON.

## Registered grammars (capability-gated by build)

The set of grammars the tool can probe depends on which `has_generated_<grammar>_parser` cfgs are active. The cfgs are set by `rust/build.rs` based on whether the corresponding generated parser is present.

| Grammar name | Always present | Cfg gate | How to enable |
|---|---|---|---|
| `return_annotation` | ✅ | (always) | Built into the library. |
| `semantic_annotation` | ✅ | (always) | Built into the library. |
| `builtin_return_annotation` | ✅ | (always) | Built into the library. |
| `builtin_semantic_annotation` | ✅ | (always) | Built into the library. |
| `ebnf` | ❌ | `ebnf_dual_run` feature + `has_generated_ebnf_parser` cfg | `--features ebnf_dual_run` plus generated-parser presence. |
| `json` | ❌ | `has_generated_json_parser` | Generated parser must be present. |
| `regex` | ❌ | `has_generated_regex_parser` | Generated parser must be present (default in main builds). |
| `systemverilog` | ❌ | `has_generated_systemverilog_parser` | Set `PGEN_SYSTEMVERILOG_PARSER_PATH=/path/to/systemverilog_parser.rs` before `cargo build`. See the SV parser book's [Build Recipe](../systemverilog_parser_book/src/build-recipe.md). |
| `systemverilog_preprocessor` | ❌ | `has_generated_systemverilog_preprocessor_parser` | Similar — gate-only generation. |
| `vhdl` | ❌ | `has_generated_vhdl_parser` | Similar. |
| `rtl_const_expr` | ❌ | `has_generated_rtl_const_expr_parser` | Similar. |
| `rtl_frontend` | ❌ | `has_generated_rtl_frontend_parser` | Similar. |

To list the active set in a given binary, run `parseability_probe --supports <name>` and check the error message — when a grammar is unavailable the error lists the supported set.

## Profiles

Some grammars expose multiple host profiles; pass `--profile <name>` to select one. Profiles documented per parser:

| Grammar | Profiles | Documented in |
|---|---|---|
| `regex` | `regex_default` | `docs/regex_parser_book/` |
| `systemverilog` | `sv_2017`, `sv_2023` | `docs/systemverilog_parser_book/` |
| `vhdl` | (TBD) | `docs/contracts/PGEN_VHDL_PARSER_INTEGRATION_CONTRACT.md` |
| _(others — see per-grammar contract)_ | | |

If `--profile` is omitted, the registry's default for the grammar is used. For grammars with no profile-specific dispatch the flag is ignored.

## Exit codes

| Exit code | Meaning |
|---|---|
| `0` | Success — `--supports` registered the grammar; `--parse` accepted the input; `--parse-dump-ast{,-pretty}` wrote the dump. |
| `1` | Operational failure — adapter unavailable for the grammar, parse rejected the input, file IO error, etc. The error message on stderr describes the specific failure. |
| `2` | Usage error — wrong arg count, malformed flag, conflicting flags. The full usage banner is printed to stderr. |

CI scripts should distinguish `0` (sample is valid) from `1` (sample is invalid OR the grammar isn't available); `--supports` is the way to disambiguate the latter from the former.

## Output format examples

### `--parse` success

```text
parse_full passed for grammar 'regex' on '/tmp/sample.regex'
```

### `--parse` failure

```text
Error: parse_full rejected sample for grammar 'regex' on '/tmp/bad.regex': unexpected token at byte 14
```

### `--parse-dump-ast` success (no truncation)

```text
parse_full passed for grammar 'regex' on '/tmp/sample.regex' (AST dump: /tmp/sample_ast.json)
```

### `--parse-dump-ast` success (truncated)

```text
parse_full passed for grammar 'regex' on 'big.regex' (AST truncation diagnostics: max_bytes_exceeded, full_bytes=2421389, max_bytes=1048576, written_bytes=1048576)
```

The dump file is still valid JSON; the truncation happens at a node boundary and the file ends with a closing brace. Consumers should parse the file and check whether the top-level object contains a `truncated: true` indicator if they need precise full-vs-truncated distinction.

## Common usage patterns

### Quick verification of a parser integration

```bash
echo "test" > /tmp/probe.txt
parseability_probe --parse regex /tmp/probe.txt --profile regex_default
# Expected: parse_full passed for grammar 'regex' on '/tmp/probe.txt'
```

### AST inspection during development

```bash
parseability_probe --parse-dump-ast-pretty regex /tmp/probe.txt /tmp/probe_ast.json --profile regex_default
cat /tmp/probe_ast.json | jq .
```

### Bug-report reproducer

Per `docs/contracts/PGEN_PARSER_ISSUE_REPORTING_PROTOCOL.md`, downstream bug reports should include:

```bash
parseability_probe --parse-dump-ast-pretty <grammar> <repro_input.txt> <repro_ast.json> --profile <profile>
```

…with the resulting `<repro_ast.json>` attached to the bug report so PGEN maintainers can immediately inspect what the parser produced for the reproducer input.

### Capability probing in CI

```bash
if parseability_probe --supports systemverilog; then
    parseability_probe --parse systemverilog /path/to/source.sv --profile sv_2017
else
    echo "SV adapter not in this build; skipping SV gate"
fi
```

### Tracing a parse

```bash
PGEN_TRACE_VERBOSITY=high parseability_probe --parse regex /tmp/probe.txt --profile regex_default --trace
# Trace lands on stderr.

parseability_probe --parse regex /tmp/probe.txt --profile regex_default \
    --trace --trace-log-file /tmp/probe_trace.log
# Trace lands in the file.
```

## Notes for downstream consumers

- **The AST dump JSON shape is the same** as what `parse_grammar_profile_ast_dump_named` returns programmatically. Per-rule shapes are documented in each parser's mdBook (see [regex parser book](../regex_parser_book/src/welcome.md), [systemverilog parser book](../systemverilog_parser_book/src/welcome.md)).
- **Determinism**: same input + same parser-release version → identical AST dump bytes. JSON object keys are emitted in canonical (alphabetical) order; number formatting is canonical. See each parser book's `ast-envelope.md` chapter.
- **Performance**: not optimized for batch use. The tool runs one parse per invocation through the embedding-API path with all the same per-call dispatch the API does. For high-throughput batch parsing, embed `pgen::embedding_api` directly. For PGEN-internal perf measurement see `regex_perf_probe` / `regex_perf_probe_embedding_api` (regex-specific microbenches alongside this tool).

## See also

- `docs/contracts/PGEN_PARSER_ISSUE_REPORTING_PROTOCOL.md` — bug-report protocol that mandates parseability_probe invocations.
- `docs/contracts/PGEN_RELEASED_PARSER_BUG_LEDGER.md` — ledger entries cite the parseability_probe command that reproduces each fixed bug.
- Per-grammar mdBooks — the AST shapes the tool's dump output represents.
- `rust/docs/EMBEDDING_API_CONTRACT.md` — the underlying programmatic API the tool wraps.

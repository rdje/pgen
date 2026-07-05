# `grammars/scratch/` — the PGEN scratch slot (parse-harness approach 3)

This directory is a **blessed, throwaway parser slot**. It exists so you can parse an
arbitrary input against an arbitrary grammar using the **entire shipped
`parseability_probe` toolbox** — with semantics *identical to the shipped generated
parsers* — without wiring a brand-new registered grammar for a one-off probe.

It is the **authoritative-by-construction** delivery of the PARSE-HARNESS capability
(tree `docs/tasks/PARSE-HARNESS.md`, approach 3): the probe runs through the *real*
`register → codegen → drive` pipeline, so whatever it reports is exactly what a
first-class shipped parser would report. The only trusted surface is the small,
one-time scratch registry wiring, which is covered by an integration test.

## The slot

- `scratch.ebnf` — the active probe grammar **and** the committed integration-test
  fixture. Its entry rule is named `scratch` so the registered slot name is stable;
  helper rules can be named anything. Overwrite the body freely to probe.

The generated artifacts (`generated/scratch_parser.rs`, `generated/scratch.json`) are
**git-ignored** and never enter the tracked set. `scratch.ebnf` **is** tracked.

## Workflow

```bash
# 1. Edit grammars/scratch/scratch.ebnf (keep the entry rule named `scratch`).

# 2. Regenerate the scratch parser artifact and rebuild the probe:
make -C rust SHELL=/bin/bash focus_scratch
(cd rust && cargo build --release --features generated_parsers --bin parseability_probe)

# 3. Drive it with the full toolbox (see TOOLBOX.md):
printf 'hello, world!' > /tmp/in.txt
./rust/target/release/parseability_probe --parse scratch /tmp/in.txt
./rust/target/release/parseability_probe --parse-dump-ast-pretty scratch /tmp/in.txt /tmp/out.json
PGEN_TRACE_VERBOSITY=debug ./rust/target/release/parseability_probe \
    --parse scratch /tmp/in.txt --trace-rules scratch

# Static analysis + trustworthiness on the synthetic grammar:
./rust/target/debug/ast_pipeline grammars/scratch/scratch.ebnf --lint-grammar
./rust/target/debug/ast_pipeline grammars/scratch/scratch.ebnf \
    --report-certificate-coverage --entry-rule scratch --count 40 --seed 0
```

## Why this is safe

- `scratch` is **additive and cfg-gated** (`has_generated_scratch_parser`): with no
  built artifact, the slot is simply absent — `--parse scratch` reports the grammar as
  unsupported, exactly as for any un-built grammar. It cannot affect any shipped
  grammar's behavior.
- The registry dispatch calls the generated parser's **entry-rule-agnostic**
  `parse_full()`, so the slot works no matter what the probe grammar's entry rule
  contains — the registration is stable across arbitrary probe grammars.

## Restore before committing

`scratch.ebnf` is the committed fixture the scratch integration test asserts against.
If you edited it for a probe, restore the default before committing:

```bash
git checkout grammars/scratch/scratch.ebnf
```

See the **"The Parse Harness"** chapter in `docs/book/` for the full design, the trust
architecture, and the other two delivery approaches (compile-and-run, interpreter).

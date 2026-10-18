# Speed: the Bar After the Nexsim Release

**No speed number is published yet.** Speed work on SystemVerilog starts once the parser is released to Nexsim. This
page fixes the bar and how it will be measured now, so that every number published later is read against the same
protocol. The work is tracked as task tree `SV-SPEED` (`docs/tasks/SV-SPEED.md`).

## The directive

The director, 2026-10-18:

> After the SV parser is released to NEXSIM we will focus on speed and the SystemVerilog run as fast as it possibly
> can, as we need to be on par with commercial parsers and be lighting fast.

> I can't access commercial tool, so we will have to be on par or even beat slang and Verilator and go beyond.

## The reference tools

Commercial tools are not available here, so the bar is set by the two leading open-source SystemVerilog front ends.

| Tool | Revision | Modes compared | Note |
|---|---|---|---|
| slang | `4106501b`, the revision vendored at `stimuli/sv/subs/slang` (its CMake declares version 11.0) | `-E` (preprocess only), `--parse-only` (*"Stop after parsing input files, don't perform elaboration or type checking"*) | built from that revision's own source, exported read-only from the submodule to `rust/target/` |
| Verilator | 5.050 (host install) | `-E` (*"Preprocess, but do not compile"*), `--lint-only` | has no parse-only mode: `--lint-only` also elaborates, so it is reported as its own column and never as a parse time |
| PGEN | the released `parseability_probe` | `--parse systemverilog … --preprocess` | preprocess → parse, the pipeline Nexsim runs (see [The Preprocess→Parse Pipeline](preprocess-parse-pipeline.md)) |

## The bar

The quantity is the [speed ratio](glossary.md#speed-ratio): PGEN's wall time divided by a reference tool's wall time
for the same file, on the same host, doing the same work. Below 1, PGEN is faster.

- **Parity** is a ratio at or below 1 against the FASTER of slang and Verilator — per file, per corpus suite (the
  geometric mean of its files) and over the whole corpus.
- **Beyond** is the director's word: reaching parity opens the next, stricter bar; it does not end the campaign.
- Peak memory (RSS) is measured the same way, as a ratio.

A ratio, never a raw second count: the same parser reads faster on a faster machine without changing at all, while a
ratio on one host compares the parsers rather than the hardware.

**Reading an example** (illustrative numbers, not a measurement). If one file takes PGEN 0.30 s, slang `--parse-only`
0.12 s and Verilator `-E` 0.20 s, the faster reference is slang, and the file's ratio is 0.30 ÷ 0.12 = **2.5**: PGEN
is 2.5 times slower than the bar on that file. Parity on that file means 0.12 s or less.

## The commands, side by side

For one file with one include directory (the leaf `SV-SPEED.1` fixes how each corpus file's include directories and
defines reach each tool — the same ones the pipeline gives it):

```bash
# PGEN — preprocess + parse, the released probe
rust/target/release/parseability_probe --parse systemverilog top.sv --profile sv_2017 --preprocess --include-dir rtl/include

# slang — preprocess + parse, no elaboration
slang --parse-only -I rtl/include top.sv

# Verilator — preprocess only (a separate column: --lint-only elaborates too)
verilator -E -Irtl/include top.sv > /dev/null
```

## The floor: correctness is never traded

A change that makes the parser faster and moves anything it decides does not land, however large the gain. Before and
after every speed change, the same identity checks a release re-key runs must come back unchanged: every corpus verdict
(`REKEY-MOVERS: IDENTICAL`), every accepted tree (`AST-IDENTITY-SWEEP … ast_moved=0`), the adjudication manifests, and
the certificate (`fully_certified`).

## The order of work

| Leaf | What it delivers |
|---|---|
| `SV-SPEED.1` | the benchmark: the reference builds, each tool's mode, the population, include directories and defines, repeat count and statistic, allocator parity (slang links `mimalloc` by default) |
| `SV-SPEED.2` | the first reading: per-file ratios, per-suite and overall geometric means, the worst files, peak memory, and `uvm_pkg.sv` on its own |
| `SV-SPEED.3` | attribution: a profiler's account of where PGEN spends its time (preprocessing, macro expansion, parsing, building the tree, I/O), ranked |
| `SV-SPEED.4` | the changes themselves, one leaf each, each landing only with the floor green and its before → after ratio measured |

# The Preprocess→Parse Pipeline

Real SystemVerilog is never parsed raw. A design or verification file is littered with compiler
directives — `` `include `` pulls in package and macro headers, `` `define `` / `` `ifdef `` select
what is live, and libraries such as UVM are built almost entirely out of macros — so the text the
parser must judge is the text that exists **after** preprocessing. PGEN's deliverable for
SystemVerilog is therefore a pipeline, not a parser alone:

```text
source file(s) ──► SystemVerilog PREPROCESSOR ──► expanded, directive-free text ──► SystemVerilog PARSER ──► AST
                   (macros · `include · `ifdef)          (+ a byte-indexed SOURCE MAP)              (+ verdict)
```

This chapter describes that pipeline as it ships: what each stage is, the **compilation context**
you hand it, how to run the whole thing in **one command**, how a rejection is mapped **back to the
line you wrote**, and what the pipeline can and cannot yet do. (The bare parser — a grammar
generated from `grammars/systemverilog.ebnf` — is documented in the rest of this book; the
preprocessor's *directive* grammar has its own book, the SystemVerilog Preprocessor Parser reference,
which is a different deliverable from the expansion stage described here.)

## The two stages

| Stage | What it does | Where it lives | Its own CLI |
|---|---|---|---|
| **Preprocessor** (expansion) | object-like and function-like macros with defaults, `` `" `` stringize, ``` `` ``` token paste, nested expansion, `` `undef ``, `` `__FILE__ `` / `` `__LINE__ ``; `` `include `` with a search path, cycle and depth guards; `` `ifdef `` / `` `ifndef `` / `` `elsif `` / `` `else `` / `` `endif ``; a byte-indexed source map, an event log and diagnostics | `rust/src/sv_preprocessor.rs` | `ast_pipeline INPUT.sv --preprocess-systemverilog` |
| **Parser** | the generated SystemVerilog parser, profile `sv_2017` / `sv_2023` / `verilog_2005` | `generated/systemverilog_parser.rs` | `parseability_probe --parse systemverilog FILE` |

The two compose through one type: `PreprocessedOutput { text, source_map, events, diagnostics,
included_files }`. The parser reads `text`; the source map is what turns a byte offset in that text
back into `file:line:column` of the source.

## The compilation context

A preprocessor without a context is a guess. Three things a real front end always knows are the
context the pipeline takes, and each has the same spelling on both CLIs:

| Context | Flag (both `ast_pipeline --preprocess-systemverilog` and `parseability_probe --preprocess`) | Meaning |
|---|---|---|
| include search path | `--sv-include-dir DIR` (repeatable, ordered) | where `` `include "x.svh" `` is looked for after the including file's own directory |
| predefined macros | `--sv-define NAME` / `--sv-define NAME=TEXT` (repeatable, ordered) | seeded into the macro table before the first line is read, exactly as if `` `define NAME TEXT `` stood at the top of the unit; `NAME` alone defines it with **empty** text (the meaning a bare `` `define NAME `` line has). A predefine is an ordinary definition: `` `ifdef ``, `` `undef `` and the redefinition policy treat it as one |
| the compilation unit | `--sv-unit-file F` (repeatable, ordered) | the files preprocessed **before** the input, under the **same macro table**; the input is the unit's last file |

The unit flag is not a convenience. IEEE 1800-2017 §22.5.1 says: *"Once a text macro name has been
defined, it can be used anywhere in the compilation unit where it is defined. There are no other
scope restrictions once inside the compilation unit"*, and §3.12.1 makes *"all files on a given
compilation command line"* one compilation unit. A macro defined in the first file of a unit is in
force in the last; the pipeline keeps ONE macro table across the list and produces one expansion
per file. What `` `resetall `` does to that table is nothing — §22.3: *"Directives that do not have
a default (e.g., `` `define `` and `` `include ``) are not affected by `` `resetall ``."*

## One command

```bash
# preprocess, then parse — in one process, no scratch file
./rust/target/release/parseability_probe --parse systemverilog hw/ip/uart/rtl/uart.sv \
    --profile sv_2017 --preprocess \
    --sv-include-dir hw/ip/prim/rtl --sv-include-dir stimuli/sv/uvm/uvm-core-2020.3.1/src \
    --sv-define SYNTHESIS
# parse_full passed for grammar 'systemverilog' on 'hw/ip/uart/rtl/uart.sv' [preprocessed]

# the AST of the EXPANDED text, same flags
./rust/target/release/parseability_probe --parse-dump-ast-pretty systemverilog top.sv out.json \
    --profile sv_2017 --preprocess --sv-define UVM

# a compilation unit: defines.svh first, then the input, one macro table
./rust/target/release/parseability_probe --parse systemverilog dut.sv --profile sv_2017 \
    --preprocess --sv-unit-file defines.svh
```

`--preprocess` is SystemVerilog-only (`--preprocess` with `json` is refused by name), and the three
context flags are refused without it — a context with nothing to apply it to is a typo, not a
no-op. The library flags compose unchanged: `--lib-out` on a package file and `--lib-in` on the
file that imports it work exactly as they do without `--preprocess`, so a multi-file design is
`bootstrap files with --lib-out, then the top with --lib-in`, each file preprocessed in its turn.

### Three outcomes, three exit codes

| Exit | Meaning | What you see |
|---|---|---|
| `0` | preprocessed and **accepted** | `parse_full passed … [preprocessed]` |
| `1` | preprocessed and **rejected** by the parser | `parse_full rejected … [furthest_position=N, …] [origin=FILE:LINE:COL]` |
| `3` | the **preprocessor refused** the input — the parser never ran | `preprocess_failed for grammar 'systemverilog' on '…': …` on stderr |

The third code exists so a corpus lane can tell *"the language did not accept this"* from *"the
input could not even be expanded"* — an unresolved `` `include ``, an unterminated `` `ifdef ``, an
illegal `` `define ``. Those are different populations with different owners.

## A rejection names the line you wrote

The parser reports positions in the text it parsed — the expanded text, which exists nowhere on
disk. Under `--preprocess` the probe maps the deepest position any branch reached
(`furthest_position=`, the diagnostic that locates a defect; see the debugging chapter of the
platform book) back through the preprocessor's source map and appends `[origin=FILE:LINE:COL]`:

```text
# a syntax error inside an INCLUDED file names that file and its line
parse_full rejected sample … : Parser did not consume full input at position 12
    [furthest_position=42, +30 bytes deeper than surface position] [origin=inc/bad.svh:3:11]

# a syntax error inside a MACRO EXPANSION names the line of the macro's USE
parse_full rejected sample … : … [furthest_position=38, …] [origin=macro.sv:4:13]
```

The rule behind the second line: the preprocessor emits one source-map chunk per output line, and
an expanded macro's chunk records the line the macro was **used** on — the expansion has no line
of its own. The column is a character column (a two-byte `é` before the position counts once).
Paths are printed relative to the working directory when they sit under it. A position the map
cannot place (an empty expansion) reads `origin=unknown` rather than nothing.

An opt-in summary line describes the expansion itself:

```bash
PGEN_REPORT_PREPROCESS=1 ./rust/target/release/parseability_probe --parse systemverilog f.sv --profile sv_2017 --preprocess
# preprocess: input='f.sv' unit_predecessors=0 include_dirs=0 defines=0 output_bytes=37 source_map_entries=1 included_files=1 macro_expansions=1 warnings=0
```

It is opt-in for the same reason the stack-probe line is: PGEN's corpus runner captures the probe's
stderr as a per-file measurement column, and an unconditional line would edit a tracked artifact.

## What the pipeline does today, measured

- The seven curated real-design cases of the external-corpus triage gate (UVM's two packages,
  scr1, friscv, VeeR) run through the single command with the manifest's include directories:
  the expansion of `uvm_pkg.sv` (3 022 806 bytes from 166 included files) is **byte-identical**
  to the shell gate's, and scr1 / friscv / VeeR accept on both profiles.
- UVM's `uvm_pkg.sv` currently **rejects** at `src/base/uvm_transaction.svh:443` —
  `const local uvm_event_pool events = new("events");` — on both profiles. That is not a
  preprocessing defect: IEEE 1800-2017 A.2.7 gives a `const`-first class property only the
  alternative `const { class_item_qualifier } data_type const_identifier [ = constant_expression ] ;`,
  and `new(…)` is not a `constant_expression` (the `local const …` order is derivable through
  `data_declaration`, which admits a `class_new` initializer). Every commercial simulator accepts
  the UVM spelling; PGEN accepted it too until release `1.0.2xx` of 2026-09-04 made `local` a
  reserved word (it had lexed as an identifier and the line mis-parsed into an accept). The policy
  question — a documented tool-extension arm for a construct the de-facto standard library relies
  on, versus the letter of Annex A — is open in `docs/tasks/SVPP-EXPANSION.md` (`.5`).
- A single file parsed alone can only report what it contains. The first corpus smoke turned an
  RTL file the raw lane could not judge (`hw/ip/uart/rtl/uart.sv`, deferred as macro-dependent)
  into a clean **accept**, and turned two DV files' macro barrier into the barrier behind it: a
  class type declared in another file (`uvm_reg_addr_t`, `dv_base_env_cfg`). The pipeline removes
  the preprocessing barrier; the cross-file declaration barrier is the library mechanism's
  (`--lib-out` / `--lib-in`) and the parser's store gates, and the corpus lane measures its size.

## Boundaries

- The expansion engine is hand-written Rust (`rust/src/sv_preprocessor.rs`), not a grammar; its
  *directive* structure is separately described by the `systemverilog_preprocessor` grammar, whose
  parser does not expand anything. The two are different deliverables and this chapter is about
  the first.
- The pipeline preprocesses one compilation unit at a time and parses its last file; a
  multi-file design still needs the library flags for cross-file declarations.
- Nothing yet reads a build system's own context (`.core`, `.f`, `+incdir+`); the context is what
  you pass on the command line.

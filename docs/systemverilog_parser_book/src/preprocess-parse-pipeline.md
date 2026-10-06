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
| **Preprocessor** (expansion) | object-like and function-like macros with defaults, `` `" `` stringize, ``` `` ``` token paste, nested expansion, `` `undef ``, `` `__FILE__ `` / `` `__LINE__ ``; `` `include `` with a search path, cycle and depth guards; `` `ifdef `` / `` `ifndef `` / `` `elsif `` / `` `else `` / `` `endif ``; a byte-indexed source map, an event log and diagnostics | `rust/src/families/systemverilog/preprocessor.rs` (the family's component, behind the engine's `preprocessing_stage`) | `ast_pipeline INPUT.sv --preprocess systemverilog` |
| **Parser** | the generated SystemVerilog parser, profile `sv_2017` / `sv_2023` / `verilog_2005` | `generated/systemverilog_parser.rs` | `parseability_probe --parse systemverilog FILE` |

The two compose through one type: `PreprocessedOutput { text, source_map, events, diagnostics,
included_files }`. The parser reads `text`; the source map is what turns a byte offset in that text
back into `file:line:column` of the source.

## The compilation context

A preprocessor without a context is a guess. Three things a real front end always knows are the
context the pipeline takes, and each has the same spelling on both CLIs:

| Context | Flag (both `ast_pipeline --preprocess systemverilog` and `parseability_probe --preprocess`) | Meaning |
|---|---|---|
| include search path | `--include-dir DIR` (repeatable, ordered) | where `` `include "x.svh" `` is looked for after the including file's own directory |
| predefined macros | `--symbol NAME` / `--symbol NAME=VALUE` (repeatable, ordered; alias `--define`) | seeded into the macro table before the first line is read, exactly as if `` `define NAME TEXT `` stood at the top of the unit; `NAME` alone defines it with **empty** text (the meaning a bare `` `define NAME `` line has). A predefine is an ordinary definition: `` `ifdef ``, `` `undef `` and the redefinition policy treat it as one |
| the compilation unit | `--unit-file F` (repeatable, ordered) | the files preprocessed **before** the input, under the **same macro table**; the input is the unit's last file |
| a formal inside a plain `"…"` string | `--stage-opt plain_string_substitution=standard\|paste_delimited\|all` (default `standard`) | what a formal's name inside a plain string literal in MACRO TEXT means: `standard` — the string is opaque, IEEE 1800-2017 §22.5.1's own `` `define H(x) "Hello, x" `` prints `Hello, x`; `paste_delimited` — only a formal marked by paste tokens on both sides (`"…[``i``]…"`, OpenTitan's idiom) is substituted; `all` — every formal substituted and every paste joined (Icarus's reading). The same spelling on `ast_pipeline --preprocess-systemverilog` |

The spellings are the neutral engine's (`ENGINE-UNIVERSAL-SERVICES.113`, 2026-09-29): the family's own
knobs travel as `--stage-opt KEY=VALUE` — `ast_pipeline INPUT.sv --preprocess systemverilog
--list-stage-options` lists them: `include_max_depth`, `include_path_policy`, `macro_redefine_policy`,
`conditional_symbol_policy`, `conditional_expr_policy`, `plain_string_substitution`,
`strict_warning_codes` — and the pre-`.113` spellings (`--preprocess-systemverilog`, `--sv-include-dir`,
`--sv-define`, `--sv-unit-file`, `--sv-keyword-profile`, the `--sv-*-policy` flags,
`--sv-disallow-macro-redefine`, the three `--sv-*-json` outputs) are accepted through release `1.0.247`,
each rewritten to the neutral form with one deprecation line on stderr, and removed after it.

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
    --include-dir hw/ip/prim/rtl --include-dir stimuli/sv/uvm/uvm-core-2020.3.1/src \
    --symbol SYNTHESIS
# parse_full passed for grammar 'systemverilog' on 'hw/ip/uart/rtl/uart.sv' [preprocessed]

# the AST of the EXPANDED text, same flags
./rust/target/release/parseability_probe --parse-dump-ast-pretty systemverilog top.sv out.json \
    --profile sv_2017 --preprocess --symbol UVM

# a compilation unit: defines.svh first, then the input, one macro table
./rust/target/release/parseability_probe --parse systemverilog dut.sv --profile sv_2017 \
    --preprocess --unit-file defines.svh
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
# preprocess: input='f.sv' unit_predecessors=0 include_dirs=0 symbols=0 output_bytes=37 source_map_entries=1 included_files=1 events=5 warnings=0
```

It is opt-in for the same reason the stack-probe line is: PGEN's corpus runner captures the probe's
stderr as a per-file measurement column, and an unconditional line would edit a tracked artifact.

## What the pipeline does today, measured

The pipeline lane (`stimuli/run_pipeline_lane.sh`, joined against the raw lane by
`stimuli/sv/pipeline_vs_raw.py`; the tracked report is
`stimuli/sv/characterization/pipeline_characterization.md`, whose identity block names the probe,
grammar, parser, context and dumps every number below depends on) parses every source file of a
corpus suite through the chain under a context DERIVED from the corpus itself
(`stimuli/sv/pipeline_context.py`: for every `` `include `` an includer's own directory does not
satisfy, the one directory that does — any file of the tree, whatever its suffix, since `.4d`; a tie
between copies of one text resolved per file to the nearest copy; an ambiguous name is reported, never
guessed, and a directory the derivation would emit that also holds an ambiguous name is refused (a suite
directory) or withheld for its includer (a per-file one), since `.4d` — and, since
`.4c.1`, the defines a file's own driver passes for it: verilator's `t_x.py` flags and sv-tests'
`:defines:` header, a literal value only, applied per file as `--symbol`; a value the driver
computes at run time and a file several variant drivers configure differently are reported, not
guessed. These are the test's own configuration, not a tool identity: `.6b.3` measured and declined
the identity predefines — and, since `.4c.2`, the unit predecessors a file's own harness compiles
before it, applied as `--unit-file` in order: verilator's source-file arguments, slang's `.f` command
files, sv2v's explicit multi-file runs; a `-f` list, a glob, a `--siloed` run and an ordering conflict
are reported, not applied — and, since `.6d.6`, the predecessors a suite's whole FLOW compiles before every
file, hand-curated with their source: OpenTitan's dvsim builds one compilation unit that starts with UVM and
compiles `dv_utils_pkg.sv` and the RTL before any DV package, so `uvm_macros.svh`, `dv_macros.svh`,
`prim_assert.sv` and, since `.6d.7`, `cip_macros.svh` precede every OpenTitan row; and, since `.6d.8`, VeeR's
generated configuration header `snapshots/default/common_defines.vh` — the first source its Makefile
compiles (`tools/Makefile:167`, `:210`), which no VeeR file `include`s by name — precedes every VeeR row,
with the two other entries of the Makefile's `defines` list after it for fidelity). The first
measurement, on the two suites the raw lane cannot judge at all (every one of their rows is
`deferred:chained_only` in the raw answer key), release probe, `sv_2017`, 45 s wall:

| suite | rows | raw lane pass | pipeline pass | pipeline fail | preprocessor refused | Δ pass |
|---|---:|---:|---:|---:|---:|---:|
| OpenTitan | 3 983 | 817 | 1 073 | 2 880 | 30 | +256 |
| UVM 2020.3.1 (file by file) | 174 | 33 | 28 | 146 | 0 | −5 |

Read the transitions, not the delta. On OpenTitan 328 rows went `fail → pass` — RTL and DV files
whose only barrier was a macro or an `` `include `` — and 69 went `pass → fail`; **all 69 are hollow
raw passes**: the raw parse had accepted a file that only `include`s others because the directive
is opaque to it, and the pipeline pulled the included text in and judged it (its rejection lands in
the included file, not in the row's own). The same holds for UVM's six. Not one `pass → fail` lands
in the row's own file.

The 30 preprocessor refusals are the context, not the engine: 29 are `` `include ``s of a name
that exists under several `top_*` variants or nowhere in the tree (a per-file context only a
build system's `.core` file supplies), and 1 is `` `include `REQUESTS_FILE `` — refused at that time by a
ruling that read §22.4's Syntax 22-1 (`"filename"` and `<filename>`) and not §22.5.1, which says in a sentence
that *"the `` `include `` directive can be followed by a macro, instead of a string literal"*. The ruling was
wrong and is retracted (`SVPP-EXPANSION.5`(m), 2026-10-02): the macro is now expanded first and what it yields
is read as the filename. The engine's
own refusals in the first run of this lane — a `` `define `` whose formal list spans lines, a
conditional directive inside a macro body used mid-line, `` `ifdef X // comment `` reading the
comment as part of the symbol — were fixed the same day (`SVPP-EXPANSION.5`), and the re-run moved
exactly the rows they had blocked: five refusals became parser verdicts and four rejections became
accepts, none the other way.

**What the rejections sit on once the macro barrier is gone.** The origin line of every remaining
rejection, read from the tree and bucketed by construct:

| construct at the origin line | rows | share of 3 026 |
|---|---:|---:|
| a class header with a base class (`class X extends Y`) | 2 049 | 67.7 % |
| other (covergroups, assertions, …) | 786 | 26.0 % |
| a parameter port list with a type default (`#(type CFG_T = other_file_cfg`) | 80 | 2.6 % |
| a `::`-scoped name | 45 | 1.5 % |
| a `uvm_*` type name, a `typedef`, a subroutine header | 63 | 2.1 % |
| the `const local … = new(…)` construct of `uvm_transaction.svh:443` | 2 | 0.1 % |

**The whole lane.** Every suite of the 16 336-row corpus through the chain (`sv_2017`; measured on
the **debug** probe with a 600 s per-file deadline, 432 s wall — see the note on the host below —
where the first reading used the release probe at 60 s and 155 s wall), each suite under its derived
context — a name that exists under several directories resolved per file to the includer's nearest
candidate, a tie reported:

| suite | rows | raw pass | pipeline pass | pipeline fail | refused | timeout | Δ pass |
|---|---:|---:|---:|---:|---:|---:|---:|
| verilator | 3 263 | 2 084 | 2 955 | 291 | 17 | 0 | +871 |
| iverilog | 3 799 | 3 214 | 3 550 | 239 | 10 | 0 | +336 |
| OpenTitan | 3 983 | 817 | 1 077 | 2 897 | 9 | 0 | +260 |
| ispras-sv-tests | 1 266 | 1 125 | 1 254 | 12 | 0 | 0 | +129 |
| sv2v | 953 | 714 | 803 | 130 | 20 | 0 | +89 |
| scr1 | 50 | 8 | 49 | 1 | 0 | 0 | +41 |
| sv-tests | 1 028 | 856 | 894 | 129 | 5 | 0 | +38 |
| Surelog | 828 | 711 | 742 | 74 | 12 | 0 | +31 |
| friscv | 441 | 38 | 61 | 348 | 32 | 0 | +23 |
| verible | 152 | 120 | 129 | 23 | 0 | 0 | +9 |
| Cores-VeeR-EL2 | 102 | 19 | 23 | 79 | 0 | 0 | +4 |
| slang | 92 | 72 | 73 | 16 | 3 | 0 | +1 |
| black-parrot | 205 | 36 | 33 | 2 | 170 | 0 | −3 |
| UVM (file by file) | 174 | 33 | 28 | 146 | 0 | 0 | −5 |
| **total** | **16 336** | **9 847** | **11 671** | **4 387** | **278** | **0** | **+1 824** |

The raw lane's 5 693 `match` rows stay green through the pipeline (5 693 of 6 064 `match` rows
pass; the 370 that do not are `must_reject` rows correctly rejected, and one refused). Of the 278
refusals, 170 are black-parrot's: its sources include headers its build system generates and the
vendored tree does not carry — a permanent gap of the corpus, not of the engine — and 32 are
friscv's hardfloat test includes of the same kind. Two raw passes became rejections **in the row's
own file** — a fragment whose whole expansion is a `` `__FILE__ `` string, and one owed — and
**19 fixtures now refuse at the preprocess stage** rather than reaching the parser: the
`:should_fail_because:` chapter-22 tests, verilator's `_bad` and sv2v's `error/` fixtures and the
Surelog IncludeAndMacros tops, whose `mode.vh` transcribes the standard's own *"illegal, only one
argument"* line. Six of those had been **passing** in both lanes (an intended-fail fixture the
parser accepted because the preprocessor bound a missing actual to nothing); the three 60 s
timeouts of the first reading (a macro naming itself twice, expanded exponentially) are refusals in
milliseconds; and four OpenTitan coverage interfaces are recognised that were not, because their
`` `DV_FCOV_INSTANTIATE_CG `` had been mis-expanded for six months (below).

**What the preprocessor refuses, and why.** Since `SVPP-EXPANSION.5`(g)(h) the expansion stage
raises its own errors — the single command reports them as `preprocess_failed` (exit 3) at the
ORIGIN file and line, each carrying the standard's sentence:

- `E_SVPP_MISSING_MACRO_ARGUMENT` — the actual list is shorter than the formals and a formal it
  leaves unbound has no default (IEEE 1800-2017 §22.5.1: *"… error if any of the remaining formal
  arguments does not have a default specified"*; its example `` `MACRO1 ( 1 ) // ILLEGAL: b and c
  omitted, no default for c ``). An EMPTY actual is a different thing and stays legal: present and
  blank, it is *"replaced by nothing"* (`` `MACRO1 ( , 2, ) ``) or takes the default.
- `E_SVPP_EXTRA_MACRO_ARGUMENT` — more actuals than formals (*"It shall be an error to specify more
  actual arguments than the number of formal arguments"*; `` `D(,,) ``).
- `E_SVPP_RECURSIVE_MACRO` — a macro expands, directly or through others, to a usage of itself; the
  message names the chain (`` `TOP -> `BOTTOM1 -> `BOTTOM -> `TOP ``). The standard's exception —
  *"an actual argument to a macro or a default may contain a usage of itself"* — is decided by
  **who wrote the usage**: every byte of an expansion carries whether the caller wrote it (a source
  line, or text that came in through an actual or a default) or a macro body produced it. A
  caller-written usage starts its own expansion context; a body-produced one is checked against
  the stack of bodies being rescanned. That distinction is what lets OpenTitan nest a
  `` `DV_SPINWAIT `` inside the actual of another (its inner body re-enters a macro the outer
  expansion still holds) while `` `define A(x) `B(`A(x)) `` is still refused: the `` `A `` handed to
  `B` is A's own body text.

**What the expansion stage rewrites, and what it leaves.** Four rules landed with `SVPP-EXPANSION.6b.2`,
each one a sentence of the standard that a corpus row had been failing on:

- **A one-line comment in macro text is not substituted** (§22.5.1: *"the comment shall not become part
  of the substituted text"*). `` `define N 21 */* foo */2//foo `` expands to `21 */* foo */2` — the block
  comment stays, it is white space to the parser; a string keeps its slashes; a continued line keeps its
  continuation even when the comment sat on it.
- **Nor is a one-line comment written inside an ACTUAL** (`SVPP-EXPANSION.5`(l), 2026-10-02 — the same
  sentence, read of the text that gets substituted). `` `M(a == b // why, exactly<newline>) `` used to expand to
  `if (!(a == b // why, exactly)) …`: the comma inside the comment correctly did not split the argument, but
  the comment's text was carried onto the expansion's single line and commented out the rest of the call —
  silently (`diagnostics=0`). Now the comment is dropped up to its line feed when the actual is bound; a
  block comment stays, and a `//` inside a string literal is the string's. OpenTitan writes this in its
  `` `DV_CHECK_RANDOMIZE_WITH_FATAL(req, … // TODO … ) `` calls and in RTL assertion macros: four corpus
  rows moved `fail → pass` under the pipeline and none the other way.
- **The paste token joins its sides in an object macro too** (§22.5.1: *"A `` delimits lexical tokens
  without introducing white space"*): `` `define YUCK /``* garbage *``/ `` expands to a comment, as a
  function macro's substitution always did. In a source line the token stays for the parser to judge.
- **A well-formed `` `line `` directive is consumed** (§22.12: *"the `line directive shall set the line
  number and file name of the following line"*). The expansion stage is the tool that maintains the current
  file and line, so `` `__FILE__ `` and `` `__LINE__ `` after it read the assigned location, the event log
  records the directive (`line_directive`), and the parser never sees it. A malformed one — a level other
  than 0, 1 or 2, a missing parameter, a comment on its line — is not this stage's to consume: it passes
  through, and the parser's own §22.12 production refuses it, so the answer key's must-reject rows on
  malformed `` `line `` lines keep their verdict under the pipeline.
  Its parameters may be MACRO USAGES — `` `line `new_line_num `foo(line) 1 `` — and are expanded first
  (IEEE 1800-2023 §22.2: *"Macro expansion shall occur within a compiler directive"*; 1800-2017 §22.5.1: a
  defined text macro *"can be used anywhere in the compilation unit where it is defined"*); the directive is
  consumed when what they yield is well-formed, and reads exactly as the literal form does
  (`SVPP-EXPANSION.5`(n); before, the macros were expanded on a pass-through line and a raw `` `line ``
  reached the parser, which has no production for it inside a procedural block).
- **A comment between the formals of a `` `define `` is refused, and the refusal names the clause.**
  §22.5.1 (both editions): *"The formal argument names shall be simple_identifiers, separated by commas and
  optionally white space"*; §5.3's white space is spaces, tabs, newlines and formfeeds, and the standard
  writes *"white space or a comment"* where it admits one (§22.4, on the `` `include `` line). No sentence of
  either edition admits a comment there, so `` `define ARGPAR(a, /* c */ b) `` and a `//` after a formal are
  refused (`SVPP-EXPANSION.5`(o)); a NEWLINE between formals is white space and is accepted. Three corpus
  files write the comment form — counted by the lane itself, the rows whose refusal carries this reason — and
  all three are tool test suites probing the edge: verilator `t_preproc.v:277` (its own comment reads
  *"comments MIGHT not be legal"*), iverilog `macro_with_args.v:17` and sv2v `macro_arg_comment.sv:1`. No
  design file in the corpus writes it, so no opt-in tolerance is offered for it.
- **Three readings follow the EDITION of the profile the text will be parsed under** (`SVPP-EXPANSION.5`(p);
  `--profile sv_2023` and `sv_2023_tolerant` select IEEE 1800-2023, every other profile and the unprofiled reading
  select 1800-2017). They are the three places where 1800-2023's clause 22 says more than 1800-2017's:

  | text | under the 2017 reading | under the 2023 reading | the sentence |
  |---|---|---|---|
  | a block comment in macro text — `` `define Y(a) a /* c */ + 1 `` | the comment is part of the substituted text | the comment is NOT substituted (one space stands where it was) | 1800-2023 §22.5.1: *"If a one-line comment or block comment (see 5.4) is included in the text, then the comment shall not become part of the substituted text"*; 1800-2017 says it of one-line comments only |
  | a block comment that the macro text opens and the NEXT line closes — `` `define X 1 /* multi `` / `line */ + 2` | REFUSED, naming the clause: the newline ended the macro text inside a comment (*"shall not be split across … Comments"*) | the comment carries the `` `define `` across its newlines; `X` is `1 + 2` | 1800-2023 §22.5.1: *"A newline character that is not contained in a triple-quoted string literal or in a block comment and is not preceded by a backslash shall end the macro text"* |
  | a macro USAGE as the argument of `` `ifdef `` / `` `ifndef `` / `` `elsif `` / `` `undef `` — `` `ifndef `ID(HAVE) `` | the argument is read as written (Syntax 22-6 takes a `text_macro_identifier`) | the usage is expanded first and the directive reads what it yields | 1800-2023 §22.2: *"Macro expansion shall occur within a compiler directive"* |

  The 2017 column is the strict reading of that edition, not a guess at tool behaviour: where the older text has no
  sentence the newer one adds, the older reading stays as it was. Each of the first two rows that changes a verdict is
  pinned as an edition twin — one file accepted under `sv_2023`, its byte twin rejected under `sv_2017`.
- **Include nesting is bounded by DEPTH, not by a file's reappearance** (`SVPP-EXPANSION.5`(q); §22.4: *"The
  number of nesting levels for include files shall be finite. Implementations may limit the maximum number of
  levels to which include files can be nested, but the limit shall be at least 15"*). A file that includes
  itself under a guard (`` `ifndef INCLUDED `` / `` `define INCLUDED `` / `` `include "self.sv" `` / `` `else `` …)
  nests once and is finite — sv2v's `file_line.sv` and `include_self.sv` write that shape — and is read; an
  unguarded loop is refused when it passes the stage's limit of 64 levels, and the refusal cites §22.4 and shows
  the chain's last steps. Through `1.0.250` any file already on the include stack was refused as an
  "include cycle", guarded or not.
- **An escaped identifier is a macro name** (`SVPP-EXPANSION.5`(q); 1800-2017 Syntax 22-2's
  `text_macro_identifier` is an `identifier`, which A.9.3 makes simple OR escaped; 1800-2023 §22.5.1 says it
  outright and gives `` `define \M@CRO3 (a=5, b=0, c="C") … `` / `` `\M@CRO3 ( 1 ) `` as its example). The
  name is read without its backslash — §5.6.1: *"an escaped identifier \cpu3 is treated the same as a
  nonescaped identifier cpu3"* — so `` `define \escaped `` is used as `` `escaped `` or `` `\escaped ``; one
  terminating white-space character is consumed, so a `(` right after it opens the formal list (1800-2023:
  *"only the single white space character terminating the identifier name … shall separate the opening
  parenthesis from the macro name"*); a usage spelled with the backslash is recognised where the run up to
  white space IS a defined name, so the escaped-quote token `` `\`" `` of a macro string keeps its meaning.
  Through `1.0.250` such a `` `define `` was refused as *"missing macro name"* (iverilog
  `escaped_macro_name.v`).
- **A one-line comment inside macro text continues the text when its line ends in backslash-newline, and the
  comment is dropped** — 1800-2023 §22.5.1 says it outright (*"If a one-line comment is followed by a backslash
  and newline character, the comment ends before the backslash and the macro text continues on the next
  line"*), and the 2017 text's two sentences read together say the same; measured licensed under both
  profiles and pinned (`SVPP-EXPANSION.5`(r)). **A backslash followed by white space before the newline does
  NOT continue the macro text** on any profile: 2017 *"preceded by a backslash"*, 2023 *"immediately preceded"*
  — the macro ends at that line and the following lines are source. The corpus writes that shape at two sites,
  both tests (verilator's own `BSSPACE` lint-error test is one); no stage option exists for it.
- **A function-like macro may open its parentheses on the next line** (§22.5.1 requires the parentheses;
  §5.3 lets white space, newlines included, separate the tokens): `` `FOO `` at the end of a line followed by
  `(1, 2)` on the next is one invocation.

Four more landed with `SVPP-EXPANSION.6b.5`, each named by a row of the disposition table:

- **`$display` is one token.** A formal named `display` is not substituted inside the system task name
  (A.9.3 `system_tf_identifier`): `` `define MACRO_D(display) $display(display); `` expands to
  `$display("hi")`, not `$"hi"("hi")`.
- **A block comment opened on an earlier line is comment text** (§5.4), whatever it contains: a `` `else ``
  inside it is not a directive, and the text after an `` `include `` filename belongs to the next logical
  line, so a `/* multi-line` opened there keeps its close.
- **The coverage-control macros are predefined** — IEEE 1800-2017 §40.3.1 says the tool defines
  `` `SV_COV_START `` … `` `SV_COV_PARTIAL `` (fifteen names, the clause's values); a source `` `define ``
  of the same name supersedes.
- **A §34 protected envelope is consumed as opaque.** From the `` `pragma protect `` line that carries
  `begin_protected` to the one that carries `end_protected`, every line is the envelope's — key blocks,
  data blocks, their encoded payload — and none of it reaches the parser; the pragma lines outside the
  envelope pass through as before. An envelope with no `end_protected` is refused.

The same slice found a defect no parser could see: the formal-list scanner treated an apostrophe as
a string quote — SystemVerilog has none (§5.9) — so `` `define DV_FCOV_INSTANTIATE_CG(NAME_, COND_ =
1'b1, ARGS_ = ()) `` had been read as *two* formals since the scanner's first commit, a third actual
dropped without a word and `` new``ARGS_ `` pasted into the legal identifier `newARGS_`. Only a rule
that *refuses* (extra actuals) made it visible; the fix moved four OpenTitan files to `pass`.

**The order around a paste** (`SVPP-EXPANSION.5`(i)). A ``` `` ``` token *"delimits lexical tokens
without introducing white space, allowing identifiers to be constructed"* (§22.5.1), and the identifier
being constructed is very often a **macro name**: UVM's `` `uvm_pack_``TYPE``(ARG, p) ``, OpenTitan's
`` `dv_``SEV_(…) ``, Surelog's `` ```rname``_ADR ``. Two rules decide what such a chain means, and both
are Verilator's measured readings (its own `t_preproc.v` goldens) and sv2v's:

- **An operand that is a defined macro usage is expanded before the join.** `` `A``b```C `` with
  `` `A `` = `a` and `` `C `` = `c` is `abc`; `` XXS_```SOME `` with `` `SOME `` = `some` is
  `XXS_some`; `` `XTYPE``_frame `` with `` `XTYPE `` = `sonet` is the identifier `sonet_frame` — and
  stays one even when a macro named `sonet_frame` exists, because the joined text is not looked up
  again.
- **An undefined usage joined by a paste is looked up as the whole constructed name, in place.**
  `` `QA``_b `` with no `QA` and a `QA_b` is `` `QA_b ``'s expansion; `` `CHOICE_``FOO```SUFFIX `` with
  `` `SUFFIX `` = `_MAGIC` is `` `CHOICE_FOO_MAGIC ``'s — not `` `CHOICE_FOO `` followed by `_MAGIC`,
  which is what the engine produced until this slice (`1_MAGIC`, a token no parser accepts) because the
  substitution step had deleted the pastes before the rescan could see what they joined. A constructed
  function-like usage collects its argument list as written, so an actual's own macro usage is still
  expanded *"only after substitution into the outer macro text"*, the order §22.5.1 gives every actual.

The reference is sv2v's `test/lex/macro_macro.sv`, whose own `macro_macro.v` expects `$display(3, 4)`:

```systemverilog
`define SUFFIX _MAGIC
`define CHOICE_FOO 1
`define CHOICE_FOO_MAGIC 3
`define MACRO2(A, B) `CHOICE_``A```SUFFIX , `CHOICE_``B```SUFFIX
$display(`MACRO2(FOO, BAR));     // 3 , 4 — Verilator 5.050 and sv2v; the engine read 1_MAGIC , 2_MAGIC
```

⚠️ The tools do not all agree here, and the disagreement is recorded rather than smoothed over: Icarus
13.0 reads `1_MAGIC` (it looks the left side up as soon as it is joined) and, for a base name that is
itself a defined macro, `my_prefix_my_suffix` where Verilator reads `` `PREFIX_my_suffix ``. The standard's
own sentence — a paste *constructs* an identifier — is the whole-name reading, which is also the one
sv2v's reference and Verilator's goldens take, so it is the one shipped. Over the corpus the change
moved exactly one expansion of the 188 files that carry a paste at all, the sv2v row above; the record,
with the three-way comparison against both tools re-runnable
(`docs/tasks/artifacts/svpp_expansion/paste_order/probe.sh`), is in `docs/tasks/SVPP-EXPANSION.md`. What
that comparison also shows, and what is deliberately **not** decided by this rule: a macro usage
substituted into a `` `"…`" `` string region is not expanded here while both tools expand it, and a
formal inside a plain `"…"` string is substituted here (as Icarus does) while Verilator keeps the
string verbatim — `SVPP-EXPANSION.5`(j) and (k), owned, next.

**The string region** (`SVPP-EXPANSION.5`(j)). §22.5.1 says what a `` `"…`" `` region is in one sentence: *"the
expansion shall include the quotation mark, substitution of actual arguments, and expansions of embedded
macros"* — three things. The engine did the first two, on one of its two paths: the renderer lived in the
function-macro substitution step, so an OBJECT macro's region was never rendered at all (OpenTitan's
`` `define SSCTRL_PATH `"`SSCTRL_HPATH`" `` reached the parser with its markers on and was refused inside
`{`SSCTRL_PATH, ".", signal}`), and a usage embedded in a region — through an actual (`` `stringify(`name1) ``)
or in the body (`` `"`SSCTRL_HPATH`" ``) — was never expanded because the rescan does not enter a string. The
escaped quote `` `\`" `` was mis-rendered too: the close scan matched its last two bytes, so the clause's own
`` `define msg(x,y) `"x: `\`"y`\`"`" `` came out as `"left side: `\"right side`\""` where the clause prints
`"left side: \"right side\""`. One renderer now does all three, in the rescan, for both paths, and it applies to
expansion text only:

```systemverilog
`define SSCTRL_HPATH tb.dut.u_otbn_core.u_otbn_start_stop_control
`define SSCTRL_PATH  `"`SSCTRL_HPATH`"
return {`SSCTRL_PATH, ".", signal};   // {"tb.dut.u_otbn_core.u_otbn_start_stop_control", ".", signal}
`define msg(x,y) `"x: `\`"y`\`"`"
$display(`msg(left side,right side));  // $display("left side: \"right side\"");  — the clause's example
$display(`"standalone`");              // unchanged: the form is defined for macro text (sv2v string_macro.sv:9 is
                                       //   pinned must-reject on it); Verilator renders it, Icarus leaves it
```

An undefined usage inside a region, or a function-like one with no argument list, stays as written, backtick and
all (Surelog's own golden for its `PreprocString` test keeps the bare `` `message ``). A rendered literal is ONE
line (§5.9): an embedded macro whose body spans lines — OpenTitan's `` `DV_LC_TX_DIST `` inside a stringified
`` `DV_CHECK_STD_RANDOMIZE_WITH_FATAL `` condition — has each newline rendered as a space, which is Verilator's
reading (its `twoline` golden `"first        second"`) and the only one a parser can read; Icarus leaves the raw
newline, and so did this engine for a multi-line actual until now. With the region rendered,
an empty argument list `()` inside one is read as the clause's *"An actual argument may be empty … substituted by
… nothing if no default is specified"* — one empty actual when the macro declares formals — where the engine had
counted zero actuals and refused `` `thru() `` as missing (`.5`(j′)); both tools read it empty. Over the corpus the
change moved 20 of the 483 expansions that carry a region and 7 of the 107 that carry an empty invocation — every
one a rendering, three OpenTitan sequence files and four OpenTitan testbenches now carrying real hierarchical
paths inside their strings, and exactly one verdict: the OpenTitan interface row above. What sv2v's `macro.sv`
does — closing a region with a bare `"` — is sv2v's extension (Verilator refuses the file, the clause says a `"`
*"must have a terminating ""*) and is left as written; the row is a pinned must-reject either way. The three-way
comparison is re-runnable (`docs/tasks/artifacts/svpp_expansion/string_region/probe.sh`).

**A plain string in macro text is opaque** (`SVPP-EXPANSION.5`(k)). The same clause that defines the `` `" `` form
says of a plain one: *"the use of " always starts a string literal and must have a terminating ". Any characters
embedded inside this string literal, including `", become part of the string in the replaced macro text"* — and its
own worked example is `` `define H(x) "Hello, x" ``, which *"will print: Hello, x"*. The engine had been substituting
a formal there, which is Icarus's reading and which MANGLES a message whose prose happens to be a formal's name — the
census read 66 such sites against 6 that rely on it: iverilog's `` `define check(value, expected, error) `` printed
`"  8'h5 %0h, got %0h"` for `"  expected %0h, got %0h"`, OpenTitan's `pwrmgr_base_vseq.sv` printed `"in %0d fast clock
cycles_before_rst_lc_src"`, UVM's field macros warned about `"UVM_ALL_ON|UVM_HEX"` instead of `"FLAG"`. The six
that rely on substitution are design code written for commercial simulators — OpenTitan's
`` "tb.dut.gen_classes[``i``].u_accu.u_prim_count.cnt_q[0]" ``, where the paste tokens MARK the formal — so the two
tool readings are the named values of one policy, with the standard's as the default:

```text
--stage-opt plain_string_substitution=standard          "Hello, x"                         (the clause; Verilator 5.050)
--stage-opt plain_string_substitution=paste_delimited   "tb.dut.gen_classes[0]…" — only a ``formal`` is substituted, prose stays
--stage-opt plain_string_substitution=all               every formal substituted, every paste joined (Icarus 13.0)
```

Over the corpus the default moved 67 of 16 388 expansions and every changed line sits inside a plain string; the
paste-order probe now agrees with Verilator on all fourteen of its rows. ⚠️ A consumer feeding OpenTitan-style DV code
names `paste_delimited`; nobody is served by `all` except a reproduction of Icarus, which is why it is spelled out
rather than left as an unnamed default.

**`` `begin_keywords `` regions are rendered, not ignored** (`SVPP-EXPANSION.8`). IEEE 1800-2017 §22.14
lets a block of source select *which identifiers are reserved as keywords*, from a version of IEEE 1364
or IEEE 1800: inside `` `begin_keywords "1364-2001" `` … `` `end_keywords ``, `logic` is an ordinary
name. A generated parser reserves one set per PROFILE and cannot switch mid-file, so the expansion stage
renders the difference as TEXT — every token the parse profile reserves and the region's version does not
is re-emitted as an ESCAPED identifier:

```systemverilog
`begin_keywords "1364-2001"     // the source, as its author wrote it
module top;
  reg [63:0] logic;             // OK: `logic` is not a keyword in 1364-2001
endmodule
`end_keywords
```

```systemverilog
`begin_keywords "1364-2001"     // what the parser is handed
module top;
  reg [63:0] \logic ;           // §5.6.2: "A SystemVerilog keyword preceded by an escape
endmodule                       //  character is not interpreted as a keyword"
`end_keywords
```

§5.6.1 makes the two spellings the same identifier (*"an escaped identifier `\cpu3` is treated the same
as a nonescaped identifier `cpu3`"*), so nothing about the design changes — only which words the parser
may bind as names. What decides the rendering:

- **The profile.** `--profile` reaches this stage on both commands (`ast_pipeline --preprocess
  systemverilog --profile P`; `--sv-keyword-profile` accepted through `1.0.247`); under `verilog_2005` the parser never reserved `logic`, so
  the same region renders nothing. Omitted, the parser's unprofiled reading is used, which reserves
  IEEE 1800-2017 Table B.1.
- **The version tables are the LRM's own** — Table 22-1 … Table 22-6 and §22.14.4's
  "1364-2001-noconfig" exclusion list — derived from the in-repo standard text and held equal to it on
  every commit by the `SV-KEYWORD-VERSIONS` doctrine, never typed by hand.
- **Regions nest** (*"Each nested pair is stacked"*) and run *"even across source code file boundaries,
  until the matching `` `end_keywords `` directive or the end of the compilation unit"*, so the stack is
  carried between the files of a `--unit-file` unit exactly as the macro table is. sv2v's
  `test/lex/begin_keywords.sv` turns on exactly this: inside a `"1364-2005"` region nested in a
  `"1364-1995"` one, `task automatic bar;` keeps `automatic` as a keyword while `logic` becomes a name.
- **It is lexical.** A keyword named in a comment, a string literal, a numeric literal, a `$`-name or an
  escaped identifier is not a token of the design and is left exactly as written; the source map is
  re-indexed by the exact byte delta of each rendering, so a rejection still names the line its author
  wrote.
- **What it cannot do it reports.** Text can unreserve a keyword; it cannot reserve a non-keyword. A
  region naming a version that reserves MORE than the profile raises
  `W_SVPP_KEYWORDS_VERSION_ABOVE_PROFILE` and is applied only in the direction text can express; one
  naming a specifier the profile's own edition does not define raises
  `W_SVPP_KEYWORDS_VERSION_NOT_IN_EDITION` (the parser then refuses the directive); an unpaired
  `` `end_keywords `` raises `W_SVPP_UNMATCHED_END_KEYWORDS`; and a specifier no version of the standard
  defines is `E_SVPP_UNKNOWN_KEYWORDS_VERSION` — §22.14's own *"It shall be an error if an
  implementation does not recognize the version_specifier"*.

The event log records each directive (`keywords_region`) with its specifier, how many words the region
unreserves and how many tokens it actually rendered.

**The deferred population, re-read.** The raw answer key is honest about what a one-file parse cannot
decide: a must-accept file whose raw parse stops on a macro use, a conditional or an `` `include `` is
*explained* by a position test, and a multi-file design file carries no verdict at all. The tracked
`stimuli/sv/characterization/pipeline_readjudication.md` (with its per-row `.tsv`; `SVPP-EXPANSION.6a`) joins
those rows with what the pipeline actually did to them. Of the 1 434 explained-away must-accept rows,
**1 199 parse once preprocessed** — the explanation held and the row now testifies for the parser — while
**223 still fail in their own text**: the explanation was wrong, and each is a defect candidate awaiting a
disposition (103 of them are a class header whose base class lives in `uvm_pkg`). Of the 4 927 chained-only
rows, 1 530 are accepted under their derived context, 2 936 still reject in their own file (1 924 on a
`class … extends` header whose base is declared elsewhere — the cross-file fact the library flags exist
for), 236 land in an included file and 225 are refused. A `chained:` outcome is not yet a `match`: the
answer key holds no expected verdict for those rows, and one is derived from the suite's own convention
before the coverage tuple moves. A refuted row keeps that label until someone has looked at it and written down what it is: the tracked
`stimuli/sv/characterization/refuted_dispositions.tsv` carries one line per row looked at — the disposition
and its evidence, in the answer key's own style — and the report splits the refuted rows by it. The
dispositions are a closed vocabulary: the base class lives in another file (the library design's rows), the
file's portable branch is a vendor's own error stub behind its identity macro, a macro the suite's driver
defines on its command line, a macro a predecessor file of the unit defines, a parser question routed to the
corpus tree (legal-looking text the parser refuses, with the clause), a key question (the parser is right to
refuse and the row's expectation is what needs a ruling), an expansion-stage rule not yet implemented. A
disposition for a row that has since moved is refused, so the table cannot carry a stale claim. Every refuted
row carries one; the report's `refuted_awaiting_disposition` is the count still to be looked at.

**The rows the raw key deferred to this lane by design (`.6c.3`, 2026-10-06).** The raw answer key files a
sv-tests test whose header says `:type: preprocessing` as `deferred:svpp_owned` — *"conformance owned by
the preprocessor lane"* — and the preprocess→parse lane IS that lane. Its re-adjudication now reads those
rows' expectation the way the raw key reads every other sv-tests row, through the key's own function in
its pipeline mode: a `:should_fail_because:` header means must_reject, its absence means must_accept. Of
the 94 such rows on the 2026-10-06 reading, **86 positive tests parse once preprocessed** (`svpp_owned:
corroborated`), 4 should-fail tests are refused by the preprocessor with a clause-cited code (`E_SVPP_*`:
the rejection the header expects, at the expansion stage), 2 are rejected in their own text as expected —
and **2 should-fail tests the pipeline ACCEPTS**: the standard's own illegal example of a macro text split
across a string literal (`22.5.1--define-expansion_21.sv`, §22.5.1 *"shall not be split across the
following lexical tokens: … String literals"*) and the redefinition of a compiler directive as a macro name
(`22.5.1--define-expansion_23.sv`, `` `define define "illegal" ``, §22.5.1 *"it shall be illegal to redefine
a compiler directive as a macro name"*). Both are over-acceptances of the preprocessor, on the bar under
the pipeline (its fifth number reads 2) until `SVPP-EXPANSION.5`(s) and (t) land. The other suites'
svpp-owned rows (22 accepted, 28 rejected, 25 refused) have no header convention derived yet and keep no
verdict.

**The file's own configuration (`.4c.1`).** A test suite's driver does not run a file bare: verilator's
`t_x.py` passes `+define+NAME[=VALUE]` / `-DNAME[=VALUE]`, sv-tests' header carries `:defines:`. The
derived context now includes them per file (`stimuli/sv/pipeline_context_defines.tsv`, 55 rows on 48 files:
52 verilator, 3 sv-tests — a name the same command line `-U`s again is not one of them), applied as
`--symbol` before the suite's rows. The deriver emits a LITERAL
value only and reports the rest instead of guessing — 3 files whose driver computes the value at run time
(`"+define+SIM_CYCLES=" + str(test.cycles)`), 24 files that several variant drivers configure differently
(`t_time_sc_ns.py` … `t_time_sc_sec.py`), and it skips the spans whose `-D` reaches the C++ compiler or
iverilog (`-CFLAGS`, `make_flags=[CPPFLAGS_ADD=…]`, `iv_flags`) and the `*_common.py` helpers that are not
drivers — the first, permissive scan had emitted 94 rows, 38 of them C++ defines or a helper's
conditional flags; the strict scan emits 56. Re-run on the whole lane, exactly 5 rows moved, all among the
files with defines: 3 `fail → pass` (sv-tests `preprocessor-macro_0/1.sv` under `TEST_VAR` / `VAR_1=2 VAR_2=5`,
verilator `t_assert_inside_cond.v` under `T_ASSERT_INSIDE_COND`) and 2 `pass → fail`. 🔎 **A driver's define
can select a vendor-extension branch**: `t_vpi_module.v` and `t_vpi_public_depth.v` are standard
SystemVerilog only WITHOUT their driver's `+define+USE_DOLLAR_C32` — with it, the selected branch is
`` `systemc_header `` / `extern "C" int mon_check();` / `` `verilog ``, Verilator's directives (not among
clause 22's) around C++ text no production derives; the row is right to reject under its own configuration
and is pinned `must_reject` with a reproducer/control measured on both profiles, as is `t_gen_missing.v`,
whose define selects the branch above its `` `error `` stub and exposes a bare `begin … end` inside a
generate region (the `.3.56` ruling, A.4.2) — a row the `driver_define` disposition had been hiding. Six
`driver_define` rows remain, each now naming why its define cannot be derived (computed, or a tie).

**The file's own unit (`.4c.2`).** A harness also decides what is compiled BEFORE a file in the same
compilation unit, and §22.5.1 makes a macro unit-wide once defined. The derived context now carries those
predecessors per file (`stimuli/sv/pipeline_context_units.tsv`, 44 rows on 17 files), applied as
`--unit-file` in order, from three declaration dialects: verilator's source-file arguments in the
driver's SV flag lists (the harness puts them before the top file), slang's `tests/unittests/data/*.f`
command files (`-F` expanded, `-v` library files skipped, the units unioned in declaration order), sv2v's
`test/*/run.sh` explicit multi-file `convert` / `runAndCapture` lines (sv2v lexes its inputs as one unit
unless `--siloed`). A `-f` list (Verilator's own option-file syntax), a glob, a `--siloed` run and a pair two
declarations order differently are reported, not applied. Re-run on the whole lane, exactly 3 rows moved,
all `fail → pass`, all among the files with predecessors, none the other way: sv2v `siloed/module.sv` after
`package.svh`, slang `test2.sv` after `test.sv`, verilator `t_hier_block_import.v` after its `_def.vh`. The
fourth `cross_file_define` row stays: 🔎 **its `.6b.4` disposition had inferred a pairing from the file
names** (`file_uses_define_in_file_with_no_eol.sv` after `file_with_no_eol.sv`) — no declaration in the
vendored slang tree makes it, so the deriver reports nothing for it; hand-given, the pair accepts on both
profiles, and the row's basis now says exactly that.

**A suite's build-wide symbol (`.6d.4`).** One kind of define no driver carries per file: the symbol a
whole flow compiles with. OpenTitan's dvsim passes `+define+UVM` to every DV build
(`hw/dv/tools/dvsim/common_sim_cfg.hjson:65`), and three DV files guard their `include "uvm_macros.svh"` on
it — without the symbol `dv_utils_pkg.sv` left every `` `uvm_* `` and `` `DV_* `` macro below the guard
unexpanded and rejected at `:215:24` on a raw `` `uvm_fatal ``, with 36 packages importing it rejecting
behind it. The deriver reserves a hand-curated row kind for exactly this (`HAND_DEFINES`, one row
`opentitan define UVM hand <source>` in `stimuli/sv/pipeline_context.tsv`, the source cited), applied as
`--symbol UVM` to every OpenTitan row. Re-run on the whole lane: ten OpenTitan packages `fail → pass`, zero
the other way, and the library pass publishes 264 packages (was 256).

**A suite's unit predecessors (`.6d.6`).** IEEE 1800-2017 §22.5.1 makes a macro unit-wide once defined, and
OpenTitan's dvsim builds one unit: `{UVM_HOME}/src/uvm_pkg.sv` first (it includes `uvm_macros.svh`), then
`dv_utils_pkg.sv` (which includes `dv_macros.svh`), the RTL (`prim_assert.sv`) and only then the DV env and
test packages — which therefore use `` `uvm_object_utils ``, `` `DV_CHECK_* ``, `` `gfn ``, `` `ASSERT_I ``
without including anything. Parsed one file at a time, 15 of the 38 packages still rejecting after `.6d.4`
stopped at exactly such a raw macro call. The deriver now carries hand-curated suite-wide unit
predecessors (`HAND_UNITS`, three rows `opentitan unit_file <path> hand <source>` in
`stimuli/sv/pipeline_context.tsv`), applied as `--unit-file` in order to every OpenTitan row — about
0.37 s per row. Re-run on the whole lane: 195 OpenTitan rows `fail → pass`, zero the other way, zero outside
OpenTitan; the library publishes 273 packages (368 of OpenTitan's 396); the members read through their unit
`accepted_in_unit` 999 → 1 287 and `unreached_in_unit` 846 → 566; the tuple under the pipeline
`11053/2606/2677/1391/0`. The membership table shrank from 5 130 to 2 392 rows while naming the same 2 058
files: the rows that went were macro headers a predecessor had already brought into the unit, whose guarded
`` `include `` in the package now expands to nothing.

**The library pass (`.6d.1`).** A design is not a bag of files. A file of OpenTitan, black-parrot,
VeeR, scr1, friscv or UVM imports typedefs, classes and parameters that a *package* in another file
declares, and the parser only admits such a name where its grammar is fact-gated (a typedef name as a port
type, a class name after `extends`, a parameter name in a range — the store gate of `SV-CORPUS-GRAD.13w`)
once the package's facts are in scope. The grammar already exports and imports those facts
(`@export_to_library` on `package_declaration`, `@import_from_library` on `package_import_item`;
`--lib-out DIR` / `--lib-in DIR`). The lane now builds that library before any per-file row runs
(`stimuli/sv/pipeline_library.py`): every package file of the design suites, its `import X::` edges read
from its preprocessed text, a topological order, each file parsed with its own derived context, the library
so far as `--lib-in` and a private `--lib-out`, and a passing file's exports published — at once for a
package one file declares, and for a package several files declare (`prim_pkg` has four, one per vendor)
only when every declaring file passed and their fact sets agree; a disagreement is reported, not resolved
by guessing which variant the build meant. A rejecting file exports nothing: the engine writes no facts
for a rejected parse, so every package importing it stays dark on its names, and the report's *"The
library the lane built"* table says how many. Then every row runs with `--lib-in` of the result.

Since `1.0.247` the UVM library is in it (`SVPP-EXPANSION.6d.3`): `uvm_pkg.sv` exports `uvm_pkg` and
`uvm_polling_pkg`, `uvm_compat_pkg.sv` exports beside it, and 142 of the 168 class files `uvm_pkg.sv`
`include`s are accepted in their unit (none were before). The OpenTitan DV packages that import `uvm_pkg`
had mostly been accepted earlier, by the fact-gate escapes that admit a `uvm_*` name without its facts;
48 still rejected at their own first barrier — an unexpanded `` `uvm_fatal `` behind an `` `ifdef UVM ``
the derived context did not satisfy. With the `UVM` symbol in OpenTitan's context (`SVPP-EXPANSION.6d.4`,
below) ten of them pass and 121 more members are accepted in their unit; with the three macro headers the
flow compiles first as unit predecessors (`.6d.6`, below) 13 more packages and 195 rows pass. The 25 that
remain are three populations, each named per row: 8 stop at a construct 1800-2023 admits and 1800-2017 does
not (unsized parameter arrays, an indexed event trigger — `SV-CORPUS-GRAD.38`, a 2023 reading of the
pipeline), 8 at a per-IP build define whose value differs between the IP's simulation configurations
(`SVPP-EXPANSION.6d.5`, parked: a variant disagreement is reported, not guessed), and 9 at a grammar
construct past every macro — `randomize() with { … local::x … }` (admitted by `1.0.248`, `SV-CORPUS-GRAD.39`: a
`local::`-qualified method receiver), a lifetime in a method prototype (`extern function automatic`,
`pure virtual task automatic`), a hierarchical name as a `cross` item, a macro argument carrying a line
comment into its stringized text — the SV lane's work. Since then `1.0.248` and `SVPP-EXPANSION.5`(l) took
three packages out of that last group (`i2c_agent_pkg`, `spi_agent_pkg`, `alert_esc_agent_pkg`); 22 remain: the 8
for the 2023 reading, 9 behind per-IP defines (`core_ibex_test_pkg` joined them: past its comment it stops at
`` 32'h`BOOT_ADDR ``), and 5 grammar constructs.

**The lane has a second reading, under `sv_2023`** (`SV-CORPUS-GRAD.38`, 2026-10-02). A *reading* is one run of
the whole lane — the library pass and every row — under one edition profile.
`PGEN_PIPELINE_PROFILE=sv_2023 stimuli/run_pipeline_lane.sh all` produces the second one into `_sv2023`-suffixed
dumps, so the two never share a path, and every run now leaves a *provenance sidecar* beside its dump (the
profile, the binary's digest, the dumps' digests) — the record of which run wrote it. The two-edition census
(`docs/tasks/artifacts/sv_corpus_grad/pipeline_sv2023_lane/census.md`) joins the readings per file and refuses
unless both came from one binary. Measured: 40 rows pass under 2023 only (`2023_only_pass`) and none under 2017
only (`2017_only_pass = 0`: the newer edition loses nothing the older accepts, now measured through the
preprocessor and the library); 3 more package files pass — the three `rstmgr_env_pkg.sv`, whose
`parameter string LIST_OF_LEAFS[] = {` is 1800-2023's unsized parameter dimension — and 51 member files the
2017 reading leaves unreached are reached behind them. **The primary reading stays `sv_2017`**: the
verdict-coverage tuple is defined over it and did not move. Of the 8 packages that stop at a 2023 construct
under 2017, only those 3 pass; the other 5 clear the construct and meet a second barrier — `foreach
(prv_read[i][j])` (three files), `solve … before` inside a `foreach` constraint, and one more per-IP define.
Under the 2023 reading 19 OpenTitan packages still rejected. Read one by one (`SV-CORPUS-GRAD.41`): 10 behind
per-IP defines, 1 behind a per-target include (riscv-dv's `riscv_core_setting.sv`), 1 behind a macro header the
build makes unit-wide (`cip_macros.svh`), and 7 at a **tool tolerance**.

**The fourth unit predecessor (`SVPP-EXPANSION.6d.7`, 2026-10-02).** `cip_macros.svh` is a guarded, macro-only
header that `cip_base_pkg.sv` includes INSIDE the package every CIP env imports; OpenTitan's build compiles that
package first, so `` `DV_MUBI8_DIST `` and the header's other macros are unit-wide there and were a raw backtick in a
per-file parse. With the header as a sourced suite-wide predecessor, 38 OpenTitan DV rows moved `fail → pass`
under BOTH readings (37 of them testbench tops, `tb.sv`) and none the other way; `csrng_env_pkg` passes and publishes.
OpenTitan package files still failing: 21 under `sv_2017`, 18 under `sv_2023`. Five testbench tops advanced
without passing — each now stops at a per-IP define (`` `EN_MASKING ``, `` `SRAM_TYPE ``, `` `INSTR_EXEC ``,
`` `SEC_VOLATILE_RAW_UNLOCK_EN ``).

**Tool tolerances: five spellings the big simulators accept and this parser rejects, under both editions.** Each
was asked of the standard's TEXT, not only of Annex A, and each has a reproducer pin that must keep rejecting
(`stimuli/sv/adjudication_repros/`), so a widening can only happen as a decision. All five are accepted by the
opt-in profile below — the first four since `1.0.249`, the fifth since `1.0.250` (`SV-CORPUS-GRAD.45`):

| spelling | where OpenTitan writes it | what the standard says | corpus files |
|---|---|---|---:|
| `foreach (a[i][j])` — a bracket pair before the loop-variable bracket | `flash_otf_read_entry.sv:129` | §12.7.3: the argument is *"an identifier that designates any type of array followed by a comma-separated list of loop variables enclosed in square brackets"*; write `foreach (a[i, j])` | 21 |
| `solve y before x[i];` inside a `foreach` constraint | `adc_ctrl_env_var_filter_cfg.sv:64` | A.1.10: `solve … before` is a constraint BLOCK item; a `foreach` holds constraint expressions | 5 |
| `cross cp_a, cfg.vif.sig;` — a dotted name as a cross item | `sysrst_ctrl_in_out_inverted_vseq.sv:49` | §19.6: *"Expressions cannot be used directly in a cross; a coverage point must be explicitly defined first"* | 3 |
| `pure virtual task automatic t();` / `extern function automatic T f();` — a lifetime in a prototype | `sec_cm_base_if_proxy.sv:21`, `tl_seq_item.sv:190` | A.2.6 / A.2.7 prototypes have no lifetime; §8.24 lists what a prototype declares, and §8.6 makes every class method automatic already | 2 |
| `binsof(cfg.vif.sig)` — a name of THREE or more parts inside `binsof(…)` (a two-part name is `cover_point.bin` and parses) | `sysrst_ctrl_in_out_inverted_vseq.sv:60` | §19.6.1: `binsof` *"yields the bins of its expression, which can be either a coverage point … or a coverage point bin"*; Syntax 19-4 `bins_expression ::= variable_identifier \| cover_point_identifier [ . bin_identifier ]`; declare a coverpoint for the variable and name it | 4 |

The corpus counts are a text scan (`docs/tasks/artifacts/sv_corpus_grad/dialect_constructs/scan.py`). A file that
uses one of these is rejected at that line under `sv_2017` and `sv_2023` alike.

**Since `1.0.249` there is an opt-in profile that accepts exactly these spellings: `sv_2023_tolerant`**
(`SV-CORPUS-GRAD.43`, by director decision; the fifth row joined in `1.0.250` by a call the director delegated,
`SV-CORPUS-GRAD.45` — it is the reference form of the third row, the only way a select expression can name a dotted
cross item). It is `sv_2023` plus the five spellings and nothing else; the edition
profiles are unchanged and remain the default. Ask for it by name — `--profile sv_2023_tolerant` on the probe,
`PGEN_PIPELINE_PROFILE=sv_2023_tolerant` for a reading of the lane — and each tolerated spelling comes back under
its own AST `kind` ([The Json Carrier](json-carrier.md), *The Opt-In Tolerance Profile*). Measured on release `1.0.250`'s own reading (`PGEN_PIPELINE_PROFILE=sv_2023_tolerant`, the census at
`docs/tasks/artifacts/sv_corpus_grad/pipeline_sv2023_tolerant_lane/census.md`): 18 rows and 7 OpenTitan package files
pass under the tolerant profile that `sv_2023` rejects (`sec_cm_pkg`, `tl_agent_pkg`, `adc_ctrl_env_pkg`,
`sysrst_ctrl_env_pkg`, `flash_ctrl_env_pkg` on three tops), five more packages publish (283 against 278), 337 of the
498 members unreached under the primary reading are reached, and nothing `sv_2023` accepts is lost
(`2023_only_pass = 0`). The fifth spelling moved exactly three rows when it joined in `1.0.250` —
`sysrst_ctrl_env_pkg.sv` and the two ibex coverage interfaces `core_ibex_fcov_if.sv` / `core_ibex_pmp_fcov_if.sv` —
the three files named before the measurement, and nothing else under any reading. Under the tolerant reading 11
OpenTitan package files still reject: 10 behind per-IP defines and 1 behind a per-target include — context the
corpus does not supply, not syntax.

What the preprocessor REFUSES on the primary reading is a census too, not a residue (`SVPP-EXPANSION.4d`,
2026-10-06, the lane's own reason column): 263 (268 before the slice) rows. 168 of them are black-parrot files
whose first `` `include `` names BaseJump STL (`bsg_defines.sv`, `bsg_noc_links.svh`, `bsg_cache.svh`), a library the
corpus does not vendor — honestly absent on the volume, and left so. The rest are top-specific files OpenTitan
and HardFloat select with a build target (`riscv_core_setting.sv` in twelve copies of eleven texts; the `xbar`
autogen pair; `HardFloat_specialize.vi` in three texts, equidistant from every `source/*.v` that includes it),
build outputs, and error tests by name — unbalanced conditionals, macro-arity and recursion tests, the four
include loops, two unquoted include paths. The derivation that supplies the context
(`stimuli/sv/pipeline_context.py`) was itself wrong in two places the census exposed: its include-candidate
population was the source files only (`.sv` / `.svh` / `.v` / `.vh`), so a `.vi` or `.vams` header the tree held
exactly once was reported *absent* (IEEE 1800-2017 §22.4 constrains neither the filename nor its suffix); and
a tie between copies of one text was reported *ambiguous*, when the choice cannot change the expansion. Both
are fixed — every file is a candidate, an identical tie resolves per file to the nearest copy — and the fix
came with the guard it needed: a directory the derivation would emit that also holds an AMBIGUOUS name would
let the preprocessor resolve that name by the side door, so a suite directory in that position refuses the
derivation and a per-file one is withheld for its includer. The guard fired on its first real run, on the fix's
own first cut (OpenTitan's darjeeling `pwrmgr` sequence library, one text for `pwrmgr_vseq_list.sv`, holds a
`pwrmgr_base_vseq.sv` that differs per top), and the rule was redesigned before anything was measured.
Measured on the three readings after the fix: five rows left the refused class under each — two HardFloat `specialize.v` files and two iverilog `.vams` tests now parse, the friscv testbench now reaches the parser and stops there — none moved the other way, and twenty HardFloat `source` rows now stop at the honest three-way tie behind the header the fix resolved. The tuple under the pipeline reads `11173/2606/2557/1270/0`.

Measured on the whole lane (the same debug probe, 689 s wall): **425 rows `fail → pass` and none the other
way** — OpenTitan 1 077 → 1 444 accepted of 3 983, VeeR 23 → 81 of 102 (its 58 rows had all stopped in
`el2_param.vh:182`, a typedef of `el2_pkg`). The library: 407 package files, 230 accepted, 170 rejected,
7 refused by the preprocessor, 166 packages published (31 of them declared by several files whose exports
agree). What still rejects is measured, not hidden: 🔎 **164 of OpenTitan's 168 rejecting packages import
`uvm_pkg` directly** — and `uvm_pkg.sv` rejects at `uvm_transaction.svh:443` on the `const local` qualifier
order the standard does not derive (the director's `.5`(d) call), so it exports nothing and the whole DV
chain (`cip_base_test`, `dv_base_env_cfg`, …) waits on that one ruling; 🔎 **six register packages are
declared per top and DISAGREE** (`clkmgr_reg_pkg`, `gpio_reg_pkg`, `otp_ctrl_reg_pkg`, `ast_reg_pkg`,
`ibex_pmp_reset_pkg`, `pwrmgr_pkg`: earlgrey and darjeeling have different register maps), so they are not
published and the two RTL packages that need one (`otp_ctrl_part_pkg.sv`, both tops) still reject — the
build declaration (`.core`) picks the variant, and reading it stays priced, not built; 🔎 **44 imported
packages are declared by no file in the tree** (`*_ral_pkg`, the generated register-abstraction packages
OpenTitan's flow emits at build time).

**The unit-membership reading (`.6d.2`).** An `` `include ``d class file has no `import` of its own, so
per file it rejects on its first cross-file name whatever the library holds — yet the library pass
parsed every one of its bytes inside its package unit. The pass now records which files each unit's
preprocessed text came from (the preprocessor's source map), and the re-adjudication reads a member's
verdict *through its unit*: `accepted_in_unit` when a unit including it passes, the per-file verdict
kept when the unit's rejection lands in this very file, `unreached_in_unit` when every unit including
it rejects elsewhere — the member's bytes were never judged, and the unit's origin is named rather than
the member's first cross-file name. An accepted member counts as adjudicated in the design suites,
exactly as a file accepted under its context does; an unreached member stays dark, with its reason.
Measured: 2 060 distinct member files across the design suites (OpenTitan 1 906); of the 2 059 members the
answer key routes, **none is accepted in a unit today and 1 923 are unreached** (OpenTitan 1 783, UVM 140),
89 are the file their unit's rejection lands in. 🔎 The reason is one ruling: every OpenTitan and UVM
package that `` `include ``s class files imports `uvm_pkg`, and `uvm_pkg.sv` exports nothing until the
`const local` question is ruled — the reading is in place and idle, and the same run converts the 1 923
the day the ruling lands. The dark population is now named by its unit's origin (`clkmgr_base_test.sv:5`,
`uvm_transaction.svh:443`) rather than by each member's first cross-file name.

**The denominator, read twice.** The platform book's verdict-coverage tuple
(`adjudicated/routed/no-verdict/dark/axis-2-bar`) now carries two readings of the same rows, published
together and held by the same gate: the raw lane's `7486/2606/6244/4037/9`, and `9135/2606/4595/3312/14`
under the pipeline. The second is the first with the re-adjudication table applied: 1 218 explained-away
rows corroborated, 1 704 design-corpus files accepted under their derived context and library (must-accept by the
suite's own convention — a shipping design compiles), 448 must-reject rows rejected where the raw lane
never reached the construct (28 of them the key questions of the disposition table, pinned with the
clause and a reproducer each, 3 the rows a file's own driver define reaches — `.4c.1`, and 11 the pipeline-routed parser questions re-read at the furthest token — `SV-CORPUS-GRAD.12b.1`), and 5 must-accept rows still refused in their own text (three of `.12b.1`'s four grammar families shipped: the spaced `@ ( * )` as `1.0.228` — nine pipeline rows `fail → pass` — and the `$`-indexed receiver with the sv_2023 `+%-` range as `1.0.229`, whose streaming-slice row proved the standard's own refusal and is pinned) — the bar's
members, each with its clause, until the corpus tree rules them. 2 935 rows are dark under the pipeline
on a declaration another file holds; that is the library design's population. The report also re-reads every row the key expects **rejected** — including
a pin the raw lane could not cash because its parse stopped on a macro use above the construct: 34 verilator
rows (a typedef reaching a type through two or more hierarchical steps, a type named through an interface
instance in a type assignment, an enum with a range but no base type) reject at the pinned line under the
pipeline on both profiles, and no row the standard cannot derive is accepted beyond the two the raw lane
already knew. The report is held to a fresh derivation on every commit, like the lane's.

⛔ **A note on the host, not the engine.** The reading above was taken on the debug probe because no
release build of the crate could be completed on the development host that day: a per-process
memory watchdog independent of the repository's own guard kills any process above 14 GB, and the
release build's compile now peaks just past it (the operations chapter of the platform book records
the numbers). A build profile cannot change a verdict; the deadline was widened to 600 s so the
slower probe could not manufacture a `timeout` (the slowest non-timeout row of the release reading
was 5.75 s), and the identity block of the tracked report names the debug binary.

Two thirds of the residue was one fact when that reading was taken: the base class of a `class …
extends` lives in another file of the compilation unit (a package the row `import`s, or a sibling
`.sv`), and the parser's store gate refuses a base it has not seen declared — the strict reading
`SV-PARSE-STRICT.2` escaped for net types and not for class bases. That is a cross-file DECLARATION
barrier, exactly the one the library flags (`--lib-out` / `--lib-in`) exist for.

**Since `SVPP-EXPANSION.6e` (2026-10-06) the residue is a census, derived and tracked** —
`docs/tasks/artifacts/sv_corpus_grad/rejects_own_file_census/census.md`, re-derived after every lane
reading, its identity held by the `PIPELINE-LANE-CURRENCY` doctrine's fourth arm. Every row the
re-adjudication files as *rejected in its own file* is re-expanded under the lane's own context and
classed by what the parser actually SAW, not by the source line the stop maps back to. On the
reading of 2026-10-06 the 310 rows fall as follows: 143 are the base-class barrier above (126 of
them files only ever read through the package that `include`s them); **47 stop at a raw backtick in
the expanded text — an undefined macro reached the parser, a CONTEXT gap that happened one stage
late** (OpenTitan's per-IP build defines such as `EN_MASKING` and `RNG_BUS_WIDTH`, which live in the
IPs' `*_sim_cfg.hjson` build options; VeeR's `RV_*` configuration macros, whose `common_defines.vh`
the VeeR Makefile compiles first — supplied since `.6d.8`, three rows now pass; two BlackParrot and one friscv header chain); 55 are `include`
fragments of a module, testbench or package whose standalone reading is not a claim about the
parser (riscv-dv's `riscv_core_setting.sv`, the `xbar` autogen pair, VeeR's `el2_dec_csr_equ_*.svh`);
11 are the same declaration barrier in data position (`mubi4_t sel_mux;` — the type is declared in
a file the row did not see); 7 are the 2023 text's (a parameter's unsized `[]`, which 1800-2023's
`param_assignment` admits and 1800-2017's does not; `` `begin_keywords "1800-2023" ``) and 5 the
tolerant profile's; 16 are error tests by name, verilator preprocessor tests whose own driver runs
`-E`, and verible module-body fragments that declare themselves so. The 26 that remain are
adjudicated one by one in `SVPP-EXPANSION.6e`: invalid SystemVerilog the standard does not derive
(`a ~& b`, a `modport` inside a `generate`, a trailing comma in a parameter list, a `do` port name
under an SV profile, an `if` arm of two statements without `begin`/`end`, a module named after a
synthesis cell `$_DLATCH_P_`), three more cross-file declaration barriers the mechanical shape test
could not see, three non-SystemVerilog test inputs, one preprocessor question routed to `.5`(r) (a
`` `define `` whose `// comment` line ends in a backslash), and five spellings the ecosystem accepts
that no edition derives — `$fatal("…")` without its finish number, `#(.Param)` without
parentheses, `static task` before the keyword outside a class, a size with a leading zero
(`01'b0`), `#a.b` as a delay — recorded as candidates for the opt-in tolerance profile and not
admitted.

- The seven curated real-design cases of the external-corpus triage gate (UVM's two packages,
  scr1, friscv, VeeR) run through the single command with the manifest's include directories:
  the expansion of `uvm_pkg.sv` (166 included files) matched the shell gate's byte for byte until
  `SVPP-EXPANSION.6b.5`, which found the gate's reference (3 022 806 bytes) wrong in 98 lines — all
  inside block comments: the preprocessor of 2026-08-24 had honoured `` `define `` lines inside a
  `/* … */` (`uvm_tlm_imps.svh` keeps older macro versions in one) and expanded macro uses inside
  others. The expansion is now 3 020 226 bytes, every changed hunk audited inside a comment, and
  scr1 / friscv / VeeR accept on both profiles.
- UVM's `uvm_pkg.sv` — the unit-level reading of that suite — **parses on both profiles since
  `1.0.247`** (`SV-CORPUS-GRAD.37`, ledger `SV-0132`, 2026-10-01). From the 2026-09-04 release that
  made `local` a reserved word until then it rejected at `src/base/uvm_transaction.svh:443`,
  `const local uvm_event_pool events = new("events");`: IEEE 1800-2017 A.1.9 gives a `const`-first
  class property only `const { class_item_qualifier } data_type const_identifier [ = constant_expression ] ;`,
  and `new(…)` is not a `constant_expression` (the `local const …` order is derivable through
  `data_declaration`, which admits a `class_new` initializer). Every commercial simulator accepts the
  UVM spelling, and §8.19's prose describes it as a constant with an initial value. **Recorded
  deviation (tool extension, on by default):** `class_property`'s const arm admits a `class_new`
  initializer — `( assign ( class_new | constant_expression ) )?` — under the director's standing
  ruling that a change must not make the parser reject what the ecosystem writes; the typed shape is
  the one the arm always declared. The policy record: `docs/tasks/SVPP-EXPANSION.md` (`.5`(d), closed).

## Boundaries

- The expansion engine is hand-written Rust (`rust/src/families/systemverilog/preprocessor.rs` — the
  SystemVerilog family's own component since `ENGINE-UNIVERSAL-SERVICES.113`, published to the engine as one
  neutral preprocessing stage, `rust/src/preprocessing_stage.rs`, which knows nothing of the backtick), not a
  grammar; its
  *directive* structure is separately described by the `systemverilog_preprocessor` grammar, whose
  parser does not expand anything. The two are different deliverables and this chapter is about
  the first.
- The pipeline preprocesses one compilation unit at a time and parses its last file; a
  multi-file design still needs the library flags for cross-file declarations.
- Nothing yet reads a build system's own context (`.core`, `.f`, `+incdir+`); on the command line
  the context is what you pass, and in the corpus lane it is derived from the corpus — a shared
  directory per suite, the nearest candidate per file for a name that exists in several, and a
  reported tie where the tree cannot decide (a tie between copies of one text is not one); and a
  directory that would resolve an ambiguous name by the side door is never emitted (`.4d`).

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

The pipeline lane (`stimuli/run_pipeline_lane.sh`, joined against the raw lane by
`stimuli/sv/pipeline_vs_raw.py`; the tracked report is
`stimuli/sv/characterization/pipeline_characterization.md`, whose identity block names the probe,
grammar, parser, context and dumps every number below depends on) parses every source file of a
corpus suite through the chain under a context DERIVED from the corpus itself
(`stimuli/sv/pipeline_context.py`: for every `` `include `` an includer's own directory does not
satisfy, the one directory that does; an ambiguous name is reported, never guessed — and, since
`.4c.1`, the defines a file's own driver passes for it: verilator's `t_x.py` flags and sv-tests'
`:defines:` header, a literal value only, applied per file as `--sv-define`; a value the driver
computes at run time and a file several variant drivers configure differently are reported, not
guessed. These are the test's own configuration, not a tool identity: `.6b.3` measured and declined
the identity predefines — and, since `.4c.2`, the unit predecessors a file's own harness compiles
before it, applied as `--sv-unit-file` in order: verilator's source-file arguments, slang's `.f` command
files, sv2v's explicit multi-file runs; a `-f` list, a glob, a `--siloed` run and an ordering conflict
are reported, not applied). The first
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
build system's `.core` file supplies), and 1 is `` `include `REQUESTS_FILE `` — refused by ruling,
because IEEE 1800-2017 §22.4 Syntax 22-1 admits `"filename"` and `<filename>` only. The engine's
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

**The file's own configuration (`.4c.1`).** A test suite's driver does not run a file bare: verilator's
`t_x.py` passes `+define+NAME[=VALUE]` / `-DNAME[=VALUE]`, sv-tests' header carries `:defines:`. The
derived context now includes them per file (`stimuli/sv/pipeline_context_defines.tsv`, 55 rows on 48 files:
52 verilator, 3 sv-tests — a name the same command line `-U`s again is not one of them), applied as
`--sv-define` before the suite's rows. The deriver emits a LITERAL
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
`--sv-unit-file` in order, from three declaration dialects: verilator's source-file arguments in the
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
OpenTitan's flow emits at build time). The `` `include ``d class files of a passing package are still
per-file rows without an `import` of their own — the unit-membership reading is `.6d.2`.

**The denominator, read twice.** The platform book's verdict-coverage tuple
(`adjudicated/routed/no-verdict/dark/axis-2-bar`) now carries two readings of the same rows, published
together and held by the same gate: the raw lane's `7498/2606/6232/4025/9`, and `9129/2606/4601/3318/30`
under the pipeline. The second is the first with the re-adjudication table applied: 1 216 explained-away
rows corroborated, 1 696 design-corpus files accepted under their derived context and library (must-accept by the
suite's own convention — a shipping design compiles), 436 must-reject rows rejected where the raw lane
never reached the construct (28 of them the key questions of the disposition table, pinned with the
clause and a reproducer each, and 3 the rows a file's own driver define reaches — `.4c.1`), and 21 must-accept rows still refused in their own text — the bar's new
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

Two thirds of the residue is one fact: the base class of a `class … extends` lives in another file
of the compilation unit (a package the row `import`s, or a sibling `.sv`), and the parser's
store gate refuses a base it has not seen declared — the strict reading `SV-PARSE-STRICT.2` escaped
for net types and not for class bases. That is a cross-file DECLARATION barrier, exactly the one
the library flags (`--lib-out` / `--lib-in`) exist for; sizing it was the point of the measurement,
and the number is now on the table for the leaf that decides how a per-file lane supplies it.

- The seven curated real-design cases of the external-corpus triage gate (UVM's two packages,
  scr1, friscv, VeeR) run through the single command with the manifest's include directories:
  the expansion of `uvm_pkg.sv` (166 included files) matched the shell gate's byte for byte until
  `SVPP-EXPANSION.6b.5`, which found the gate's reference (3 022 806 bytes) wrong in 98 lines — all
  inside block comments: the preprocessor of 2026-08-24 had honoured `` `define `` lines inside a
  `/* … */` (`uvm_tlm_imps.svh` keeps older macro versions in one) and expanded macro uses inside
  others. The expansion is now 3 020 226 bytes, every changed hunk audited inside a comment, and
  scr1 / friscv / VeeR accept on both profiles.
- UVM's `uvm_pkg.sv` — the unit-level reading of that suite — currently **rejects** at
  `src/base/uvm_transaction.svh:443`, `const local uvm_event_pool events = new("events");`, on both
  profiles. That is not a preprocessing defect: IEEE 1800-2017 A.2.7 gives a `const`-first class
  property only the alternative
  `const { class_item_qualifier } data_type const_identifier [ = constant_expression ] ;`, and
  `new(…)` is not a `constant_expression` (the `local const …` order is derivable through
  `data_declaration`, which admits a `class_new` initializer). Every commercial simulator accepts
  the UVM spelling; PGEN accepted it too until the release of 2026-09-04 made `local` a reserved
  word (it had lexed as an identifier and the line mis-parsed into an accept). The policy question
  — a documented tool-extension arm for a construct the de-facto standard library relies on, versus
  the letter of Annex A — is open in `docs/tasks/SVPP-EXPANSION.md` (`.5`).

## Boundaries

- The expansion engine is hand-written Rust (`rust/src/sv_preprocessor.rs`), not a grammar; its
  *directive* structure is separately described by the `systemverilog_preprocessor` grammar, whose
  parser does not expand anything. The two are different deliverables and this chapter is about
  the first.
- The pipeline preprocesses one compilation unit at a time and parses its last file; a
  multi-file design still needs the library flags for cross-file declarations.
- Nothing yet reads a build system's own context (`.core`, `.f`, `+incdir+`); on the command line
  the context is what you pass, and in the corpus lane it is derived from the corpus — a shared
  directory per suite, the nearest candidate per file for a name that exists in several, and a
  reported tie where the tree cannot decide.

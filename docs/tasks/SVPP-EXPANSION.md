# SVPP-EXPANSION — the SystemVerilog preprocessing / expansion stage (the missing front-end link for NEXSIM)

## Metadata

- Tree ID: `SVPP-EXPANSION`
- Status: **`active`** — ⭐⭐ **ACTIVATED 2026-09-09 by the DIRECTOR's STEER** recorded in `MEMORY.md`'s
  resume pointer at `feb45c4b` (*"BUILD THE PREPROCESS→PARSE PIPELINE — develop the preprocessor and the SV
  parser TOGETHER, not in isolation … the raw-parse corpus benchmark is grammar-development SCAFFOLDING, NOT
  the deliverable … wire a driver that runs the preprocessor FIRST with a real compilation context (include
  dirs, `+define`s, the unit's file list), then parses the expanded output, and measure end-to-end recognition
  on the macro-heavy corpus the raw lane defers … If anything is missing, say so"*). That steer SUPERSEDES the
  2026-06-08 sequencing gate below (build-later, after the locked program): the pipeline is on the SV release
  path and inside the lane lock ([[project_nexsim_sv_signoff_delivery_focus]]), not a departure from it.
  Was `proposed` (own-now / build-later, director 2026-06-08) from creation until this activation.
- Family / slice-id prefix: `PGEN-SVPP-EXPANSION-<NNNN>`
- Roadmap lane: front-end workbench / downstream-consumer service (NEXSIM SV front-end) — composes the
  `systemverilog_preprocessor` (svpp) directive parse with the `systemverilog` (sv) parser
- Created: `2026-06-08`
- Owner: repo-local workflow
- Decision record: [[project_svpp_expansion_stage_for_nexsim]]
- Disciplines: [[feedback_research_grounded_sota_no_trial_and_revert]] (survey IEEE 1800 §22 +
  slang/Verible/Verilator preprocessing models BEFORE building), [[feedback_tools_first_no_guessing]],
  [[feedback_ast_pipeline_parser_agnostic]] (parser-agnostic where the svpp directive model is general),
  [[feedback_always_signoff_decisions]].

## The frame (why this tree exists)

PGEN's svpp parser parses SystemVerilog preprocessor *directives* into a typed AST (`` `define `` /
`` `ifdef `` / `` `include `` / macro bodies / condition exprs) and passes non-directive code through as
`non_directive_text`; it does **not** expand macros, resolve conditionals, or inline includes (no
substitution engine). The sv parser parses SV source directly and only *tolerates* a directive inline via
the opaque `compiler_directive` rule — it does **not** expand it either (`systemverilog.ebnf:178`). So the
two parsers do **not** compose into the classic preprocess→parse pipeline a downstream consumer expects;
an **expansion stage** must sit between them.

Real SystemVerilog cannot be elaborated without preprocessing (macros expand into arbitrary tokens;
conditionals select live code; includes pull in package/interface/typedef definitions). So for NEXSIM (the
primary SV+VHDL integration target — a simulator/elaborator-facing consumer), an un-expanded parse tree is
not a usable design representation. Per the director (2026-06-08): **PGEN ought to deliver the whole
package; the SV + SVPP front-end stack is not release-complete for NEXSIM without the expansion stage.**

The svpp + sv parsers remain valid deliverables AS PARSERS; the expansion stage is a SEPARATE deliverable —
a **transformation** over the svpp directive AST, not a parser — consistent with PGEN's "parsers from EBNF"
doctrine and its front-end-workbench / compiler-elaborator-enablement direction.

## Goal

Deliver a SystemVerilog preprocessing / expansion stage that, given SV source, produces **expanded,
sv-parseable source** (or an equivalent expanded token stream) by composing the svpp directive parse with
macro substitution, conditional resolution, and `` `include `` inlining — so the full SV front-end
(svpp parse → expand → sv parse) is a usable, trustworthy service for downstream consumers (NEXSIM first).

## Non-goals

- A new parser. This is a transformation/service over the EXISTING svpp directive AST + macro table.
- VHDL (no C-style preprocessor — SV-specific).
- Full elaboration / typing / binding (owned by the compiler-elaborator lane). Expansion stops at producing
  sv-parseable expanded source; semantic elaboration is downstream.
- Changing the svpp or sv grammars' accept sets (the expansion consumes svpp output and targets the existing
  sv parser).

## Acceptance criteria

- **Macro substitution:** object-like and function-like macros (with arguments), `` `" `` stringize,
  `` ` ` `` token-paste, nested/recursive expansion with a cycle/recursion guard; `` `undef `` honored;
  predefined/built-in macros (`` `__FILE__ `` / `` `__LINE__ `` / `` `line `` …) where in scope.
- **Conditional resolution:** `` `ifdef `` / `` `ifndef `` / `` `elsif `` / `` `else `` / `` `endif ``,
  arbitrarily nested; only the live branch survives.
- **`include` inlining:** path resolution (search list), recursion/cycle guard, line/file provenance.
- **Composition proof:** the expanded output re-parses cleanly through the `systemverilog` parser; round-trip
  + coverage proof; provenance/source-map preserved enough for downstream diagnostics (anchor fidelity).
- **Parser-agnostic where general:** the directive-AST→expansion machinery should reuse the svpp directive
  model generically where possible (not a one-off SV hack).
- **Lockstep on landing:** SV + SVPP integration contracts + release policy updated to document the front-end
  pipeline (svpp parse → expand → sv parse); book chapter; this tree; decision record.

## Task tree

- ID: `SVPP-EXPANSION.1`
  Status: ✅ **`done`** 2026-09-09 (`PGEN-SVPP-EXPANSION-0002`, pure docs)
  Goal: SCOPING — measure what EXISTS of the preprocess→parse pipeline, what is MISSING for a real
  compilation context, and what the existing expansion engine already refuses on the macro-heavy corpus;
  re-plan the leaves against that measurement (the 2026-06-08 leaf plan assumed no expansion engine existed).
  Acceptance: every claim below carries the command that produced it; the leaf list `.2`–`.7` is owned and
  ordered; the architecture question is recorded as a FINDING with a measured basis, not re-litigated.
  Verification: `rust/target/svpp_e1/pp_census.sh` (the preprocess-only census, 17 s), the triage gate's
  `summary.txt` of 2026-08-24, the tracked lane reports — all quoted in the `.1` record below.
  Commit: `PGEN-SVPP-EXPANSION-0002`
- ID: `SVPP-EXPANSION.2`
  Status: ✅ **`done`** 2026-09-09 (`PGEN-SVPP-EXPANSION-0003`; ENGINE tier — `rust/src/sv_preprocessor.rs` +
  `rust/src/main.rs`, ZERO grammar / codegen / generated bytes; record + checklist below)
  Goal: the COMPILATION CONTEXT (code, `rust/src/sv_preprocessor.rs` + `rust/src/main.rs`): (a) predefined
  macros — `SvPreprocessorConfig.defines` seeded into the macro table before the first line is read, CLI
  `--sv-define NAME[=VALUE]` (repeatable; the `+define+` of every commercial front end); (b) UNIT-level macro
  continuity — an ordered file list preprocessed under ONE macro table, so a macro defined in file *n* is
  visible in file *n+1* (IEEE 1800-2017 §22.5.1 `section-22-compiler-directives.md:272-273`: *"Once a text
  macro name has been defined, it can be used anywhere in the compilation unit where it is defined. There are
  no other scope restrictions once inside the compilation unit"*; §3.12.1 `:406`: *"All files on a given
  compilation command line make a single compilation unit"*), exposed as a library entry that takes the unit's
  file list and returns one `PreprocessedOutput` per file, and as CLI `--sv-unit-file F` (repeatable, ordered:
  the files preprocessed BEFORE `INPUT` in the same unit); (c) `` `resetall `` clears the table per §22.3.
  Acceptance: unit tests for each (a define from the CLI expands; a macro defined in unit file 1 expands in
  file 2 and NOT in a fresh unit; `` `resetall `` clears); the existing focused preprocessor suite unchanged;
  `ast_pipeline --preprocess-systemverilog` byte-identical output on every current call (no flag ⇒ no change).
- ID: `SVPP-EXPANSION.3`
  Status: ✅ **`done`** 2026-09-09 (`PGEN-SVPP-EXPANSION-0004`; PROBE + DOCS tier — `rust/src/bin/parseability_probe.rs`,
  ZERO grammar / codegen / generated / engine bytes; record + checklist below — and it found the `uvm_pkg`
  finding routed to `.5`(d))
  Goal: the SINGLE-COMMAND CHAIN — `parseability_probe --parse systemverilog FILE --preprocess
  [--sv-include-dir D]… [--sv-define N[=V]]… [--sv-unit-file F]…` (and the `--parse-dump-ast[-pretty]` twins):
  preprocess IN-PROCESS, parse the expanded TEXT through the registry (never a scratch file), and on a
  rejection map `furthest_position` back to the ORIGIN `file:line:column` through the preprocessor's own
  source map — the map has existed since 2026-02-27 and nothing has ever read it. One `preprocess:` summary
  line on stderr (included files, expansions, diagnostics) in the opt-in style of `PGEN_REPORT_*`, because
  `stimuli/run_external_corpus.sh` captures the probe's stderr as a corpus column (`SV-CORPUS-GRAD.12a`).
  Acceptance: the seven triage-gate cases reproduce their 14/14 through the single command (`--lib-in`/`--lib-out`
  composing unchanged); a rejection inside an INCLUDED file names that file and line; a rejection inside a
  macro EXPANSION names the use site; the SV parser book gains its "Preprocess→Parse Pipeline" chapter and the
  platform book's CLI chapter documents the flags (lockstep, not a mention —
  [[feedback_lockstep_means_fully_documented_not_mentioned]]).
- ID: `SVPP-EXPANSION.4`
  Status: ✅ **`done`** 2026-09-09 (`PGEN-SVPP-EXPANSION-0005`; INSTRUMENT + CORPUS-LANE tier — three tracked
  tools, one tracked context, one tracked report; ZERO Rust / grammar / codegen / generated bytes; record +
  checklist below). The whole-lane run (16 336 rows) and the `.core`-derived per-file context are `.4`(b),
  opened below.
  Goal: the CORPUS PIPELINE LANE — a tracked per-suite compilation context (`stimuli/sv/pipeline_context.tsv`:
  suite → include dirs + defines, seeded from the suites' own `.core` / `.f` / `+incdir+` declarations where
  they exist — OpenTitan has 818 `.core` files and ibex `.f` lists with `+incdir+` lines; nothing in PGEN reads
  them today), a runner lane beside the raw one (`results_pipeline.tsv` — the same row set, each row parsed
  through `.3`'s chain), and the comparison report (`fail → pass`, `pass → fail`, preprocess-fail by class,
  per suite) — measured FIRST on the 4 157 UVM + OpenTitan rows the raw lane carries entirely as
  `deferred:chained_only`, then on the whole 16 336-row lane. Discharges the chaining half of
  `SV-CORPUS-GRAD.4`.
  Acceptance: the lane runs under the memory guard, timeout-bounded, repo-relative paths, byte-reproducible;
  the report pre-registers the raw lane's numbers beside its own; every `pass → fail` is adjudicated by name.
- ID: `SVPP-EXPANSION.4b`
  Status: `pending` — **FRONTIER** (opened 2026-09-09 by `.4`)
  Goal: (i) the WHOLE lane — every suite of the 16 336-row corpus through the pipeline, the context
  derivation extended to every suite (`SUITES` in `pipeline_context.py`), the report re-rendered; (ii) the
  PER-FILE context from the suites' own declarations — OpenTitan's 818 `.core` files and ibex's `.f` lists
  with `+incdir+` — so the 23 `pp_fail` rows whose `` `include `` name exists under several `top_*`
  variants get the directory their build system would give them; (iii) a currency gate for the tracked
  report (the identity block is already checked by `IDENTITY-CARRIERS`; a `--check` of the context is
  wired into the runner — what is missing is the doctrine row that re-derives the report's headline from
  the dump, `CORPUS-CENSUS-CURRENCY`'s shape).
- ID: `SVPP-EXPANSION.5`
  Status: `pending` (three defects named by `.1`'s census; more expected from `.4`)
  Goal: the preprocessor DEFECTS the measurement finds, each its own slice with the toolbox chain
  (reproduce → root cause → fix → before/after → no regression): (a) a NEWLINE inside a `` `define ``'s
  formal-argument list — §22.5.1 `:46-47` *"separated by commas and optionally white space"*, §5.3 white space
  includes newlines — refused as `unterminated macro parameter list` (OpenTitan
  `hw/dv/sv/cip_lib/seq_lib/cip_base_vseq__tl_errors.svh:8`, reached from 4 census rows); (b) conditional
  directives INSIDE a macro body mis-nested on the expansion re-entry — `` `else without matching `` /
  `` `endif without matching `` (`hw/ip/tlul/rtl/tlul_assert.sv` via `prim_assert.sv`'s `ASSERT_RPT`,
  `hw/vendor/lowrisc_ibex/dv/formal/check/top.sv` via `peek/compare_helper.sv`) — and the DIAGNOSTIC defect
  those two expose: the re-entry restarts line numbering, so the error names `tlul_assert.sv:4` (a comment
  line) and `compare_helper.sv:3`; (c) a RULING — `` `include `MACRO `` (`hw/ip/aes/pre_dv/aes_tb/rtl/aes_tb_pkg.sv:39`)
  is outside Syntax 22-1 (`"filename"` | `<filename>` only), so the refusal is LRM-correct: record it as a
  named tool-extension class (a policy knob, default OFF under [[feedback_sv_strict_lrm_compliance_default]]);
  (d) ⭐⭐ **the `uvm_pkg` finding `.3` made** (a DIRECTOR CALL, see *Open questions*): UVM 2020.3.1's
  `const local uvm_event_pool events = new("events");` (`src/base/uvm_transaction.svh:443`) is not Annex-A
  derivable in either edition — A.2.7 gives a `const`-first class property only
  `const { class_item_qualifier } data_type const_identifier [ = constant_expression ] ;` and `new(…)` is
  no `constant_expression` — so the parser rejects `uvm_pkg.sv` (both profiles, `furthest_position=1015525`)
  since `e0a32496` (`.3.37`, 2026-09-04) made `local` a reserved word; before that the line mis-parsed
  into an ACCEPT with `local` read as an identifier. Every commercial simulator accepts the spelling;
  the `local const …` order IS derivable (through `data_declaration`'s `class_new` initializer). Bisected
  with the interpreter over the 32 grammar commits since the gate's 2026-08-24 run; the fix shape is a
  documented tool-extension arm on `class_property` versus the letter of Annex A; (e) housekeeping: the
  26 pre-existing preprocessor tests write to the OS temp dir — move them to `create_repo_scratch_dir`.
- ID: `SVPP-EXPANSION.6`
  Status: `pending`
  Goal: RE-ADJUDICATE the preprocessing-deferred population under the pipeline — the 1 434
  `divergence:explained_svpp_*` rows (must-accept rows the raw lane explains away by preprocessing), the 4 927
  `deferred:chained_only` rows and the 173 `deferred:svpp_owned` rows (lane report of `PGEN-SV-CORPUS-GRAD-0386`,
  `stimuli/sv/characterization/adjudication_summary.md`) — so `SV-CORPUS-GRAD.13d`'s 1 527 expansion-owned DARK
  rows and `.12b`'s 263 position-undecidable rows get the verdict only expansion can give them, and the axis-2
  denominator moves through `SV-CORPUS-DENOMINATOR`'s own gate.
- ID: `SVPP-EXPANSION.7`
  Status: `pending` (runs in lockstep with every slice; closes the tree)
  Goal: the CONTRACT + BOOK statement of the front end — the SV integration contract's front-end section
  (svpp expansion → sv parse, the context flags, the source-map contract), the svpp parser book's status
  (expansion is a sibling deliverable, not the directive parser's), the platform book's family page, the
  DONE-BAR register if a family row's next gap changes.

## Current frontier

| Order | Leaf | Status | Why next |
| --- | --- | --- | --- |
| 1 | `.5` | `pending` | the three census defects + `.3`'s `uvm_pkg` finding (d, a director call) + housekeeping (e); toolbox chain each — `.4` measured that the engine's own refusals are 6 of 4 157 rows, so this is small and next |
| 2 | `.4b` | `pending` | the whole lane + the `.core`-derived per-file context + the report's currency gate |
| 3 | `.6` | `pending` | the deferred population re-adjudicated; the axis-2 denominator moves — `.4` sized its largest class (the cross-file base-class fact, 2 048 of 3 025 rejections) |
| 4 | `.7` | `pending` | contract + book lockstep, closes the tree |

## `.1` SCOPING — ✅ done 2026-09-09 (`PGEN-SVPP-EXPANSION-0002`): what exists, what is missing, what the engine refuses today

⭐ **THE 2026-06-08 FRAME WAS WRONG ABOUT ONE FACT, AND THE LEAF PLAN INHERITED IT.** The tree's own frame
says *"neither parser expands, and there is no PGEN stage that feeds svpp's output into sv"*. The first half
is true of the two GENERATED parsers; the second half was false when written: a hand-written expansion engine
has existed since **2026-02-27** (`636f35c0`, *"feat: add Rust SV preprocessor execution stage and
ast_pipeline CLI mode"*, `rust/src/sv_preprocessor.rs`, 2 540 lines today), and a SHELL composition of it with
the SV parser has run the seven curated real-design cases since Phase Q
(`rust/scripts/sv_external_corpus_triage_gate.sh`, manifest
`rust/test_data/grammar_quality/systemverilog_external_corpus_triage_v0.json`). So the leaves `.2`–`.4` as
planned — *build* macro substitution, conditional resolution and `` `include `` inlining — describe work that
was done before the tree was created. What is genuinely missing is narrower and different, and it is measured
below.

### What EXISTS (each with the command that shows it)

| piece | where | evidence |
|---|---|---|
| the expansion engine: object + function macros with defaults, `` `" `` stringize, ``` `` ``` paste, nested expansion (depth ≤ 16), `` `undef ``, `` `__FILE__ `` / `` `__LINE__ ``, `` `include `` with search dirs + cycle + depth guards, `` `ifdef/`ifndef/`elsif/`else/`endif `` with policies, a byte-indexed SOURCE MAP, an event log, diagnostics, the shared source-text decoder | `rust/src/sv_preprocessor.rs` (`preprocess_systemverilog_file`, `PreprocessedOutput`) | `git log --oneline --follow -- rust/src/sv_preprocessor.rs \| tail -1` → `636f35c0` |
| the CLI | `ast_pipeline INPUT.sv --preprocess-systemverilog [--output F] [--sv-include-dir D]… [--sv-source-map-json] [--sv-event-log-json] [--sv-diagnostics-json] + 6 policy flags` | `rust/src/main.rs:329-578`, `:1022-1127` |
| the shell composition, 7 cases × 2 profiles, `bootstrap_files` parsed with `--lib-out` then the case with `--lib-in` | `sv_external_corpus_triage_gate.sh` (`make -C rust sv_external_corpus_triage_gate`; operator-invoked, no CI lane) | last run `rust/target/sv_external_corpus_triage_gate/summary.txt`, `generated_at_utc: 2026-08-24T21:15:58Z`: `cases_executed: 14 preprocess_pass_total: 14 parse_pass_total: 14 parse_fail_total: 0 preprocessed_bytes_max: 3022806 parse_max_ms: 85168` — `uvm_pkg` and `uvm_compat_pkg` on both profiles among them |
| the in-memory parse entry the chain needs | `parser_registry::parse_sample_detail_with_options(grammar, &text, profile, &LibraryOptions)` (`rust/src/parser_registry.rs:1950`) | called by `parseability_probe --parse` (`rust/src/bin/parseability_probe.rs:556`) |
| the corpora | UVM `stimuli/sv/uvm/uvm-core-2020.3.1/` (174 rows), OpenTitan `stimuli/sv/subs/opentitan/` (3 983 rows; 818 `.core` files, 3 976 `.sv`, 35 `.svh`; ibex vendored inside it with `.f` lists carrying `+incdir+`) | `docs/tasks/artifacts/sv_corpus_grad/verdict_coverage/coverage.md:51,56` |

### What is MISSING (the director asked to be told)

1. **No predefined macros.** `SvPreprocessorConfig` has no `defines` field and the CLI no `--sv-define`; every
   `+define+` a build system passes (`ifdef UVM`, `ifdef VERILATOR`, `SYNTHESIS`, `FPV_ON`, …) is silently
   FALSE. → `.2`(a).
2. **No compilation UNIT.** `preprocess_systemverilog_file` builds a fresh macro table per call, so the
   §22.5.1 rule that a macro is visible for the rest of the compilation unit (`:272-273`) holds only inside one
   file and its includes. A unit's second file cannot see the first file's `` `define ``s. → `.2`(b).
3. **No single command.** The probe parses RAW text (`--parse` reads the file and hands it to the registry);
   the chain exists only as shell orchestration in one gate, per curated case. → `.3`.
4. **The source map is write-only.** `PreprocessedOutput.source_map` is emitted to JSON on request and read
   by nothing; a rejection in expanded text reports an offset into a file that does not exist on disk. → `.3`.
   Census: `git grep -n 'source_map\|sv_source_map_json\|SourceMapEntry' -- rust/src rust/scripts scripts stimuli | grep -v '^rust/src/sv_preprocessor.rs'`
   → 7 lines, all in `rust/src/main.rs:530-1122`, every one a WRITE (`serde_json::to_string_pretty(&output.source_map)`)
   or a `.len()` for the summary line; no consumer maps a position through it.
5. **No corpus-scale measurement.** `stimuli/run_external_corpus.sh` never preprocesses (`parse_one()` at
   `:400` runs `--parse` on the raw file); the 4 157 UVM + OpenTitan rows carry no verdict at all
   (`deferred:chained_only`), and 1 434 further rows are ADJUDICATED as *fails because it needs
   preprocessing* on a whole-file + positional text test (`preproc_dependency()` /
   `svpp_can_explain_failure()`), never by preprocessing. → `.4`, `.6`.
6. **Nothing reads the suites' own context declarations** (`.core`, `.f`, `+incdir+`). → `.4`.
   Census: `git grep -n -E '\.core\b|\+incdir\+|fusesoc' -- rust/src rust/scripts scripts 'stimuli/*.sh' 'stimuli/sv/*.py' rust/Makefile`
   → 1 line, `stimuli/sv/adjudicate_external_corpus.py:1450`, a self-test FIXTURE string of the not-source-text
   detector (`"+incdir+../rtl\n-f files.f\n"`), not a reader of any `.core` or `.f` file.

### What the existing engine REFUSES today — the preprocess-only census (`rust/target/svpp_e1/pp_census.sh`, 2026-09-09)

Every UVM + OpenTitan row preprocessed ALONE with the current `ast_pipeline` and a plausible include path
(the UVM `src/` + every OpenTitan directory holding a `.svh`, 20 dirs, `rust/target/svpp_e1/include_dirs.txt`);
nothing parsed (the probe on disk was stale — fingerprint `a1ccdf4a…` against the parser's `423042d2…`):

```text
files=4157 rc0=4079 rc_nonzero=78 timeouts=0        (17 s wall, 6-way)
   71  include 'X' was not found in search paths     58 distinct names; 57 EXIST elsewhere in the corpus
                                                     (an include-PATH gap → .4), 1 absent (`sail.sv`)
    4  unterminated macro parameter list             all `cip_base_vseq__tl_errors.svh:8` — a formal list
                                                     spanning three lines with no backslash → .5(a)
    1  `else without matching `ifdef/`ifndef … tlul_assert.sv:4     → .5(b)  (line 4 is a comment)
    1  `endif without matching `ifdef/`ifndef … compare_helper.sv:3 → .5(b)  (reached from top.sv;
                                                     compare_helper.sv alone preprocesses rc 0)
    1  include path must be quoted or angle-bracketed   `` `include `REQUESTS_FILE `` → .5(c), LRM-correct
```

⇒ the expansion ENGINE is not the bottleneck on this population: **98.1 % preprocesses**, and 71 of the
78 refusals are the context (item 6 above), not the engine. The remaining 7 are three named defect classes.

### The architecture question — recorded, not re-litigated (🔎 for the director's review)

The tree carries a director ruling (2026-06-09, *Decisions* below): the expansion stage is a
**parse-tree transformation over the enhanced `svpp.ebnf` AST — NOT a separate token-stream engine**.
`rust/src/sv_preprocessor.rs` IS a hand-written line/token scanner with its own directive recognizer
(`parse_directive_line`, `parse_define_directive`, `parse_macro_parameter_list`, …) — the shape that ruling
declined — and it predates the ruling by three months. The 2026-09-09 steer names that file as *"the pieces
EXIST"* and directs the driver to be built on it. Both are director statements; this leaf does not choose
between them, because the DRIVER (`.2`–`.4`) is engine-agnostic — it composes *a* preprocessor with the parser
through `PreprocessedOutput`, and every acceptance criterion above holds for either engine behind that type.
What the driver produces is the evidence the choice needs: the census above says the hand-written engine
preprocesses 98.1 % of the hardest population, and `.4`/`.5` will say how many of its residual refusals are
engine defects versus context. **Recommendation:** keep the existing engine for the measurement; decide the
re-platforming (svpp-AST transformation, the 2026-06-09 shape) on `.4`'s numbers — a defect count in the
single digits argues for fixing the scanner, a class of defects the scanner's shape cannot express argues for
the ruling. Recorded durably in [[project_preprocess_parse_pipeline_is_the_sv_deliverable]].

## Decisions

- **(director 2026-06-08)** PGEN must deliver the SV expansion stage for NEXSIM; the SV+SVPP front-end is not
  release-complete without it. The svpp + sv parsers remain valid AS PARSERS; expansion is a separate
  transformation deliverable. See [[project_svpp_expansion_stage_for_nexsim]].
- **(director 2026-06-08, sequencing)** Build-later: sequenced strictly AFTER the locked program (every
  existing parser cert-coverage WIRED + clean + `UNKNOWN`=0). Do not pivot to building this until then.
- **(director 2026-09-09, ACTIVATION + the deliverable)** *"BUILD THE PREPROCESS→PARSE PIPELINE — develop
  the preprocessor and the SV parser TOGETHER, not in isolation … the correct flow for EVERY file is
  preprocessor → expanded (directive-free) text → SV parser; the raw-parse corpus benchmark is
  grammar-development SCAFFOLDING … NOT the deliverable … wire a driver that runs the preprocessor FIRST with a
  real compilation context (include dirs, `+define`s, the unit's file list), then parses the expanded output,
  and measure end-to-end recognition on the macro-heavy corpus the raw lane defers (UVM … OpenTitan). If
  anything is missing, say so."* Supersedes the 2026-06-08 sequencing. The leaves `.2`–`.7` are that steer,
  decomposed; the *"anything missing"* is the six-item list in the `.1` record.
- **(director 2026-06-09, ARCHITECTURE — go all-in PGEN-native)** The expansion stage is a **parse-tree
  transformation over the enhanced `svpp.ebnf` AST** — NOT a separate token-stream engine and NOT a separate
  grammar. RATIONALE (objective, no fundamental blocker found): the industry token-stream model exists to
  dodge one wall — *macros cross SV structural boundaries, so you can't preprocess over a parse tree of the
  SV grammar*. svpp's AST is **not** an SV syntax tree; it is a flat, source-ordered `pp_item*` **structured
  token/directive stream** (directives with internal structure + opaque `non_directive_text` runs), so it
  NEVER hits that wall (it makes no SV-structural assumptions). Walking svpp's `pp_item` list with a stateful
  macro table IS token-stream preprocessing, with the directive grammar recognized DECLARATIVELY (EBNF =
  single source of truth) instead of ad-hoc scanner code. Every hard case maps cleanly: boundary-crossing
  macro fragments + conditional-split constructs are text runs to substitute/keep/drop; token-paste/stringize
  + rescan/recursion are engine logic over the modeled macro-body atoms; balanced function-macro args are a
  context-free `macro_actual_args` rule; cross-boundary re-lexing (`` `HALF ``→`3.14` must not fuse with a
  following ` e2`) is handled by PGEN's **lexical-faithfulness pillar (LEXICAL-ANNOTATIONS)** — an area PGEN
  is AHEAD; provenance is structured (AST spans). The token-stream model's only genuine edges
  (performance / streaming / malformed-input tolerance) are NON-FUNCTIONAL — none a capability/correctness
  blocker, and all consistent with PGEN's parse-then-transform architecture + correctness-before-speed.
  GAINS going PGEN-native: directive grammar validated by the well-formedness linter + cert-coverage;
  expansion is a testable, proof-first transformation over a validated AST; structured provenance;
  architectural consistency (the transformation dual; the macro table is the same stateful left-to-right
  processing the semantic store already does). OBLIGATIONS (not blockers): `svpp.ebnf` must be a total/correct
  grammar over the directive+text surface (proven by cert-coverage/linter); macro-usage recognition + output
  must be lexically faithful (we have the pillar); full-AST performance on huge files is monitor-and-optimize-
  later. The `.1` SCOPING read of IEEE 1800 §22 + slang/Verible/Verilator now CONFIRMS no SV-specific corner
  breaks this + settles implementation details (output form, provenance carrier, token-paste/rescan corners)
  — it does NOT re-litigate the architecture.
  - **PROS/CONS (recorded 2026-06-09, balanced).** *Token-stream (industry) PROS:* streaming performance/low
    memory (single lexical pass, no full AST); conceptual simplicity (lexer + substitution loop);
    language-agnostic (grammar-blind directive filter); malformed-input tolerance; direct token-level
    differential alignment with slang/Verible/Verilator. *Token-stream CONS for PGEN:* ad-hoc directive
    recognition (NOT EBNF-validated → drifts from single-source-of-truth; linter/cert-coverage don't apply);
    a SECOND separate technology beside the EBNF/AST pipeline (more surface, inconsistent); flatter
    provenance; no reuse of the existing svpp grammar/AST. *Parse-tree (PGEN-native) PROS:* EBNF = single
    source of truth (directive grammar validated by linter + cert-coverage + well-formedness); proof-first /
    testable transformation over a validated AST (shape contracts, round-trip, deterministic gates);
    structured provenance (AST spans — what downstream linters/elaborators need); architectural consistency
    (transformation dual; macro table reuses semantic-store-style stateful processing); lexical-faithfulness
    already solved (LEXICAL-ANNOTATIONS). *Parse-tree CONS:* heavier (build+walk AST vs streaming —
    mitigable, correctness-first); a grammar-totality obligation on svpp.ebnf (provable via cert-coverage;
    `non_directive_text` catch-all is near-total); the same subtle rescan/paste semantics must be encoded
    (true of BOTH models). **WEIGHING:** every token-stream pro is NON-FUNCTIONAL (performance/simplicity/
    agnosticism/tolerance/reference-alignment) — none is a capability the parse-tree lacks; every parse-tree
    pro is a SIGNOFF-QUALITY/doctrine win (PGEN's differentiator). The parse-tree cons are real but
    non-functional + mitigable. ⇒ parse-tree, all-in.


## `.2` the COMPILATION CONTEXT — ✅ done 2026-09-09 (`PGEN-SVPP-EXPANSION-0003`): predefined macros and the compilation unit

**What landed** (`rust/src/sv_preprocessor.rs`, `rust/src/main.rs`; ZERO grammar / codegen / generated bytes):

- `SvPreprocessorConfig.defines: Vec<PredefinedMacro>` — `PredefinedMacro { name, text }` with
  `PredefinedMacro::parse("NAME" | "NAME=TEXT")` (everything after the FIRST `=` is text, verbatim; the name
  must be a legal text-macro identifier; a formal list `F(x)=…` is REFUSED, not misread as a name). Seeded
  into the macro table in `PreprocessorState::new`, in declaration order, as object-like definitions — so
  `` `ifdef ``, `` `undef `` and the redefinition policy treat a predefine exactly as a source
  `` `define ``. `NAME` alone = empty text, the meaning a bare `` `define NAME `` line has (§22.5.1).
- `preprocess_systemverilog_unit(&[PathBuf], &config) -> Result<Vec<PreprocessedOutput>>` — the files in
  order under ONE macro table (`into_carried_macros` hands the table from file *n* to file *n+1*); the text,
  source map, events, diagnostics and included-file list are per FILE (`take_output`); the vector is
  index-aligned with the input; a failure names `compilation unit file k of n ('path')`.
  `preprocess_systemverilog_file` is now the one-file unit (same `take_output`).
- CLI: `--sv-define NAME[=TEXT]` (repeatable, ordered) and `--sv-unit-file F` (repeatable, ordered — the
  files preprocessed BEFORE `INPUT`, their text discarded; `INPUT` is the unit's last file). The summary
  line gains `predefines=N unit_predecessors=M`; without either flag the one-file entry runs and the output
  is byte-identical to `1.0.227`'s.
- `` `resetall `` — NOTHING to the table, by the standard: IEEE 1800-2017 §22.3
  (`docs/systemverilog/2017/md/section-22-compiler-directives.md`, the clause's last two sentences): *"Not all
  compiler directives have a default value (e.g., `` `define `` and `` `include ``). Directives that do not have
  a default are not affected by `` `resetall ``."* Pinned by `resetall_does_not_undefine_macros`, so the
  leaf's own plan item (c) is closed as *not a change* rather than silently dropped.
- Nine tests (`predefined_macro_*` ×4, `compilation_unit_carries_the_macro_table_across_files_in_order`,
  `a_fresh_unit_does_not_see_a_previous_units_macros`, `a_unit_failure_names_the_file_that_failed`,
  `a_source_define_supersedes_a_predefine_under_the_default_policy`, `resetall_does_not_undefine_macros`);
  their scratch directories live under `rust/target/svpp_unit_tests/` on the repository volume
  (`create_repo_scratch_dir`) — ⚠️ the 26 pre-existing tests use `std::env::temp_dir()`, off-volume;
  routed, not changed here (see *Open questions*).
- Docs: `PGEN_USER_GUIDE.md` (the recipe, the controls list, the flag list),
  `docs/knowledge/ast-pipeline-cli-reference.md` (+ `KNOWLEDGE_MAP.md` regenerated),
  `docs/reference/RUST_CODEBASE_ANALYSIS.md` (the preprocessor's integration seam). ⛔ The platform book's
  CLI chapters mention no preprocess flag at all (`grep -n -i preprocess docs/book/src/cli-and-workflows.md
  docs/book/src/user-facing-surfaces.md docs/book/src/getting-started.md` → 0 lines) — the chapter `.3` owes
  covers the whole pipeline surface, these flags included.

**Acceptance checklist (`TASK-ACCEPTANCE`):**

  - [x] **REPRODUCE / ISSUE** — `.1`'s census: `rust/target/svpp_e1/pp_census.sh` → `files=4157 rc0=4079 rc_nonzero=78`; on `stimuli/sv/subs/opentitan/hw/ip/prim/rtl/prim_assert.sv` the `` `ifdef VERILATOR `` arm is dead with no way to define `VERILATOR` from the command line (`ast_pipeline … --preprocess-systemverilog --sv-event-log-json` shows the active include `prim_assert_standard_macros.svh`, never the dummy-macro arm); a macro defined in one file is invisible to the next file of the same unit (`rust/target/svpp_e1/s2/input.sv` alone: `logic [`FROM_FIRST-1:0] q;`).
  - [x] **ROOT CAUSE (WHY + WHERE)** — ops family, a capability that never existed: `git log -S'defines' --oneline -- rust/src/sv_preprocessor.rs` → **0** commits (no field ever carried a command-line define; `git log -S'PredefinedMacro'` → 0 before this leaf); WHERE `SvPreprocessorConfig` (no `defines`), `PreprocessorState::new` (`macros: HashMap::new()` per call — a fresh table per FILE), `preprocess_systemverilog_file` (the only entry, one path). WHY the standard requires the unit: IEEE 1800-2017 §22.5.1 `section-22-compiler-directives.md:272-273` (*"Once a text macro name has been defined, it can be used anywhere in the compilation unit"*), §3.12.1 `section-2-normative-references.md:406` (*"All files on a given compilation command line make a single compilation unit"*), and §22.3 for what `` `resetall `` does NOT do.
  - [x] **FIX** — engine tier (declarative: none applies — a preprocessor has no grammar surface for a command-line define; grammar: none): the field, the parser, the seeding, the unit entry, the two flags, the summary fields; nine tests.
  - [x] **ADDRESSED (verified)** — `rust/target/svpp_e1/verify_s2.sh` (log `verify_s2.log`) on the rebuilt debug `ast_pipeline` (2 m 17 s, peak 8.4 GB under the guard): (2) `--sv-define VERILATOR` on `prim_assert.sv` → `predefines=1`, output 9 264 → 8 947 bytes, the active `` `include `` events now `"prim_assert_dummy_macros.svh"`; (2b) `--sv-define 'F(x)=x'` → rc 1, *"'F(x)' is not a legal text-macro name for a predefined macro"*; (3) `--sv-unit-file first.svh` → `logic [4-1:0] q;|logic seen;` with `unit_predecessors=1` where alone it reads `logic [`FROM_FIRST-1:0] q;`; (3b) a broken predecessor → *"compilation unit file 1 of 2 ('rust/target/svpp_e1/s2/broken.svh') failed to preprocess"*. Unit tests: `cargo test --lib sv_preprocessor` → `test result: ok. 35 passed; 0 failed` (26 pre-existing + 9 new).
  - [x] **NO REGRESSION** — the no-flag path is **byte-identical** to the pre-`.2` binary's output on three real OpenTitan files (`dv_utils_pkg.sv` 23 009 B, `uart.sv` 20 405 B, `dv_base_env.sv` 3 664 B — `cmp` in `verify_s2.sh` (1)); every pre-existing preprocessor test green (26/26 of the 35); no grammar, codegen or generated byte moved (`generated-reproducibility: OK` at commit); `make -C rust SHELL=/opt/homebrew/bin/bash clippy_on_rust_change` → `clippy_source_all_targets` ok, `GENERATED-CLIPPY-CORRECTNESS: ✅ POLICY-ONLY PASS` (68 pinned lints), `clippy_generated_all_targets` ok — *"✅ clippy_on_rust_change completed"* (memory guard exit 0, peak 8.9 GB, 207 s; log `rust/target/svpp_e1/clippy_s2.log`); `bash scripts/check_doctrines.sh` ALL PASS (run at commit).
  - [x] **LOCKSTEP** — `PGEN_USER_GUIDE.md` + `docs/knowledge/ast-pipeline-cli-reference.md` (+ `KNOWLEDGE_MAP.md`) + `docs/reference/RUST_CODEBASE_ANALYSIS.md` + this leaf + `docs/TASK_TREE.md` + `MEMORY.md` + `CHANGES.md` + `DEVELOPMENT_NOTES.md`; the platform-book chapter is `.3`'s (stated above, not waived).
  - promotion: declined (the `.2` lessons in `DEVELOPMENT_NOTES.md` restate the `-0002` record's "re-measure before executing the plan" at a smaller scale; no new card).

## `.3` the SINGLE-COMMAND CHAIN — ✅ done 2026-09-09 (`PGEN-SVPP-EXPANSION-0004`): `parseability_probe … --preprocess`, and a rejection names the line you wrote

**What landed** (`rust/src/bin/parseability_probe.rs`; ZERO grammar / codegen / generated / engine bytes):

- `--preprocess` (SystemVerilog only; refused by name on any other grammar) with the three context flags
  `--sv-include-dir DIR` / `--sv-define NAME[=TEXT]` / `--sv-unit-file F` (repeatable, ordered; refused
  without `--preprocess` — a context with nothing to apply it to is a typo, not a no-op). The input is
  expanded IN-PROCESS through `preprocess_systemverilog_unit` (`.2`) and the EXPANDED text is handed to
  the registry exactly as a raw file's text is (`ProbeInput::{Raw, Preprocessed}`); `--lib-in` /
  `--lib-out` compose unchanged; `--parse-dump-ast[-pretty]` dumps the AST of the expanded text.
- **Three exit codes**: `0` accepted, `1` rejected by the parser, **`3` the preprocessor refused the
  input** (`preprocess_failed for grammar '…' on '…': …` on stderr; the parser never ran) — so a corpus
  lane can keep *could not be expanded* apart from *rejected by the language*.
- **A rejection names its origin.** `augment_rejection_with_origin` reads the rejection's
  `furthest_position=` (the surface `at position N` as a fallback) and maps it through the
  preprocessor's source map (`origin_of_position`: the chunk holding the byte names the file and line it
  was written from — an included file names itself, a macro EXPANSION names the line of its USE; the
  column counts CHARACTERS, not bytes; a position past the end lands on the last chunk's end; paths
  print relative to the working directory) and appends `[origin=FILE:LINE:COL]`; an unmappable one
  says `origin=unknown` so the absence is visible. Raw input's messages are untouched.
- `PGEN_REPORT_PREPROCESS=1` prints one `preprocess: …` line (opt-in, like `stack_probe:`, because
  `stimuli/run_external_corpus.sh` captures the probe's stderr as a corpus column).
- Both `command_parse` / `command_parse_dump_ast` now take `&GlobalOptions` instead of six / eight
  unpacked fields (clippy's `too_many_arguments` had been at 8/7 on the dump command already).
- Four unit tests (`preprocess_flags_are_parsed_in_order_and_need_preprocess`,
  `origin_maps_a_position_to_its_chunks_file_line_and_character_column` — incl. a two-byte `é` before
  the position, `rejection_position_prefers_furthest_over_surface`,
  `a_raw_rejection_is_untouched_and_a_preprocessed_one_carries_its_origin`).
- Docs (lockstep, [[feedback_lockstep_means_fully_documented_not_mentioned]]): the SV parser book's NEW
  chapter *The Preprocess→Parse Pipeline* (`docs/systemverilog_parser_book/src/preprocess-parse-pipeline.md`
  + `SUMMARY.md` + the tracked HTML rebuilt through `systemverilog_parser_book_gate`),
  `docs/reference/PARSEABILITY_PROBE.md` (flags, env, the `--parse` section, exit code 3),
  `docs/book/src/parseability-probe-debug.md` (reference card + a new section),
  `docs/book/src/cli-and-workflows.md` (a pipeline section), `TOOLBOX.md` §1.1, `PGEN_USER_GUIDE.md`.

**Verification** (`rust/target/svpp_e1/verify_s3.sh`, log `verify_s3b.log`, on the debug probe rebuilt from
this leaf's final bytes — fingerprint = parser `423042d2…`):

```text
(1) raw path unchanged (no --preprocess): dv_utils_pkg furthest_position=8829 SAME · uart parse_full passed SAME · dv_base_env furthest_position=233 SAME
(2) a rejection inside an INCLUDED file:   rc=1 furthest_position=42 origin=rust/target/svpp_e1/s3/inc/bad.svh:3:11
(3) a rejection inside a MACRO EXPANSION:  rc=1 furthest_position=38 origin=rust/target/svpp_e1/s3/macro.sv:4:13   (the `BAD(broken) use line)
(4) --sv-define UVM on an `ifdef: AST dump uvm_arm_in_ast=1 plain_arm_in_ast=0; --sv-unit-file: alone origin=unit.sv:1:18 → unit rc=0
    preprocess: input='…/unit.sv' unit_predecessors=1 include_dirs=0 defines=0 output_bytes=37 source_map_entries=1 included_files=1 macro_expansions=1 warnings=0
(5) refusals: a context flag without --preprocess rc=1 · --preprocess on json rc=1 · an unterminated `ifdef rc=3 preprocess_failed …
(6) the seven triage cases via the single command: TRIAGE-VIA-SINGLE-COMMAND: pass=10 fail=4 of 14
    PASS scr1_core_top / scr1_top_ahb / friscv_rv32i_core / friscv_pipeline / veer_el2_lsu × {2017, 2023} (≤ 1 s each, debug)
    FAIL uvm_pkg × {2017, 2023}: furthest_position=1015525 origin=stimuli/sv/uvm/uvm-core-2020.3.1/src/base/uvm_transaction.svh:443:38
    BOOT FAIL uvm_compat_pkg × {2017, 2023}: its bootstrap IS uvm_pkg.sv (the same rejection)
```

⭐⭐ **THE 14/14 THIS LEAF WAS WRITTEN AGAINST WAS A STALE READING, AND THE CHAIN IS WHAT SHOWED IT.** The
gate's tracked `summary.txt` (`generated_at_utc: 2026-08-24T21:15:58Z`, 14/14) predates 32 grammar commits.
Executed by hand on HEAD, the gate's own two steps read: expansion `ast_pipeline … --preprocess-systemverilog
--sv-include-dir …/src` → 3 022 806 bytes, **`cmp` byte-identical** to the gate's
`work/case_uvm_pkg_2017.preprocessed.sv`; then `parseability_probe --parse systemverilog <that 08-24 file>
--profile 2017` → **rc 1, `furthest_position=1015525`**. So the gate reads 10/14 on HEAD, the single command
reproduces it exactly, and the preprocessor is not the variable. The variable, bisected (all commands in
`rust/target/svpp_e1/s3/`):

```text
reproducer  class uvm_event_pool; endclass  class c;  const local uvm_event_pool events = new("events");  endclass
  r_const_local.sv    furthest_position=77       ← REJECTS
  c_local.sv          parse_full passed          (local T x = new(…))
  c_const.sv          parse_full passed          (const T x = new(…))
  r_local_const.sv    parse_full passed          (local const T x = new(…) — the other ORDER)
  c_const_local_int   parse_full passed          (const local int x = 3 — a constant initializer)
interpreter  HEAD grammar: accepted=false furthest_position=77 · grammar at 2e492d08 (2026-08-24): accepted=true
bisect over the 32 grammar commits since 2026-08-24 (5 interpreter runs):
  d239cc57 false · c4b2fe1b true · 9b5fa46a true · 70115cc2 true · e0a32496 false  ⇒ regressor e0a32496
  e0a32496 2026-09-04 PGEN-SV-CORPUS-GRAD-0366 (SV-CORPUS-GRAD.3.37: the reserved-word set becomes Annex B Table B.1)
  control: accepted=false at e0a32496, accepted=true at its parent 70115cc2
```

WHY, read in the standard: IEEE 1800-2017 A.2.7 (`docs/systemverilog/2017/md/section-41-data-read-api.md:559-563`)
and 1800-2023 A.2.7 (`docs/systemverilog/2023/md/section-8-classes.md:108-112`) — identical:
`class_property ::= { property_qualifier } data_declaration | const { class_item_qualifier } data_type
const_identifier [ = constant_expression ] ;`. With `const` FIRST the only derivation is the second
alternative, whose initializer is a `constant_expression`; `new("events")` is a `class_new`, admitted by
`variable_decl_assignment` (A.2.4) — i.e. by `data_declaration`, which is why the `local const …` order
parses — and by nothing in `constant_expression`. Until `.3.37`, `local` lexed as an identifier and the
line mis-parsed into an accept; `.3.37` was correct, and it exposed that UVM's spelling is a
tool-tolerated one. §8.19's prose (`docs/systemverilog/2017/md/section-8-classes.md`, *Constant class
properties*: *"Global constant class properties include an initial value as part of their declaration"*)
does not narrow the initial value; the syntax box does. ⇒ **NOT a preprocessor defect, NOT a grammar
regression of a legal construct: a policy question** — routed as `.5`(d) and asked below.

**Acceptance checklist (`TASK-ACCEPTANCE`):**

  - [x] **REPRODUCE / ISSUE** — before this leaf `parseability_probe --parse systemverilog FILE` parsed RAW text only (`git log -S'--preprocess' --oneline -- rust/src/bin/parseability_probe.rs` → 0 commits); the chain existed as shell orchestration in `rust/scripts/sv_external_corpus_triage_gate.sh` alone, and a rejection in expanded text named an offset into a file that exists nowhere (`furthest_position=1015525` into a 3 022 806-byte expansion).
  - [x] **ROOT CAUSE (WHY + WHERE)** — ops family: `git log -S'--preprocess' -- rust/src/bin/parseability_probe.rs` → 0 and `git log -S'source_map' -- rust/src/bin/parseability_probe.rs` → 0 — the probe never called the preprocessor and never read its source map; WHERE `command_parse` / `command_parse_dump_ast` (`read_source(input_file)` was the only input path), `strip_global_flags` (no context flags). For the finding: correctness family — `furthest_position=77` on the reproducer against four accepting controls, the interpreter A/B `accepted=true` (2026-08-24 grammar) → `accepted=false` (HEAD), bisected to `e0a32496`; WHY A.2.7 in both editions (lines cited above).
  - [x] **FIX** — probe tier (no grammar, no engine): `ProbeInput`, `load_probe_input`, `origin_of_position`, `rejection_position`, `augment_rejection_with_origin`, `report_preprocess_summary`, exit 3; the two commands narrowed to `&GlobalOptions`; four tests; the docs above.
  - [x] **ADDRESSED (verified)** — `verify_s3b.log` arms (2)–(6) above: an included file's rejection reads `origin=…/inc/bad.svh:3:11`, a macro expansion's `origin=…/macro.sv:4:13` (its use line), `--sv-define` flips the AST (`uvm_arm_in_ast=1 plain_arm_in_ast=0`), `--sv-unit-file` turns `origin=unit.sv:1:18` into `parse_full passed … [preprocessed]` with `unit_predecessors=1 … macro_expansions=1`, the three refusals rc 1 / 1 / 3, and the seven curated cases reproduce the gate's HEAD reading 10/14 through one command — with the four failing arms bisected to ONE construct.
  - [x] **NO REGRESSION** — the raw path's verdicts and `furthest_position` values are identical on the three smoke files (arm (1), `SAME` ×3 — and the message text is untouched by construction, `augment_rejection_with_origin` returns raw input's message unchanged, pinned by `a_raw_rejection_is_untouched_…`); `cargo test --features "generated_parsers ebnf_dual_run" --bin parseability_probe` → 12 passed, 0 failed; `cargo test --lib sv_preprocessor` → 35 passed, 0 failed (the `.2` tests restyled to clippy's liking, same assertions); the expansion of `uvm_pkg.sv` byte-identical to the 2026-08-24 gate artifact; no grammar / codegen / generated byte moved (`generated-reproducibility: OK` at commit); `clippy_on_rust_change` → source stage ok, `GENERATED-CLIPPY-CORRECTNESS: ✅ POLICY-ONLY PASS`, generated stage (`--all-targets --features generated_parsers,ebnf_dual_run`, the stage that compiles the probe) ok — *"✅ clippy_on_rust_change completed"*; in the feature-enabled log the only findings in the touched files are three pre-existing `collapsible_if`s (`parseability_probe.rs:252`, `:704`, `sv_preprocessor.rs:586` — none in code this tree wrote; the `.2` test-style findings and the 9/7 `too_many_arguments` are gone); `bash scripts/check_doctrines.sh` ALL PASS (run at commit).
  - [x] **LOCKSTEP** — the SV parser book chapter + `SUMMARY.md` + rebuilt tracked HTML (`systemverilog_parser_book_gate` ✅) + `docs/reference/PARSEABILITY_PROBE.md` + `docs/book/src/parseability-probe-debug.md` + `docs/book/src/cli-and-workflows.md` + `TOOLBOX.md` + `PGEN_USER_GUIDE.md` + this leaf + `docs/TASK_TREE.md` + `MEMORY.md` + `CHANGES.md` + `DEVELOPMENT_NOTES.md`. The integration contract's front-end section is `.7`'s (stated, not waived).
  - promotion: declined (the `.3` lesson — *a gate's tracked number is a reading of the tree it ran on; re-execute its steps on HEAD before writing an acceptance criterion against it* — is `docs/decisions/feedback_standing_evidence_reading_disciplines.md`'s "the frame a number arrives with", applied; no new card).

## `.4` the CORPUS PIPELINE LANE — ✅ done 2026-09-09 (`PGEN-SVPP-EXPANSION-0005`): the measurement the director asked for

**What landed** (ZERO Rust / grammar / codegen / generated bytes):

- `stimuli/sv/pipeline_context.py` (+ the tracked `stimuli/sv/pipeline_context.tsv`, 54 rows) — the
  compilation context DERIVED from the corpus: for every `` `include "NAME" `` an includer's own directory
  does not satisfy, the ONE directory (in the suite, or in a declared shared library — UVM's `src/`) that
  does, ordered by how many includers need it; an ambiguous name (29 on OpenTitan, all `top_earlgrey` /
  `top_darjeeling` / `top_englishbreakfast` variants of the same `seq_lib` list) is REPORTED with its
  candidates and never guessed; 4 absent names reported. `--write` / `--check` (the runner refuses to
  measure against a stale copy) / `--self-test` (6 checks on a synthetic tree: local, shared, ambiguous,
  absent). `PIPELINE-CONTEXT: suite=uvm-core files=174 local_includes=31 include_dirs=1 …`,
  `suite=opentitan files=4015 local_includes=2043 include_dirs=53 ambiguous_nonlocal_names=29 absent_names=4`.
- `stimuli/run_pipeline_lane.sh <suite>…` — the pipeline twin of the raw lane: every source file through
  `parseability_probe --parse systemverilog F --profile sv_2017 --preprocess <context>`, five statuses
  (`pass` / `fail` / `pp_fail` = rc 3 / `timeout` / `crash`), the dump
  `stimuli/sv/characterization/pipeline_results.tsv` (`.gitignore`d like the raw lane's; columns `suite
  status relpath secs rc furthest origin`), `PIPELINE-LANE:` summary; suite names are the raw lane's own so
  the two join.
- `stimuli/sv/pipeline_vs_raw.py` → the TRACKED `stimuli/sv/characterization/pipeline_characterization.md`:
  the identity block (probe, grammar bytes + semantic digest, generated parser, context, both dumps —
  labels in `IDENTITY-CARRIERS`'s vocabulary), the per-suite table, the raw → pipeline transition matrix
  with `pass → fail` split into HOLLOW raw passes (the rejection lands in an included file) and own-file
  ones, the by-adjudication-class table, the ORIGIN-CONSTRUCT census (the rejecting line read from the tree
  and bucketed), the top origin files / lines, the `pp_fail` reasons, the context.
- Docs: `TOOLBOX.md` (a quick-chooser row + §1.1b), `docs/book/src/diagnosing-unknowns.md` (the index row),
  the SV parser book chapter (§*What the pipeline does today, measured* rewritten around the numbers).

**THE MEASUREMENT** (`stimuli/sv/characterization/pipeline_characterization.md`; release probe
`423042d2…` = the parser; `sv_2017`; 8-way, 60 s deadline; **45 s wall, 7.2 GB peak** under the guard):

```text
PIPELINE-LANE:   files=4157 pass=1097 fail=3025 pp_fail=35 timeout=0 crash=0
PIPELINE-VS-RAW: suites=opentitan,uvm-core rows=4157 raw_pass=850 pipeline_pass=1097 pipeline_fail=3025 pp_fail=35

suite      rows  raw pass  pipeline pass  fail  pp_fail  Δ pass
opentitan  3983       817           1069  2879       35    +252     fail→pass 324 · pass→fail 68 (ALL hollow) · fail→pp_fail 31 · pass→pp_fail 4
uvm-core    174        33             28   146        0      −5     fail→pass 1   · pass→fail 6  (ALL hollow)
every row: deferred:chained_only in the raw answer key (4 157 of 4 157) — the population that had NO verdict

origin construct (the rejecting line, read from the tree):   class … extends <base>  2048 (67.7 %)
  other 790 · parameter port list with a type default 80 · `::`-scoped name 42 · uvm_* type 22 ·
  typedef 21 · subroutine header 19 · `const local … = new(…)` 2 · rand/constraint 1
pp_fail reasons: `include of a name that exists under several top_* variants 23 · a `define formal list
  spanning lines 4 (`.5`(a)) · `include `MACRO 1 (`.5`(c)) · a conditional inside a macro body 1 (`.5`(b))
  · 6 more unresolved includes of the absent / vendor kind
```

⭐ **THREE READINGS.** (1) The pipeline gives a verdict to a population the raw lane could not judge at
all, and it moves in BOTH directions honestly: +324 rows recognised (RTL and DV files whose only barrier
was a macro or an `include`), and 74 raw passes REVOKED — every one hollow, a file that only `include`s
others, accepted raw because the directive is opaque; not one `pass → fail` lands in the row's own
file. (2) The engine's own refusals are **6 of 4 157** (`.5`(a)–(c)); the other 29 `pp_fail`s are the
CONTEXT — `.4b`(ii). (3) **The barrier behind the macro barrier is now a number**: 2 048 of the 3 025
remaining rejections (67.7 %) sit on a `class X extends Y` header whose base is declared in another
file of the unit — the cross-file DECLARATION fact the store gate refuses (`SV-PARSE-STRICT.2` escaped it
for net types, not class bases; `SV-CORPUS-GRAD.3.57`'s first row is this shape), and the next 80 + 42 +
22 + 21 are the same fact in other spellings. The library flags exist for it; how a per-file lane
supplies it (a UVM library via `--lib-out`, package files parsed first) is `.6`'s design question, and
the size of the prize is known before it is designed.

**Acceptance checklist (`TASK-ACCEPTANCE`):**

  - [x] **REPRODUCE / ISSUE** — the raw lane's `adjudication_manifest.tsv`: 4 157 rows `deferred:chained_only` (every UVM and OpenTitan row — no verdict), `stimuli/sv/characterization/adjudication_summary.md` `deferred:chained_only | 4927`; the raw runner never preprocesses (`parse_one()` in `stimuli/run_external_corpus.sh:400`); the census `git grep -n -E '\.core\b|\+incdir\+|fusesoc' -- rust/src rust/scripts scripts 'stimuli/*.sh' 'stimuli/sv/*.py' rust/Makefile` → 1 line, a self-test fixture (`.1`).
  - [x] **ROOT CAUSE (WHY + WHERE)** — ops family: `git ls-files stimuli | grep -c pipeline` → 0 before this leaf (no lane, no context, no report existed); WHERE `stimuli/run_external_corpus.sh` (raw `--parse` only) and the adjudicator's `preproc_dependency()` / `svpp_can_explain_failure()` (a TEXT test standing in for a preprocessor). WHY the population needs a context: the `.1` census read 71 of 78 refusals as include-PATH gaps, and this leaf's derivation shows the path is a property of the suite (53 shared directories, 29 ambiguous names) rather than of the engine.
  - [x] **FIX** — instrument tier: the derivation, the lane, the report, the context TSV, the tracked report; docs.
  - [x] **ADDRESSED (verified)** — `PIPELINE-LANE: files=4157 pass=1097 fail=3025 pp_fail=35 timeout=0 crash=0 wall_s=45 probe=rust/target/release/parseability_probe timeout_s=60 jobs=8` and `PIPELINE-VS-RAW: … raw_pass=850 pipeline_pass=1097 … missing_raw=0` (every pipeline row has its raw twin); the transition matrix, the hollow split (74 / 0), the origin-construct census (2 048 `extends`) as quoted above; `pipeline_context.py --self-test` → `checks=6 failed=0`; `--check` → `OK`.
  - [x] **NO REGRESSION** — no Rust, grammar, codegen or generated byte moved (`generated-reproducibility: OK` at commit; the release probe's fingerprint equals the on-disk parser `423042d2…`); the raw lane's tracked artifacts are untouched (`results.tsv` sha `5fa726cd…` in the report's identity block, the same the raw report records); the smoke on `uvm-core` with the DEBUG probe and the release run agree row for row on every verdict (28 / 146, the same six hollow passes); every `IDENTITY-CARRIERS` row of the new report re-hashes on this tree (`scripts/check_identity_carriers.sh --report`); `bash scripts/check_doctrines.sh` ALL PASS (run at commit).
  - [x] **LOCKSTEP** — `TOOLBOX.md` §1.1b + chooser row, `docs/book/src/diagnosing-unknowns.md` index row, the SV parser book chapter (measured section), the tracked report, this leaf, `docs/TASK_TREE.md`, `MEMORY.md`, `CHANGES.md`, `DEVELOPMENT_NOTES.md`; the integration contract's front-end section stays `.7`'s (stated, not waived).
  - promotion: declined (the `.4` lesson — a raw pass on an include hub is hollow, read the split not the delta — is recorded in the report's own prose and TOOLBOX §1.1b; no new card).

## Open questions

- 🔎 **DIRECTOR CALL (2026-09-09, from `.3`): `const local T x = new(…)` — UVM's spelling is not Annex-A
  derivable; PGEN rejects it since `.3.37` (2026-09-04) and accepted it before only by mis-lexing `local`.**
  The letter of A.2.7 (both editions) admits only a `constant_expression` after a `const`-first class
  property, while `data_declaration` admits a `class_new` — so `local const T x = new()` parses and
  `const local T x = new()` does not, a syntactic asymmetry the standard itself carries and every
  simulator smooths over. UVM 2020.3.1 writes the non-derivable order (`uvm_transaction.svh:443`), so
  `uvm_pkg.sv` cannot pass on the strict default, and with it nothing built on UVM. Options: (a) a
  documented tool-extension arm — `const { class_item_qualifier } data_type const_identifier = class_new ;`
  — ON by default with a deviation record (the de-facto standard library is the corpus every consumer
  brings), or (b) OFF by default behind the `SV-PARSE-STRICT` policy knob, so the strict profile keeps
  the letter and a `--sv-extensions` flag admits UVM. Recommendation: (a) for the release — the north star's
  *"100 % LRM-compliant by default"* was written against over-acceptance of INVALID text, and this text is
  what the standard's own §8.19 prose describes as a constant with an initial value; recorded here so
  the decision, either way, is a ruling and not a drift. Owned by `.5`(d); until ruled, `.4`'s lane
  counts `uvm_pkg` and its dependants as rejects at this one origin.

- ⚠️ **The preprocessor's 26 pre-existing unit tests write their scratch files to `std::env::temp_dir()`**,
  which on this host is off the repository volume — against the data-locality rule
  ([[project_north_star]]: data stays on the repo volume). Found by `.2` when adding tests; the nine new tests
  use `create_repo_scratch_dir` (under `rust/target/svpp_unit_tests/`). Owed: move the 26 to the same helper
  (a test-only change, one slice, no behaviour) — routed to `.5` as a housekeeping item so it is not lost.

- ✅ **The transformation-model fork and the output-form question are OVERTAKEN by measurement (2026-09-09,
  `.1`).** An expansion engine producing expanded source TEXT plus a byte-indexed source map has existed since
  2026-02-27 and composes with the SV parser (14/14 curated cases). The open question is no longer *which model
  to build* but *whether the built one's residual defects justify the 2026-06-09 re-platforming* — answered
  by `.4`/`.5`'s counts, see the `.1` record above. The three bullets below are kept as written (the record of
  what was asked), superseded by that reading.

- **Grammar home: enhance `svpp.ebnf` vs. a separate "expression parser" EBNF? (director 2026-06-08).**
  RECORDED ANALYSIS + LEAN (to confirm at `.1` with SOTA): expansion is ~90% TRANSFORMATION LOGIC over the
  structure svpp ALREADY parses — macro table (`pp_define`/`macro_formals`/`macro_body` incl. stringize +
  token-paste atoms), conditional STRUCTURE (`pp_conditional`), and `` `include `` paths need NO grammar
  work; substitution semantics, conditional evaluation, and include inlining are engine logic, NOT grammar.
  The ONE genuine grammar gap is recognizing **macro-usage sites** (`` `NAME `` / `` `NAME(args) ``) inside
  the body/non-directive text — today buried in opaque `non_directive_text`. That is a LEXICAL-level
  enhancement to `svpp.ebnf` (split `non_directive_text` into `(plain_text | macro_usage)*` + add a
  `macro_actual_args` rule), NOT a full SV parse and NOT a separate grammar. **LEAN: enhance `svpp.ebnf`; a
  separate "expression parser EBNF" is almost certainly unnecessary** because (1) the macro/usage model is
  lexical and belongs in svpp's domain (EBNF-single-source-of-truth), and (2) standard SV preprocessor
  conditionals (`` `ifdef ``/`` `ifndef ``/`` `elsif ``) take a single **macro identifier**, NOT a C-`#if`
  boolean expression (IEEE 1800-2017 §22.6) — so there is no expression language to parse. ⚠️ SCOPING CHECK:
  svpp's current `condition_expr` models a RICHER form (`||`/`&&`/`!`/`?:` atoms) than the standard —
  confirm at `.1` whether that is a deliberate tool-extension or over-modeling.
- **Transformation model (the deeper fork under the grammar question): parse-tree vs. token-stream.** A
  parse-tree transformation walks the (enhanced) svpp AST — PGEN-native, EBNF-as-source-of-truth, reuses
  svpp (the LEAN). A token-stream transformation is the industry SOTA (slang / Verible / Verilator
  preprocess at the token level, because macros can expand across structural boundaries and preprocessing
  is inherently lexical). `.1` SCOPING decides this with IEEE 1800 §22 + the slang/Verible/Verilator
  implementations read FIRST (research-grounded discipline) — do not pick speculatively.
- Output form: expanded source TEXT (re-fed to the sv parser) vs. an expanded token stream handed to the sv
  parser directly — decide at `.1` with provenance/source-map requirements in mind (tied to the
  transformation-model fork above).
- Reuse boundary: how much of the substitution/resolution machinery can be parser-agnostic (a general
  directive-AST→expansion service) vs. SV-specific.

## Blockers

- None. The 2026-06-08 sequencing gate (build after the locked program) is SUPERSEDED by the director's
  2026-09-09 steer, which activates this work as part of the SV release path.
- ⚠️ Build-mode note, not a blocker: the release `parseability_probe` is ABSENT and the debug one is STALE
  (fingerprint `a1ccdf4a…` vs parser `423042d2…`) at activation; `.3` rebuilds it, and the corpus lane
  (`.4`) needs the release build (≈22 min / 12 GB peak under the guard).

## Verification log

| Date | Leaf | Checks | Result |
| --- | --- | --- | --- |
| `2026-06-08` | (tree creation) | director directive captured + tool-verified gap (svpp parses directives but does not expand; sv does not expand, tolerates `compiler_directive` inline; no composition stage) | tree registered (`PGEN-SVPP-EXPANSION-0001`, docs-only); build deferred behind the locked program |
| `2026-09-09` | `.1` | `rust/target/svpp_e1/pp_census.sh` (preprocess-only, 4 157 UVM + OpenTitan rows, current `ast_pipeline`): `files=4157 rc0=4079 rc_nonzero=78 timeouts=0`; the 78 classified by message (71 include-not-found / 4 formal-list / 2 conditional / 1 include-form); `rust/target/sv_external_corpus_triage_gate/summary.txt` (2026-08-24): 14/14; lane report `adjudication_summary.md`: `explained_svpp_*` 1 434, `chained_only` 4 927, `svpp_owned` 173 | `.1` DONE — the pipeline's missing pieces are the six named items, the engine refuses 7 of 4 157 on its own account; leaves `.2`–`.7` owned |

## Commit log

| Leaf | Commit subject or reference | Notes |
| --- | --- | --- |
| (tree creation) | `PGEN-SVPP-EXPANSION-0001` | tree + decision record created; registered in `docs/TASK_TREE.md` Proposed Task Trees; sequenced after the locked program |
| `.1` | `PGEN-SVPP-EXPANSION-0002` | tree ACTIVATED on the director's 2026-09-09 steer; `.1` scoping re-measured (docs only); leaves re-planned `.2`–`.7`; decision record [[project_preprocess_parse_pipeline_is_the_sv_deliverable]] |

## Changelog

- `2026-06-08` — tree created (`proposed`, own-now/build-later) per the director directive; decision record
  [[project_svpp_expansion_stage_for_nexsim]] written; sequenced strictly after the locked program.
- `2026-09-09` — ACTIVATED by the director's steer (`PGEN-SVPP-EXPANSION-0002`); `.1` SCOPING done on
  measurement (the expansion engine exists since 2026-02-27; 14/14 curated composition; 98.1 % of UVM +
  OpenTitan preprocesses alone; six missing pieces named; three engine defects named); leaves `.2`–`.7`.

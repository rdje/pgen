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
  Status: `pending` — **FRONTIER**
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
  Status: `pending`
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
  Status: `pending`
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
  named tool-extension class (a policy knob, default OFF under [[feedback_sv_strict_lrm_compliance_default]]).
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
| 1 | `.2` | `pending` | the compilation context is what every later leaf consumes: without predefines and unit-level macro continuity the chain (`.3`) parses a context no real front end would give it, and the lane (`.4`) measures the wrong thing |
| 2 | `.3` | `pending` | the single-command chain + source-mapped diagnostics; the corpus lane drives it |
| 3 | `.4` | `pending` | the measurement the director asked for: end-to-end recognition on UVM + OpenTitan, then the whole lane |
| 4 | `.5` | `pending` | the three census defects (and whatever `.4` adds), toolbox chain each |
| 5 | `.6` | `pending` | the deferred population re-adjudicated; the axis-2 denominator moves |
| 6 | `.7` | `pending` | contract + book lockstep, closes the tree |

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

## Open questions

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

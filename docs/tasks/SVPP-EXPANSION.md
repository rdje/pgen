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
  Status: ✅ **`done`** 2026-09-09 (`PGEN-SVPP-EXPANSION-0007`; INSTRUMENT + DOCTRINE tier, ZERO Rust / grammar /
  codegen / generated bytes; record + checklist below) — (i) the whole lane, (ii) the per-file context by
  NEAREST CANDIDATE (the `.core`-reading form stays owed, see the record), (iii) the `PIPELINE-LANE-CURRENCY`
  doctrine (registry 38 → 39)
  Goal: (i) the WHOLE lane — every suite of the 16 336-row corpus through the pipeline, the context
  derivation extended to every suite (`SUITES` in `pipeline_context.py`), the report re-rendered; (ii) the
  PER-FILE context from the suites' own declarations — OpenTitan's 818 `.core` files and ibex's `.f` lists
  with `+incdir+` — so the 23 `pp_fail` rows whose `` `include `` name exists under several `top_*`
  variants get the directory their build system would give them; (iii) a currency gate for the tracked
  report (the identity block is already checked by `IDENTITY-CARRIERS`; a `--check` of the context is
  wired into the runner — what is missing is the doctrine row that re-derives the report's headline from
  the dump, `CORPUS-CENSUS-CURRENCY`'s shape).
- ID: `SVPP-EXPANSION.5`
  Status: ✅ **`done`** 2026-09-09 for (a) (b) (c) (e) and a SIXTH item (f) found by the fix's own test
  (`PGEN-SVPP-EXPANSION-0006`; ENGINE tier — `rust/src/sv_preprocessor.rs` only, ZERO grammar / codegen /
  generated bytes; record + checklist below), and for (g) (h) — the two items the whole lane `.4b` found —
  plus (g′), a SILENT mis-expansion (g) exposed (`PGEN-SVPP-EXPANSION-0008`; same tier; second record +
  checklist below). (d) stays OPEN as the director call it is. **(i) — the ORDER around a ``` `` ``` paste — ✅ done
  2026-09-15 (`PGEN-SVPP-EXPANSION-0025`; ENGINE tier, `rust/src/sv_preprocessor.rs` only; third record + checklist
  below): a macro usage that is an OPERAND of a paste is expanded before the join, and an UNDEFINED usage joined by
  a paste is looked up as the WHOLE constructed name, in place, with its arguments still unexpanded (sv2v
  `test/lex/macro_macro.sv`, the `.6b.4` disposition that named this item).**
  **(j) — a `` `" `` string region: "the expansion shall include the quotation mark, substitution of actual arguments, and
  expansions of embedded macros" (IEEE 1800-2017 §22.5.1 `:366-368`) — ✅ done 2026-09-15 (`PGEN-SVPP-EXPANSION-0026`;
  ENGINE tier, `rust/src/sv_preprocessor.rs` only; record + checklist below): an embedded macro usage inside a region is
  expanded, and an OBJECT macro's region is rendered as a string at all — both readings shared by Verilator 5.050 and
  Icarus 13.0 and found by `.5`(i)'s three-way probe; a region in a SOURCE line stays verbatim (the form is defined for
  macro text, and sv2v `string_macro.sv:9` is pinned must-reject on exactly that).**
  **(k) — a formal inside a plain `"…"` string in macro text — in progress 2026-09-15 (`PGEN-SVPP-EXPANSION-0027`): IEEE
  1800-2017 §22.5.1 `:370-372` says the characters of a plain string literal in macro text "become part of the string in
  the replaced macro text", so a formal there is NOT substituted (Verilator 5.050's reading); the engine substitutes it
  (Icarus 13.0's). Found by `.5`(i)'s probe (shape `s1`). A CENSUS of the corpus first — how many bodies rely on either
  reading, in which suites — then the ruling, measured.** → ✅ done 2026-09-15 (`PGEN-SVPP-EXPANSION-0027`): the
  standard by default, `paste_delimited` / `all` behind `--sv-plain-string-substitution`; record + checklist below.
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
  Pre-scoping (2026-09-09, measured in `-0008` while its lane ran; debug probe, `.4b` context, scratch
  `rust/target/svpp_e1/s5gh/lib_*`): (i) the library flags REACH the preprocessed path — `command_parse` hands
  `--lib-in` / `--lib-out` to `parse_sample_detail_with_options` after `load_probe_input`, so no plumbing is owed;
  (ii) a package export carries the fact the store gate consults — the 2026-08-24 triage artifact
  `case_uvm_compat_pkg_2017.lib/package/uvm_pkg.facts.json` holds 1 364 `type_name` facts, 766 with
  `declaration_family = class`, and `known_unscoped_base_class_type_identifier`'s predicate is exactly
  `fact_attribute_equals(type_name, $body, declaration_family, class)`; (iii) with that (STALE) library as
  `--lib-in`, `dv_utils_pkg.sv`'s rejection moves `:61:19` (`uvm_reg_addr_t start_addr;`) → `:215:24` (a
  `` `uvm_fatal `` use inside an `if` — the next barrier, to diagnose with the toolbox), while
  `alert_esc_agent_pkg.sv` (`:16` `parameter uint …` — `uint` is `dv_utils_pkg`'s typedef), `dv_base_reg_pkg.sv`
  (`:15`, the same) and `dv_lib_pkg.sv` (`dv_base_env_cfg.sv:7` `#(type RAL_T = dv_base_reg_block)`) do not move:
  they stop on OPENTITAN's own package chain, so the design is a DEPENDENCY-ORDERED export pass over `import
  X::*` (uvm → dv_utils → dv_base_reg → dv_lib → dv_base_agent → the agents → the IP env packages), each
  package parsed as its own unit with `--lib-in` of its predecessors and `--lib-out` of itself; (iv) the
  base-class census of the `.4b` dump (2 243 failing rows whose origin line is an `extends` header): 1 217
  OpenTitan bases are IP-specific (`*_base_vseq`, `*_env_cfg` — the tail of that chain), 607 are `dv_*` /
  `cip_*`, and 405 are `uvm_*` across five suites (OpenTitan 180, sv-tests 103, uvm-core 102, Surelog 12, VeeR 8)
  — the rows a UVM library alone unlocks. ⛔ (v) **A REJECTING `uvm_pkg.sv` EXPORTS NOTHING** — `--lib-out` on
  HEAD (rejects at `uvm_transaction.svh:443:38`, the `.5`(d) construct) writes no `facts.json`; the 2026-08-24
  artifact exists only because the parser of that day mis-lexed `local`. So every UVM-based row of this design
  (the 405 + the whole OpenTitan chain, which imports `uvm_pkg` first) is GATED ON THE DIRECTOR's `.5`(d)
  ruling, or on an export-the-accepted-prefix engine change the ruling would make unnecessary; `.6` can start
  on the non-UVM package chains and the re-adjudication plumbing, and must say so in its record.
  Sub-leaves (2026-09-09): ✅ **`.6a`** the RE-ADJUDICATION INSTRUMENT + tracked report (`-0009`, record below);
  `.6b` the disposition of the 223 `refuted` rows (and the 29 `svpp_owned:rejects_own_file`) one by one with the
  toolbox — parser defect (→ the axis-2 bar, `SV-CORPUS-GRAD`) or answer-key error (→ a pin with the LRM sentence);
  `.6c` an EXPECTED verdict for the `chained_only` rows from their suite's own convention (a design corpus file
  is must-accept as part of its unit; a Surelog multi-file unit carries its `.sl` verdict) so a
  `chained:accepted_under_expansion` row becomes a `match` — then the verdict-coverage instrument consumes the
  re-adjudication table and the `adjudicated/routed/no-verdict/dark/axis-2-bar` tuple moves through
  `SV-CORPUS-DENOMINATOR`'s gate (the live anchor in `grammar-wellformedness.md`); `.6d` the LIBRARY design — a
  dependency-ordered `--lib-out` / `--lib-in` pass over `import X::*` for the 1 924 + 103 `extends` rows (UVM's
  part gated on `.5`(d)).
- ID: `SVPP-EXPANSION.7`
  Status: ✅ `done` 2026-09-09 (`PGEN-SVPP-EXPANSION-0021`) for the tree's current stop — re-run in lockstep if a later slice moves the front end; the tree's remaining leaf `.6d.3` is BLOCKED on the director's (d)
  Goal: the CONTRACT + BOOK statement of the front end — the SV integration contract's front-end section
  (svpp expansion → sv parse, the context flags, the source-map contract), the svpp parser book's status
  (expansion is a sibling deliverable, not the directive parser's), the platform book's family page, the
  DONE-BAR register if a family row's next gap changes.

## Current frontier

| Order | Leaf | Status | Why next |
| --- | --- | --- | --- |
| — | `.5`(i) | ✅ `done` | `PGEN-SVPP-EXPANSION-0025`: the ORDER around a paste — a usage that is a paste operand is expanded first, an undefined usage joined by a paste is the whole constructed name, looked up in place with its arguments unexpanded; sv2v `macro_macro.sv` `fail → pass`; the whole paste-carrying population (188 expansions) diffed before/after — exactly one moved; measured against Verilator 5.050 and Icarus 13.0 (they split; the standard's sentence and sv2v's reference decide) |
| — | `.5`(j) | ✅ `done` | `PGEN-SVPP-EXPANSION-0026`: the `` `" `` region — one renderer in the rescan for both macro paths (an embedded usage expanded, an object macro's region rendered, `` `\`" `` a quote, a rendered literal one line), `()` one empty actual; FOUR OpenTitan rows `fail → pass`, tuple `9140/2606/4590/3310/5`; the whole corpus diffed (46 of 16 388 expansions moved, all renderings); measured against Verilator 5.050 and Icarus 13.0 with the four splits recorded |
| — | `.5`(k) | ✅ `done` | `PGEN-SVPP-EXPANSION-0027`: a plain `"…"` string in macro text is OPAQUE by default (§22.5.1 `:370-372` and its own `Hello, x`; Verilator's reading) — the census read 66 prose sites the engine had been MANGLING against 6 paste-delimited idiom sites; the two tool readings are the named policies `paste_delimited` (OpenTitan's idiom) and `all` (Icarus's) on `--sv-plain-string-substitution` |
| — | `.7` | ✅ `done` | `PGEN-SVPP-EXPANSION-0021`: the SV integration contract's *Front End — the preprocess→parse pipeline* section (the command, the four-part context, the expansion guarantees and refusal codes, the source-map rule with the last-mapped-byte caveat, the library's facts files), the preprocessor contract's *Runtime Execution Stage* section (expansion = a sibling deliverable), the svpp book's status note, the platform page's *read twice* note, the user guide's single command; the two parser books' tracked HTML mirrors rebuilt (they had lagged four commits; the platform book's HTML is untracked output) |
| — | `.6d.3` | ✅ `done` | `PGEN-SVPP-EXPANSION-0029`: `uvm_pkg.sv` EXPORTS (release `1.0.247`'s lane run: `published_packages=256`, `uvm-core=2/2`), its 168 `include`d class files read through their unit — `unreached_in_unit` 139 → 0, `accepted_in_unit` 2 → 142 — and the tuple under the pipeline +143 adjudicated; the 2026-09-09 price re-measured: the OpenTitan DV packages had been converted BEFORE the ruling by the fact-gate escapes (140 of 186 pass, before and after), 48 still reject at their own first barrier — an unexpanded `` `uvm_fatal `` behind `` `ifdef UVM `` (→ `.6d.4`) |
| — | `.6d.4` | ✅ `done` | `PGEN-SVPP-EXPANSION-0030`: OpenTitan's build-wide `UVM` symbol as a hand-curated suite define (`HAND_DEFINES`, one row in `pipeline_context.tsv`, source `common_sim_cfg.hjson:65`); `dv_utils_pkg.sv` includes 22 files instead of 2 and passes; the whole lane re-run: `PIPELINE-MOVERS … moved=10` (all OpenTitan `fail → pass`), the library 256 → 264 packages, OpenTitan members `accepted_in_unit` 878 → 999 / `unreached_in_unit` 960 → 846, the tuple under the pipeline `10439/…` → `10570/2606/3160/1874/0`; the 38 packages that remain reject at 38 distinct grammar / fact-gate origins past the macro barrier (the SV lane's) |
| — | `.8` | ✅ `done` | `` `begin_keywords `` / `` `end_keywords `` (IEEE 1800-2017 §22.14) — the preprocessor renders every token that is a keyword of the parse profile but NOT of the region's version as an ESCAPED identifier (§5.6.1), from the LRM's own per-version tables (Tables 22-1 … 22-6); five rows of the axis-2 bar (Surelog `1364_2005/dut.v`, ispras `22.14.01_01.sv` + `22.14.01_04.sv`, sv-tests `5.6.4--compiler-directives-begin-keywords.sv`, sv2v `test/lex/begin_keywords.sv`); opened 2026-09-14 by `SV-CORPUS-GRAD.3.60`'s reconnaissance (`PGEN-SV-CORPUS-GRAD-0394`), DONE the same day (`PGEN-SVPP-EXPANSION-0022`): 68 pipeline rows moved and every one carries a §22.14 directive — 67 `fail → pass`, 1 `pass → fail` (the clause's own negative example), the bar `9 → 8`; the tables DERIVED from three in-repo standards and gate-held by the new doctrine `SV-KEYWORD-VERSIONS` |
| — | `.6d.2` | ✅ `done` | `PGEN-SVPP-EXPANSION-0020`: the unit-membership reading — the pass records each unit's member files (the source map; 5 074 rows), the re-adjudication reads a member's verdict through its unit (`accepted_in_unit` / kept / `unreached_in_unit`); measured: 0 accepted, 1 923 unreached, 89 kept — all behind the UVM gate; the tuple unmoved |
| — | `.6d.1` | ✅ `done` | `PGEN-SVPP-EXPANSION-0019`: the LIBRARY PASS — 407 package files of the design suites parsed in dependency order, 166 packages published, every row run with `--lib-in`; 425 rows `fail → pass`, 0 the other way (OpenTitan 1 077 → 1 444, VeeR 23 → 81); 🔎 164/168 rejecting OpenTitan packages import `uvm_pkg` (the (d) gate); six per-top register packages disagree (not published); 44 generated `*_ral_pkg` imports absent from the tree |
| — | `.4c.2` | ✅ `done` | `PGEN-SVPP-EXPANSION-0018`: the unit predecessors a file's OWN harness compiles first, derived per file (`pipeline_context_units.tsv`, 44 rows on 17 files; `-f` lists, globs, `--siloed` and ordering conflicts REPORTED); the whole lane re-run — exactly 3 movers, all `fail → pass`; 🔎 a `.6b.4` disposition had inferred a pairing no declaration makes — corrected forward; `.4c` CLOSED |
| — | `.4c.1` | ✅ `done` | `PGEN-SVPP-EXPANSION-0017`: the defines a file's OWN driver passes, derived per file (`pipeline_context_defines.tsv`, 56 rows; computed values and ties REPORTED, C++ / iverilog spans and helpers skipped); the whole lane re-run — exactly 5 movers (3 `fail → pass`, 2 `pass → fail`), 3 rows pinned `must_reject` with a reproducer/control each; 🔎 a driver define can select a vendor-extension branch |
| — | `.6c.2` | ✅ `done` | `PGEN-SVPP-EXPANSION-0016`: the 28 key-question rows pinned `must_reject` with the clause and a reproducer/control each; the raw key demotes them, the pipeline cashes them (433 `must_reject:match_under_expansion`) |
| — | `.6c.1` | ✅ `done` | `PGEN-SVPP-EXPANSION-0015`: the verdict-coverage tuple READ UNDER THE PIPELINE, published beside the raw one through `SV-CORPUS-DENOMINATOR`'s own gate — `7529/2606/6201/3994/9` raw, `8669/2606/5061/3779/30` under the pipeline |
| — | `.6b.5` | ✅ `done` | `PGEN-SVPP-EXPANSION-0014`: the four engine rules — 10 rows `fail → pass` (+1 refusal → pass), 0 regressions; the UVM expansion CORRECTED (the 2026-08-24 artifact honoured `` `define ``s inside block comments) |
| — | `.6b` | ✅ `done` | `.6b.1`–`.6b.4`: every one of the 223 refuted rows dispositioned with evidence — 34 pinned, 11 recognised by four engine rules, 183 in the table (103 cross-file base class, 27 key questions, 21 parser questions routed to `SV-CORPUS-GRAD`, 10 preprocessor items, 9 vendor-identity branch, 9 driver defines, 4 cross-file defines); `refuted_awaiting_disposition=0` |
| 2 | `.6c` | `pending` | expected verdicts for `chained_only` from the suite convention; the coverage tuple moves through the denominator gate |
| 3 | `.6d` | `pending` | the dependency-ordered library pass (1 924 + 103 `extends` rows); UVM's part waits on (d) |
| — | `.6a` | ✅ `done` | `PGEN-SVPP-EXPANSION-0009`: the re-adjudication instrument + tracked report; the deferred population has its first measured verdicts under the pipeline |
| 2 | `.7` | `pending` | contract + book lockstep, closes the tree; (d) waits for the director |
| — | `.5`(g)(h)(g′) | ✅ `done` | `PGEN-SVPP-EXPANSION-0008`: a missing actual, an extra actual and a recursive macro are REFUSED at the preprocess stage with the standard's sentence; the apostrophe-as-quote misreading of a formal list fixed — the census back at 73, the three timeouts gone |

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
  (`create_repo_scratch_dir`) — and so do the 26 pre-existing ones since `.5`(e) made `create_temp_dir` a
  shim over the same helper (`grep -c 'temp_dir()' rust/src/sv_preprocessor.rs` = 0 at HEAD; the
  *Open questions* entry that said otherwise is corrected).
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

## `.5` the preprocessor DEFECTS — ✅ done 2026-09-09 (`PGEN-SVPP-EXPANSION-0006`) for (a) (b) (c) (e) (f); (d) RULED AND CLOSED 2026-10-01 (option (a), release `1.0.247`, `SV-CORPUS-GRAD.37`) — it was the director's

**What landed** (`rust/src/sv_preprocessor.rs`; ZERO grammar / codegen / generated bytes; 7 new tests, 42/42):

- **(a) a `` `define ``'s formal-argument list may span lines with no backslash.** IEEE 1800-2017 §22.5.1
  (`section-22-compiler-directives.md`, the clause's *"If formal arguments are used …"* paragraph): *"The
  formal argument names shall be simple_identifiers, separated by commas and optionally white space"*, and
  §5.3's white space includes newlines. `define_formal_list_unclosed` keeps appending lines until the
  list's `)` closes; only then does the macro TEXT's backslash-newline rule apply. Test
  `a_define_formal_list_may_span_lines_without_a_backslash` (the OpenTitan `cip_base_vseq__tl_errors.svh:8`
  shape, defaults with a nested `#(…)`).
- **(b) a conditional directive is honoured wherever it appears.** IEEE 1800-2017 §22.6 (`:5` of the
  clause): *"These directives may appear anywhere in the source description."* Before: recognised at a
  line's start only, so (i) a macro whose body OPENS with `` `ifdef `` broke when used mid-line inside
  another macro's body (OpenTitan `prim_assert.sv`'s `ASSERT_ERROR` via `tlul_assert.sv`; ibex
  `top.sv` → `compare_helper.sv`) — the `` `ifdef `` landed mid-line and was ignored while the `` `else ``
  at a line start was honoured, hence *"`else without matching"*; (ii) `assign a = `ifdef B b `else c
  `endif ;` in plain source passed through VERBATIM; and (iii) — found by the before-state measurement —
  `` `ifdef X // a trailing comment `` read the symbol as `X // a trailing comment`, found it undefined
  and took the `` `else `` branch with `X` DEFINED, silently under the default policy. Now:
  `first_midline_conditional` splits a line at the first conditional directive that follows text
  (outside strings and comments; a `` `define `` line is exempt — its conditionals belong to the macro
  text), the text before it is its own logical line, the directive and its remainder the next, at the
  same line number (`pending_tail`); the five arms take their ARGUMENT — an identifier, or
  `[!]defined(SYM)` for `` `elsif `` — and hand the remainder on as text, consuming a trailing comment
  (`split_conditional_argument`, `split_conditional_expression`, `remainder_as_pending`). Tests
  `a_conditional_directive_may_appear_mid_line`,
  `a_macro_body_starting_with_a_conditional_expands_when_used_mid_line`,
  `a_trailing_comment_after_a_conditional_symbol_is_not_part_of_the_symbol` (under the `error` symbol
  policy, so any misread symbol would now refuse).
- **(b′) an expansion's lines are the USE line.** `preprocess_text_internal` takes `line_base`; the
  re-entry on a macro expansion passes the use line, so events, diagnostics and — the half `.3`'s origin
  mapping depends on — the source-map chunks of a re-entered expansion name the line of the use, never
  `1..n` of the expansion. Test `a_re_entered_expansions_diagnostic_names_the_use_line` (an unmatched
  `` `else `` inside a body used on line 8 reports `top.sv:8`, not `:3`).
- **(c) RULED: `` `include `MACRO `` stays refused, naming the standard.** IEEE 1800-2017 §22.4 Syntax
  22-1 admits `` `include "filename" `` and `` `include <filename> `` only; a macro-valued path is a tool
  extension PGEN does not admit under [[feedback_sv_strict_lrm_compliance_default]]. The refusal now
  reads *"… (IEEE 1800-2017 §22.4 Syntax 22-1: … a macro-valued path is a tool extension PGEN does not
  admit); got '`REQUESTS_FILE'"*. Test `include_of_a_macro_valued_path_is_refused_naming_the_standard`.
  OpenTitan `aes_tb_pkg.sv:39` is the one corpus row; it stays `pp_fail` by ruling.
- **(e)** every preprocessor test's scratch directory is on the repository volume — `create_temp_dir`
  delegates to `create_repo_scratch_dir` (`rust/target/svpp_unit_tests/`), so the 26 older tests moved
  without being rewritten.
- **(e′) formatting.** `rust/src/sv_preprocessor.rs` is `rustfmt --edition 2024 --check` clean after this
  leaf (it carried 80 pre-existing drift lines in 7 hunks; the session's additions had raised it to 313 in 18,
  so the file was formatted wholesale — deterministic, no semantic change, 42/42 after). ⚠️ The repository
  does not enforce rustfmt; `rust/src/bin/parseability_probe.rs` (6 pre-session hunks → 16 after `.3`) and
  `rust/src/main.rs` (22 → 24 after `.2`) are NOT reformatted here — a wholesale format of those two changes
  committed pre-existing code and is a housekeeping slice of its own, routed to `.4b`'s tidy list.
- **(f) an EMPTY actual argument takes the formal's default** — found by (a)'s test, which used the
  OpenTitan call `` `MK(t1, 0, , sqr) `` and got `seq_t_` bound to nothing. IEEE 1800-2017 §22.5.1's own
  examples (`section-22-compiler-directives.md:251-267`): `` `MACRO1 ( , 2, 3 ) // argument a omitted,
  replaced by default ``, `` `MACRO3 ( ) // all arguments replaced by defaults ``. `expand_function_macro`
  now treats a whitespace-only actual as omitted when the formal has a default (a formal without one
  still binds to ""). Test `an_empty_actual_takes_the_formals_default` (the LRM's MACRO1 / MACRO3 lines
  verbatim). ✅ **(g) — DONE in `-0008`, record below** — the OTHER half of that example: `` `MACRO2 (1) // ILLEGAL: b requires an
  argument `` (`section-22-compiler-directives.md:257-266`) — an omitted actual for a formal WITHOUT a
  default is an ERROR the preprocessor must raise, and today it binds `""` and hands the parser text it
  rejects (whole-lane `.4b`: sv-tests `22.5.1--define-expansion_7.sv` / `_8.sv`, both
  `:should_fail_because:`, read `pass → fail` with the rejection in the row's own file — the right verdict
  at the wrong stage). ✅ **(h) — DONE in `-0008`** — a RECURSIVE macro was expanded exponentially until the depth guard (16)
  stops it — verilator `t_pp_circdef_bad.v` (`` `define SEL_NUM_BITS `WIDTH-`SEL_NUM_BITS +: `SEL_NUM_BITS ``,
  two self-references ⇒ 2^16 growth) TIMES OUT at 60 s, `t_pp_circ_subst_bad.v` (`` `define e fun `e ``)
  leaves `` `e `` for the parser to reject; the standard's rule on recursive text macros is the sentence
  to cite when fixing. Both routed here from `.4b`, each a small engine slice with a reproducer in hand. ⭐ It moves one of the three `.2` smoke expansions — pre-registered by the test before the
  corpus was touched: `dv_utils_pkg.sv:296` `` `DV_CHECK_STRNE_FATAL(text, "", , $sformatf("%m")) `` now
  expands its empty third actual to the formal's default `""` (`…, "", ""))` where it read `…, "", ))`,
  a parse error the raw lane never reached).

**Verification** (`rust/target/svpp_e1/verify_s5.sh`, log `verify_s5.log`, on the rebuilt debug binaries):

```text
(1) the .1 census re-run, same include path:  files=4157 rc0=4084 rc_nonzero=73   (was rc0=4079 rc_nonzero=78)
    refusals after: 72 include-not-found (71 as before + top.sv, which now reaches its ABSENT `../build/psgen-jg.sv`
                    having got past its `endif) · 1 the (c) ruling (`include `REQUESTS_FILE)
    rows whose rc moved 1→0: cip_base_pkg.sv, cip_base_vseq__tl_errors.svh, cip_base_vseq.sv, cip_seq_list.sv (a),
                    tlul_assert.sv (b) — exactly the five pre-registered, 0 the other way
(2) byte-identity: uvm_pkg.sv 3 022 806 B IDENTICAL to the 2026-08-24 gate artifact · uart.sv IDENTICAL ·
    dv_base_env.sv IDENTICAL · dv_utils_pkg.sv DIFFERS in ONE line — the (f) fix, pre-registered
(3) the census's files: cip_base_pkg.sv rc 0 (511 465 B, 35 includes) · tlul_assert.sv rc 0 (40 923 B) ·
    top.sv rc 1 on the absent `../build/psgen-jg.sv` (the next barrier, not this one) · aes_tb_pkg.sv rc 1 by ruling
(4) the (b) reproducers through the single command: b_nested / b_midline / b_direct all `parse_full passed … [preprocessed]`
(5) the 14 pre-registered "mid-line" files: 11 expand with no surviving conditional, 2 keep one only inside a `//` comment
    (a false positive of the pre-registration grep), 1 is top.sv's absent include
```

**Acceptance checklist (`TASK-ACCEPTANCE`):**

  - [x] **REPRODUCE / ISSUE** — `rust/target/svpp_e1/pp_census.sh` (`.1`): `rc_nonzero=78`, of which 4 `unterminated macro parameter list` (`cip_base_vseq__tl_errors.svh:8`), 2 `` `else `` / `` `endif `` *without matching* (`tlul_assert.sv:4`, `compare_helper.sv:3` — expansion-local line numbers), 1 `include path must be quoted` (`aes_tb_pkg.sv:39`); minimal reproducers `rust/target/svpp_e1/s5/b_nested.sv` (*"`else without matching `ifdef/`ifndef at …:3"*), `b_midline.sv` (the directives pass through verbatim), `trailing_comment.sv` (`logic no;` emitted with `X` defined; `W_SVPP_INVALID_CONDITIONAL_SYMBOL: X // a trailing comment`).
  - [x] **ROOT CAUSE (WHY + WHERE)** — correctness family: the pre-fix expansion of `b_midline.sv` (the directives passed through verbatim) parsed by the probe → `furthest_position=22`, the byte of the un-honoured `` `ifdef `` in `assign a = `ifdef B …` — the parser's stop names the exact token the preprocessor left behind; `--sv-diagnostics-json` on `trailing_comment.sv` → `W_SVPP_INVALID_CONDITIONAL_SYMBOL` with detail `X // a trailing comment` and `W_SVPP_UNDEFINED_CONDITIONAL_SYMBOL` — the symbol read past its end; `--sv-event-log-json` on `b_nested.sv` shows no `ifdef` event for the mid-line `` `ifdef UVM `` while the line-start `` `else `` is honoured (*"`else without matching"*); ops family: `git log -S'first_midline_conditional' -- rust/src/sv_preprocessor.rs` → 0 commits (no mid-line recognition ever existed). WHERE `preprocess_text_internal` (`parse_directive_line(&initial_trimmed)` on the whole line — line-start only; the five arms taking `rest.trim()` as the symbol; the `define` continuation loop joining backslash-continued lines only; the re-entry passing no line base) and `expand_function_macro` (an empty actual bound before the default was consulted). WHY: §22.6 *"anywhere in the source description"*, §22.5.1 *"separated by commas and optionally white space"* and its MACRO1 / MACRO3 examples, §22.4 Syntax 22-1.
  - [x] **FIX** — engine tier (a preprocessor has no grammar): `first_midline_conditional`, `pending_tail`, `split_conditional_argument` / `split_conditional_expression` / `remainder_as_pending`, `define_formal_list_unclosed`, `line_base`, the empty-actual default, the §22.4 message; 7 tests.
  - [x] **ADDRESSED (verified)** — `verify_s5.log` (1): `rc_nonzero 78 → 73`, the five pre-registered rows `1 → 0` and none the other way; (3) `tlul_assert.sv` rc 0, `cip_base_pkg.sv` rc 0; (4) the three reproducers `parse_full passed … [preprocessed]`; `cargo test --lib sv_preprocessor` → `42 passed; 0 failed` (35 + 7), the trailing-comment test under the `error` symbol policy with `diagnostics` empty.
  - [x] **NO REGRESSION** — `uvm_pkg.sv`'s 3 022 806-byte expansion **byte-identical** to the 2026-08-24 gate artifact; `uart.sv` and `dv_base_env.sv` byte-identical to the `.2`-era outputs; the one moved expansion (`dv_utils_pkg.sv`, one line) is the pre-registered (f) fix; the corpus lane re-run on the FINAL release probe (rebuilt from the final bytes at the 16 GB guard budget, peak 14.2 GB): `PIPELINE-LANE: files=4157 pass=1101 fail=3026 pp_fail=30 timeout=0 crash=0 wall_s=28` against `.4`'s `pass=1097 fail=3025 pp_fail=35` — the per-row diff is EXACTLY the pre-registered classes: 5 `pp_fail → fail` (the four `cip_*` (a) rows and `tlul_assert.sv` (b), each now a parser verdict with an origin) and 4 `fail → pass` (`prim_esc_tb.sv`, `prim_flop_2sync/tb.sv`, `ast.sv`, `ast_main.sv` — the (b) shapes), **0 `pass → fail`, 0 `pass → pp_fail`**; the tracked `pipeline_characterization.md` re-rendered (OpenTitan 817 → 1 073 pass, 69 hollow raw passes, `extends` 2 049 of 3 026); 35 pre-existing preprocessor tests green (42/42); no grammar / codegen / generated byte moved (`generated-reproducibility: OK` at commit); `clippy_on_rust_change` → source stage ok, `GENERATED-CLIPPY-CORRECTNESS: ✅ POLICY-ONLY PASS`, generated stage ok — *"✅ clippy_on_rust_change completed"* (the feature-enabled log's only finding in `sv_preprocessor.rs` is the pre-existing `collapsible_if` at the `define` collection, not this leaf's code); `bash scripts/check_doctrines.sh` ALL PASS (run at commit).
  - [x] **LOCKSTEP** — the SV parser book chapter (the refusal sentence and the measured table re-rendered from the re-run; HTML rebuilt), the tracked `pipeline_characterization.md` re-rendered, this leaf, `docs/TASK_TREE.md`, `MEMORY.md`, `CHANGES.md`, `DEVELOPMENT_NOTES.md`; the svpp CLI docs need no change (no flag moved). ⭐ The acceptance gate's correctness family gains the preprocessor's own instruments — `W_SVPP_*` / `E_SVPP_*`, `preprocess_failed`, `--sv-event-log-json` / `--sv-diagnostics-json` — registered in `scripts/check_diagnosis_evidence.sh` and `TOOLBOX.md`'s family table IN THIS COMMIT, as the toolbox's rule demands (they are what located (b)); the gate's four probe banks re-run: 6/6, 13/13, 9/9, 10/10.
  - promotion: declined (the `.5` lesson — a directive is a token, not a line — is stated in the code and in `DEVELOPMENT_NOTES.md`; no new card).

## `.4b` the WHOLE LANE, the per-file context, the currency doctrine — ✅ done 2026-09-09 (`PGEN-SVPP-EXPANSION-0007`)

**What landed** (ZERO Rust / grammar / codegen / generated bytes):

- **(i) the whole lane.** `stimuli/sv/pipeline_context.py`'s suite roster is DERIVED from the tree (every
  directory under `stimuli/sv/subs/` + `uvm-core`; `--list-suites`), the runner takes `all`, and the context
  covers 14 suites (74 directory rows; friscv, ispras-sv-tests and sv2v need none — their includes are all
  local, which is why deriving the roster from the context TABLE had skipped them, the defect this leaf's
  first run made). ⛔ Its first run also died at its own summary line — *"syntax error near unexpected
  token `then'"* — because the script was EDITED while that bash instance was still reading it (bash
  reads a script incrementally); 13 676 rows of correct work, one garbled line. Recorded in TOOLBOX §1.1b.
- **(ii) the per-file context.** A name that exists under several directories is resolved PER FILE to the
  NEAREST candidate — the candidate sharing the longest path prefix with the includer, what a per-top file
  list does for the `top_earlgrey` / `top_darjeeling` / `top_englishbreakfast` variants of one `seq_lib` —
  written to the tracked `stimuli/sv/pipeline_context_files.tsv` (41 rows: OpenTitan 24 resolving 10 names,
  Surelog 10, verilator 7) and applied by the runner to that file FIRST; a TIE stays ambiguous and reported
  (OpenTitan 19 names, all `ip_templates` vs `top_*` at equal distance). Measured on OpenTitan alone
  against `.5`'s run: `pp_fail 30 → 9`, `fail 2 880 → 2 901`, `pass 1 073` unchanged — 21 refusals became
  parser verdicts, none became passes. ⛔ The `.core`-READING form (FuseSoC YAML: filesets, `depend`,
  `is_include_file`) stays owed: the nearest-candidate rule is derived from the tree and covers 22 of the
  29 ambiguous names; reading the build system's own declaration would cover the 19 ties and the absent
  generated headers, and needs a YAML reader this repository does not carry — routed as `.4c`, priced,
  not built.
- **(iii) the `PIPELINE-LANE-CURRENCY` doctrine** (`scripts/check_pipeline_lane_currency.sh`, registry
  38 → 39, `DOCTRINE_ENFORCEMENT.md` §10, the book's count and derived chapter re-rendered by
  `scripts/report_doctrines.sh`): (1) both tracked context tables equal a fresh derivation
  (`pipeline_context.py --check`, always evaluable, HARD); (2) the tracked report equals a fresh render of
  the two dumps (`pipeline_vs_raw.py --check`, the `parse binary` digest row masked as provenance — the
  reading `IDENTITY-CARRIERS` gives it; the dumps being `.gitignore`d, an absent one is NOT EVALUATED,
  loudly, exit 0). `--self-test` 4/4: a GREEN control on the real tree, a deleted context row refused, an
  edited report number refused, the absent-dump path reported. Founding measurement in the registry text:
  the lane's number moved `1 097 → 1 101` within hours of being published, which is what a doctrine is for.
- The report tool's `--check`, the runner's `all` and per-file rows, TOOLBOX §1.1b, the platform book's
  toolbox index (unchanged row, still accurate), the SV parser book chapter (§*The whole lane*).

**THE MEASUREMENT** (`stimuli/sv/characterization/pipeline_characterization.md`; release probe = parser
`423042d2…`; `sv_2017`; 8-way, 60 s; **155 s wall, 11.4 GB peak**):

```text
PIPELINE-LANE:   files=16336 pass=11673 fail=4401 pp_fail=259 timeout=3 crash=0
PIPELINE-VS-RAW: rows=16336 raw_pass=9847 pipeline_pass=11673 pipeline_fail=4401 pp_fail=259 timeout=3 missing_raw=0
Δ pass by suite: verilator +872 · iverilog +337 · opentitan +256 · ispras +129 · sv2v +91 · scr1 +41 · sv-tests +40 ·
                 Surelog +31 · friscv +23 · verible +9 · VeeR +4 · slang +1 · black-parrot −3 · uvm-core −5
hollow raw passes revoked (rejection in an INCLUDED file): 80 (opentitan 69, uvm 6, slang 2, verilator 2, VeeR 1)
own-file pass → fail: 5 — ALL preprocessor fixtures whose raw pass was hollow:
   sv-tests 22.5.1--define-expansion_7.sv / _8.sv (`:should_fail_because:` — `D() on a two-formal macro, no defaults; → .5(g))
   verilator t_pp_circ_subst_bad.v (`define e fun `e — a circular substitution; → .5(h))
   slang unittests/data/file_uses_defn.svh (its whole expansion is a `__FILE__ string — a fragment)
   verilator t_preproc_debugi.v (`BAR(aa,bb) — owed a read)
timeouts: 3 — Surelog DiffSimpleIncludeAndMacros/top.v, SimpleIncludeAndMacros/top.v, verilator t_pp_circdef_bad.v (→ .5(h))
pp_fail 259: black-parrot 170 (generated headers the vendored tree does not carry — permanent, corpus-owned),
   friscv 32 (hardfloat test includes, same kind), sv2v 18, verilator 13, opentitan 9, iverilog 9, Surelog 4, slang 3, sv-tests 1
by the raw answer key: match 6064 → 5693 pass / 370 fail / 1 refused; deferred:chained_only 4927 → 1526 pass / 3182 fail / 217 refused / 2 timeout;
   explained_svpp_macro_use 1093 → 986 pass; explained_svpp_conditional 198 → 193 pass; explained_svpp_include 139 → 21 pass / 111 fail
origin construct (4 401 rejections): class … extends <base> 2182 · other 1903 · parameter port list 102 · typedef 66 · `::` 63 ·
   subroutine header 41 · uvm_* 26 · new(…) 11 · import 5 · rand 2
```

⭐ **THREE READINGS.** (1) The pipeline lifts the corpus from 9 847 to 11 673 accepted rows (+1 826 net), and
1 434 rows the raw answer key had *explained away* as preprocessing-blocked now carry a real verdict:
986 + 193 + 21 pass, the rest reject on the barrier behind (the `include` class mostly on cross-file
declarations). (2) The 80 revoked raw passes and the 5 own-file movers are the lane doing its job — every
own-file mover is a fixture whose raw pass was wrong, and two of them expose engine items (g)(h). (3) The
denominator of `.6`'s question moved: the cross-file base-class fact is now 2 182 of 4 401 rejections
(49.6 % of the whole lane's residue, 67.7 % on OpenTitan's), and 259 refusals are the corpus's own
absent generated headers, which no context can supply.

**Acceptance checklist (`TASK-ACCEPTANCE`):**

  - [x] **REPRODUCE / ISSUE** — `.4` covered 4 157 of 16 336 rows (`PIPELINE-VS-RAW: suites=opentitan,uvm-core rows=4157`); 29 OpenTitan include names were ambiguous and produced 23 `pp_fail`s; no doctrine held the tracked report or context to a re-derivation (`git ls-files scripts | grep -c pipeline` → 0 before this leaf).
  - [x] **ROOT CAUSE (WHY + WHERE)** — ops family: `git ls-files scripts | grep -c pipeline` → 0 (no currency check existed), the suite roster read from the context TABLE rather than the tree (`awk … pipeline_context.tsv` in the runner's usage — three suites with only local includes had no row), and the first whole-lane run's *"syntax error near unexpected token `then'"* from editing `stimuli/run_pipeline_lane.sh` while it ran (`bash -n` clean before and after: the file was fine, the running instance's read position was not); the ambiguity: `pipeline_context.py`'s report listed 29 names with ≥ 2 candidates, 22 of them with a unique nearest candidate by path prefix.
  - [x] **FIX** — instrument tier: `registered_suites()` / `--list-suites` / `all`; `nearest_candidate` + the per-file table + the runner's per-file rows; `pipeline_vs_raw.py --check`; the doctrine script with its self-test; registry / §10 / book count / derived chapter.
  - [x] **ADDRESSED (verified)** — `PIPELINE-LANE: files=16336 pass=11673 fail=4401 pp_fail=259 timeout=3 crash=0 wall_s=155`; `PIPELINE-VS-RAW: … rows=16336 … missing_raw=0`; OpenTitan `pp_fail 30 → 9` with `pass 1073` unchanged (the per-file rows); `check_pipeline_lane_currency.sh --self-test` → `arms=4 failed=0`; `bash scripts/check_doctrines.sh` → `ALL 39 enforced doctrines PASS` with `<meta:mirror>` 39, `<meta:book-count>` 39, `<meta:book-content>` equals a fresh derivation.
  - [x] **NO REGRESSION** — no Rust / grammar / codegen / generated byte moved (`generated-reproducibility: OK` at commit; the release probe's fingerprint = the parser `423042d2…`); the `.5` subset reproduces inside the whole lane row for row except the 21 OpenTitan rows the per-file context moved `pp_fail → fail` (pre-registered: the 23 ambiguous-include refusals; `pass` byte-for-byte unchanged at 1 073 / 28); the raw lane's tracked artifacts untouched; `PIPELINE-LANE-CURRENCY: OK` on the tree; `IDENTITY-CARRIERS: OK` (the report's 7 rows re-hash); the doctrine's four probe arms observed firing.
  - [x] **LOCKSTEP** — `TOOLBOX.md` §1.1b, the SV parser book chapter (whole-lane section; HTML rebuilt), `DOCTRINE_ENFORCEMENT.md` §10, `docs/book/src/gate-flow.md` (count), `docs/book/src/the-enforced-doctrines.md` (derived), the tracked context tables + report, this leaf (+ `.5`(g)(h) opened, `.4c` priced), `docs/TASK_TREE.md`, `MEMORY.md`, `CHANGES.md`, `DEVELOPMENT_NOTES.md`.
  - promotion: declined (the `.4b` lesson — never edit a running bash script — is in TOOLBOX §1.1b and `DEVELOPMENT_NOTES.md`; no new card).

## `.5`(g)(h)(g′) the LAST preprocessor defects the whole lane named — ✅ done 2026-09-09 (`PGEN-SVPP-EXPANSION-0008`)

**What landed** (`rust/src/sv_preprocessor.rs` only; ZERO grammar / codegen / generated bytes; 6 new tests,
48/48; the file stays `rustfmt --edition 2024 --check` clean):

- **(g) a MISSING actual for a formal without a default is refused; so is an EXTRA actual.** IEEE 1800-2017
  §22.5.1 (`section-22-compiler-directives.md:224-226`): *"… error if any of the remaining formal arguments
  does not have a default specified … It shall be an error to specify more actual arguments than the number
  of formal arguments"*, and its examples (`:251-267`): `` `MACRO1 ( 1 ) // ILLEGAL: b and c omitted, no
  default for c ``, `` `D(,,) // illegal, more actual than formal arguments ``. `expand_function_macro` now
  raises `E_SVPP_EXTRA_MACRO_ARGUMENT` when the actual list is longer than the formals and
  `E_SVPP_MISSING_MACRO_ARGUMENT` when it is shorter and a formal it leaves unbound has no default, each
  naming the macro, the counts, the formal and the standard's sentence; the probe reports it as
  `preprocess_failed` (rc 3) at the ORIGIN file and line. ⭐ The standard's asymmetry is encoded, not
  smoothed: an EMPTY actual (present, blank — `` `MACRO1 ( , 2, ) ``, `` `MACRO2 (1, , 3) ``) for a formal
  without a default is *"replaced by nothing"* and stays legal ((f) kept), a MISSING one (the list is short)
  is the error. Test `a_missing_actual_for_a_formal_without_a_default_is_refused` — the LRM's MACRO1 / MACRO2
  / D lines verbatim, the legal ones asserted to expand to the standard's own expansions, the illegal ones
  to refuse with the code.
- **(h) a RECURSIVE macro is refused, not expanded — and the exception follows PROVENANCE.** IEEE
  1800-2017 §22.5.1 (`section-22-compiler-directives.md:342-343`): *"It shall be an error for a macro to
  expand directly or indirectly to text containing another usage of itself (a recursive macro). However,
  an actual argument to a macro or a default may contain a usage of itself"*. `expand_macros_in_text`
  carries `expanding` — the stack of macro names whose BODY is being rescanned — and a usage of a name on
  that stack raises `E_SVPP_RECURSIVE_MACRO` naming the whole chain (*"`TOP -> `BOTTOM1 -> `BOTTOM ->
  `TOP"*). The exception is decided by WHO WROTE THE USAGE: every byte of an expansion's text carries a
  provenance flag (`user_mask`) — a source line is all the caller's; inside an expansion the bytes an
  actual or a default supplied keep the flags they were written with (`parse_macro_invocation_arg_ranges`
  locates each actual in the text it came from; `substitute_function_macro_body` returns the mask with the
  text, the paste tokens removed DURING the scan so the two stay aligned), and the bytes the body produced
  are `false`. A caller-written usage starts its OWN expansion context (a fresh stack — the caller wrote
  finitely many, so they cannot loop); a body-produced usage is checked against the stack. ⭐ The first
  cut (byte RANGES of the current substitution) passed the standard's `` `TOP( `TOP(b,1), `TOP(42,a) ) ``
  example and was WRONG on the corpus: the whole-lane re-run refused OpenTitan `chip_env_pkg.sv` at
  `chip_sw_lc_raw_unlock_vseq.sv:57` — `` `DV_SPINWAIT( … `DV_SPINWAIT(…) … ) `` — because the inner
  usage sat in a region of the outer substitution (exempt) but its own body's `` `DV_SPINWAIT_EXIT ``
  did not, while the OUTER `DV_SPINWAIT_EXIT` was still on the stack; and a fresh stack for every
  region-resident usage would have laundered `` `define A(x) `B(`A(x)) `` (the `` `A `` handed to B is
  A's body text) into an infinite expansion. Provenance settles both. The depth guard (16) stays as the
  backstop it always was (a default that names its own macro without end is bounded by it). Tests
  `a_recursive_macro_is_refused_not_expanded` (the direct `` `define e fun `e `` of verilator
  `t_pp_circ_subst_bad.v`, the two-self-reference
  `` `define SEL_NUM_BITS `WIDTH-`SEL_NUM_BITS +: `SEL_NUM_BITS `` of `t_pp_circdef_bad.v` — the one that
  TIMED OUT at 60 s, now refused in milliseconds — and an indirect `A → B → A` naming the chain),
  `an_actual_may_contain_a_usage_of_the_macro_itself` (the `TOP` example),
  `a_usage_written_inside_an_actual_starts_its_own_expansion_context` (the OpenTitan nested-spinwait
  shape) and `an_actual_a_body_wrote_does_not_launder_recursion` (`` `define A(x) `B(`A(x)) `` refused
  as `` `A -> `B -> `A ``).
- **(g′) — FOUND BY (g): an apostrophe in a formal's default was read as an opening quote.** The first (g)
  candidate raised the `.1` census `73 → 78`: five OpenTitan `aes/dv/err_injection_if/fi_*.sv` refused
  *"`DV_FCOV_INSTANTIATE_CG invoked with 3 arguments for 2 formals"* on a macro declared with THREE —
  `` `define DV_FCOV_INSTANTIATE_CG(NAME_, COND_ = 1'b1, ARGS_ = ()) `` (`dv_fcov_macros.svh:35`).
  `split_macro_parameter_tokens` kept an `in_single_quote` state, so the `'` of `1'b1` opened a "string"
  that swallowed the following comma: the list read as `NAME_` and `COND_` with the garbage default
  `1'b1, ARGS_ = ()`. SystemVerilog has no single-quoted string — §5.9 (`section-32-h-12ab-f001.md:99`):
  *"A string literal is a sequence of characters enclosed by double quotes"*; an apostrophe in a default is a
  based literal's (`1'b1`) or an assignment pattern's (`'{…}`). The single-quote state is REMOVED (the
  double-quote state and its escapes stay). Test `an_apostrophe_in_a_formals_default_is_not_a_quote` (the
  OpenTitan formal list verbatim, a `'{1, 2}` default, a `"s,t"` default that must still hide its comma).
  🔎 **The defect was SILENT since the formal-list scanner's first commit (2026-03-17) and the parser could not see it.** With
  `ARGS_` not a formal, the body's `` NAME_``_inst = new``ARGS_; `` pasted to `x_inst = newARGS_;` — an
  assignment from an IDENTIFIER, legal SystemVerilog — so the 20 OpenTitan files that invoke the macro were
  expanded WRONG and two of them (`ibex_icache_fcov_if.sv`, `rom_ctrl_cov_if.sv`) were `pass` rows of `.4b`
  on that wrong text; a third actual (five `fi_*.sv` rows) was dropped without a word. Only a rule that
  REFUSES (extra actuals) made it visible. The 24 invoking files (20 + 4 of `DV_FCOV_SIGNAL_GEN_IF`, whose
  `DEFAULT_ = '0` was the LAST formal so its count was right) are pre-registered as the (g′) movers: their
  expansion bytes change; their verdicts are read, not assumed.
- **the paste-while-scanning refactor** — `substitute_function_macro_body` removes the ``` `` ``` paste
  token DURING its scan (it was a `.replace("``", "")` over the whole substituted text afterwards), so the
  argument regions it returns stay aligned with the text the rescan sees; the `.5` expansions are
  byte-identical (verification (1)).
- **a `.4b` defect owned here: the `PIPELINE-LANE-CURRENCY` report arm was a treadmill.** The tracked
  report carries a *"Measured at `HEAD` = …"* line naming the commit the RENDER ran on, and the arm masked
  the probe-digest row only — so the report published by `-0007` read DRIFT on the very next commit
  (`afed3050+dirty` vs `20bb1939+dirty`, the first `check_doctrines.sh` run of this slice), and every
  later commit of the repository would have had to re-render it to pass. That line is provenance of the
  same kind as the probe digest; `pipeline_vs_raw.py --check` now masks both (`PROVENANCE_PREFIXES`), the
  doctrine script's header says so, its `--self-test` stays 4/4 (green control, deleted context row, edited
  report number, absent dump) and the arm reads OK on this tree before the lane is re-run.

**Verification** (`rust/target/svpp_e1/verify_s5gh.sh`; logs `verify_s5gh_final.log` = the first (g)
candidate, `verify_s5gh_final2.log` = the final bytes; the Surelog bisect `s5gh/surelog_bisect/{A,B}.log`;
all on the debug binaries rebuilt from the final bytes, the lane on the release probe rebuilt from them):

```text
(1) byte-identity: dv_utils_pkg / uart / dv_base_env IDENTICAL to the .5 bytes · uvm_pkg.sv 3 022 806 B IDENTICAL
    to the 2026-08-24 gate artifact  (the paste-while-scanning refactor moved nothing)
(2) the .1 census: files=4157 rc0=4084 rc_nonzero=73 timeouts=0   — the first (g) candidate read rc_nonzero=78:
    the five fi_*.sv rows "invoked with 3 arguments for 2 formals" = (g′); after (g′) back at 73, the same 73
(3) the fixtures through the single command (rc 3 = preprocess_failed, all < 1 s where the first candidate needed
    4–57 s on _7.sv for the unrelated parser stage of a file the preprocessor now refuses first):
    22.5.1--define-expansion_7.sv  E_SVPP_MISSING_MACRO_ARGUMENT     22.5.1--define-expansion_8.sv  E_SVPP_EXTRA_MACRO_ARGUMENT
    t_pp_circ_subst_bad.v          E_SVPP_RECURSIVE_MACRO            t_pp_circdef_bad.v (was TIMEOUT)  E_SVPP_RECURSIVE_MACRO 0 s
    file_uses_defn.svh / t_preproc_debugi.v: rc 1 furthest_position=0 / =229 — parser verdicts, unchanged (not this leaf's)
(4) the two Surelog timeouts (DiffSimpleIncludeAndMacros/top.v, SimpleIncludeAndMacros/top.v) under their per-file
    context: both refuse in 0 s — E_SVPP_MISSING_MACRO_ARGUMENT at mode.vh:36 `D("msg1") — the LRM's own
    "illegal, only one argument", transcribed into the test. The .4b TIMEOUT's cause is the file's last four lines,
    an INDIRECT cycle (`define BOTTOM `TOP / `define TOP `BOTTOM1 / `define BOTTOM1 `BOTTOM / `TOP): with the
    illegal usages removed (variant B) → E_SVPP_RECURSIVE_MACRO at top.v:47 "`TOP -> `BOTTOM1 -> `BOTTOM -> `TOP"
(5) the (g′) reproducer s5gh/fcov.sv: rc 0, `x_inst = new();` for the default and the three-actual call expanded
(6) PRE-REGISTERED before the lane re-run (written before the release probe finished building): against the .4b dump,
    exactly these rows move — 22.5.1--define-expansion_7.sv, _8.sv, t_pp_circ_subst_bad.v: fail → pp_fail (own-file
    fixtures, the refusal now at the preprocess stage); t_pp_circdef_bad.v and the two Surelog top.v: timeout → pp_fail;
    the 24 (g′) invoking files: expansion bytes change, verdicts READ (the two .4b passes expected to stay pass on
    `new()` where they passed on `newARGS_`); every other row identical; timeouts 3 → 0
(7) build incident — DIAGNOSED, not hidden: THREE release builds of the final bytes died in the bin's fat-LTO link at
    the same point (rustc "signal: 15, SIGTERM" at 16:35:19 after 1 111 s and at 16:44:06 after 457 s; "signal: 9,
    SIGKILL" at 16:54:04 after 462 s with SIGTERM ignored for the build step); every guard marker reads reason=none
    (peaks 14 663 / 14 831 / 15 053 MB under the 16 384 MB budget) — the guard sent none of them. THE KILLER IS THE
    HOST WATCHDOG: `~/.memwatch/memwatch.sh` (a launchd agent the director installed 2026-06-11 after kernel panics,
    `KILL_RSS_MB=14336`, "kill any single owned process above 14 GB RSS") — its log: "2026-09-09 16:54:04 KILL
    pid=36311 — single process rss 14946MB > 14336MB — cmd=[rustc --crate-name parseability_probe …]". The `.4b`
    release build passed the same day with a 14.2 GB tree peak — the fat-LTO link of `parseability_probe` sat just
    under the line and the (g)(h) bytes put it over. Two profile overrides were then tried, both env-only
    (`Cargo.toml` untouched): `CARGO_PROFILE_RELEASE_LTO=thin` moved the peak into the LIB's compile (KILL at
    16:59:16, 14 507 MB, `crate-name pgen`), and `LTO=off` + `CODEGEN_UNITS=16` (cargo's own release defaults)
    died in the same lib compile at 14 354 MB (17:08:22) — 18 MB over the line: the release FRONTEND of this crate
    (236 MB of generated parsers, opt-level 3) sits on the watchdog's line by itself. NO release probe of these
    bytes exists on this host. The lane below therefore ran on the DEBUG probe (built 16:14 from the final bytes,
    the verify script's binary, `PGEN_PARSE_PROBE_BIN`, recorded in the report's identity block) with a 600 s
    per-file deadline (`PGEN_PIPELINE_TIMEOUT_S=600`): a build profile cannot change a verdict, the slowest
    non-timeout row of the `.4b` dump is 5.75 s, and a debug probe runs about an order of magnitude slower, so the
    deadline was widened to keep the `timeout` class meaning what it means. ⛔ STANDING BLOCKER for the director
    (also in MEMORY / the operations chapter): a `cargo build --release` of the probe cannot complete on this host
    until the watchdog's line is raised, the release profile is revisited for this crate, or the build moves to a
    bigger host — every release-probe gate (`sota_exit_gate`, the corpus lanes) is behind that.
(8) lane incident, owned: the first debug-probe run at the raw lane's 8 jobs BREACHED the guard (tree 14 797 MB >
    the 14 336 MB budget — a debug probe holds ~1.8 GB each), the guard killed the tree (exit 97), and the chain's
    render then PUBLISHED the tracked report from the EMPTY dump the killed run left behind (`rows=0`): two
    instrument defects, both fixed in this slice — `pipeline_vs_raw.py` now REFUSES (exit 2, nothing written) to
    render a dump with no rows, and `run_pipeline_lane.sh` writes its dump to a temporary path and moves it into
    place only at its summary step, so a killed run leaves the previous dump intact. The dump and the report were
    restored from the `.4b` copies (`--check` OK) before the 6-job re-run.
(9) THE LANE, final bytes (debug probe, 6 jobs, 600 s deadline, 432 s wall; the dump published atomically; the
    report re-rendered with the probe recorded): PIPELINE-LANE: files=16336 pass=11671 fail=4387 pp_fail=278 timeout=0
    crash=0 — against .4b's pass=11673 fail=4401 pp_fail=259 timeout=3. Row-for-row against the .4b dump: 23 rows
    moved, 2 kept their status with a new origin, 16 311 identical.
      fail → pp_fail 10: the two chapter-22 :should_fail_because: tests (_7 MISSING, _8 EXTRA), verilator
                         t_pp_circ_subst_bad.v and t_pp_recursedef_bad.v (RECURSIVE), the six Surelog top_N.v of the
                         two IncludeAndMacros suites (mode.vh:36 `D("msg1") — the LRM's "illegal, only one argument")
      timeout → pp_fail 3: t_pp_circdef_bad.v (RECURSIVE, 0 s), the two Surelog top.v (mode.vh:36)
      pass → pp_fail 6:  SIX OVER-ACCEPTANCES REVOKED — iverilog br_gh72b_fail.v (regress-sv.list: CE), sv-tests _6 and
                         _12 (:should_fail_because:), sv2v test/error/macro_overapplied.sv and macro_underapplied.sv,
                         verilator t_preproc_defarg_bad.v — every one an intended-fail fixture the raw AND the pipeline
                         lane had passed
      fail → pass 4:     the (g′) movers — OpenTitan kmac_cov_if.sv, sysrst_ctrl_cov_if.sv, otp_ctrl_cov_if.sv ×2 (both
                         tops): `DV_FCOV_INSTANTIATE_CG with three actuals now expands to `new (…)` where it read
                         `newARGS_` — real design files RECOGNISED
      origin only 2:     Surelog top_2.v ×2 (pp_fail both times; the missing-actual refusal at mode.vh:36 now precedes
                         the `include `MACRO ruling at :52)
      the 24 (g′) invoking files: INSTANTIATE_CG 20 — 14 fail→fail, 2 pass→pass, 4 fail→pass; SIGNAL_GEN_IF 4 — unchanged
      0 pass → fail, 0 pass → timeout, 0 crash. The ONE row the first (h) cut had moved fail → pp_fail (OpenTitan
      chip_env_pkg.sv, the nested `DV_SPINWAIT) is back at its .4b verdict and origin (:64:18) under the provenance rule.
```

**Acceptance checklist (`TASK-ACCEPTANCE`):**

  - [x] **REPRODUCE / ISSUE** — `.4b`'s whole-lane report: sv-tests `22.5.1--define-expansion_7.sv` / `_8.sv` `pass → fail` in the row's own file (the right verdict at the wrong stage), verilator `t_pp_circ_subst_bad.v` the same, and three `timeout` rows (`t_pp_circdef_bad.v`, the two Surelog `top.v`); minimal reproducers in the tests (the LRM's lines) and `rust/target/svpp_e1/s5gh/fcov.sv` for (g′).
  - [x] **ROOT CAUSE (WHY + WHERE)** — correctness family: `--sv-event-log-json` on `t_pp_circdef_bad.v` showed the expansion events of `SEL_NUM_BITS` nested to the depth guard (2 self-references ⇒ 2^16 growth — the timeout); the `.4b` rows' `furthest_position` on `_7.sv` / `_8.sv` sat on the text of the illegal expansion (`$display("start", , "end")` with a formal bound to nothing); the first (g) candidate's own refusal `E_SVPP_EXTRA_MACRO_ARGUMENT … 3 arguments for 2 formals` on a three-formal macro located (g′); ops family: `git log -S'in_single_quote' -- rust/src/sv_preprocessor.rs` → three commits (`636f35c0` 2026-02-27 the preprocessor's first commit, `17e3995d` 2026-03-17 *"Harden SV preprocessing for external corpora"* — the one that introduced `split_macro_parameter_tokens`, `git log -S'fn split_macro_parameter_tokens'` → that commit alone — and `3817607f` 2026-03-19), so the misreading is as old as the formal-list scanner, 2026-03-17. WHERE: `expand_function_macro` (bound a missing actual to `""`), `expand_macros_in_text` (no notion of the macro being rescanned; the depth guard the only stop), `split_macro_parameter_tokens` (the single-quote state). WHY: §22.5.1 `:224-226`, `:251-267`, `:342-343`; §5.9 `section-32-h-12ab-f001.md:99`.
  - [x] **FIX** — engine tier (a preprocessor has no grammar): the two argument-count refusals, the `expanding` stack with the actual-argument exception, the apostrophe rule, the paste-while-scanning refactor; 4 tests.
  - [x] **ADDRESSED (verified)** — verification (3): the two chapter-22 tests and the two verilator tests refuse at the preprocess stage with the code and the standard's sentence, `t_pp_circdef_bad.v` in 0 s where it timed out; (4): the two Surelog timeouts refuse in 0 s and their cycle is named once the illegal usages are out of the way; (5): the OpenTitan three-formal macro expands with all three actuals; `cargo test --lib sv_preprocessor` → `46 passed; 0 failed` (42 + 4).
  - [x] **NO REGRESSION** — verification (1): the four `.5` expansions byte-identical, `uvm_pkg.sv`'s 3 022 806 B to the 2026-08-24 gate artifact; (2): the `.1` census at 73, the same 73 rows; verification (9): the whole lane row-for-row against the `.4b` dump — 23 movers, every one named and explained (19 intended-fail fixtures now refused at the preprocess stage, six of them over-acceptances the lanes had PASSED; 4 real OpenTitan files recognised; 3 timeouts gone), **0 `pass → fail`**, 16 311 rows identical; the first (h) cut's one real-design false refusal caught by the same diff and fixed before commit; `clippy_on_rust_change` → *"✅ clippy_on_rust_change completed"* (`CLIPPY_RC=0`, peak 12 625 MB under the 16 384 MB guard); no grammar / codegen / generated byte moved (`generated-reproducibility: OK` at commit); `bash scripts/check_doctrines.sh` ALL PASS (run at commit).
  - [x] **LOCKSTEP** — the SV parser book chapter (the whole-lane table and the movers paragraph re-rendered from the re-run; HTML rebuilt), `TOOLBOX.md` §1.1b (the own-file-mover sentence), the tracked `pipeline_characterization.md` re-rendered, this leaf (the `-0006` record's OPEN markers closed), the frontier, `docs/TASK_TREE.md`, `MEMORY.md`, `CHANGES.md`, `DEVELOPMENT_NOTES.md`. No flag moved, so the svpp CLI docs need no change; the three `E_SVPP_*` codes are covered by the `[WE]_SVPP_[A-Z_]{4,}` family registered in `-0006`.
  - promotion: declined for a card; the (g′) lesson — a rule that REFUSES is the only instrument that sees a silent mis-expansion — goes to `DEVELOPMENT_NOTES.md` and this record's 🔎.

## `.6a` the RE-ADJUDICATION instrument — ✅ done 2026-09-09 (`PGEN-SVPP-EXPANSION-0009`): the deferred population's first verdicts under the pipeline

**What landed** (INSTRUMENT + DOCTRINE tier; ZERO Rust / grammar / codegen / generated bytes):

- **`stimuli/sv/pipeline_readjudication.py`** joins the answer key (`adjudication_manifest.tsv`, READ never
  written), the `.12a` per-row audit (`explained_svpp_audit/after/audit.tsv`: the position verdict) and the
  `.13c` audit (`dark_chained_only/audit.tsv`: the stratum and cross-file channel) with the pipeline lane's
  dump, and writes the tracked **`stimuli/sv/characterization/pipeline_readjudication.md`** and the per-row
  **`pipeline_readjudication.tsv`** (6 534 rows: `suite relpath raw_class prior_stratum pipeline_status landing
  construct verdict origin`). Verdict vocabulary: `corroborated` / `refuted` / `hollow` / `refused_context` /
  `refused_engine` for the must-accept `explained_svpp_*` rows, and the `chained:` / `svpp_owned:` forms
  (`accepted_under_expansion`, `rejects_own_file`, …) for rows the key holds no expected verdict for — a
  `chained:accepted_under_expansion` row is NOT a `match` until `.6c` derives its expected verdict from the
  suite's convention (corpus-expected-from-SPEC, never from the fix). `--check` derive-and-diffs both outputs
  (the probe row and the HEAD line masked as provenance; an absent dump NOT EVALUATED; an empty dump REFUSED).
- **`PIPELINE-LANE-CURRENCY` gains arm (3)** — the re-adjudication report and table equal a fresh render on
  every commit (`scripts/check_pipeline_lane_currency.sh`; `--self-test` arm E: an edited verdict in the
  per-row table is refused; 5/5); the registry row, `DOCTRINE_ENFORCEMENT.md` §10 and the derived chapter say so.
- `SV-CORPUS-GRAD.12b` and `.13d` now point at the report's numbers (their prose totals — 263, 1 527 — predate
  the audits' last re-run; the tracked artifacts hold 257 and 1 460, and the artifacts are the denominators).

**THE MEASUREMENT** (`pipeline_readjudication.md`, from the `-0008` lane: debug probe, 600 s):

```text
rows re-read: 6 534 (every manifest row of the three populations has a lane row)
explained_svpp (1 434 must-accept rows the raw lane explained away):
    corroborated 1 199 · refuted 223 · refused_engine 7 · refused_context 5
    by .12a position verdict: STUCK-ON-SVPP-TOKEN 1 167 → 1 091 corroborated / 67 REFUTED / 9 refused
                              IN-WINDOW 253 → 98 / 152 / 3 ;  PAST-LAST-TICK 4 → 0 / 4 / 0 ;  not-in-audit 10 → 10 / 0
    the 223 refuted by construct: `extends` 103 (sv-tests `*-uvm.sv` — every sv-tests explained_svpp_include row —
                              the base class in uvm_pkg), other 95, `typedef` 18, function/task 3, import 1, `#(` 1, `::` 1, - 1
    by suite: verilator 85 (80 macro_use, 3 conditional, 2 protected_envelope), sv-tests 105, sv2v 10, slang 9,
              iverilog 8, Surelog 3, verible 2, ispras 1
chained_only (4 927): accepted_under_expansion 1 530 · rejects_own_file 2 936 · hollow 236 · refused_context 210 · refused_engine 15
    the .13d expansion-owned stratum (IN-WINDOW 1 415 + PAST-LAST-TICK 45 = 1 460): accepted 432 · rejects_own_file 658 ·
                              hollow 158 · refused 212   ← the deferral's textual half RESOLVED for 432 rows
    FACT-GATED-CROSS-FILE 2 057: 2 056 rejects_own_file, 1 refused — unmoved, as .13c predicted (no preprocessor supplies a sibling's type)
    ONE-SIDED (raw pass) 1 188: 1 098 still pass · 78 hollow · 1 own-file · 11 refused
    the 2 936 own-file rejections by construct: `extends` 1 924, other 811, `#(` 75, `::` 47, typedef 24, function/task 23, uvm_* 21, …
svpp_owned (173): accepted 111 · rejects_own_file 29 · refused_engine 30 · refused_context 2 · hollow 1
```

🔎 **FINDINGS (for the director's review, also in `SV-CORPUS-GRAD.12b` / `.13d`):** (1) 223 must-accept rows the
raw answer key had EXPLAINED AWAY still fail in their own text once preprocessed — including 67 the position
test had DECIDED (`STUCK-ON-SVPP-TOKEN`): the directive was where the parse stopped, not why. Axis 2's honest
bound is no longer 319–582; 223 candidates await a disposition each (`.6b`). (2) 103 of them plus 1 924
chained-only rows are one construct — a class header whose base is declared in another file (UVM for the
sv-tests, the OpenTitan / VeeR package chains for the rest): **2 027 rows on one cross-file fact**, the library
design's prize, and UVM's part of it is gated on the `.5`(d) ruling (a rejecting `uvm_pkg.sv` exports nothing).
(3) 432 of the 1 460 expansion-owned dark rows are ACCEPTED under the pipeline — they need only an expected
verdict to become `match` rows, the first movement of the axis-2 denominator that expansion has ever produced.

**Acceptance checklist (`TASK-ACCEPTANCE`):**

  - [x] **REPRODUCE / ISSUE** — `SV-CORPUS-GRAD.12b` (263 position-undecidable rows, parked: *"the only instrument that settles them is SVPP-EXPANSION itself"*) and `.13d` (1 527 dark rows expansion owns, `todo` until `.6` reports); `pipeline_characterization.md`'s by-class table showed the outcomes existed (`deferred:chained_only` 1 530 pass / 3 172 fail / 225 pp_fail) with no row-level verdict anywhere.
  - [x] **ROOT CAUSE (WHY + WHERE)** — instrument family: the deferred rows' only instrument was a TEXT test (`.12a` position, `.13c` backtick reach) because no lane parsed them preprocessed; the lane exists since `.4b` and its dump carries every row's `origin` (`[origin=FILE:LINE:COL]`, `.3`), so the verdict is a JOIN, not a new measurement — census: `grep -c chained_only stimuli/sv/corpus_verdict_coverage.py` → the coverage instrument buckets the class as NO VERDICT with no pipeline input (`git log -S'pipeline_results' -- stimuli/sv/corpus_verdict_coverage.py` → 0 commits). WHERE: a new instrument beside `pipeline_vs_raw.py`, reusing its join (`suite_relpath`), buckets (`origin_bucket`) and identity block. WHY: `.12b`/`.13d`'s own dispositions.
  - [x] **FIX** — the instrument, the tracked report + table, the doctrine arm (3) + self-test arm E, the pointers in `SV-CORPUS-GRAD.12b` / `.13d`, TOOLBOX §1.1b, the SV parser book chapter.
  - [x] **ADDRESSED (verified)** — `python3 stimuli/sv/pipeline_readjudication.py --probe rust/target/debug/parseability_probe` → `rows=6534 missing_lane_rows=0 …` (the summary above); `--check` → OK; `check_pipeline_lane_currency.sh` → arms (1)(2)(3) OK; `--self-test` → 5/5 (green control, deleted context row, edited report number, absent dump NOT EVALUATED, edited re-adjudication verdict refused).
  - [x] **NO REGRESSION** — the answer key is untouched (`git diff --stat stimuli/sv/characterization/adjudication_manifest.tsv` → empty); the lane dump and `pipeline_characterization.md` unchanged; `bash scripts/check_doctrines.sh` ALL PASS (run at commit); no Rust byte moved.
  - [x] **LOCKSTEP** — TOOLBOX §1.1b, the SV parser book chapter (HTML rebuilt), `DOCTRINE_ENFORCEMENT.md` §10 + the derived chapter, `SV-CORPUS-GRAD.md` (`.12b`, `.13d`), this tree (`.6` sub-leaves, the frontier), `docs/TASK_TREE.md`, `MEMORY.md`, `CHANGES.md`, `DEVELOPMENT_NOTES.md`.
  - promotion: declined (the lesson — a position test is a hypothesis about a parse, and only the parse settles it — is `.12b`'s own statement; no new card).

## `.6b.1` the 34 verilator rows — ✅ done 2026-09-09 (`PGEN-SVPP-EXPANSION-0010`): pinned `must_reject` with the standard's line; the pipeline cashes the pins

**What landed** (ANSWER-KEY + INSTRUMENT tier; ZERO Rust / grammar / codegen / generated bytes):

- **34 pins in `EXTRA_PINNED`** (`stimuli/sv/adjudicate_external_corpus.py`, the `.13y` family extended): 17 typedefs
  reaching a type through TWO OR MORE hierarchical steps (`typedef io.types.a_t a_t;`) — IEEE 1800-2017 A.2.1.3
  `type_declaration` (`section-41-data-read-api.md:825`) admits exactly one `interface_instance_identifier
  constant_bit_select . type_identifier` before the new name, and 1800-2023 A.2.1.3 (`section-Annex_A-normative-formal-syntax.md:800`)
  narrows even that to an interface PORT (§6.18 `section-6-data-types.md:1839-1840`: hierarchical references to type
  identifiers are not allowed; references through ports are not hierarchical); 16 `localparam type x = if0.rq_t;` — a type
  named through an interface instance in a type assignment (A.2.2.1 `data_type` `:842`, A.2.4 `type_assignment` `:1013`,
  the `.13y` ruling); 1 `typedef enum [3:0] {…}` — a range with no base type (A.2.2.1 `:850`, `enum_base_type` `:868-873`).
  ⭐ Each pin was verified BOTH ways before it was written: the corpus row rejects at the pinned line on sv_2017 AND sv_2023
  under the pipeline; the minimal reproducers reject on both (`localparam type u = i.t;` at 87, `typedef enum [3:0] {…}` at
  22) while their LRM-legal controls accept (`localparam type u = p::t;`, `typedef enum logic [3:0] {…}`); and the
  ONE-STEP form `typedef i.t u;` ACCEPTS on both — the letter of 2017 A.2.1.3 — so the parser draws the line exactly where the
  standard does.
- **The raw answer key demotes them, by design** — `reject_arm_demotion`: the raw parse stops on a macro use above the
  construct, so the 34 rows read `deferred:svpp_owned` with `expected = out_of_scope_with_cause` and the pin's cite in the
  basis (measured to scratch before publishing: `34 < explained_svpp_macro_use → 34 > deferred:svpp_owned`, nothing else
  moved). The permanent-deferral ceiling of `deferred:svpp_owned` was raised `173 → 207` in the register
  (`permanent_deferral_ceilings.json`) naming the argument, as `.13f` requires; the coverage artifacts, the provenance
  census and the live tuple (`grammar-wellformedness.md`: `7563/2606/6167/3960/9 → 7529/2606/6201/3994/9`, the bar
  unchanged at 9) re-derived — the three corpus gates green.
- **`pipeline_readjudication.py` gains the `must_reject` population** — every row the key expects rejected: the manifest's
  `expected`, OR a pin the raw lane demoted, read from the adjudicator's own pin tables (`import adjudicate_external_corpus`,
  `pinned_expected()`; the manifest is still never written). Verdicts `must_reject:match_under_expansion` (rejected in its own
  file), `must_reject:accepts_invalid` (a pipeline pass on text the standard cannot derive — an over-acceptance only the
  pipeline can see), `hollow`, `refused_*`.

**THE MEASUREMENT** (`pipeline_readjudication.md`, re-rendered on the same `-0008` lane dump):

```text
must_reject (406): match_under_expansion 404 · accepts_invalid 2 (the two known unexplained_accepts_invalid rows) · 0 hollow / refused
    by stratum: `match` 369 → 369 match_under_expansion ;  `pinned, raw lane demoted` 35 → 35 (the 34 + verilator t_dpi_import.v, the .3.43 pin)
    ⇒ the pipeline reveals NO new over-acceptance among the rows the key expects rejected
explained_svpp: refuted 223 → 189 (the 34 left the population), corroborated 1 199 unchanged
```

**Acceptance checklist (`TASK-ACCEPTANCE`):**

  - [x] **REPRODUCE / ISSUE** — `.6a`'s report: 34 `refuted` verilator rows on `typedef` / `localparam type` lines (the `a typedef` construct bucket, 18, and 16 of `other`); minimal reproducers `rust/target/svpp_e1/s6b1/*.sv`.
  - [x] **ROOT CAUSE (WHY + WHERE)** — correctness family: the pipeline's `[origin=FILE:LINE:COL]` on each row lands on the second dot / the `=` of the type assignment / the `[` after `enum` (`furthest_position=22` and `=87` on the reproducers); the AST dump of the one-step control shows the grammar's own `interface_type` alternative (`instance`, `source_type`) — the parser follows 2017 A.2.1.3 to the letter; ops family: the `.13y` block in `EXTRA_PINNED` already ruled the interface-instance type assignment (`git log -S'THROUGH AN INTERFACE INSTANCE' -- stimuli/sv/adjudicate_external_corpus.py` → the `.13y` commit) — these 34 are the rows the RAW lane never reached to apply it. WHERE: the answer key (pins), the re-adjudication instrument (the `must_reject` population). WHY: A.2.1.3 `:825` / 2023 `:800`, A.2.2.1 `:842` / `:850` / `:868-873`, A.2.4 `:1013`, 2023 §6.18 `:1839-1840`.
  - [x] **FIX** — answer-key tier (pins with cites) + instrument tier (the population); no grammar or engine byte: the parser is CORRECT on every one of the 34.
  - [x] **ADDRESSED (verified)** — the re-render: `must_reject:match_under_expansion=404`, the 34 among the 35 demoted pins, `accepts_invalid=2` (both pre-existing), `refuted 223 → 189`; `--check` OK for both lane reports; `check_pipeline_lane_currency.sh --self-test` 5/5.
  - [x] **NO REGRESSION** — the pipeline lane's dump is byte-identical to `-0008`'s (untouched; both lane reports `--check` OK on it) and no Rust byte is staged (the `-0008` clippy verdict stands); the manifest diff is exactly the 34 rows (class and expected), nothing else; v2005 manifest unchanged, sv2023 manifest the same 34; `check_sv_corpus_denominator.sh` OK on the new anchor, `check_corpus_census.sh sv` OK, `check_corpus_key_integrity.sh` OK (the provenance census re-derived: `clause_cited` grew by the 34 cites); the lane dump untouched; `bash scripts/check_doctrines.sh` ALL PASS (run at commit).
  - [x] **LOCKSTEP** — the SV parser book chapter, TOOLBOX §1.1b, `SV-CORPUS-GRAD.md` (`.13y` pointer), this tree, `docs/TASK_TREE.md`, `MEMORY.md`, `CHANGES.md`, `DEVELOPMENT_NOTES.md`.
  - promotion: declined (the `.13y` ruling already carries the lesson; no new card).

## `.6b.2` four engine items the refuted rows named — ✅ done 2026-09-09 (`PGEN-SVPP-EXPANSION-0011`): each a sentence of the standard

**What landed** (`rust/src/sv_preprocessor.rs` only; ZERO grammar / codegen / generated bytes; 4 tests, 52/52; rustfmt-clean):

- **(i) the ``` `` ``` paste token joins its sides in an OBJECT macro** — §22.5.1 (`section-22-compiler-directives.md:380`):
  *"A `` delimits lexical tokens without introducing white space"*. `expand_macros_in_text` drops the token
  when rescanning macro-produced text (`user_mask.is_some()`), as a function macro's substitution always did;
  in a source line it stays for the parser. sv2v `macro_vendor_comment.sv` (`` `define YUCK /``* … *``/ ``)
  reached the parser with the token in it. Test `a_paste_token_in_an_object_macro_joins_its_sides`.
- **(ii) a well-formed `` `line `` directive is CONSUMED** — §22.12 (`:791-795`): *"The compiler shall maintain
  the current line number and file name … The `line directive shall set the line number and file name of the
  following line to those specified … only white space may appear on the same line … All parameters … are
  required."* `parse_line_directive` accepts exactly `number "filename" level` with the level 0 | 1 | 2 and
  nothing else on the line; `preprocess_text_internal` keeps a `line_override` (file, number, physical start)
  so `` `__FILE__ `` / `` `__LINE__ `` and the messages of the following lines read the assigned location
  (`exp_file` / `exp_line` at the four expansion sites) while the events and the source map keep the physical
  one; the event log records `line_directive`. ⭐ A MALFORMED directive is not this stage's to consume: it passes
  through as before and the parser's own §22.12 production refuses it, so `SV-CORPUS-GRAD.3.39`'s must_reject
  rows (`line_char1/2`, `line_eof`, `line_level`) keep their verdict under the pipeline — verified in the lane:
  0 `pass → fail`, the `must_reject:match_under_expansion` count unchanged at 404. iverilog `line_directive.v`
  (the directive inside `initial begin … end`) rejected at the line after it. Test
  `a_line_directive_is_consumed_and_renames_the_following_lines` (the iverilog gold's shape: `__FILE__` →
  `"real_source.v"`, `__LINE__` 10, 11; the level-3 line passes through).
- **(iii) a one-line comment in macro TEXT is not substituted** — §22.5.1 (`:187-188`): *"If a one-line comment
  (that is, a comment specified with the characters //) is included in the text, then the comment shall not
  become part of the substituted text."* `strip_line_comments_from_macro_text` cuts each line of the macro text
  at a `//` outside a string literal, a `` `"…`" `` macro string (the `` `\`" `` escaped quote respected) and
  a block comment; a backslash the comment swallowed is given back so the continuation stays the text's; block
  comments stay (white space to the parser). sv2v `macro.sv`'s `` `define NUMBER_12 21 */* foo */2//foo ``
  commented out the `)` of every use. Test `a_one_line_comment_in_macro_text_is_not_substituted`.
- **(iv) a function-like macro may open its parentheses on the NEXT line** — §22.5.1 (`:225`) requires the
  parentheses; §5.3 lets white space, newlines included, separate the tokens. `has_unclosed_function_macro_invocation`
  answers true when a function-like macro's name closes the line, so `collect_multiline_macro_invocation` reads
  on and `parse_macro_invocation_args` (which already skipped newlines) finds the `(`. sv2v `macro_whitespace.sv`.
  Test `a_function_like_macro_may_open_its_parentheses_on_a_later_line`.

**Pre-registration** (before the build): defines with `//` in their text — 0 in UVM + OpenTitan (the `.5` byte
identity must hold), 19 files corpus-wide; object macros with a paste token — 18 corpus-wide; files with a
`` `line `` directive — 21; the `(`-on-the-next-line shape not countable by grep. Movers READ, not assumed.

**Verification** (`rust/target/svpp_e1/verify_6b2.log`, the lane `s6b2_lane.log`, the row diff against
`s6b2_pipeline_results_before.tsv`):

```text
(1) byte-identity: dv_utils_pkg / uart / dv_base_env IDENTICAL to the .5 bytes · uvm_pkg 3 022 806 B IDENTICAL to the 2026-08-24 artifact
(2) the .1 census: files=4157 rc0=4084 rc_nonzero=73 (unchanged)   (3) the (g)(h) fixtures: unchanged (E_SVPP_* in 0–4 s)
(4) THE LANE (debug probe, 600 s, 436 s wall): PIPELINE-LANE: files=16336 pass=11682 fail=4376 pp_fail=278 timeout=0 crash=0
    against -0010's 11671 / 4387 / 278: 11 rows moved, ALL fail → pass, 0 the other way —
      (ii)  iverilog line_directive.v, fileline2.v; verilator t_force_release.v, t_cast_types.v (a `line inside a block)
      (iii) iverilog macro_comment2.v, multiply_large.v, pr355.v, pr534.v, pr985.v (a // comment in a define's text)
      (i)   sv2v macro_vendor_comment.sv          (iv) sv2v macro_whitespace.sv
    4 rows advanced their origin with the same verdict: sv2v macro.sv :118 → :137, macro_iv.sv :135 → :154 (the next
    barrier — .6b.4), verilator t_pp_display.v :35 → :40; 16 321 rows identical
(5) re-adjudication: corroborated 1 199 → 1 205, refuted 189 → 183, must_reject:match_under_expansion 404 (unchanged: the
    malformed `line rows still refuse), accepts_invalid 2 (unchanged)
(6) clippy_on_rust_change ✅ (CLIPPY_RC=0, peak 13 623 MB under the 16 384 MB guard — and under the host watchdog's 14 336 MB)
```

**Acceptance checklist (`TASK-ACCEPTANCE`):**

  - [x] **REPRODUCE / ISSUE** — `.6a`'s `refuted` rows sv2v `macro_vendor_comment.sv`, `macro_whitespace.sv`, `macro.sv` (`NUMBER_11/12`), iverilog `line_directive.v`; each reproduced with `ast_pipeline --preprocess-systemverilog --output` (the expansion still carried ``` `` ```, the `` `line `` line, the `//foo`, the unexpanded `` `FOO ``).
  - [x] **ROOT CAUSE (WHY + WHERE)** — correctness family: the probe's `[origin=…]` on each row (`line_directive.v:8:131` = the byte before the `` `line `` line; `macro.sv:118:1`) and the expanded text beside it; the LRM sentences §22.5.1 `:187-188` / `:380`, §22.12 `:791-795`, §5.3; ops family: `git log -S'"line"' -- rust/src/sv_preprocessor.rs` → 0 commits (no `line` arm ever existed; the directive took the unknown-directive pass-through). WHERE: `expand_macros_in_text` (the paste branch), the directive dispatch (no `line` arm), `parse_define_directive` (no comment stripping), `has_unclosed_function_macro_invocation` (only a `(` on the same line).
  - [x] **FIX** — engine tier: the four rules above, `parse_line_directive`, `strip_line_comments_from_macro_text`, the `line_directive` event kind; 4 tests.
  - [x] **ADDRESSED (verified)** — the lane's 11 `fail → pass` movers are exactly the four classes (verification (4)); `refuted 189 → 183`; `cargo test --lib sv_preprocessor` → `52 passed; 0 failed`.
  - [x] **NO REGRESSION** — the four `.5` expansions byte-identical (uvm_pkg 3 022 806 B to the 2026-08-24 artifact); the `.1` census 73 unchanged; 0 `pass → fail`, 0 `pass → pp_fail`, 16 321 rows identical; the must_reject rows unchanged (404 / 2 — the malformed `` `line `` rows still refuse); `clippy_on_rust_change` ✅; no grammar / codegen / generated byte moved (`generated-reproducibility: OK` at commit); `bash scripts/check_doctrines.sh` ALL PASS (run at commit).
  - [x] **LOCKSTEP** — the SV parser book chapter (the four rules; HTML rebuilt), this tree, `docs/TASK_TREE.md`, `MEMORY.md`, `CHANGES.md`, `DEVELOPMENT_NOTES.md`; the tracked lane reports re-rendered. The `E_SVPP_` / `--sv-event-log-json` families already cover the new event kind.
  - promotion: declined (the chapter carries the four rules; no new card).

## `.6b.3` the disposition table, and the tool-identity decision — ✅ done 2026-09-09 (`PGEN-SVPP-EXPANSION-0012`)

**What landed** (INSTRUMENT + ANSWER-KEY-STYLE tier; ZERO Rust / grammar / codegen / generated bytes):

- **`stimuli/sv/characterization/refuted_dispositions.tsv`** — `.6b`'s tracked disposition table: one line per
  `refuted` row that has been looked at (`suite relpath disposition basis`), written like the answer key's pins,
  each with its evidence. `pipeline_readjudication.py` reads it (identity block row; a disposition for a row that
  is not, or no longer, `refuted` under the lane is REFUSED — a stale claim cannot ride along), renders a
  `refuted` row as `refuted:<disposition>`, splits the refuted rows by disposition in the report and prints
  `refuted_awaiting_disposition` in its summary line — the `.6b` worklist, derived. The vocabulary is closed
  (`DISPOSITION_KINDS`): `cross_file_base_class`, `vendor_identity_branch`, `driver_define`, `parser_question`,
  `preprocessor_item`; an unknown one is refused.
- **The tool-identity decision, MEASURED then made.** The verilator suite tests its own identity macro in 126
  files (`` `ifdef/`ifndef VERILATOR `` / `verilator`); the lane's derived context predefines nothing (the raw
  answer key's stance). Defining `VERILATOR` + `verilator` for the suite (the debug probe, the suite's include
  context): **5 `fail → pass`** (`t_dpi_display.v`, `t_dpi_display_bad.v`, `t_dpi_sys.v`, `t_dpi_threads.v`,
  `t_display_p_elab.v` — their portable branch is the vendor's `` `error `` stub) and **24 `pass → fail`**
  (`t_extend.v`, `t_extend_class.v`, `t_func_rand.v`, `t_hier_block_libmod.v`, `t_stream_dynamic.v`,
  `t_trace_primitive.v`, `t_udp_noname.v`, `t_unroll_nested.v`, the `t_vlt_warn*.v` … — the Verilator-only
  branch carries constructs the standard cannot derive), 87 pass → pass, 9 fail → fail. ⇒ the lane KEEPS the raw
  key's context (no tool-identity predefines: `HAND_DEFINES` stays empty) — a per-file verdict depends on the
  branch parsed, and the portable branch is the one the raw answer key was written against; the 5 rows are a fact
  about their CONTEXT, recorded as `vendor_identity_branch`, not an engine or grammar defect.
- **119 of the 183 refuted rows dispositioned**: 103 `cross_file_base_class` (every sv-tests `*-uvm.sv` row: a
  class header whose base is in `uvm_pkg` — `.6d`'s population, gated on `.5`(d)); 9 `vendor_identity_branch`
  (the verilator rows above and their kin — `t_cover_line.v`, `t_dpi_result_type.v`, `t_gen_intdot2.v`,
  `t_var_types.v`); 7 `driver_define` — each with the driver line that defines the macro (`t_gate_ormux.py` /
  `t_hier_block_perf.py` `+define+SIM_CYCLES=`, `t_gen_missing.py` `+define+T_GEN_MISSING`, `t_time_sc_10_ns.py`
  `+define+TEST_EXPECT=200ns`, `t_time_vpi_100s10ms.py` `+define+time_scale_units=100s …`,
  `t_cover_line_tri_gate_cond.py` `+define+T_COND`, `t_to_string_emitted_class_hierarchy1.py` `-DOBJ_TYPE=…`) —
  the `.4c` population (reading drivers) sized at 7 rows here. **64 await** `.6b.4`: sv2v `macro.sv` /
  `macro_iv.sv` (`:137` / `:154`, the stringize `` `"…`" `` forms), `string_macro.sv`, `macro_macro.sv`, the slang
  driver tests (`` `FOO `` from `-I %data` includes the derived context does not carry), the verilator `_unsup` /
  `_bad` rows (`expect_filename` = Verilator's own expected error — an answer-key question), and ~30 parser
  questions (`foreach (x[i, j]);` with a null body, `bins auto[4]`, `->ev[i]`, `qa1[$-1].delete`, a `generate`
  region, …) to route to `SV-CORPUS-GRAD`.

**Acceptance checklist (`TASK-ACCEPTANCE`):**

  - [x] **REPRODUCE / ISSUE** — `.6a`'s 183 `refuted` rows with no way to record what each one is; the 9 verilator rows dying on `` `ifndef VERILATOR `error `` (origin `t_dpi_display.v:9:10` = the byte before the stub).
  - [x] **ROOT CAUSE (WHY + WHERE)** — correctness family: the probe's `[origin=…]` per row and the measured 5 / 24 / 87 / 9 movement with the predefines (`--sv-define VERILATOR --sv-define verilator`); ops family: `git ls-files stimuli/sv/characterization/ | grep -c disposition` → 0 before this slice (no disposition surface existed; `HAND_DEFINES` in `pipeline_context.py` has been empty since `.4`). WHERE: the instrument (no disposition input), the context (no tool-identity define — by decision). WHY: the raw key's context stance; the measured trade.
  - [x] **FIX** — the table, the instrument's disposition input + refusal + report split + summary field; 119 dispositions with evidence; `HAND_DEFINES` left empty on purpose.
  - [x] **ADDRESSED (verified)** — `pipeline_readjudication.py` → `refuted:cross_file_base_class=103 refuted:vendor_identity_branch=9 refuted:driver_define=7 refuted_awaiting_disposition=64`; `--check` OK; `check_pipeline_lane_currency.sh --self-test` 5/5.
  - [x] **NO REGRESSION** — the lane dump byte-identical to `-0011`'s (no lane re-run; no Rust byte staged, the `-0011` clippy verdict stands); `corroborated` 1 205 and the `must_reject` counts unchanged; the answer key and the manifests untouched; `bash scripts/check_doctrines.sh` ALL PASS (run at commit).
  - [x] **LOCKSTEP** — the SV parser book chapter (the disposition paragraph; HTML rebuilt), TOOLBOX §1.1b, this tree, `docs/TASK_TREE.md`, `MEMORY.md`, `CHANGES.md`, `DEVELOPMENT_NOTES.md`.
  - promotion: declined (the chapter and TOOLBOX carry the mechanism; no new card).

## `.6b.4` the last 64 refuted rows — ✅ done 2026-09-09 (`PGEN-SVPP-EXPANSION-0013`): every refuted row has a disposition

**What landed** (ANSWER-KEY-STYLE tier; ZERO Rust / grammar / codegen / generated bytes):

- The disposition vocabulary gains **`cross_file_define`** (a PREDECESSOR FILE of the unit defines the macro — §22.5.1's
  unit-wide macro; the per-file lane carries no unit list) and **`key_question`** (the parser is RIGHT to refuse: the
  text is not derivable, or the fixture exercises a tool's error path — the expectation is the question, routed to the
  answer key as a pin in the `.13y` family).
- **64 rows dispositioned, each with the measured expansion and the standard's clause** — the whole refuted population
  now reads: 103 `cross_file_base_class` · 27 `key_question` · 21 `parser_question` · 10 `preprocessor_item` · 9
  `vendor_identity_branch` · 9 `driver_define` · 4 `cross_file_define`; **`refuted_awaiting_disposition = 0`**.
- What the looks found (the toolbox on every row: `ast_pipeline --preprocess-systemverilog` for the expansion, the probe
  on the row and on the expanded text where the origin looked wrong):
  - ⭐ **The origin of a rejection is the last MAPPED byte, not the offending one.** Five slang rows read
    `string s = `FOO;` at the origin; the expansion is a legal string literal (`` `FOO → `BAR → `__FILE__ ``) and the parse
    stops on the NEXT item, a bare `begin end` at module level — the `.3.56` ruling. The origin mapping is right about
    the last byte the source map covers; the report's construct bucket read the wrong line. (A `.7` lockstep note.)
  - ⭐ **IEEE 1800-2017 §40.3.1 PREDEFINES the coverage-control macros** (`section-40-code-coverage-control-and-api.md:96-106`:
    *"The following predefined `define macros represent basic real-time coverage capabilities"*) — `` `SV_COV_START `` …
    `` `SV_COV_ERROR `` are the tool's, not an include's: three rows (sv-tests 20.14, Surelog CovMacro, verilator
    `t_cover_sys_unsup`) and an engine rule for `.6b.5`.
  - **`$display(display)` in a macro body**: the formal `display` was substituted INSIDE the system task name
    (`$"display"("display")`); A.9.3's `system_tf_identifier` is one token — an engine rule for `.6b.5` (2 rows).
  - **A block comment across directive lines**: Surelog PPComment opens `/*` on a source line and the `` `else `` inside the
    comment is honoured; iverilog br_gh782a puts `/* multi-line` after an `` `include `` filename and the include consumes
    the opener — two engine rules for `.6b.5`.
  - **The §34 protected envelope** (3 rows: the base64 payload read as source) — an engine rule for `.6b.5`.
  - **Existing rulings, applied**: a bare begin/end in a generate region or at module level (`.3.56`: 3 verilator `generate`
    rows + the 5 slang rows), loop variables in separate bracket pairs (`.13y` ForeachSquare: `t_randomize_array`), a
    parameter value assignment without parentheses (`.3.51`: Surelog PoundParam), a MIXED port list (`.13y`: `t_clk_concat2`),
    an edition contrast (`.13x.d`: `inside {[a +/- b]}` is 1800-2023).
  - **New must-reject candidates for the key** (`key_question`, each with the clause): mixed named/ordered parameter
    assignments (A.4.1.1), a replication inside an assignment pattern (A.8.1), `;` between an assertion and its `else`
    (A.6.10), a select on a function call's result (A.8.4), `bins x[4];` with no right-hand side (A.2.11), `` `uselib ``
    (no edition's directive), Verilator PGO configuration text in a `.v`, the `` `" `` macro-string forms outside macro
    text or closed by a plain `"` (§22.5.3.2), `N'{…}` (2017 has no sized pattern), `do` as a Verilog net name on the
    SV lane (Table B.1), UVM macros in lint fixtures that include nothing, a slang compat-mode constraint form.
  - **21 parser questions routed to `SV-CORPUS-GRAD`** with the construct and the clause: a foreach with a null body
    (A.6.8 / A.6.4, 2), an event trigger on an array element (A.6.5 / A.9.3, 2), a method call without parentheses
    (A.8.2), a streaming concatenation as an assignment target (A.8.1 / A.8.5), `foreach` in a constraint block (A.1.10),
    `std::randomize … with` (18.12), the `always` property operator (A.2.10), a prefix `++i` for-step (A.6.8 / A.8.3), a
    `$c(…)` system call, `@ ( * )` with spaces (9.4.2.2), five iverilog rows where a net / port / function / argument
    name shadows a `$unit` typedef (3.13 / 6.18 — a store-gate scoping question), and three rows whose chain starts at the
    probe's furthest token (`t_dpi_var`, `t_param_hier_bad`, sv2v `macro_macro`).
  - **Context facts**: 4 `cross_file_define` (sv2v siloed `package.svh`, slang `test.sv` → `test2.sv`, `file_with_no_eol.sv`,
    verilator `t_hier_block_import_def.vh` passed on the driver's command line) and 2 more `driver_define` (sv-tests'
    own `:defines:` header; `t_assert_inside_cond` — measured to PASS with its driver's define).

**Acceptance checklist (`TASK-ACCEPTANCE`):**

  - [x] **REPRODUCE / ISSUE** — `.6b.3` left `refuted_awaiting_disposition=64`; each row reproduced with `ast_pipeline --preprocess-systemverilog --output` (the expansion) and the probe (`[origin=…]`), `rust/target/svpp_e1/s6b4/`.
  - [x] **ROOT CAUSE (WHY + WHERE)** — correctness family: the probe's `furthest_position=1091` on the expanded slang text landed on `begin end` two lines past the mapped origin (the origin is the last mapped byte); the expansions beside each origin (`$"display"("display")`, `always @ ( * ) …`, the orphaned `comment */`); ops family: `git ls-files stimuli/sv/characterization/ | grep -c disposition` → 1 (the table exists since `-0012`; its vocabulary lacked the two classes the residue needed). WHERE: the table (no rows for the 64), the vocabulary. WHY: the clauses cited per row.
  - [x] **FIX** — 64 disposition rows with evidence; two vocabulary entries; the corpus-tree hand-off (`SV-CORPUS-GRAD.md`, the `.12b` pointer).
  - [x] **ADDRESSED (verified)** — `pipeline_readjudication.py` → `refuted_awaiting_disposition=0` (103 / 27 / 21 / 10 / 9 / 9 / 4); `--check` OK; `check_pipeline_lane_currency.sh --self-test` 5/5.
  - [x] **NO REGRESSION** — the lane dump byte-identical to `-0011`'s (no lane re-run; no Rust byte staged, the `-0011` clippy verdict stands); the answer key and the manifests untouched; `corroborated` 1 205 and the `must_reject` counts unchanged; `bash scripts/check_doctrines.sh` ALL PASS (run at commit).
  - [x] **LOCKSTEP** — the SV parser book chapter (the vocabulary; HTML rebuilt), TOOLBOX §1.1b, `SV-CORPUS-GRAD.md` (the routed rows), this tree (`.6b` DONE, `.6b.5` opened), `docs/TASK_TREE.md`, `MEMORY.md`, `CHANGES.md`, `DEVELOPMENT_NOTES.md`.
  - promotion: declined (the table and the chapter carry it; no new card).

## `.6b.5` four engine rules the dispositions named — ✅ done 2026-09-09 (`PGEN-SVPP-EXPANSION-0014`): and the UVM expansion corrected

**What landed** (`rust/src/sv_preprocessor.rs` only; ZERO grammar / codegen / generated bytes; 4 tests, 56/56; rustfmt-clean):

- **(a) `$display` is one token in substitution** — IEEE 1800-2017 A.9.3 `system_tf_identifier ::= $[a-zA-Z0-9_$]{…}`:
  a formal named like the tail of a system task name is not substituted inside it (`substitute_function_macro_body`,
  both scans: `bytes[start - 1] == b'$'`). sv2v `macro.sv` / `macro_iv.sv`: `` `define MACRO_D(display) $display(display); ``
  expanded to `$"display"("display")`. Test `a_formal_named_like_a_system_task_is_not_substituted_inside_it`.
- **(b) a block comment opened on an earlier line is comment text** (§5.4) — `preprocess_text_internal` carries
  `in_block_comment` across lines (`leaves_block_comment_open` decides at each source line's end, strings and `//`
  respected); a line inside the comment is emitted verbatim and never scanned for a directive; a line carrying the
  close emits whole when the close ends it (the first cut dropped that line's terminator and fused `*/endfunction` —
  caught by the UVM diff, verification (1)) or splits into the comment head and its own logical tail. And the text
  after an `` `include ``'s filename is the next logical line's (`include_directive_remainder` → `pending_tail`, the
  `.5`(b) rule). Surelog PPComment (`assign c= d;/*` then `` `else `` / `` `endif `` inside the comment were honoured),
  iverilog br_gh782a (`` `include "x.vi"  /* multi-line `` lost its opener). Test
  `a_block_comment_hides_directive_lines_and_survives_an_include_line`.
- **(c) the coverage-control macros are predefined** — IEEE 1800-2017 §40.3.1
  (`section-40-code-coverage-control-and-api.md:96-140`: *"The following predefined `define macros represent basic
  real-time coverage capabilities"*): `SV_COV_PREDEFINED_MACROS` (15 names, the clause's values) seeded in
  `PreprocessorState::new` before the context's predefines, so a context or source `` `define `` supersedes. Test
  `the_coverage_control_macros_are_predefined`.
- **(d) a §34 protected envelope is consumed as opaque** — from the `` `pragma protect `` line carrying
  `begin_protected` to the one carrying `end_protected` (§34.3 `section-34-protected-envelopes.md:47-50`; nested
  envelopes counted, `protected_depth`), every line is the envelope's and none reaches the parser; other `` `pragma ``
  lines keep their pass-through; an unterminated envelope refuses (`E_SVPP_UNTERMINATED_PROTECTED_ENVELOPE`); the event
  log records `protected_envelope` twice. Test `a_protected_envelope_is_consumed_as_opaque` (the ispras rot13 shape).

**Verification** (`rust/target/svpp_e1/verify_6b5.log`, `s6b5_lane.log`, the audit `s6b5/audit_uvm_diff.py`):

```text
(1) byte-identity: dv_utils_pkg / uart / dv_base_env IDENTICAL to the .5 bytes.  uvm_pkg DIFFERS from the 2026-08-24 artifact:
    3 022 806 → 3 020 226 B, 98 changed lines in 10 hunks — AUDITED: every hunk sits inside a block comment of the new
    expansion (hunks=10 in_block_comment=10 elsewhere=0). uvm_tlm_imps.svh:46-103 is a /* … */ block holding OLDER
    definitions of `UVM_BLOCKING_PUT_IMP / `UVM_NONBLOCKING_PUT_IMP (the live ones follow at :108 / :113): the 2026-08-24
    artifact had CONSUMED the commented-out `define lines as definitions (and expanded `uvm_info uses inside other block
    comments, uvm_reg_map.svh); the new expansion keeps the comments as comments. The artifact is SUPERSEDED, not matched.
(2) the .1 census: files=4157 rc0=4084 rc_nonzero=73 (unchanged)   (3) the (g)(h) fixtures: unchanged
(4) THE LANE (debug probe, 600 s, 447 s wall): PIPELINE-LANE: files=16336 pass=11692 fail=4367 pp_fail=277 timeout=0 crash=0
    against -0013's 11682 / 4376 / 278: 10 rows moved — 9 fail → pass: Surelog CovMacro/dut.sv, PPComment/dut.sv,
    PragmaProtect/top.sv; ispras 34.03.01_02.sv; iverilog br_gh782a.v; sv-tests 20.14--coverage.sv; sv2v macro_iv.sv;
    verilator t_cover_sys_unsup.v, t_lint_pragma_protected.v — and 1 pp_fail → pass: iverilog pr2823414.v ("C-Style comment
    in a skipped `ifdef is losing the newline" — the `endif was read inside the comment: unterminated conditional block).
    0 pass → fail, 0 pass → pp_fail. The terminator fix changed no verdict between the two runs (1 origin moved one line).
(5) re-adjudication: corroborated 1 205 → 1 213; the 8 dispositioned preprocessor_item rows that PASSED were REFUSED by the
    instrument as stale (the mechanism working) and dropped from the table; sv2v macro.sv re-dispositioned key_question
    (its next barrier: a `" macro string closed by a plain "); t_lint_pragma_protected_bad.v stays preprocessor_item
    (a malformed envelope — its driver's fails=True); refuted_awaiting_disposition=0; --check OK; self-test 5/5
(6) clippy_on_rust_change ✅ (CLIPPY_RC=0, peak 13 242 MB)
```

**Acceptance checklist (`TASK-ACCEPTANCE`):**

  - [x] **REPRODUCE / ISSUE** — the 10 `preprocessor_item` rows of the disposition table, each reproduced with `ast_pipeline --preprocess-systemverilog --output` (`$"display"("display")`, the `` `else `` honoured inside `/* … */`, the orphaned `comment */`, the base64 payload in the expansion).
  - [x] **ROOT CAUSE (WHY + WHERE)** — correctness family: the expansions beside each origin (`[origin=macro.sv:137:6]`, `dut.sv:6:19`, `br_gh782a.v:10:45`, `34.03.01_02.sv:14`), the parser's rejection of `$"display"` at the pasted token; ops family: `git log -S'in_block_comment' -- rust/src/sv_preprocessor.rs` → the line scanners tracked comments WITHIN a line only (no cross-line state ever existed); `git log -S'SV_COV' -- rust/src/sv_preprocessor.rs` → 0 commits. WHERE: `substitute_function_macro_body` (token boundary), `preprocess_text_internal` (no cross-line comment state, the include arm's remainder, no envelope state), `PreprocessorState::new` (no §40.3.1 seeds). WHY: A.9.3, §5.4, §22.5.1, §40.3.1 `:96-140`, §34.3 `:47-50`.
  - [x] **FIX** — engine tier: the four rules, `leaves_block_comment_open`, `include_directive_remainder`, `protect_pragma_has_keyword`, `SV_COV_PREDEFINED_MACROS`, the `protected_envelope` event kind; 4 tests.
  - [x] **ADDRESSED (verified)** — verification (4): 10 rows `fail`/`pp_fail → pass`, exactly the rows the four rules named plus pr2823414.v; `cargo test --lib sv_preprocessor` → `56 passed; 0 failed`.
  - [x] **NO REGRESSION** — three `.5` expansions byte-identical; the UVM expansion's every changed line audited inside a block comment (the reference artifact was wrong and is superseded — a 🔎 finding); the `.1` census 73 unchanged; 0 `pass → fail`, 0 `pass → pp_fail`; the `must_reject` rows unchanged (404 / 2 — the malformed `` `line `` and envelope rows still refuse); `clippy_on_rust_change` ✅; no grammar / codegen / generated byte moved (`generated-reproducibility: OK` at commit); `bash scripts/check_doctrines.sh` ALL PASS (run at commit).
  - [x] **LOCKSTEP** — the SV parser book chapter (the four rules; the UVM byte-identity claim corrected; HTML rebuilt), this tree, `docs/TASK_TREE.md`, `MEMORY.md`, `CHANGES.md`, `DEVELOPMENT_NOTES.md`; the tracked lane reports and the disposition table re-rendered.
  - promotion: declined (the chapter carries the rules; no new card).

🔎 **FINDING (director's review): the reference expansion of `uvm_pkg.sv` used by the external-corpus triage gate since
2026-08-24 (`case_uvm_pkg_2017.preprocessed.sv`, 3 022 806 B) was produced by a preprocessor that honoured `` `define ``
lines inside block comments (`uvm_tlm_imps.svh:46-103`) and expanded macro uses inside them. Every earlier "byte-identical
to the 2026-08-24 artifact" claim in this tree compared against that artifact; the claims were true of the bytes and the
bytes were wrong in 98 lines, all inside comments. The parser's verdict on `uvm_pkg.sv` is unchanged (it rejects at
`uvm_transaction.svh:443`, the `.5`(d) call). The triage gate's tracked reference will be re-baselined when that gate is
next re-run on HEAD (it is stale on `.3`'s finding already).

## `.6c.1` the coverage tuple under the pipeline — ✅ done 2026-09-09 (`PGEN-SVPP-EXPANSION-0015`): the axis-2 denominator moves through its own gate

**What landed** (INSTRUMENT + DOCTRINE tier; ZERO Rust / grammar / codegen / generated bytes):

- **`stimuli/sv/corpus_verdict_coverage.py` reads the re-adjudication table** (`--pipeline-readjudication`, default
  the tracked TSV; absent → the reading is NOT EVALUATED, loudly) and buckets every manifest row a second time, UNDER
  THE PIPELINE (`PIPELINE_CLASSES`, each verdict with its *why*; a row the pipeline did not re-read keeps its raw
  bucket). ⛔ Expected verdicts still come from the spec and the suite convention, never from the fix: a
  `chained:accepted_under_expansion` row is ADJUDICATED only in a DESIGN corpus (the adjudicator's `DESIGN_SUITES`:
  a shipping design's file is valid as part of its unit — 1 271 rows; 260 accepted rows of other suites stay NO
  VERDICT, no per-file expectation), `corroborated` (1 213) and `must_reject:match_under_expansion` (404) are
  ADJUDICATED matches, `refuted:parser_question` (21) is ADJUDICATED as a defect CANDIDATE on the bar (a must-accept
  row still refused in its own text once preprocessed, legal by the clause the disposition names; the corpus tree
  rules each), every other disposition / hollow / refused verdict is NO VERDICT with its reason, and
  `chained:rejects_own_file` (2 935) is DARK UNDER THE PIPELINE — the barrier behind the macro barrier. Output: a
  `## Under the PIPELINE` section of `coverage.md` and the tracked **`pipeline_coverage.tsv`** (bucket / class / rows /
  pct, plus a `TUPLE` row).
- **`SV-CORPUS-DENOMINATOR` gains the second reading** (`scripts/check_sv_corpus_denominator.sh`): the tracked
  `pipeline_coverage.tsv` must equal a fresh derivation (or the reading is NOT EVALUATED where the lane dump is
  absent), its `TUPLE` is derived, and the live anchor paragraph must carry BOTH tuples, the raw one first; the
  registry row, `DOCTRINE_ENFORCEMENT.md` §10 and the derived chapter say so; the anchor in
  `docs/book/src/grammar-wellformedness.md` carries both.
- The family-status blocks derive their sentence from `coverage.tsv` (unchanged) and stay green.

**THE MEASUREMENT** (`coverage.md`, *Under the PIPELINE*):

```text
raw                 adjudicated 7 529 / routed 2 606 / no-verdict 6 201 / dark 3 994 / bar 9
under the pipeline  adjudicated 8 669 / routed 2 606 / no-verdict 5 061 / dark 3 779 / bar 30
    ADJUDICATED +1 140:  corroborated 1 213 · design-corpus files accepted under expansion 1 271 · must_reject matches 404
                         · parser_question candidates 21 (the bar 9 → 30)
    NO VERDICT −1 140:   chained:rejects_own_file 2 935 (dark under the pipeline: 1 924 on an extends header) ·
                         chained accepted outside DESIGN_SUITES 260 · hollow 236 · refused 225 · the dispositions
                         (cross_file_base_class 103, key_question 27, vendor_identity_branch 9, driver_define 9,
                         cross_file_define 4, preprocessor_item 2) · svpp_owned 173
    DARK −215:           3 994 → 3 779 (the accepted design rows and the corroborated rows leave the dark)
```

🔎 **FINDINGS (director's review):** (1) the bar under the pipeline is **30**, not 9 — the 21 rows are must-accept text
the parser refuses once the macro barrier is gone, each named with its clause in the disposition table and routed to
`SV-CORPUS-GRAD`; the raw 9 was a reading of a lane that never reached them. (2) 1 271 design-corpus files are
ADJUDICATED under the pipeline by the suite convention alone — 2 935 more are dark there on a cross-file declaration
the per-file lane cannot supply: the library design (`.6d`) is where the next thousand verdicts are, and UVM's part of
it waits on the `.5`(d) ruling.

**Acceptance checklist (`TASK-ACCEPTANCE`):**

  - [x] **REPRODUCE / ISSUE** — the raw tuple `7529/2606/6201/3994/9` counted 4 927 chained-only rows as NO VERDICT and 1 434 explained rows as ADJUDICATED-explained while the pipeline had re-read every one of them (`.6a`); the coverage instrument had no pipeline input (`git log -S'pipeline_readjudication' -- stimuli/sv/corpus_verdict_coverage.py` → 0 commits).
  - [x] **ROOT CAUSE (WHY + WHERE)** — correctness family: the re-adjudication table's verdict counts (the `.6a` report) against the coverage buckets — the same rows, two readings, only one published; ops family: `git ls-files docs/tasks/artifacts/sv_corpus_grad/verdict_coverage/ | grep -c pipeline` → 0 before this slice (no artifact carried the reading under the pipeline), and `git log -S'pipeline_readjudication' -- stimuli/sv/corpus_verdict_coverage.py` → 0 commits. WHERE: `corpus_verdict_coverage.py` (no second reading), the gate (one tuple), the anchor (one tuple). WHY: the tree's `.6` goal — the axis-2 denominator moves through `SV-CORPUS-DENOMINATOR`'s own gate.
  - [x] **FIX** — `PIPELINE_CLASSES` + `pipeline_reading()` + the md section + `pipeline_coverage.tsv`; the gate's second tuple (freshness, derivation, the anchor); the anchor; the registry / §10 / derived chapter.
  - [x] **ADDRESSED (verified)** — `corpus_verdict_coverage.py` → `PIPELINE: 8669/2606/5061/3779/30`; `check_sv_corpus_denominator.sh` OK with both tuples; `check_family_status.sh` OK.
  - [x] **NO REGRESSION** — the raw tuple unchanged (`7529/2606/6201/3994/9`; `coverage.tsv` and `strata.tsv` byte-identical to `-0010`'s); the manifests untouched; no Rust byte staged (the `-0014` clippy verdict stands); `bash scripts/check_doctrines.sh` ALL PASS (run at commit).
  - [x] **LOCKSTEP** — the SV parser book chapter (the tuple under the pipeline), `DOCTRINE_ENFORCEMENT.md` §10 + the derived chapter, the platform book's anchor, this tree, `docs/TASK_TREE.md`, `MEMORY.md`, `CHANGES.md`, `DEVELOPMENT_NOTES.md`.
  - promotion: declined (the chapter and the doctrine text carry it; no new card).

## `.6c.2` the 28 key-question rows pinned — ✅ done 2026-09-09 (`PGEN-SVPP-EXPANSION-0016`)

**What landed** (ANSWER-KEY tier; ZERO Rust / grammar / codegen / generated bytes):

- **28 pins in `EXTRA_PINNED`** (`stimuli/sv/adjudicate_external_corpus.py`), one per `key_question` row of the disposition
  table, each carrying the row's clause (from `.6b.4`) and a MEASUREMENT: for the eleven construct families with no earlier
  ruling, a minimal reproducer that REJECTS on sv_2017 AND sv_2023 and a control that ACCEPTS on both
  (`rust/target/svpp_e1/s6c2/*.sv`: mixed named/ordered parameter assignments `fail@81`; a replication inside an assignment
  pattern `fail@65`; a `;` before an assertion's `else` `fail@49`; `bins auto[4];` with no right-hand side `fail@77`; a sized
  `5'{…}` pattern `fail@39`; `wire do;` `fail@14`; the `` `" `` macro-string form in source `fail@27` and one closed by a plain
  `"` `fail@32`; a UVM macro in an include-less class `fail@20`; `` `uselib `` inside a module `fail@9` — exactly as
  `` `resetall `` or any unknown directive there, while an unknown directive at UNIT level passes as opaque text, a documented
  tolerance); for the families with a ruling (`.3.56` bare begin/end ×8, `.13y` foreach brackets and the mixed port list, `.3.51`
  `#4`), the ruling's reproducer stands and the corpus row is measured under the pipeline on both profiles. ⭐ Two are EDITION
  CONTRASTS (the `.13x.d` shape): a select on a function call's result — 1800-2023 A.8.4 `primary ::= … |
  function_subroutine_call [ [ range_expression ] ]` (`section-Annex_A-normative-formal-syntax.md:3087`) admits ONE select and
  2017 none (`s = f()[4];` rejects on sv_2017, accepts on sv_2023; `t_bitsel_concat.v` then rejects on sv_2023 at `:97`
  `valarr()[1][2]`, two selects); and the `inside {[a +/- b]}` range — 2023 §11.4.13 (`t_inside_tolerance_unsup.v:18` accepts
  on sv_2023, where the row then rejects at `:20` on `[1490 +%- 10]`). 🔎 **The sv_2023 profile refuses `+%-`, a form 1800-2023
  admits beside `+/-` — a gap of the 2023 grammar, routed to `SV-CORPUS-GRAD`'s 2023 lane (`.14`).**
- **The raw key demotes them by design**: measured to scratch first — exactly 28 rows moved, 22 `explained_svpp_macro_use /
  _conditional → deferred:svpp_owned` and 6 `explained_svpp_include → deferred:chained_only` (the `include` arm of
  `reject_arm_demotion`), nothing else; the `svpp_owned` ceiling raised `207 → 229` with the argument. The re-adjudication
  instrument's `expected_of` now reads a pin behind BOTH demotion arms (`deferred:chained_only` too), and every one cashes:
  `must_reject:match_under_expansion 404 → 433`. The 28 rows left the disposition table (147 remain), the raw tuple
  `7529/2606/6201/3994/9 → 7501/2606/6229/4022/9`, the tuple under the pipeline `8669/2606/5061/3779/30 → 8698/2606/5032/3750/30`,
  the bar unchanged at 9 / 30; the manifests, summaries, coverage artifacts, provenance census, the dark-row audit, the sv2023
  parity report and the family-status blocks re-derived.

**Acceptance checklist (`TASK-ACCEPTANCE`):**

  - [x] **REPRODUCE / ISSUE** — the 28 `key_question` rows of `refuted_dispositions.tsv` (`.6b.4`): must-accept rows the parser is right to refuse.
  - [x] **ROOT CAUSE (WHY + WHERE)** — correctness family: the probe's `furthest_position` on each reproducer (`=81`, `=65`, `=49`, `=77`, `=39`, `=14`, `=27`, `=32`, `=20`, `=9`) against its accepting control, on both profiles, and each corpus row's `[origin=…]` on both profiles under the pipeline; ops family: `git ls-files stimuli/sv/adjudicate_external_corpus.py` + `grep -c 'SVPP-EXPANSION.6c.2' stimuli/sv/adjudicate_external_corpus.py` → 0 before this slice. WHERE: the answer key's expectations (the suite rules keyed vendor-tolerated text must_accept). WHY: the clauses named per pin (A.1.4, A.4.2, A.4.1.1, A.8.1, A.8.4, A.6.10, A.2.11, A.6.8, §22.5.3.2, §22.5.1, Table B.1, 2023 A.8.4 / §11.4.13).
  - [x] **FIX** — answer-key tier: 28 pins with cites and measurements; the instrument's `expected_of` (both demotion arms); the ceiling with its argument; the table trimmed.
  - [x] **ADDRESSED (verified)** — the adjudicator to scratch: exactly 28 rows moved (22 + 6); the re-adjudication `must_reject:match_under_expansion=433`, `refuted_awaiting_disposition=0`, `--check` OK; `check_sv_corpus_denominator.sh` OK on the new anchor (both tuples).
  - [x] **NO REGRESSION** — the lane dump byte-identical to `-0014`'s (no lane re-run; no Rust byte staged, the `-0014` clippy verdict stands); the bar unchanged (9 raw / 30 under the pipeline); `corroborated` 1 213 unchanged; the three corpus gates OK; `bash scripts/check_doctrines.sh` ALL PASS (run at commit).
  - [x] **LOCKSTEP** — the platform book's anchor, the SV parser book chapter (the numbers), `SV-CORPUS-GRAD.md` (the pins and the 2023 gap), this tree, `docs/TASK_TREE.md`, `MEMORY.md`, `CHANGES.md`, `DEVELOPMENT_NOTES.md`.
  - promotion: declined (the pins carry their own measurements; no new card).

## `.4c.1` the drivers' DEFINES — ✅ done 2026-09-09 (`PGEN-SVPP-EXPANSION-0017`): every file under its own driver's configuration

**What landed** (LANE + ANSWER-KEY tier; ZERO Rust / grammar / codegen / generated bytes):

- **A third derived context table** — `stimuli/sv/pipeline_context_defines.tsv` (`suite relpath define source`), written
  and checked by `stimuli/sv/pipeline_context.py` beside the two include tables (`derive_defines`, `driver_defines`,
  `render_defines`; the `--check` and the `PIPELINE-LANE-CURRENCY` doctrine's arm (1) cover all three; 4 new self-test
  checks, 14 in all). verilator: the same-stem `t_x.py`'s `+define+NAME[=VALUE]` / `-DNAME[=VALUE]` tokens, or a SINGLE
  variant driver naming the file as `top_filename`; sv-tests: the `:defines:` header. ⛔ Derived, never guessed: a value
  the driver computes at run time (`"+define+SIM_CYCLES=" + str(test.cycles)`, an f-string) is REPORTED as `computed`
  (3 files), a file several variant drivers configure differently is REPORTED as a `tie` (24 files), the spans whose
  `-D` reaches the C++ compiler or iverilog (`-CFLAGS`, `make_flags=[CPPFLAGS_ADD=…]`, `CPPFLAGS_DRIVER=…`, `iv_flags`)
  are skipped, and a `*_common.py` helper is not a driver. MEASURED: the first, permissive scan emitted 94 rows —
  `VL_NO_LEGACY` ×5, `CPP_MACRO=cplusplus`, `WAVES` ×7 (iverilog-only), `TEST_HDR_TRACE=1` and `trace_two_common.py`'s
  conditional flags among them — the strict scan emits 56 (53 verilator on 45 files, 3 sv-tests). 🔎 **A `-D` in a
  Python test driver is not necessarily an SV define** — a third of the permissive rows were C++ flags.
- **The lane applies them** — `stimuli/run_pipeline_lane.sh`'s `context_args` emits `--sv-define` for the file's rows
  after its include rows and before the suite's (`CONTEXT_DEFINES`); the report's identity block names the table by
  digest and a `### Per-file defines` section counts it per suite; the atomic-publish staging file is `.gitignore`d.
- **The whole lane re-run** (debug probe `d7e17a94…`, the `-0016` probe unchanged; 6 jobs, 600 s, 463 s wall, peak
  9 955 MB under the 14 336 MB guard): 16 336 rows, `pipeline pass 11 692 → 11 693`; the dump diffed row by row
  against the `-0016` dump (`rust/target/svpp_e1/s4c1/diff.py`): **exactly 5 movers, all among the files with
  defines** — `fail → pass` sv-tests `5.6.4--…preprocessor-macro_0.sv` (`TEST_VAR`), `…macro_1.sv` (`VAR_1=2 VAR_2=5`),
  verilator `t_assert_inside_cond.v` (`T_ASSERT_INSIDE_COND`); `pass → fail` `t_vpi_module.v` and
  `t_vpi_public_depth.v` (`USE_DOLLAR_C32`, both at `[origin=…:20:5]`); 0 movers elsewhere.
- **Three pins** (`EXTRA_PINNED`, a `.4c.1` block): 🔎 **a driver's define can select a VENDOR-EXTENSION branch** —
  under `USE_DOLLAR_C32` the two `t_vpi_*` rows' selected branch is `` `systemc_header `` / `extern "C" int mon_check();`
  / `` `verilog ``, Verilator's own directives (neither among IEEE 1800-2017 clause 22's compiler directives) around
  C++ text no production of Annex A derives: the file is standard SV only WITHOUT its driver's configuration.
  Reproducer `rust/target/svpp_e1/s4c1/vendor_directive_repro.sv` `fail@9` (`:1:10`) on sv_2017 AND sv_2023, the control
  (the three lines removed) accepts on both; at unit level the directive passes as opaque text and the C++ line is
  refused (`:2:7`). And `t_gen_missing.v`: under `T_GEN_MISSING` the parse leaves the `` `error `` stub behind and stops
  at line 30 `generate` + `begin: g` — a bare begin/end inside a generate region, the `.3.56` ruling (A.4.2);
  `gen_bare_begin_repro.sv` `fail@144` (`:3:11`) on both profiles, the control (wrapper removed, the `;` after
  `begin: foo_inst` kept — a legal empty `package_or_generate_item_declaration`) accepts on both. ⭐ The
  `driver_define` disposition had been HIDING a `.3.56` row. The adjudicator moved exactly these 3 rows
  (`explained_svpp_macro_use → deferred:svpp_owned`, the conditional arm); the `svpp_owned` ceiling `229 → 232` with
  the argument; the re-adjudication cashes them: `must_reject:match_under_expansion 433 → 436`, `corroborated` 1 213
  (+2 −2), `refuted:driver_define 9 → 6`, `refuted_awaiting_disposition=0`.
- **The disposition table** — 147 → 144 rows: the three rows that stopped being refuted left it (the instrument
  refuses a stale disposition); the six `driver_define` rows that remain now each name WHY the define cannot be derived
  (computed: `t_gate_ormux.v`, `t_hier_block_perf.v`; a tie: `t_time_sc.v`, `t_time_vpi.v`, `t_tri_gate.v`,
  `t_to_string_emitted_class_hierarchy.v`), and the kind's definition says so (instrument + derived report).
- **The denominator moved through its gate**: raw `7501/2606/6229/4022/9 → 7498/2606/6232/4025/9`, under the pipeline
  `8698/2606/5032/3750/30 → 8701/2606/5029/3746/30`; the bar unchanged at 9 / 30; the manifests, summaries, coverage
  artifacts, provenance census, the dark-row audit, the sv2023 parity report, the family-status blocks and the anchor
  re-derived.

**Acceptance checklist (`TASK-ACCEPTANCE`):**

  - [x] **REPRODUCE / ISSUE** — the 9 `driver_define` rows of `refuted_dispositions.tsv` (`.6b.3`): the derived context carried no driver defines, so a file was measured under a configuration its own driver never ran.
  - [x] **ROOT CAUSE (WHY + WHERE)** — ops family: `git ls-files stimuli/sv/pipeline_context*.tsv` listed two tables and no defines table; `stimuli/run_pipeline_lane.sh` `context_args` emitted `--sv-include-dir` only. Correctness family, per mover: the probe's `[origin=…]` with and without the driver's define (`t_vpi_module.v:20:5`, `t_vpi_public_depth.v:20:5`, `t_gen_missing.v:30:11`, `t_assert_inside_cond.v:23:20` before) and `furthest_position=9` / `=144` on the reproducers against their accepting controls, on both profiles.
  - [x] **FIX** — the deriver (`derive_defines`, strict spans, helpers excluded, computed/tie reporting), the lane's arm, the report's identity row and section, three pins with cites and measurements, the ceiling with its argument, the table trimmed and reworded.
  - [x] **ADDRESSED (verified)** — `pipeline_context.py --self-test` 14/14; `--check` OK; the lane re-run: exactly 5 movers, all among the files with defines, 0 elsewhere; the adjudicator moved exactly 3 rows; `must_reject:match_under_expansion=436`, `refuted_awaiting_disposition=0`; `check_sv_corpus_denominator.sh` OK on both tuples.
  - [x] **NO REGRESSION** — the probe binary byte-identical to `-0016`'s (digest `d7e17a94…`; no Rust byte staged, the `-0014` clippy verdict stands); 16 331 of 16 336 rows byte-identical in verdict, the 5 movers each read; the bar unchanged (9 raw / 30 under the pipeline); `corroborated` 1 213 unchanged; `PIPELINE-LANE-CURRENCY` OK + `--self-test` 5/5; the three corpus gates and `check_family_status.sh` OK; `bash scripts/check_doctrines.sh` ALL PASS (run at commit).
  - [x] **LOCKSTEP** — `TOOLBOX.md` §1.1b, the SV parser book chapter (the paragraph, the numbers), the platform book's anchor, the family-status blocks, the doctrine's header, this tree, `docs/TASK_TREE.md`, `MEMORY.md`, `CHANGES.md`, `DEVELOPMENT_NOTES.md`.
  - promotion: declined (the pins carry their own measurements; no new card).

## `.4c.2` the harness's UNIT LISTS — ✅ done 2026-09-09 (`PGEN-SVPP-EXPANSION-0018`): every file after the predecessors its own harness compiles first; `.4c` closed

**What landed** (LANE tier; ZERO Rust / grammar / answer-key / codegen bytes):

- **A fourth derived context table** — `stimuli/sv/pipeline_context_units.tsv` (`suite relpath order unit_file source`), written
  and checked by `stimuli/sv/pipeline_context.py` (`derive_units`, `driver_unit_files`, `units_from_lists`, `render_units`;
  the `--check` and the doctrine's arm (1) cover all four tables; 6 new self-test checks, 20 in all). Three declaration
  dialects, each read as the harness states it: verilator — the source-file arguments (`t/…`, `$TEST_ROOT/…` → the test's
  own directory) in the same-stem driver's SV flag lists, in command order, the top file excluded (`driver.py:1099` puts
  `*flags` before `top_filename`); slang — `tests/unittests/data/*.f` command files (`-F` expanded, `-v` library files
  skipped; `unit.f` `--library mylib test.sv test2.sv -DFOOBAR -Ilibtest`, `cmd3.f` `test2.sv -F cmd4.f test4.sv`), the units
  UNIONED in declaration order; sv2v — `test/*/run.sh` explicit multi-file `convert "…" $out FILES` / `runAndCapture FILES`
  lines (sv2v lexes its inputs as one unit; `--siloed` says so by its own help text). ⛔ Derived, never guessed: a `-f` list
  (5 verilator drivers; Verilator's own option-file syntax with `+define+A+B` forms and env substitutions — not read), a glob
  (`runAndCapture *.sv`, 12 lines of `write/run.sh`), a `--siloed` run and a pair two declarations order differently (judged
  on a snapshot, dropped from both sides) are REPORTED (`unit` lines), never applied. 44 rows on 17 files: verilator 12
  files / 38 rows (`t_sv_cpu.v` alone 18), slang 3 / 4, sv2v 2 / 2. And a `.4c.1` precision fix found on the way: a
  `-DNAME` the same command line `-U`s again (`t_flag_define.py` `-DCMD_UNDEF -UCMD_UNDEF`) is not a define the driver
  passes — one row left the defines table (56 → 55).
- **The lane applies them** — `context_args` emits `--sv-unit-file` for the file's rows first, in order (`CONTEXT_UNITS`);
  the report's identity block names the table by digest and a `### Per-file unit predecessors` section counts it per suite.
- **The whole lane re-run** (the unchanged debug probe `d7e17a94…`; 6 jobs, 600 s, 468 s wall, peak 9 390 MB under the guard):
  16 336 rows, `pipeline pass 11 693 → 11 696`; diffed row by row against the `-0017` dump
  (`rust/target/svpp_e1/s4c1/diff_4c2.py`): **exactly 3 movers, all `fail → pass`, all among the 17 files with predecessors**
  — sv2v `test/siloed/module.sv` after `package.svh` (was `:8:18`), slang `data/test2.sv` after `test.sv` (was `:3:8`),
  verilator `t_hier_block_import.v` after `t_hier_block_import/t_hier_block_import_def.vh` (was `:20:16`); 0 the other way;
  the 14 other files with predecessors (t_sv_cpu.v with 18 among them) `pass → pass` — no predecessor's own preprocessing
  turned a row. Each mover pre-measured with the hand-given predecessor on sv_2017 AND sv_2023 (rc 0, `[preprocessed]`).
- **The fourth `cross_file_define` row is a corrected claim** — 🔎 `.6b.4` had written *"the test pairs the two files as one
  unit"* for `file_uses_define_in_file_with_no_eol.sv`; RE-CHECKED: no unittest `.cpp` and no `data/*.f` in the vendored slang
  tree names either file (`grep -rl` over the tree: only the data files themselves and an unrelated `single-unit-no-eol.sv`)
  — the pairing had been read off the file names. MEASURED: hand-given as `--sv-unit-file`, the pair accepts on both profiles;
  standalone the row rejects at `:3:31` on the unexpanded usage. The row's basis now says that; the deriver reports nothing
  for it (a unit nobody declares is not derived); the kind's definition (instrument + derived report) names the `.4c.2`
  state. The disposition table 144 → 141 rows; `refuted:cross_file_define 4 → 1`, `corroborated 1 213 → 1 216`,
  `refuted_awaiting_disposition=0`.
- **The denominator moved through its gate** (no key change): under the pipeline `8701/2606/5029/3746/30 →
  8704/2606/5026/3743/30`, the raw tuple `7498/2606/6232/4025/9` unmoved; the bar unchanged at 9 / 30; the family-status
  blocks and the anchor re-derived. `.4c` (drivers `.4c.1` + unit lists `.4c.2`) is CLOSED; reading OpenTitan's `.core` /
  `.f` build declarations stays priced, not built (`.4b`).

**Acceptance checklist (`TASK-ACCEPTANCE`):**

  - [x] **REPRODUCE / ISSUE** — the 4 `cross_file_define` rows of `refuted_dispositions.tsv` (`.6b.4`): a macro a predecessor file of the unit defines, which the per-file lane did not carry.
  - [x] **ROOT CAUSE (WHY + WHERE)** — ops family: `git ls-files stimuli/sv/pipeline_context*.tsv` listed three tables and no units table; `stimuli/run_pipeline_lane.sh` `context_args` emitted no `--sv-unit-file`. Correctness family, per row: the probe's `[origin=…]` standalone (`module.sv:8:18`, `test2.sv:3:8`, `t_hier_block_import.v:20:16`, `…no_eol.sv:3:31`) against rc 0 `[preprocessed]` with the hand-given predecessor, on both profiles; the slang pairing claim re-checked by `grep -rl` over the vendored tree (no declaration).
  - [x] **FIX** — the deriver (three dialect readers, the union/conflict rule, reporting), the `-U` rule, the lane's arm, the report's identity row and section, the table trimmed and the fourth row's basis corrected, the kind's definition.
  - [x] **ADDRESSED (verified)** — `pipeline_context.py --self-test` 20/20; `--check` OK; the lane re-run: exactly 3 movers, all `fail → pass`, all among the files with predecessors, 0 elsewhere; `refuted:cross_file_define=1`, `refuted_awaiting_disposition=0`; `check_sv_corpus_denominator.sh` OK on both tuples.
  - [x] **NO REGRESSION** — the probe binary byte-identical to `-0016`'s (digest `d7e17a94…`; no Rust byte staged, the `-0014` clippy verdict stands); 16 333 of 16 336 rows byte-identical in verdict, 0 `pass → fail`; the bar unchanged (9 raw / 30 under the pipeline); the raw tuple unmoved; `PIPELINE-LANE-CURRENCY` OK + `--self-test` 5/5; the three corpus gates and `check_family_status.sh` OK; `bash scripts/check_doctrines.sh` ALL PASS (run at commit).
  - [x] **LOCKSTEP** — `TOOLBOX.md` §1.1b, the SV parser book chapter (the paragraph, the numbers), the platform book's anchor, the family-status blocks, the doctrine's header, this tree, `docs/TASK_TREE.md`, `MEMORY.md`, `CHANGES.md`, `DEVELOPMENT_NOTES.md`.
  - promotion: declined (no new card; the derivation carries its own self-test).

## `.6d.1` the LIBRARY PASS — ✅ done 2026-09-09 (`PGEN-SVPP-EXPANSION-0019`): the design suites' packages in dependency order, every row with the library

**What landed** (LANE tier; ZERO Rust / grammar / answer-key / codegen bytes — the engine's `--lib-in` / `--lib-out` and the grammar's
`@export_to_library` / `@import_from_library` were already there; this slice is the PASS that uses them on the corpus):

- **The mechanism, measured first** (scratch `rust/target/svpp_e1/s6d/`): `prim_mubi_pkg.sv --lib-out` writes `package/prim_mubi_pkg.facts.json`
  (8 `type_name` facts, `declaration_family = typedef`); `rom_ctrl_mux.sv` (dark on `mubi4_t` at `:21:16`) ACCEPTS with `--lib-in` of it; a
  same-directory chain works (`pinmux_reg_pkg` + `prim_pad_wrapper_pkg` exported, `pinmux_pkg.sv` `:19:15 → pass` with `--lib-in`/`--lib-out`
  on the same dir, exporting itself); a NAMED import works (`rram_ctrl_pkg.sv` `import rram_ctrl_reg_pkg::name` `:211:44 → :221:12`, the next
  barrier a `mubi4_t` not yet in that scratch library).
- **`stimuli/sv/pipeline_library.py`** — inventory (every `*.sv`/`*.svh` of `DESIGN_SUITES` declaring `package NAME;`), edges from the
  PREPROCESSED text (`ast_pipeline --preprocess-systemverilog` with the file's derived context; a refused expansion falls back to the raw
  text and is noted), a topological order (Kahn, ties by path; cycles reported and parsed last — none found), each file parsed with the probe,
  its own context, `--lib-in LIB` and a private `--lib-out`, and a passing file's exports PUBLISHED into
  `stimuli/sv/characterization/pipeline_library/` — at once for a package one file declares, and for a package several files declare only
  when every declaring file passed and the normalised fact sets agree (`variants_agree`), else `variants_disagree` / `variants_incomplete`,
  reported. The per-package dump `pipeline_library.tsv` (`suite status relpath packages secs rc furthest origin exported note`; `.gitignore`d).
- **`stimuli/sv/pipeline_context_args.sh`** — the ONE sourceable place the four context tables (+ `--lib-in` when `PGEN_PIPELINE_LIB_DIR`
  is set) become probe flags; the lane sources it, the builder calls it (`bash -c 'source …; context_args S F'`) — no second copy of the awk.
  The lane runs the pass before the per-file rows (`PGEN_PIPELINE_LIBRARY=0` skips it) and every row gets `--lib-in`. `DESIGN_SUITES` now has
  ONE definition (`pipeline_context.py`; `corpus_verdict_coverage.py` imports it). The report's identity block names the library dump and a
  `## The library the lane built` section summarises it per suite.
- **The whole lane re-run** (the unchanged debug probe `d7e17a94…`; 6 jobs, 600 s; 689 s wall incl. the pass, peak 9 797 MB under the
  guard): 16 336 rows, `pipeline pass 11 696 → 12 121`; diffed row by row against the `-0018` dump (`rust/target/svpp_e1/s6d/diff_6d1.py`):
  **425 movers, ALL `fail → pass`, 0 `pass → non-pass`** — OpenTitan 367 (1 077 → 1 444 of 3 983), VeeR 58 (23 → 81 of 102; all 58 had
  stopped at `el2_param.vh:182`, `el2_pkg`'s typedef); black-parrot, uvm-core, friscv, scr1 unmoved. The library: 407 package files — 230
  pass, 170 fail, 7 refused (black-parrot's 4 + OpenTitan's 3, the `.4b` absent-header class); 166 packages published (130 OpenTitan direct
  + 31 agreeing variants, 3 black-parrot, 2 VeeR); slowest `uvm_pkg.sv` 20.8 s.
- **What still rejects, measured**: 🔎 **164 of OpenTitan's 168 rejecting packages import `uvm_pkg` directly** (all under `/dv/`; top origins
  `clkmgr/pwrmgr/rstmgr_base_test.sv:5:45` `extends cip_base_test #(…)`, `*_env_pkg.sv`) — `uvm_pkg.sv` rejects at
  `uvm_transaction.svh:443:38` (the `.5`(d) `const local` construct) and exports nothing, so the DV chain waits on the director's ruling
  (or an export-the-accepted-prefix engine change). 🔎 **Six per-top register packages DISAGREE** (`ast_reg_pkg`, `clkmgr_reg_pkg`,
  `gpio_reg_pkg`, `ibex_pmp_reset_pkg`, `otp_ctrl_reg_pkg`, `pwrmgr_pkg`: earlgrey vs darjeeling register maps) — not published; the two
  RTL package failures are exactly their cost (`otp_ctrl_part_pkg.sv` on both tops, `:535:44` / `:338:44`, a `function automatic
  otp_ctrl_core_hw2reg_t …` return type from `otp_ctrl_reg_pkg`); 21 more variant packages are `incomplete` (a declaring file rejects).
  🔎 **44 imported packages are declared by no file** (`*_ral_pkg` ×44 — the generated register-abstraction packages OpenTitan's flow emits;
  the `.4b` "absent generated headers" class, now with names). The re-adjudication: `chained:accepted_under_expansion 1 531 → 1 956`,
  `chained:hollow 236 → 178`, `chained:rejects_own_file 2 934 → 2 567`, `corroborated` 1 216 and the 436 must-reject matches unchanged, nothing
  refused; the tuple under the pipeline `8704/2606/5026/3743/30 → 9129/2606/4601/3318/30`, the raw one unmoved, the bar 9 / 30 unchanged.
- **Next**: `.6d.2` — the unit-membership reading (an `` `include ``d class file has no `import` of its own; its verdict is its package
  unit's at that file's origin: OpenTitan's 157 hollow + 2 373 own-file rows sit behind passing or rejecting packages); the (d) ruling.

**Acceptance checklist (`TASK-ACCEPTANCE`):**

  - [x] **REPRODUCE / ISSUE** — the `.13c` dark audit: 2 057 `FACT-GATED-CROSS-FILE` + 168 `CROSS-LIBRARY` rows whose failure byte is a name a package in another file declares (676 OpenTitan RTL rows among them); the pre-scoping (i)–(v) in `.6`.
  - [x] **ROOT CAUSE (WHY + WHERE)** — ops family: `git ls-files stimuli/sv/pipeline_library.py` listed nothing and `stimuli/run_pipeline_lane.sh` passed no `--lib-in`; correctness family: the probe's `[origin=…]` per mover before (`rom_ctrl_mux.sv:21:16`, `pinmux_pkg.sv:19:15`, `rram_ctrl_pkg.sv:211:44`, `el2_param.vh:182:22`) against rc 0 with the exported facts, and the facts file's `declaration_family = typedef` entry for the gating name.
  - [x] **FIX** — the builder (inventory, expansion edges, topological order, parse-and-publish with the variant rule), the sourceable context script, the lane's pass, the report's section and identity row, one `DESIGN_SUITES`.
  - [x] **ADDRESSED (verified)** — the builder's smoke run on the three small suites (7 published) then the full pass (166 published, 0 cycles); the lane re-run: 425 movers, all `fail → pass`; `check_sv_corpus_denominator.sh` OK on the new tuple; `PIPELINE-LANE-CURRENCY` OK + `--self-test` 5/5; `pipeline_context.py --self-test` 20/20.
  - [x] **NO REGRESSION** — the probe binary byte-identical to `-0016`'s (digest `d7e17a94…`; no Rust byte staged, the `-0014` clippy verdict stands); 15 911 of 16 336 rows byte-identical in verdict, 0 `pass → non-pass`; the raw tuple unmoved; the bar unchanged (9 raw / 30 under the pipeline); `corroborated` and the must-reject matches unchanged; the three corpus gates and `check_family_status.sh` OK; `bash scripts/check_doctrines.sh` ALL PASS (run at commit).
  - [x] **LOCKSTEP** — `TOOLBOX.md` §1.1b, the SV parser book chapter (the mechanism and the measurement), the platform book's anchor, the family-status blocks, the doctrine's header, this tree, `docs/TASK_TREE.md`, `MEMORY.md`, `CHANGES.md`, `DEVELOPMENT_NOTES.md`.
  - promotion: declined (the pass carries its own dump and report; no new card).

## `.6d.2` the UNIT-MEMBERSHIP reading — ✅ done 2026-09-09 (`PGEN-SVPP-EXPANSION-0020`): a member's verdict is its unit's; measured idle behind the UVM gate

**What landed** (LANE + INSTRUMENT tier; ZERO Rust / grammar / answer-key / codegen bytes):

- **The pass records membership** — `stimuli/sv/pipeline_library.py` asks the preprocessor for the source map of every package
  unit's expansion (`--sv-source-map-json`) and writes `stimuli/sv/characterization/pipeline_library_members.tsv`
  (`suite package_relpath member_relpath`; a dump, `.gitignore`d): 5 074 rows — OpenTitan 4 884, uvm-core 186, black-parrot 3, VeeR 1;
  2 060 distinct member files.
- **The re-adjudication reads a member through its unit** (`read_units`, `unit_reading`; a `unit` column in the per-row TSV): for a
  row of the explained_svpp / chained_only populations that fails per file and is a member of ≥ 1 unit — `accepted_in_unit` when a unit
  including it PASSES (its bytes were parsed there), the per-file verdict KEPT when a unit's rejection origin is in this file (the
  per-file verdict IS the unit's), `unreached_in_unit` when every including unit rejects elsewhere (never judged; the first unit named).
  must_reject and svpp_owned rows are left out by design (a fragment the key expects rejected is not accepted by membership; the
  preprocessor lane owns its rows). The report's `## The unit-membership reading` section counts it per suite; the coverage census
  buckets `accepted_in_unit` ADJUDICATED in the design suites (exactly as `accepted_under_expansion`) and `unreached_in_unit` NO
  VERDICT (dark, named); the lane report's library table gains the member-file count.
- **Measured** (the library re-derived standalone, 226 s; the same 166 packages, verdicts identical to the `-0019` run): 2 059 routed
  members re-read — **`accepted_in_unit` 0, kept 89, `unreached_in_unit` 1 923** (OpenTitan 1 783, uvm-core 140). VERIFIED against the
  dumps directly: 31 distinct members belong to a passing unit and 0 of them fail per file (they are RTL headers; every passing
  package is RTL, and RTL packages `include` little); the 2 012 (rejecting unit, failing member) pairs are the DV class files.
  🔎 **The reading is in place and IDLE behind one ruling**: every package that `` `include ``s class files imports `uvm_pkg`, and
  `uvm_pkg.sv` exports nothing until `.5`(d) is ruled — 1 923 member rows join the 164 packages and the 273 rows gated at their
  failure point as the ruling's measured price; the same lane run converts them the day it lands (`.6d.3`, blocked).
  The dark population is now NAMED by its unit's origin: `chained:rejects_own_file 2 567 → 718`, `chained:hollow 178 → 104`,
  `chained:unreached_in_unit` 1 923; the tuple under the pipeline UNMOVED at `9129/2606/4601/3318/30` (an unreached member was
  dark before and is dark now, with a reason), the raw tuple unmoved, the bar 9 / 30.

**Acceptance checklist (`TASK-ACCEPTANCE`):**

  - [x] **REPRODUCE / ISSUE** — after `.6d.1`, OpenTitan's 157 `chained:hollow` + 2 373 `chained:rejects_own_file` rows: `` `include ``d class files with no `import` of their own, rejected per file on their first cross-file name whatever the library holds.
  - [x] **ROOT CAUSE (WHY + WHERE)** — ops family: `git ls-files stimuli/sv/characterization/pipeline_library_members.tsv` lists nothing (a dump) and the instrument had no membership input; correctness family: the source map of `cip_base_pkg.sv`'s expansion (`--sv-source-map-json`: segments with `source.file`) names its members, and the per-file `[origin=…]` of a member (`chip_sw_base_vseq.sv`) is its first `extends` base while its unit's `[origin=…]` is `clkmgr_base_test.sv:5:45`-class — the unit's, not the member's.
  - [x] **FIX** — the builder's membership table; the instrument's reader, reading rule and `unit` column; the coverage classes; the report's column and section; TOOLBOX / chapter / doctrine text.
  - [x] **ADDRESSED (verified)** — the re-adjudication renders (`refuted_awaiting_disposition=0`, nothing refused); the zero-accepted count re-derived from the dumps by a second script (31 passing-unit members, 0 failing per file); `corpus_verdict_coverage.py` buckets the two new classes (no REFUSING); `check_sv_corpus_denominator.sh` OK, the tuple unmoved.
  - [x] **NO REGRESSION** — the lane dump byte-identical to `-0019`'s (no lane re-run; the library re-derived standalone with identical verdicts: 166 published, 225/168/3 OpenTitan); no Rust byte staged (the `-0014` clippy verdict stands); `corroborated` 1 216, the 436 must-reject matches, the bar 9 / 30 and both tuples unchanged; `PIPELINE-LANE-CURRENCY` OK + `--self-test` 5/5; the corpus gates and `check_family_status.sh` OK; `bash scripts/check_doctrines.sh` ALL PASS (run at commit).
  - [x] **LOCKSTEP** — `TOOLBOX.md` §1.1b, the SV parser book chapter, the doctrine's header, this tree, `docs/TASK_TREE.md`, `MEMORY.md`, `CHANGES.md`, `DEVELOPMENT_NOTES.md`.
  - promotion: declined (an instrument extension; no new card).

## `.6d.3` the UVM LIBRARY — ✅ done 2026-10-02 (`PGEN-SVPP-EXPANSION-0029`): `uvm_pkg.sv` exports, its members are read through their unit, and the price measured behind the ruling is re-measured

**What landed** (docs only; the conversion itself was done by release `1.0.247`'s own lane run, `PGEN-SV-CORPUS-GRAD-0423` — this slice measures and records it, and names the next barrier with the probe):

- **`uvm_pkg.sv` exports.** The release's lane: `PIPELINE-LIBRARY: Cores-VeeR-EL2=2/2 black-parrot=3/7 opentitan=345/396 uvm-core=2/2 published_packages=256 unresolved_imports=44 cyclic_files=0 member_rows=5074`; the dump's rows `uvm-core pass …/src/uvm_pkg.sv uvm_polling_pkg uvm_pkg 91.36 0 … uvm_pkg` (the slowest file of the pass) and `uvm-core pass …/compat/uvm_compat_pkg.sv uvm_compat_pkg 2.28 0 … uvm_compat_pkg`. The `.6d.1` run (2026-09-09) published 166 packages; today's publishes 256.
- **The members, read through their unit.** uvm-core's 168 `include`d class files, in the tracked `pipeline_readjudication.md` at `1.0.246` versus `1.0.247`: `accepted_in_unit` 2 → 142, kept (rejection lands in the file) 1 → 0, `unreached_in_unit` 139 → 0; across the chained population `chained:unreached_in_unit` 1,099 → 960 and `chained:accepted_in_unit` 880 → 1,020; the verdict-coverage tuple under the pipeline `10296/2606/3434/2150/0` → `10439/2606/3291/2005/0` — the 143 rows adjudicated are these members plus the release's five per-file movers (`uvm.sv`, `uvm_pkg.sv`, verilator's two `uvm_pkg_all_v2020_3_1_*.svh`, VeeR's `testbench/uvm/mem/hdl/tbench_top.sv`; uvm-core per file `pass` 29 → 31).
- **The 2026-09-09 price, re-measured.** (i) *164 rejecting OpenTitan packages importing `uvm_pkg`*: 48 OpenTitan package files reject today (46 of them import `uvm_pkg`); of the 186 package files under `/dv/`, 140 pass — and the same 140 passed under `1.0.246`. The DV packages were converted BEFORE this ruling by the fact-gate escapes (`SV-CORPUS-GRAD.27` the wildcard type, `.28` the parameterized class type, `.31` the class-scope wildcard; 2026-09-22/23), which let a package accept the `uvm_*` names it imports without the facts; the export changed none of their verdicts (`PIPELINE-MOVERS rows_before=16336 rows_after=16336 moved=5 added=0 lost=0`, none OpenTitan). (ii) *1 923 unreached member rows*: 960 today, all OpenTitan — uvm-core's 139 converted by this release, OpenTitan's 1 783 → 960 by the escapes earlier. (iii) *273 dark rows gated on a `uvm_*` name*: 11 (`a uvm_* name (typed, not a macro) | 11`, the same 11 before and after).
- **The next barrier, named with the probe (TOOLBOX-FIRST).** The 48 are one chain: 36 import `dv_utils_pkg`, 25 import `dv_base_reg_pkg`, and `dv_utils_pkg.sv` itself rejects — `parseability_probe --parse systemverilog stimuli/sv/subs/opentitan/hw/dv/sv/dv_utils/dv_utils_pkg.sv --profile sv_2017 --preprocess <the lane's context: 53 include dirs, 0 symbols> --lib-in stimuli/sv/characterization/pipeline_library` → `Parser did not consume full input at position 171 [furthest_position=14038, +13867 bytes deeper than surface position] [origin=stimuli/sv/subs/opentitan/hw/dv/sv/dv_utils/dv_utils_pkg.sv:215:24]`; the preprocessed text at byte 14038 reads `` `uvm_fatal($sformatf("%m"), "'elf_file' argument cannot be an empty string") `` — UNEXPANDED. `ast_pipeline --preprocess systemverilog` with the same context leaves 21 backticks in the output and the names `uvm_fatal`, `uvm_info`, `uvm_new_func`, `DV_SPINWAIT`, `DV_SPINWAIT_EXIT`, `DV_SPINWAIT_EXIT_MULTI`, `DV_ASSERT_CTRL`, `OTDBG` unexpanded: the file includes `uvm_macros.svh` only under `` `ifdef UVM `` (lines 12–14), the derived context carries no `UVM` symbol, and OpenTitan's own flow defines it build-wide — `hw/dv/tools/dvsim/common_sim_cfg.hjson:65`, `"+define+UVM"` under `// Standard UVM defines`. Three OpenTitan DV files carry the guard (`dv_utils_pkg.sv`, `dv_macros.svh`, `sw_logger_if.sv`); `dv_base_reg_pkg.sv:123:715` is the same macro. Routed as `.6d.4`: the `UVM` symbol as a hand-curated suite define — the row kind `pipeline_context.py` reserves as `HAND_DEFINES` ("predefined macros a suite is compiled with") — then the library pass and the lane re-run; expected to move the 46 packages that import `uvm_pkg` and the 960 members behind them, to be measured, not assumed.
- **What this slice does not claim.** No row moved here; every number above is the release's lane run or the tracked report read at two commits. The 2026-09-09 figures were a snapshot of a gate, and three releases had moved most of it before the gate opened — the lesson recorded in `DEVELOPMENT_NOTES.md`.

## `.6d.4` the BUILD-WIDE SYMBOL — ✅ done 2026-10-02 (`PGEN-SVPP-EXPANSION-0030`): OpenTitan's `UVM` as a hand-curated suite define, and the DV chain behind `dv_utils_pkg` opens

**What landed** (LANE + CONTEXT tier — `stimuli/sv/pipeline_context.py` (`HAND_DEFINES`: one entry), the tracked `stimuli/sv/pipeline_context.tsv` (one row), the lane's derived reports; ZERO Rust / grammar / answer-key / codegen bytes):

- **The row.** `HAND_DEFINES["opentitan"] = [("UVM", "stimuli/sv/subs/opentitan/hw/dv/tools/dvsim/common_sim_cfg.hjson:65 (+define+UVM, dvsim build_opts)")]` → `python3 stimuli/sv/pipeline_context.py --write` emits `opentitan	define	UVM	hand	<source>` (`rows=75`, was 74); `context_args opentitan <file>` now carries `--symbol UVM` for every OpenTitan row; `pipeline_context.py --check` rc 0 afterwards. The row kind existed since `.4c.1` ("predefined macros a suite is compiled with"), empty until a suite needed one.
- **The root, measured first.** `parseability_probe --parse systemverilog …/dv_utils_pkg.sv --profile sv_2017 --preprocess <context> --lib-in stimuli/sv/characterization/pipeline_library` → `preprocess: … symbols=1 output_bytes=149450 source_map_entries=4546 included_files=22` (was `symbols=0 output_bytes=23011 … included_files=2`) and `parse_full passed … [preprocessed]` (was `Parser did not consume full input at position 171 [furthest_position=14038 …] [origin=…/dv_utils_pkg.sv:215:24]`).
- **The whole lane re-run** (`bash stimuli/run_pipeline_lane.sh all`, 6 jobs, 600 s; `wall_s=1017`): `PIPELINE-LIBRARY: Cores-VeeR-EL2=2/2 black-parrot=3/7 opentitan=355/396 uvm-core=2/2 published_packages=264 unresolved_imports=44 cyclic_files=0 member_rows=5130` (was `opentitan=345/396 … published_packages=256 … member_rows=5074`); `PIPELINE-LANE: files=16336 pass=12501 fail=3558 pp_fail=277 timeout=0 crash=0` (was `pass=12491 fail=3568`); `PIPELINE-MOVERS rows_before=16336 rows_after=16336 moved=10 added=0 lost=0`, by suite and direction `{('opentitan', 'fail->pass'): 10}` — zero `pass → fail`, zero outside OpenTitan; `LIBRARY-MOVERS {('fail', 'pass'): 10}`, 38 OpenTitan package files still `fail`. The members read through their unit (`PIPELINE-READJUDICATION: … chained:accepted_in_unit=1141 … chained:unreached_in_unit=846 …`, was 1020 / 960): OpenTitan `accepted_in_unit` 878 → 999, kept 32 → 27, `unreached_in_unit` 960 → 846. The verdict-coverage tuple under the pipeline `10439/2606/3291/2005/0` → `10570/2606/3160/1874/0` (131 rows adjudicated; the raw tuple `7485/2606/6245/3864/0` unmoved); the platform book's anchor moved with it (`check_sv_corpus_denominator.sh` refused the stale anchor, then OK). `IDENTITY-CARRIERS: OK`, `corpus-key-integrity: OK`, the three `PIPELINE-LANE-CURRENCY` rows OK.
- **What still rejects, measured (the SV lane's).** 38 OpenTitan package files, every one importing `uvm_pkg` (27 `dv_utils_pkg`, 30 `dv_lib_pkg`, 22 `cip_base_pkg`, 19 `dv_base_reg_pkg`), at 38 DISTINCT origins — no macro among them: `num_interrupts = ral.intr_state.get_n_used_bits();` (×3, a class-scope method call on a register-model member), `parameter string LIST_OF_LEAFS[] = {` / `LIST_OF_READ_SEED_FORCE_PATHS[] = {` (×6, an unpacked-array parameter with an assignment pattern), `` `DV_CHECK_RANDOMIZE_WITH_FATAL(req, `` (×2, a macro whose expansion carries an inline constraint block), `endcase` (×2), `pure virtual task automatic inject_fault();`, `extern function automatic logic [DataWidth-1:0] get_written_data();`, `rand tl_agent_cfg device_agent_cfg;`, `->m_adc_push_pull_ev[channel];`, `class aes_nist_vectors extends uvm_object;` … — each a grammar or fact-gate construct past the macro barrier, named per row in the lane report's *"rejects in its own file"* table and the dark worklist; 846 members wait behind them. Not this tree's work: the preprocessor lane has nothing left to expand here.
- **What was NOT done, stated.** No per-file override, no second symbol: `UVM` is the one define OpenTitan's flow passes to every DV build and the only `` `ifdef `` the chain stopped on; RTL files carry no `UVM` guard (3 of 2 611 `/dv/` files do, none outside), so the suite-wide row changes no RTL row — `moved=10`, all DV packages.

**Acceptance checklist (`TASK-ACCEPTANCE`):**

  - [x] **REPRODUCE / ISSUE** — `.6d.3`'s probe: `parseability_probe --parse systemverilog stimuli/sv/subs/opentitan/hw/dv/sv/dv_utils/dv_utils_pkg.sv --profile sv_2017 --preprocess <the lane's context> --lib-in …` → `Parser did not consume full input at position 171 [furthest_position=14038, +13867 bytes deeper than surface position] [origin=stimuli/sv/subs/opentitan/hw/dv/sv/dv_utils/dv_utils_pkg.sv:215:24]`; `ast_pipeline --preprocess systemverilog` with the same context: 21 backticks left, `uvm_fatal` / `uvm_info` / `uvm_new_func` / `DV_SPINWAIT*` / `DV_ASSERT_CTRL` / `OTDBG` unexpanded; the library dump: 48 OpenTitan `fail` rows, 36 importing `dv_utils_pkg`.
  - [x] **ROOT CAUSE (WHY + WHERE)** — WHERE: `stimuli/sv/pipeline_context.py`, `HAND_DEFINES` (empty), and the three DV files that guard `include "uvm_macros.svh"` on `` `ifdef UVM `` (`dv_utils_pkg.sv:12-14`, `dv_macros.svh`, `sw_logger_if.sv`). WHY: the derivation reads drivers and headers (`.4c.1`) and no driver carries a build-wide flow define; OpenTitan's dvsim passes `+define+UVM` to every DV build (`hw/dv/tools/dvsim/common_sim_cfg.hjson:65`, under `// Standard UVM defines`), so without it the include is dead and the parser meets a raw macro — the signature is the probe's `furthest_position=14038` on the unexpanded `` `uvm_fatal `` at `:215:24`, and `included_files=2` versus `22` with the symbol.
  - [x] **FIX** — CONTEXT tier (data, sourced): one hand-curated suite define, the row kind the deriver reserves; no engine tier (the `--symbol` flag and the lane's `define` rows existed), no grammar tier.
  - [x] **ADDRESSED (verified)** — `dv_utils_pkg.sv` → `parse_full passed … [preprocessed]` (`symbols=1 … included_files=22`); the lane: `PIPELINE-MOVERS … moved=10`, all OpenTitan `fail → pass`; the library 256 → 264 packages; OpenTitan members `accepted_in_unit` 878 → 999, `unreached_in_unit` 960 → 846; the tuple under the pipeline `10570/2606/3160/1874/0`; `pipeline_context.py --check` rc 0; `PIPELINE-LANE-CURRENCY` (1) CONTEXT OK.
  - [x] **NO REGRESSION** — `moved=10 added=0 lost=0`, zero `pass → fail`, zero movers outside OpenTitan (the symbol reaches every OpenTitan row and changed only the ten DV packages); the raw lanes untouched by construction (the context is the pipeline's); the raw tuple `7485/2606/6245/3864/0` unmoved; `IDENTITY-CARRIERS: OK (live=14 frozen=7 …)`, `corpus-key-integrity: OK`; ZERO grammar / generated bytes (every artifact byte-identical); `bash scripts/check_doctrines.sh` → `doctrines: ALL 45 enforced doctrines PASS.`.
  - [x] **LOCKSTEP** — this section + the row, the SV book's pipeline chapter (the `.6d.4` paragraph beside `.4c.1`'s, the `.6d.3` paragraph's tail) + tracked HTML, the platform book's verdict-coverage anchor, the derived lane reports (`pipeline_characterization.md`, `pipeline_readjudication.{md,tsv}`, the coverage artifacts, the dark audits), `docs/TASK_TREE.md`, `MEMORY.md`, `CHANGES.md`.
  - promotion: declined (the `.6d.3` note already carries the lesson — the next barrier is named by the probe; the deriver's own comment names the mechanism).

## `.7` CONTRACT + BOOK lockstep — ✅ done 2026-09-09 (`PGEN-SVPP-EXPANSION-0021`): the front end stated where a downstream reader looks

**What landed** (DOCS tier; ZERO Rust / grammar / answer-key / codegen bytes; no release / schema bump — the grammar and parser unchanged at
`1.0.227` / schema `32`; the SV grammar digest unchanged, `SV-CONTRACT-CURRENCY` re-derived):

- **`docs/contracts/PGEN_SYSTEMVERILOG_PARSER_INTEGRATION_CONTRACT.md`** — a new section *Front End — the preprocess→parse pipeline*
  between the companion-book section and the `verilog_2005` profile: the single command and its exit codes (0 / 1 with
  `furthest_position=` + `[origin=…]` / 3 `preprocess_failed`, verified at `parseability_probe.rs:502-505`), the expansion alone
  (`ast_pipeline --preprocess-systemverilog` + the JSON side outputs), the FOUR-part context (include dirs, defines, unit files,
  the library) and that the lane derives it while a consumer supplies its own — never guessed; the expansion guarantees with their
  clauses and the six refusal codes (`E_SVPP_MISSING_MACRO_ARGUMENT`, `E_SVPP_EXTRA_MACRO_ARGUMENT`, `E_SVPP_RECURSIVE_MACRO`,
  `E_SVPP_UNTERMINATED_PROTECTED_ENVELOPE`, `E_SVPP_UNDEFINED_CONDITIONAL_SYMBOL`, `E_SVPP_MACRO_REDEFINED` — the module's own
  set, `grep -o` verified) and the thirteen event kinds; the source-map rule as `origin_of_position` implements it (one chunk per
  emitted line; an included line names the included file, an expansion names the USE line, character columns, past-the-end lands
  on the last chunk's end, `origin=unknown` when unmappable) with the `.6b.4` caveat — the origin is the LAST MAPPED byte at or
  before the rejection, a construct is bucketed on the furthest TOKEN; the library's facts files (`DIR/package/<name>.facts.json`,
  the `type_name` / `declaration_family` shape read from a real export), both import forms, a rejected parse exporting nothing,
  neither flag twice; and the trust boundary (the numbers are the SV parser book's, re-derived per commit; the UVM gate named).
- **`docs/contracts/PGEN_SYSTEMVERILOG_PREPROCESSOR_PARSER_INTEGRATION_CONTRACT.md`** — a *Runtime Execution Stage — the EXPANSION*
  section: the expansion is a SIBLING deliverable of the directive parser (the runtime module, the `SVPP-EXPANSION` tree, the SV
  contract's section as its contract), the behaviours a consumer of the module relies on since 2026-09-09, and why this family's
  corpus leg stays NOT MEASURED (no corpus of directive files; the expansion's corpus is the SV family's); a Scope line.
- **The books**: the svpp parser book's status chapter gains *The expansion stage is a sibling deliverable* (below the derived block);
  the platform book's family page gains *The SystemVerilog corpus is read twice* (the raw lane is scaffolding, the pipeline is the
  release reading, both tuples in the anchor, the chapter for the rest); the user guide's §6 gains the single command. ⚠️ **The
  two parser books' TRACKED HTML mirrors (`docs/systemverilog_parser_book-html/`, `docs/systemverilog_preprocessor_parser_book-html/`)
  had lagged since `-0017`** — the `.4c.1`…`.6d.2` chapter edits were never rebuilt; `mdbook build` on all three books now (the
  platform book's `docs/book-html/` is untracked output), and the LOCKSTEP line of every future slice names the rebuild.
- **The DONE-BAR register**: no family row's next gap moved (the family-status blocks re-derived equal in `-0020`).

**Acceptance checklist (`TASK-ACCEPTANCE`):**

  - [x] **REPRODUCE / ISSUE** — `grep -c -i 'preprocess\|--sv-\|lib-in\|source map\|origin=' docs/contracts/PGEN_SYSTEMVERILOG_PARSER_INTEGRATION_CONTRACT.md` = 15, none a statement of the front end (corpus footnotes); the preprocessor contract's Scope said nothing of the expansion; `git status` after `mdbook build` showed 41 files of the two tracked `-html/` mirrors modified (four commits stale).
  - [x] **ROOT CAUSE (WHY + WHERE)** — ops family: `git ls-files docs/contracts/` lists the two contracts and neither carried a front-end section; the probe's exit-3 site `parseability_probe.rs:505`, the origin rule `origin_of_position` and the refusal codes (`grep -o E_SVPP_ rust/src/sv_preprocessor.rs`) are the sources every sentence above cites.
  - [x] **FIX** — the two contract sections, the two book notes, the user guide, the HTML mirrors.
  - [x] **ADDRESSED (verified)** — `mdbook build` on the three books exits 0 with no warning; `scripts/check_sv_contract_currency.sh` OK (digest `c447ccaa…` unchanged); every command, code and rule in the sections was read from the source named beside it.
  - [x] **NO REGRESSION** — no Rust, grammar, generated, answer-key or dump byte staged; the lane dump, both tuples, the bar and every tracked report byte-identical to `-0020`; `bash scripts/check_doctrines.sh` ALL PASS (run at commit).
  - [x] **LOCKSTEP** — both contracts, three books (source; the two tracked HTML mirrors), the user guide, this tree, `docs/TASK_TREE.md`, `MEMORY.md`, `CHANGES.md`, `DEVELOPMENT_NOTES.md`.
  - promotion: declined (documentation).

## `.8` `` `begin_keywords `` regions as ESCAPED identifiers — ✅ done 2026-09-14 (`PGEN-SVPP-EXPANSION-0022`)

- **THE ROWS (measured on the `1.0.230` probe under the pipeline, 2026-09-14).** Surelog `tests/1364_2005/dut.v:8:7` `reg bit, rst, clk;`, ispras `ieee-1800-2012/22/22.14.01_01.sv:10:13` `reg [63:0] logic;`, sv-tests `tests/chapter-5/5.6.4--compiler-directives-begin-keywords.sv:18:6` `reg logic;`, sv2v `test/lex/begin_keywords.sv:3:12` `integer automatic = 2;` — four `rejects_valid` rows whose furthest token is an IEEE 1800 keyword used as an identifier inside a `` `begin_keywords "1364-2005" `` / `"1364-2001"` / `"1364-1995"` region; and ispras `22.14.01_04.sv` — `accepts_invalid`: `interface … endinterface` under `"1364-2005"` is no production (the parser reads the 1800 interface). Five of the bar's nine rows; the corpus carries no other `` `begin_keywords `` file (0 further matches over the 16 336 lane rows).
- **THE DESIGN.** §22.14 (`docs/systemverilog/2017/md/section-22-compiler-directives.md:844-1260`) is a LEXICAL rule — which identifiers are reserved — over a region the preprocessor already delimits (the grammar carries `begin_keywords_directive` / `end_keywords_directive` as items, `grammars/systemverilog.ebnf:369-377`, and does nothing with them). The parser's reserved-word regex is per PROFILE and cannot switch mid-file, and an engine-side region-aware lexer would be the first grammar-specific lexing rule in the engine. The preprocessor can do it as TEXT: inside a region whose version set is a SUBSET of the parse profile's, every token that is reserved in the profile but not in the region's version is emitted as an escaped identifier — `logic` → `\logic ` — which §5.6.1 makes the identifier `logic` in every edition; the source map keeps the original span. Then `reg [63:0] logic;` parses as a net named `logic`, `integer automatic = 2;` as a variable named `automatic`, and `interface foo; endinterface` under `"1364-2005"` becomes `\interface foo; \endinterface` — no 1800 production, REJECTED, which is the key's ruling. The per-version sets are the LRM's own Tables 22-1 (1364-1995), 22-2 (+2001), 22-3 (+2005: `uwire`), 22-4 (+1800-2005), 22-5 (+2009), 22-6 (+2012), 22.14.9 (2017 = 2012); `"1364-2001-noconfig"` is 2001 minus the eight config keywords (22.14.4). A region whose version is NOT a subset of the parse profile (a `"1800-2023"` region under `sv_2017`) is reported as a diagnostic, not rendered.
- **ACCEPTANCE (to be measured).** The five rows move (`fail → pass` × 4 under the pipeline, `pass → fail` × 1 — `22.14.01_04.sv` to `must_reject:match_under_expansion`); the raw lanes byte-inert (the raw lane does not preprocess); every other pipeline row byte-identical; the preprocessor contract's *Runtime Execution Stage* names the rendering and the source-map behaviour; pins in `stimuli/sv/adjudication_repros/` for each version specifier and one nested-region case; the preprocessor grammar / self-tests extended; the front-end book chapter names the rule.

### What shipped

IEEE 1800-2017 §22.14 is a LEXICAL rule — *"what identifiers are reserved as keywords within a block of
source code"* — and the generated parser reserves one set per PROFILE and cannot switch mid-file. The
expansion stage now renders the difference as TEXT: inside a region, every token the parse profile reserves
and the region's `version_specifier` does not is re-emitted as an ESCAPED identifier, which §5.6.2 makes
*"not interpreted as a keyword"* and §5.6.1 makes the same identifier as the unescaped spelling.

- **`rust/src/sv_keyword_versions.rs`** (new) — the version algebra and the standard's own tables. The nine
  specifiers, `reserved_words()` built by the accumulation §22.14.3/.5/.6/.7/.8 each state plus §22.14.4's
  one subtraction, `words_to_escape(profile, region)`, `region_within_profile`, the profile map. ⛔ The
  tables are DERIVED, never typed: `stimuli/sv/check_sv_keyword_versions.py` (gate / `--write` / `--print` /
  `--self-test`) reads them out of the in-repo LRM text and the new doctrine **`SV-KEYWORD-VERSIONS`**
  (registry **40 → 41**, `scripts/check_sv_keyword_versions.sh`) holds the two equal on every commit.
- **`rust/src/sv_preprocessor.rs`** — `render_keyword_regions`, a single left-to-right lexical pass over the
  EXPANDED text (so a directive a macro produced counts and one in an inactive `` `ifdef `` branch does not),
  skipping comments, string literals, escaped identifiers, numeric literals and `$`-names; the region STACK
  is compilation-UNIT state and carries across `--sv-unit-file` boundaries as the clause requires; the source
  map is re-indexed by the exact byte delta of each rendering. New event kind `KeywordsRegion`; new
  diagnostics `W_SVPP_UNKNOWN_KEYWORDS_VERSION`, `W_SVPP_KEYWORDS_VERSION_ABOVE_PROFILE`,
  `W_SVPP_KEYWORDS_VERSION_NOT_IN_EDITION`, `W_SVPP_UNMATCHED_END_KEYWORDS`,
  `W_SVPP_MALFORMED_KEYWORDS_DIRECTIVE`. New config field `parse_profile`, wired from `--profile` on the
  probe and `--sv-keyword-profile` on `ast_pipeline --preprocess-systemverilog`.
- **`stimuli/sv/run_adjudication_repros.py`** — a `preprocess` column, because a construct the PREPROCESSOR
  decides has no raw-parse claim to pin at all. Nine new pins (`preprocess_rows=9`).

### ⛔ TWO THINGS THE LEAF'S OWN PLAN GOT WRONG, BOTH CAUGHT BY MEASUREMENT BEFORE THEY SHIPPED

1. **"An unrecognized `version_specifier` is an error" (§22.14's own sentence) was BUILT, MEASURED and
   WITHDRAWN.** The first cut raised `E_SVPP_UNKNOWN_KEYWORDS_VERSION` and refused the file. Measured over
   the 113 corpus files carrying the directive, that spelling moved **7 rows `fail` → `pp_fail`** — every
   Verilog-AMS row (`"VAMS-2.3"` ×6 + `"1800+VAMS"` ×1), whose answer-key rows are keyed on the PARSER's
   verdict. ⭐ **The implementation that errors is the GRAMMAR**: `keywords_version_specifier_<profile>`
   admits exactly its own edition's list, so every other spelling is *already* a parse rejection, and
   refusing here only PREEMPTS a precise, per-edition refusal with a coarse one. Demoted to
   `W_SVPP_UNKNOWN_KEYWORDS_VERSION`; no region is opened either way (an unknown version has no known
   keyword set), and an operator who wants the clause fatal at this stage has the existing knob
   `--sv-strict-warning-codes W_SVPP_UNKNOWN_KEYWORDS_VERSION` — pinned by a unit test that drives both.
2. **"The corpus carries no other `` `begin_keywords `` file (0 further matches over the 16 336 lane rows)"
   is FALSE, and the leaf's "eight config keywords" is TEN.** A census of the directive's own text reads
   **113 files** and **11 distinct specifiers** — `1364-2005` ×68, `1800-2023` ×9, `VAMS-2.3` ×7,
   `1364-2001` ×7, `1364-2001-noconfig` ×5, `1800-2005` ×4, `1800-2012` ×3, `1364-1995` ×3, `1800-2017` ×2,
   `1800-2009` ×2, `1800+VAMS` ×1. The `.12b.1` reconnaissance had counted rows whose FURTHEST TOKEN was a
   keyword-in-identifier-position, which is the bar's question, not the construct's population; §22.14.4's
   exclusion list is ten identifiers in both standards that print it.

### ⛔ THE TWO SPLIT POINTS THE LRM EXTRACTION DESTROYS, AND THE ORACLES THAT RECOVER THEM

The PDF→md extraction DISPLACES every table body past the prose that follows its caption and MERGES
consecutive bodies, so §22.14's six tables arrive as four runs. Neither split is assumed:

| run | captions pending | how it is split |
|---|---|---|
| 102 words | Table 22-1 | whole; CHECKED EQUAL to IEEE 1364-2005 §19.11 Table 19-2 (another standard, another PDF) |
| 22 words | 22-2, 22-3 | ⛔ **not separable by any property of the 1800 text** — the merged body is strictly alphabetical, so *every* cut is admissible. Split by IEEE 1364-2005 Tables 19-3 / 19-4 (full sets): 22-2 = 19-3 − 19-2 (21), 22-3 = 19-4 − 19-3 (`uwire`, 1). Their union is then checked equal to the 1800 body |
| 120 words | 22-4, 22-5 | no second document carries these; split at the **UNIQUE** cut leaving both tables strictly increasing — the body has exactly ONE descent (`within` → `accept_on`), so 22-4 = 97 and 22-5 = 23. A body whose descent count is not *(tables − 1)* is REFUSED, never guessed |
| 4 words | Table 22-6 | whole |

Σ = 102 + 21 + 1 + 97 + 23 + 4 = **248** = IEEE 1800-2017 Annex B Table B.1, which is §22.14.9's own
sentence (*"the full set … is listed in Annex B, which reflects the combination of all version tables"*);
the 1800-2017 and 1800-2023 extractions are held equal word for word; the ten noconfig exclusions are taken
from BOTH standards and checked identical and ⊆ Table 22-2.

### ⛔ THE ONE DIRECTION THIS STAGE CANNOT RENDER — RULED OUT, NOT ROUTED (2026-09-15, `PGEN-SVPP-EXPANSION-0024`)

Text can UNRESERVE a keyword (§5.6.1: an escaped identifier is not a keyword) and it cannot RESERVE a
non-keyword. So a region naming a version that reserves MORE than the parse profile would be applied only
half way — an OVER-ACCEPTANCE the stage cannot fix — and `.8` shipped that as a WARNING
(`W_SVPP_KEYWORDS_VERSION_ABOVE_PROFILE`), which is a report, not a guarantee.

Under a director challenge it was measured and then PROVEN instead:

- **Reachable?** Over the 16 336 corpus rows there are **14 occurrences** of a specifier reserving more than
  `verilog_2005` (`1800-2005` ×4, `1800-2009` ×2, `1800-2012` ×3, `1800-2017` ×2, `1800-2023` ×5), and **none
  at all** above `sv_2017` / `sv_2023` (§22.14.9: `1800-2023` adds no reserved keyword to `1800-2017`).
- **Can it bite?** No, and the reason is structural rather than lucky. A specifier is ADMITTED by a profile
  exactly when that profile's own EDITION lists it (§22.14 Syntax 22-10, carried per profile by
  `keywords_version_specifier_*`), and an edition lists only versions at or below itself ⇒ **admitted ⇒ subset
  ⇒ never above**. Every one of those 14 occurrences is a PARSE REJECTION on the `verilog_2005` lane, because
  `keywords_version_specifier_v2005` lists the four 1364 versions only. The warning can fire only where the
  directive is already illegal.
- **Held, not asserted.** `admitted_specifiers_are_subsets` is now an arm of `check_sv_keyword_versions.py`
  under the `SV-KEYWORD-VERSIONS` doctrine: it derives each profile's set and each admitted specifier's set
  from the LRM tables and FAILS if any admitted specifier reserves a word the profile does not. A future
  edition — or a vendor specifier added to a profile's list — that broke the implication would fail the gate
  instead of quietly turning the warning into a live over-acceptance. RED control in `--self-test`: a planted
  `verilog_2005` that admits `"1800-2005"` is refused.

## Acceptance Checklist (enforced)

- [x] **REPRODUCE / ISSUE** — the four `rejects_valid` bar rows and the one `accepts_invalid` row on the
      shipped `1.0.230` release probe, under the pipeline:
      `./rust/target/release/parseability_probe --parse systemverilog stimuli/sv/subs/ispras-sv-tests/ieee-1800-2012/22/22.14.01_01.sv --profile sv_2017 --preprocess`
      → `Parser did not consume full input at position 211 [furthest_position=235, +24 bytes deeper than surface position] [origin=stimuli/sv/subs/ispras-sv-tests/ieee-1800-2012/22/22.14.01_01.sv:10:13]`
      (line 10 is `reg [63:0] logic;`), and the same shape on Surelog `1364_2005/dut.v:8:7`, sv-tests
      `5.6.4--compiler-directives-begin-keywords.sv:18:6`, sv2v `lex/begin_keywords.sv:3:12`; while
      `22.14.01_04.sv` — the clause's own `// ! TYPE: NEGATIVE` example — read
      `parse_full passed … [preprocessed]`.
- [x] **ROOT CAUSE (WHY + WHERE)** — WHY: §22.14 changes the RESERVED-WORD SET for a region, and nothing in
      the front end read the directive at all. WHERE, at both ends: the grammar carries
      `begin_keywords_directive` / `end_keywords_directive` as items (`grammars/systemverilog.ebnf:458-474`)
      and binds only the version string, while its reserved-word refusal is one regex per PROFILE
      (`reserved_non_keyword_identifier_sv` / `_v2005`, `:853-859`) with no per-region form; and the
      preprocessor fell through to the *"Unknown directive: pass through only when active"* arm of
      `preprocess_text_internal` (`rust/src/sv_preprocessor.rs`, the `_ =>` branch of the directive match),
      emitting the directive verbatim and doing nothing with it. The `furthest_position` above lands
      exactly on the keyword: the parse is refused by the identifier rule, not by the directive.
      Tool-backed by the `[origin=…]` / `furthest_position=` lines quoted in the REPRODUCE box and by
      `SV-KEYWORD-VERSIONS:` (`stimuli/sv/check_sv_keyword_versions.py --print`), which prints the exact
      per-version sets the refusal is measured against.
- [x] **FIX** — declarative-first was impossible and the reason is measured: a per-region reserved set is
      not expressible in a whitespace-insensitive EBNF whose terminals are profile-gated, and an
      engine-side region-aware lexer would be the first grammar-SPECIFIC lexing rule in the engine. The fix
      is therefore at the PREPROCESS tier, where the clause's own subject (which identifiers are reserved)
      is representable as text: `\` + word + ` `, §5.6.1/§5.6.2. **ZERO grammar bytes, ZERO codegen bytes,
      ZERO generated-parser bytes** — the parser is byte-identical
      (`generated/systemverilog_parser.rs` sha256 `711a3bc073c7e1bf…`, the same digest the probe's
      `--parser-fingerprint` reports).
- [x] **ADDRESSED (verified)** — before→after on the same nine reproducers, pre-change release probe vs
      post-change release probe (`stimuli/sv/run_adjudication_repros.py`, `preprocess=1` rows): the five
      `fixed_begin_keywords_*` rows `REJECT → ACCEPT`, `invalid_begin_keywords_1364_2005_interface.sv`
      `ACCEPT → REJECT`, and the two that must not move (`invalid_keyword_as_name_without_region.sv`,
      `control_begin_keywords_1800_2017_region.sv`) did not. Over the corpus, the pipeline lane
      (`stimuli/run_pipeline_lane.sh all`, 16 336 rows, debug probe, 8 jobs, 511 s — the binary the tracked
      report's identity block already names) reads
      `PIPELINE-LANE: files=16336 pass=12202 fail=3857 pp_fail=277 timeout=0 crash=0` against a before of
      `pass=12136 fail=3923 pp_fail=277 timeout=0`. **68 rows moved and EVERY ONE carries a §22.14
      directive** (a row-by-row join against the 113-file census: `movers NOT carrying a
      begin_keywords/end_keywords directive: 0`): **67 `fail → pass`** — iverilog 62, ispras-sv-tests 2,
      Surelog 1, sv-tests 1, sv2v 1 — and **1 `pass → fail`**, ispras `22.14.01_04.sv`, the clause's own
      negative example, so `must_reject:accepts_invalid 2 → 1` in
      `stimuli/sv/characterization/pipeline_readjudication.md`. The pipeline bar falls **9 → 8** and
      `SV-CORPUS-DENOMINATOR` re-derives and publishes the tuple `9135/2606/4595/3312/8`.
      (The 62 iverilog rows are `deferred:v2005_profile_lane` in the answer key — files iverilog itself
      compiles as `-g2005` — so the ADJUDICATED count is unchanged at 9 135; they are a pass-count move,
      not a verdict promotion.)
- [x] **NO REGRESSION** — `bash scripts/check_doctrines.sh` → **ALL 41 enforced doctrines PASS** (including
      the new `SV-KEYWORD-VERSIONS`, whose `--self-test` drives six arms: a planted missing word, a planted
      foreign word, a planted wrong profile mapping, a planted missing admitted specifier, a two-descent
      body, and a GREEN control on the real tree). `python3 stimuli/sv/run_adjudication_repros.py` →
      `checked=1191 armed=231 listed=598 multi_profile_rows=454 preprocess_rows=9 failures=0`.
      `cargo test --lib sv_preprocessor` → 69 passed / 0 failed (13 new); `cargo test --lib
      sv_keyword_versions` → 6 passed / 0 failed. `make -C rust SHELL=/opt/homebrew/bin/bash
      clippy_on_rust_change` → strict source lint ok, generated-parser stage pass, correctness roster 68
      lints intact. The three RAW lanes are byte-inert **by construction** — `render_keyword_regions` is
      called only from `preprocess_systemverilog_file` / `_unit`, which `--parse` without `--preprocess`
      never enters — and that is MEASURED rather than argued: all three re-run into a scratch directory
      (`PGEN_CORPUS_OUT_DIR=rust/target/svpp8/raw_*`, the release probe, the tracked parameters) and the
      verdict columns diffed against the tracked files — **`results.tsv`, `results_sv2023.tsv` and
      `results_v2005.tsv` are each IDENTICAL**, `sv_2017` pass 9 847 / fail 6 489, `sv_2023` 9 872 / 6 464,
      `verilog_2005` 2 313 / 293, 0 rows moved on any lane.
- [x] **LOCKSTEP** — the SV integration contract's *Front End* section (the rendering, the profile flag, the
      five diagnostics, the AST consequence for a consumer), the preprocessor contract's *Runtime Execution
      Stage*, the SV parser book's *The Preprocess→Parse Pipeline* chapter (a worked before/after pair and
      the five rules), `PGEN_USER_GUIDE.md`, `TOOLBOX.md` §1.1 + the new §5.16 + the group-5 diagnosis
      vocabulary, `DOCTRINE_ENFORCEMENT.md` §10, `docs/book/src/the-enforced-doctrines.md` (re-derived by
      `scripts/report_doctrines.sh --markdown`), `docs/book/src/gate-flow.md`'s doctrine count `40 → 41`,
      and the verdict-coverage anchor in `docs/book/src/grammar-wellformedness.md`. No release / schema bump
      and no DONE-BAR row change: the grammar and the generated parser are unchanged at `1.0.230` /
      schema `33`, and SystemVerilog stays `Mostly Done`.

## `.5`(i) the ORDER around a ``` `` ``` paste — ✅ done 2026-09-15 (`PGEN-SVPP-EXPANSION-0025`): a usage that is a paste operand is expanded first; an undefined usage joined by a paste is the WHOLE constructed name

**The row.** sv2v `test/lex/macro_macro.sv` — the one `refuted:preprocessor_item` row of the disposition table
that named this item (`.6b.4`): `` `define MACRO2(A, B) `CHOICE_``A```SUFFIX , `CHOICE_``B```SUFFIX `` used as
`` $display(`MACRO2(FOO, BAR)); ``. sv2v's own reference `macro_macro.v` expects `$display(3, 4)` — the pasted
name `CHOICE_FOO_MAGIC` looked up after `` `SUFFIX `` expands — and the engine emitted `1_MAGIC , 2_MAGIC`, a token no
parser accepts (`[origin=…macro_macro.sv:16:20]`, `furthest_position=74`).

**The standard.** IEEE 1800-2017 §22.5.1 (`docs/systemverilog/2017/md/section-22-compiler-directives.md:380-381`):
*"A `` delimits lexical tokens without introducing white space, allowing identifiers to be constructed from
arguments"*; and `:323-326`: *"The macro text and argument defaults may contain usages of other text macros. Such
usages shall be substituted after the outer macro is substituted … If an actual argument or an argument default
contains a macro usage, the macro usage shall be expanded only after substitution into the outer macro text."* The
clause names no order for a paste whose operand is itself a usage; what it does say is that a paste CONSTRUCTS an
identifier, and the identifier being constructed here is a macro name.

**The oracles — two tools this repository did not build, both installed on this host, run rather than quoted**
(`docs/tasks/artifacts/svpp_expansion/paste_order/probe.sh`, its run in `probe_run.txt`): Verilator 5.050
(`verilator -E -P`) reads `3 , 4` and `5 , 5` — sv2v's reference exactly; Icarus 13.0 (`iverilog -E`) reads
`1_MAGIC , 2_MAGIC` — the engine's old reading — and `_MAGIC , _MAGIC` for line 17, which nobody else reads. So the
tools SPLIT on this row, and the leaf does not pretend otherwise: the whole-name reading is the one the clause's own
sentence supports (a paste constructs ONE identifier; looking a prefix up mid-construction reads "delimits lexical
tokens" as "terminates a lexical token"), it is sv2v's and Verilator's, and Verilator's tracked `t_preproc.out`
pins four more of its consequences that the engine already read the same way on the OBJECT-macro path: `C5`
(`` `A``b```C `` → `abc`), `Q1` (`` `QA``_b `` → `zzz` — *"the existence or non-existence of a base define can make
a difference"*), `XACTION3` (`` `XTYPE``_frame `` → `sonet_frame`, not re-expanded although `sonet_frame` is a
macro), the bug1225 families (`` XXS_```SOME `` → `XXS_some`). Five shapes under `shapes/` sharpen the split:
`s2` (actual is an undefined usage → whole name; all three agree), `s3` (actual is a DEFINED usage → `x_y`;
Verilator and Icarus agree and the engine had read the whole name `from_whole_name` — a SECOND divergence this
slice closes), `s5` (defined left usage AND defined whole name → `a_b`; all three agree), `s1` and `s4` (routed
below).

**Root cause (WHY + WHERE).** `substitute_function_macro_body` (`rust/src/sv_preprocessor.rs`) DELETED every
``` `` ``` token while it substituted the formals, so `` `CHOICE_``FOO```SUFFIX `` reached the rescan
(`expand_macros_in_text`) as `` `CHOICE_FOO`SUFFIX `` and the first defined prefix won — the paste's operand
`` `SUFFIX `` could not be expanded first because nothing remembered there had been a paste. The object-macro
path had no substitution step, so its body was rescanned WITH its pastes, and `.6b.2`(i) had taught that rescan
to drop a paste in expansion text: left to right, an undefined `` `CHOICE_ `` stays verbatim, the paste is
dropped, `` `SUFFIX `` expands, and the second pass looks `` `CHOICE__MAGIC `` up whole. That is why line 17
(`MACRO3`, an object macro) was right and line 16 (`MACRO2`, a function macro) was wrong — one construct, two
code paths, two answers. The deletion is as old as the file: `git log --diff-filter=A -- rust/src/sv_preprocessor.rs`
→ `636f35c0d` (2026-02-27), and `git show 636f35c0d:rust/src/sv_preprocessor.rs | grep -n 'replace("``", "")'` → line
914, `let pasted = substituted.replace("``", "");` — every paste removed AFTER substitution and BEFORE the rescan;
`-0008` (`.5`(g)(h)) moved that removal into the substitution scan to keep the provenance mask aligned and kept its
effect. No order was ever chosen; it fell out of which path ran first.

**Pre-registration (before a byte moved).** Corpus census, `grep -rlF '``'` over `stimuli/sv/subs` +
`stimuli/sv/uvm`: **184 files** carry a paste at all; **12** carry a paste followed by a macro usage
(`-F '```'`: sv2v ×2, Surelog PpLppdr, iverilog `sv_macro3a/b.v` + `br_gh374.v`, verilator `t_preproc.v` +
`_str_undef.v` + `_strify_join.v` + `_ttempty.v`, OpenTitan `dv_macros.svh` (inside a `` `" `` region) and
`usb_osc.sv` (inside a comment)); **17** carry a macro usage immediately before a paste
(`'(^|[^`])`[A-Za-z_][A-Za-z0-9_$]*``'`) — UVM's `` `uvm_pack_``TYPE``(…) `` / `` `m_uvm_``TYPE``_resize(…) ``
in `uvm_object_defines.svh` and Verilator's four vendored copies, OpenTitan's `` `dv_``SEV_(…) `` ×18 and
`` `DV_MUBI``WIDTH_``_DIST ``, sw_logger_if's `` `_``n``_ARGS ``, black-parrot's `` `decompress_``csr_name_mp``_s ``,
and the test files above — and **every one of those left operands is undefined as a macro name in its tree**
(`uvm_pack_` `uvm_unpack_` `m_uvm_` `dv_` `uvm_` `DV_MUBI` `_` `REPEAT_` `decompress_` `compress_` `CHOICE_`
`inv1` `MOD`: 0 `` `define ``s each; `PREFIX` `A` `QA` `XTYPE` are defined only inside their own Verilator/Icarus
tests, where the expansion sits inside a `` `" `` region or on the object path that already read them right), so
the expected mover set was **exactly one expansion: `macro_macro.sv` line 16**, the four `.5` reference
expansions (`dv_utils_pkg.sv`, `uart.sv`, `dv_base_env.sv`, `uvm_pkg.sv` 3 022 806 B) byte-identical, and one lane
row `fail → pass`.

**What shipped** (`rust/src/sv_preprocessor.rs` only; ZERO grammar / codegen / generated bytes; 5 tests, 74/74):

- `substitute_function_macro_body` KEEPS the paste tokens in body text and in actuals — a paste is a token of the
  expansion text and the rescan is the one place that knows what its operands are. Inside any string literal (a
  `` `" `` region, or a plain `"…"` in the body, now tracked with its `\"` escapes) a paste is removed at
  substitution as before, because the rescan never enters a string; `push_actual` takes `strip_paste`.
- `expand_macros_in_text`: at an UNDEFINED usage immediately followed by ``` `` ```, in expansion text
  (`user_mask.is_some()` — a source line keeps its paste for the parser), the new `join_paste_chain` walks the
  chain left to right: an identifier fragment is appended as written (digits included, `` `_``3``_ARGS ``); a
  DEFINED usage operand — object or function-like, its parentheses taken along by
  `parse_macro_invocation_arg_ranges` — is expanded through `expand_macros_in_text` on exactly its own bytes and
  its own mask slice, so provenance, the `.5`(h) recursion check and the depth guard are the ordinary ones, and
  its expansion appended; an UNDEFINED usage operand is kept as written, backtick and all, so the constructed
  name fails its lookup and the parser reports the byte; a paste followed by neither (`` `X``( ``) joins nothing
  and ends the chain. The WHOLE constructed name is then looked up by the ordinary path at the chain's end —
  in place, a function-like result collecting its argument list still UNEXPANDED (§22.5.1's order for every
  actual: `` `uvm_pack_``int``(ARG, p) `` becomes `` `uvm_pack_int(ARG, p) `` with `ARG` as written). A DEFINED
  left operand never reaches the walker: the ordinary path expands it and the loop drops the paste after it —
  the same expand-first reading (`s3`, `s5`, `C5`).
- Formatting: `rust/src/sv_preprocessor.rs` is `rustfmt --edition 2024 --check` clean after this leaf, formatted wholesale as
  `.5`(e′) did — 16 drift hunks, three in this slice's lines and the rest left by `.8` (`push_event(KeywordsRegion …)` at
  `:731` among them); deterministic, no semantic change, the focused tests and `clippy_on_rust_change` run on the
  formatted bytes.
- Tests: `a_usage_on_the_right_of_a_paste_is_expanded_before_the_join` (sv2v's file verbatim),
  `a_defined_usage_as_a_paste_operand_is_expanded_before_the_join` (C5, XACTION3, `s3`, `s5`),
  `an_undefined_usage_joined_by_a_paste_is_looked_up_as_the_whole_name` (Q1, `name1`/`name3`, XXE/XXS/XYS — each a
  line of `t_preproc.out` / `t_preproc_str_undef.out`), `a_constructed_function_usage_takes_its_arguments_unexpanded`
  (UVM's shape, Surelog's ```rname``_ADR, sv2v's `` `inv1``inv2(str) `` incl. the empty-`inv1` form),
  `a_paste_inside_a_plain_string_in_a_macro_body_is_unchanged` (the bytes stay exactly as before; (k) below).

**Measured** (rebuilt debug `ast_pipeline` / `parseability_probe`; the pre-change binaries snapshotted before the
rebuild linked over them, `rust/target/s5i/ast_pipeline.before`):

```text
(1) POPULATION-DIFF — every corpus file carrying a paste (184) + the four .5 reference expansions, expanded
    standalone under the .5 include context by BOTH binaries (rust/target/s5i/expand_one.sh, pop_before/ vs pop_after/):
    files=188 identical=187 moved=1 — sv2v macro_macro.sv line 4: `$display(1_MAGIC , 2_MAGIC);` → `$display(3 , 4);`
    rc distribution identical (175 rc 0 / 13 rc 1 — include-not-found refusals, the same 13); uvm_pkg.sv 3 022 806 B,
    dv_utils_pkg.sv, uart.sv, dv_base_env.sv IDENTICAL
(2) PASTE-ORDER-PROBE (docs/tasks/artifacts/svpp_expansion/paste_order/probe_run.txt): rows=14 pgen_eq_verilator=8
    pgen_eq_iverilog=8 verilator_eq_iverilog=9 — the six PGEN≠Verilator rows are sv_macro3a/b, t_preproc_str_undef,
    t_preproc_strify_join, s1, s4: every one the `" region or the plain-string question routed below, none the paste order
(3) the .1 census (rust/target/svpp_e1/pp_census.sh, 4 157 UVM + OpenTitan rows preprocess-only): files=4157 rc0=4084
    rc_nonzero=73 timeouts=0 — unchanged since .5
(4) cargo test --lib sv_preprocessor: 74 passed / 0 failed (69 + 5)
(5) THE LANE (stimuli/run_pipeline_lane.sh all, debug probe, 8 jobs, 60 s) — run TWICE, and the second is the measurement:
    run 1 (17:54, the host still settling a build): files=16336 pass=12199 fail=3859 pp_fail=277 timeout=1 — TWO movers against the
      tracked dump (12199 / 3860 / 277 / 0): sv2v macro_macro.sv fail → pass, and OpenTitan top_darjeeling …/pinmux_reg_top.sv
      pass → timeout at 60.06 s (45.28 s in the tracked run; its two sibling 1.3 MB register files read 57.85 s and 51.24 s in the
      same run against 43.99 / 41.11 s; the whole run's row-seconds 2 828 → 3 505, +24 %, rows ≥ 30 s 4 → 12). The file carries
      no paste and its EXPANSION under the lane's own context is BYTE-IDENTICAL on both binaries (1 339 145 B), so the parser was
      handed the same input — a load artefact, and the .26 class of timing drift.
    run 2 (18:19, idle host, wall 443 s, peak 7 356 MB): files=16336 pass=12200 fail=3859 pp_fail=277 timeout=0 crash=0 —
      EXACTLY ONE mover against the tracked dump: sv2v test/lex/macro_macro.sv fail → pass (origin :16:20 → none); 0 the other
      way, 0 origin moves; the trio at 49.11 / 47.28 / 43.84 s, row-seconds 2 990, rows ≥ 30 s back at 4. Run 2 is the dump.
(6) re-adjudication: corroborated 1223 → 1224, refuted:preprocessor_item 2 → 1 (the disposition row RETIRED — the instrument
    refuses a disposition for a row that moved), refuted_awaiting_disposition=0; PIPELINE-LANE-CURRENCY (1)(2)(3) OK;
    coverage: raw 7486/2606/6244/4039/6 UNCHANGED, pipeline 9135/2606/4595/3314/5 → 9136/2606/4594/3313/5 (adjudicated +1,
    no-verdict −1, dark −1, the bar unmoved); check_sv_corpus_denominator.sh OK after the book anchor moved
(7) pins (run_adjudication_repros.py): debug probe checked=1210 armed=231 listed=605 preprocess_rows=11 failures=0 (was
    1206 / 231 / 603 / 9 — the two new rows); release probe (the lowmem host recipe, installed at the provenance path)
    checked=1210 armed=231 listed=605 multi_profile_rows=461 preprocess_rows=11 failures=0 — identical
(8) the three RAW lanes (`stimuli/run_external_corpus.sh sv | sv2023 | sv2005`, the rebuilt release probe installed at the provenance path `rust/target/release/parseability_probe`, into `PGEN_CORPUS_OUT_DIR` and diffed on the verdict columns): all three IDENTICAL — `sv_2017` 9 844 / 6 492 (16 336 rows), `sv_2023` 9 869 / 6 467, `verilog_2005` 2 313 / 293 (2 606 rows), 0 rows moved on any lane — byte-inert by construction (`--parse` without `--preprocess` never enters the expansion) and measured anyway
(9) make -C rust SHELL=/opt/homebrew/bin/bash clippy_on_rust_change on the FORMATTED bytes → ✅ clippy_on_rust_change completed. (strict source lint, the generated-parser stage, GENERATED-CLIPPY-CORRECTNESS: ✅ POLICY-ONLY PASS — 68 pinned lints, all still in clippy::correctness; guard exit 0, peak 7 263 MB); cargo test --lib sv_preprocessor re-run on the same bytes: 74 passed / 0 failed
```

**ROUTED — owned, next, each with the measurement that found it** (the three-way probe above, rows `NO`):

- **`.5`(j) — a macro usage inside a `` `" `` string region.** (1) A usage substituted into the region through an
  actual is not expanded: `` `define stringify(text) `"text`" `` with `` `stringify(`name1) `` gives `"`name1"` here,
  `"my_prefix_my_suffix"` in BOTH Verilator 5.050 and Icarus 13.0 (iverilog `sv_macro3a.v`, whose tracked
  `gold/sv_macro3.gold` says the same; verilator `t_preproc_str_undef.v`, whose `.out` says `"`PREFIX_my_suffix"` /
  `"my_prefix_suffix"`). §22.5.1 `:325-326`: the usage in the actual *"shall be expanded only after substitution
  into the outer macro text"* — substituted into the region, then expanded. (2) An OBJECT macro's `` `" `` region is
  never rendered as a string at all: verilator `t_preproc_strify_join.v`'s `` `define STRIFY `"`FOO``-```BAR``-```QUX```" ``
  expands to `` `"foo-bar-qux`" `` here and `"foo-bar-qux"` in both tools (its `.out`); only the function-macro path
  has a region renderer (`substitute_function_macro_body`). Both rows are `svpp_owned:accepted_under_expansion` /
  `corroborated` under the lane today because the SV grammar tolerates a stray directive token and a string may
  contain a backtick — the verdict is right, the bytes are wrong, and a downstream consumer reads the bytes.
  Shape `s4` is the same question one level deeper (a constructed function usage whose target stringizes its
  argument: `"`Z"` here, `"z"` in Verilator; Icarus cannot construct a function usage by paste at all and errors).
- **`.5`(k) — a formal inside a plain `"…"` string in a macro body.** verible's
  `lint/testdata/macro_string_concatenation.sv` `` `define FOO(arg) "foo``arg``bar" ``: `"fooxbar"` here and in
  Icarus 13.0, `"foo``arg``bar"` verbatim in Verilator 5.050 — and §22.5.1 `:370-372` reads with Verilator: *"the
  use of " always starts a string literal and must have a terminating ". Any characters embedded inside this
  string literal, including `", become part of the string in the replaced macro text"* — the `` `" `` form exists
  because a plain string does not substitute. A corpus census of formals inside plain strings in `` `define `` bodies
  is the first step; UVM writes `` `"ARG`" `` for exactly this reason, so the price may be small. Not decided here:
  this slice keeps the plain-string bytes exactly as they were (test 5 pins it), and the choice is a separate
  ruling with its own census.

**Acceptance checklist (`TASK-ACCEPTANCE`):**

  - [x] **REPRODUCE / ISSUE** — `./rust/target/debug/parseability_probe --parse systemverilog stimuli/sv/subs/sv2v/test/lex/macro_macro.sv --profile sv_2017 --preprocess` → `Parser did not consume full input at position 0 [furthest_position=74, +74 bytes deeper than surface position] [origin=stimuli/sv/subs/sv2v/test/lex/macro_macro.sv:16:20]`; `./rust/target/debug/ast_pipeline --preprocess-systemverilog stimuli/sv/subs/sv2v/test/lex/macro_macro.sv --output …` → line 4 `$display(1_MAGIC , 2_MAGIC);` where sv2v's reference reads `$display(3, 4);`.
  - [x] **ROOT CAUSE (WHY + WHERE)** — correctness family: the probe's `furthest_position=74` / `[origin=…:16:20]` lands on `1_MAGIC`, the first byte after the wrongly-joined `CHOICE_FOO`; the expansion text (`--preprocess-systemverilog --output`) beside Verilator's `verilator -E -P` and Icarus's `iverilog -E` on the same bytes (`PASTE-ORDER-PROBE: rows=14 pgen_eq_verilator=8 pgen_eq_iverilog=8 verilator_eq_iverilog=9`; before the fix the sv2v row read `NO / yes / NO`). WHERE: `substitute_function_macro_body`'s paste branch (`i += 2; continue;` — the token deleted while substituting) and `push_actual`'s strip loop, against `expand_macros_in_text`'s paste branch (`.6b.2`(i), drop-in-expansion-text), which is why the OBJECT path (`MACRO3`, line 17 → 5) was right and the FUNCTION path (`MACRO2`, line 16) was wrong. WHY: the operand `` `SUFFIX `` must expand BEFORE its side is joined, and the joined name must be looked up WHOLE — §22.5.1 *"allowing identifiers to be constructed"*.
  - [x] **FIX** — engine tier (a preprocessor has no grammar; declarative and grammar tiers do not apply): the paste survives substitution outside string literals; `join_paste_chain` in the rescan; `push_actual(strip_paste)`; five tests. ZERO grammar / codegen / generated bytes.
  - [x] **ADDRESSED (verified)** — `macro_macro.sv`: `1_MAGIC , 2_MAGIC` → `3 , 4` (POPULATION-DIFF (1), the ONE mover of 188); the probe's sv2v row `PGEN=V` `NO → yes`; `cargo test --lib sv_preprocessor` 74/74 with the five oracle-golden tests; the whole lane (run 2, idle host): `files=16336 pass=12200 fail=3859 pp_fail=277 timeout=0` against the tracked `12199 / 3860 / 277 / 0` — EXACTLY ONE row moved, `macro_macro.sv` `fail → pass`, 0 the other way; the re-adjudication `corroborated 1223 → 1224`, the pipeline tuple `9135/2606/4595/3314/5 → 9136/2606/4594/3313/5` through `SV-CORPUS-DENOMINATOR`'s gate; pins `preprocess_rows 9 → 11`, `failures=0`.
  - [x] **NO REGRESSION** — POPULATION-DIFF: 187 of 188 paste-carrying expansions BYTE-IDENTICAL before → after (uvm_pkg 3 022 806 B, the three OpenTitan `.5` references included), the 13 refusals the same 13; the `.1` census `73` unchanged; the lane's other 16 335 rows verdict-identical (run 2; run 1's single timeout is a measured load artefact on a byte-identical expansion, recorded above, not a verdict); the raw tuple `7486/2606/6244/4039/6` unmoved; the three RAW lanes (`stimuli/run_external_corpus.sh sv | sv2023 | sv2005`, the rebuilt release probe installed at the provenance path `rust/target/release/parseability_probe`, into `PGEN_CORPUS_OUT_DIR` and diffed on the verdict columns): **all three IDENTICAL** — `sv_2017` 9 844 / 6 492 (16 336 rows), `sv_2023` 9 869 / 6 467, `verilog_2005` 2 313 / 293 (2 606 rows), 0 rows moved on any lane — byte-inert by construction (`--parse` without `--preprocess` never enters the expansion) and measured anyway; `make -C rust SHELL=/opt/homebrew/bin/bash clippy_on_rust_change` on the FORMATTED bytes → `✅ clippy_on_rust_change completed.` (strict source lint, the generated-parser stage, `GENERATED-CLIPPY-CORRECTNESS: ✅ POLICY-ONLY PASS — 68 pinned lints, all still in clippy::correctness`; guard exit 0, peak 7 263 MB); `cargo test --lib sv_preprocessor` re-run on the same bytes: 74 passed / 0 failed; no grammar / codegen / generated byte moved (`generated/` untouched); `bash scripts/check_doctrines.sh` ALL PASS (run at commit, see the verification log).
  - [x] **LOCKSTEP** — the SV parser book's *The Preprocess→Parse Pipeline* chapter (*The order around a paste*, with the sv2v example and the recorded tool split), the SV integration contract's *Front End* guarantees, the preprocessor contract's *Runtime Execution Stage*, the tracked probe + shapes + run under `docs/tasks/artifacts/svpp_expansion/paste_order/`, two pins (`fixed_paste_operand_macro_usage.sv`, `control_paste_operand_macro_usage_object.sv`, `preprocess=1`), the disposition table (the row retired — it is `corroborated` now and the instrument REFUSES a disposition for a row that moved), the re-adjudication / coverage artifacts and the live tuple, this tree, `docs/TASK_TREE.md`, `MEMORY.md`, `CHANGES.md`, `DEVELOPMENT_NOTES.md`. No release / schema bump: the grammar and the generated parser are unchanged.
  - promotion: declined for now — the lesson (*one construct, two code paths, two answers: the path that had no substitution step was right by accident*) is stated in the code and in `DEVELOPMENT_NOTES.md`.

## `.5`(j) the `` `"…`" `` string region — ✅ done 2026-09-15 (`PGEN-SVPP-EXPANSION-0026`): an embedded usage is expanded, an object macro's region is rendered, an escaped quote is a quote — and (j′) `()` is one empty actual

**The clause.** IEEE 1800-2017 §22.5.1 (`docs/systemverilog/2017/md/section-22-compiler-directives.md:366-368`): *"An `"
overrides the usual lexical meaning of " and indicates that the expansion shall include the quotation mark, substitution
of actual arguments, and expansions of embedded macros. This allows string literals to be constructed from macro
arguments."* — three things, and the engine did one of them, on one of its two paths. And `:220-221` for (j′): *"An
actual argument may be empty or white space only, in which case the formal argument is substituted by the argument
default if one is specified or by nothing if no default is specified."*

**The rows — found by `.5`(i)'s three-way probe, every `NO` row of it.** (1) An embedded usage arriving through an
ACTUAL was not expanded: iverilog `sv_macro3a.v` `` `stringify(`name1) `` read `"`name1"` here and
`"my_prefix_my_suffix"` in BOTH Verilator 5.050 and Icarus 13.0 — and in iverilog's own tracked `gold/sv_macro3.gold`;
verilator `t_preproc_str_undef.v` the same (its `.out`: `"`PREFIX_my_suffix"` / `"my_prefix_suffix"`). (2) An OBJECT
macro's region was never rendered at all — the object path has no substitution step and the renderer lived in the
substitution: verilator `t_preproc_strify_join.v` read `` `"foo-bar-qux`" `` here and `"foo-bar-qux"` in both tools
(its `.out`), and — the design row — OpenTitan `hw/ip/otbn/dv/uvm/env/otbn_ssctrl_if.sv`'s
`` `define SSCTRL_PATH `"`SSCTRL_HPATH`" `` reached the parser as `` {`"tb.dut…`", ".", signal} `` and was refused at
`:15:13` (`chained:rejects_own_file`, a DARK row) where both tools read
`{"tb.dut.u_otbn_core.u_otbn_start_stop_control", ".", signal}`. (3) The ESCAPED quote: the clause's own example
(`:376-379`) `` `define msg(x,y) `"x: `\`"y`\`"`" `` *"expands to `$display("left side: \"right side\"");`"* — the engine
emitted `"left side: `\"right side`\""` because the close finder matched the escaped quote's LAST TWO BYTES and ended the
region at the first `` `\`" `` (iverilog `sv_macro2.v` and sv-tests `22.5.1--define-expansion_25.sv` are that example
verbatim). (j′) verilator `t_pp_display.v:35` `` `msg(`thru(),) ``: rendering the region expands the `` `thru() `` inside
it, and the engine read `()` as ZERO actuals and refused it as MISSING (`.5`(g)) — on an ordinary line too, measured on
the pre-change binary — where both tools read one EMPTY actual (`$display(": \"\"")`) and the clause's own
`` `MACRO3 ( ) // all arguments replaced by defaults `` needs the same reading.

**Root cause (WHY + WHERE).** `substitute_function_macro_body` rendered the region INTO a plain `"…"` string at
substitution time, and `expand_macros_in_text` never enters a string — so an embedded usage (the clause's third thing)
could not be expanded, and the object-macro path, which has no substitution step, passed `` `" `` through verbatim
(`out.push_str("`\"")`) and never rendered at all. Its close finder scanned for the two bytes `` `" `` without stepping
over `` `\`" ``. `parse_macro_invocation_arg_ranges` pushes the last segment only `if s < e || !args.is_empty()`, so
`()` yields no range and `expand_function_macro` counts zero actuals.

**Pre-registration (before a byte moved)** — `rust/target/s5j/region_census.py` over FULL bodies (continuations joined):
**468 files** carry a region in a `` `define ``; 3 of them in an OBJECT macro (otbn_ssctrl_if.sv, sv2v
`string_macro.sv`, verilator `t_preproc_strify_join.v`), 3 embed a usage in the body's own text (the two above + Surelog
`PreprocString/dut.sv`), 19 carry a paste inside a region, 6 the unclosed legacy form; a usage arriving through an
ACTUAL is invisible to a body census, so the class is diffed instead (below). For (j′): **103 files** carry an empty
invocation `` `NAME() ``; the frequent names are formal-less (`foo8()`, `foo64()`) or fully defaulted
(`DV_ALERT_IF_CONNECT()` ×41), whose reading does not change; `D()` (two no-default formals) stays MISSING on its second
formal. Expected: `otbn_ssctrl_if.sv` `fail → pass` under the pipeline (DARK → adjudicated, a design suite), byte moves
with no verdict move on the corroborated rows above, `t_pp_display.v` unchanged in verdict (its `:40` source-line
region is the pinned must-reject shape, `.6c.2`), sv2v `macro.sv` a must-reject row either way.

**What shipped** (`rust/src/sv_preprocessor.rs` only; ZERO grammar / codegen / generated bytes; 8 tests, 82/82;
rustfmt-clean):

- `expand_usage_at` — the one place a usage at a byte is expanded on exactly its own bytes and mask slice (builtins
  included), factored out of `.5`(i)'s chain walker and shared with the region renderer.
- `find_macro_string_close` — the closer of a region, stepping over `` `\`" `` whole.
- `render_macro_string_region` — ONE renderer, in the rescan, for BOTH paths: an embedded usage expanded through
  `expand_usage_at` (undefined, or function-like with no argument list → kept as written, backtick and all — Surelog's
  own golden for `PreprocString` keeps its bare `` `message ``), `` `\`" `` → `\"`, a paste joins, every other byte —
  a plain `"` included — is the string's. Reached from `expand_macros_in_text` at `` `" `` in EXPANSION text only.
- `substitute_function_macro_body` no longer renders: it substitutes the actuals INTO the region and emits it with its
  markers on, escaped quotes and pastes copied as written, for the renderer. The unclosed legacy form is untouched.
- ⛔ **A SOURCE line's second pass stays a source line.** The rescan's second pass carried `Some(&[])` for every text,
  so a source line holding a usage was re-read as expansion text and its own `` `" `` rendered — which turned sv2v
  `string_macro.sv:9` (`` $display(`"`FOO`"); ``, PINNED must-reject by `.6c.2`: the form is defined for macro text)
  into an ACCEPT. Test 4 pins the verbatim source-line region. Verilator 5.050 renders a source-line region
  (`"standalone"`), Icarus 13.0 leaves it, the clause scopes it to macro text — recorded, not followed.
- (j′) `expand_function_macro`: `()` is one empty actual when the macro declares formals, the empty list of a
  formal-less macro otherwise.
- Tests: `a_macro_usage_embedded_in_a_string_region_is_expanded`, `an_object_macros_string_region_is_rendered`,
  `an_escaped_quote_in_a_string_region_renders_as_a_quote` (the clause's example), `a_string_region_in_a_source_line_stays_verbatim`,
  `an_undefined_or_argumentless_usage_in_a_region_stays_verbatim`, `pastes_inside_a_string_region_still_join` (UVM's
  `` `"ARG[+]`" ``), `a_region_arriving_through_an_actual_is_rendered_where_it_lands` (Verilator's nested `quux`),
  `an_empty_argument_list_is_one_empty_actual_when_the_macro_has_formals`.

**Measured** (rebuilt debug binaries; the post-`-0025` binaries snapshotted first, `rust/target/s5j/ast_pipeline.before`):

```text
(1) POPULATION-DIFF, three needles then the WHOLE CORPUS (docs/tasks/artifacts/svpp_expansion/string_region/population_diff.sh,
    8-way, the .5 include context, before = the post--0025 binary):
      needle=`"  files=483 identical=463 moved=20     needle=`NAME() files=107 identical=100 moved=7
      needle=``  files=188 identical=180 moved=8      needle=ALL  files=16388 identical=16342 moved=46, rc moved on 0
    ⛔ the needles are BLIND to a region that reaches a file through an `` `include `` (OpenTitan's `DV_CHECK_*` /
    `DV_STRINGIFY` live in dv_macros.svh): ALL found 26 more OpenTitan files than the region needle — packages,
    interfaces, vseq lists, testbenches — every one a rendering; a scan of every changed line for an odd number of
    unescaped quotes reads 0 (no literal left open across a line).
(2) STRING-REGION-PROBE (probe_run.txt): rows=12 pgen_eq_verilator=8 pgen_eq_iverilog=7 verilator_eq_iverilog=6 —
    the four PGEN≠Verilator rows: string_macro.sv (Verilator refuses the FILE, "Unterminated string" on sv2v's
    plain-quote closer), PreprocString + s4 (MEM_ROOT undefined: Verilator refuses, Icarus drops `message, PGEN keeps
    the bytes Surelog's golden keeps), s2 (the source-line region: Verilator renders, the clause and the .6c.2 pin do not)
(3) cargo test --lib sv_preprocessor: 83 passed / 0 failed (74 + 9); clippy_on_rust_change ✅ on the formatted bytes
    (GENERATED-CLIPPY-CORRECTNESS: ✅ POLICY-ONLY PASS, 68 lints; guard peak 10 608 MB)
(4) THE LANE (stimuli/run_pipeline_lane.sh all, debug probe, 8 jobs, 60 s, idle host, wall 410 s, peak 10 235 MB): files=16336 pass=12202 fail=3857 pp_fail=277 timeout=0 crash=0 against the tracked 12200 / 3859 / 277 / 0 — SIX rows moved, every one named above: opentitan otbn_ssctrl_if.sv, prim_onehot_check_if.sv, prim_sparse_fsm_flop_if.sv, dma_sys_tl_if.sv fail → pass; iverilog pr1741212.v (a source-line paste, no answer key) and verilator t_preproc_strify_join.v (three bare strings at file scope, svpp_owned) pass → fail; two origin moves with the verdict kept (sv2v macro.sv :142:33 → :142:14, t_pp_display.v the same line); the disposition table needs no row retired (0 dispositioned rows now pass); re-adjudication chained:accepted_under_expansion 1964 → 1968, chained:rejects_own_file 710 → 706, corroborated 1224 unchanged; PIPELINE-LANE-CURRENCY (1)(2)(3) OK; coverage: raw 7486/2606/6244/4039/6 UNCHANGED, pipeline 9136/2606/4594/3313/5 → 9140/2606/4590/3310/5 (adjudicated +4, no-verdict −4, dark −3, the bar unmoved); check_sv_corpus_denominator.sh OK after the anchor moved
(5) pins (`stimuli/sv/run_adjudication_repros.py`): debug probe `checked=1214 armed=231 listed=607 preprocess_rows=13 failures=0` and the rebuilt release probe (the lowmem host recipe, installed at the provenance path) the same — 1210 / 605 / 11 before, the two new rows (`fixed_string_region_object_macro.sv` ACCEPT, `invalid_string_region_source_line.sv` REJECT — the pin runner refused a REJECT under class `control`, whose class IS the claim, so the negative control carries class `invalid` as the earlier keyword-region control does); the three RAW lanes (`stimuli/run_external_corpus.sh sv | sv2023 | sv2005`, release probe, into `PGEN_CORPUS_OUT_DIR`, verdict columns diffed): all three IDENTICAL — `sv_2017` 9 844 / 6 492 (16 336 rows), `sv_2023` 9 869 / 6 467, `verilog_2005` 2 313 / 293 (2 606 rows), 0 rows moved on any lane
```

**Two more things the class diff decided, both stated rather than smoothed over:**

- **A string literal is ONE line.** §5.9 (`docs/systemverilog/2017/md/section-32-h-12ab-f001.md:101`): *"A string
  literal shall be contained in a single line unless the newline character is immediately preceded by a \"* — and a
  macro body's backslash-newline *"shall be replaced in the expanded macro with a newline"* (§22.5.1), so an embedded
  usage whose body spans lines (`` `DV_LC_TX_DIST `` inside `` `DV_CHECK_STD_RANDOMIZE_WITH_FATAL ``'s stringified
  condition, OpenTitan `cip_base_pkg.sv:118`) put a raw newline INSIDE the literal — eight lines of one string — which the
  parser refuses (`$display("a\nb")` REJECT, `"a \<nl> b"` ACCEPT, measured). Verilator renders the newline as a space
  (its `twoline` golden `"twoline: \"first        second\""`), Icarus leaves it raw. The one-line reading is the only one
  a parser can read: every newline in a rendered literal becomes a space, its indentation kept (test 9, Verilator's
  golden verbatim). ⭐ It also repaired what the OLD renderer had been emitting for a multi-line ACTUAL written by the
  caller — `prim_onehot_check_if.sv` (3-line literal → 1), `spi_agent_pkg.sv` (6 → 1), `dma_sys_tl_if.sv` (2 → 1).
- **A paste in a SOURCE line stays for the parser — consistently now.** iverilog `pr1741212.v:485`
  `` $display( "cos(%s): %f", ``MATH_2_PI, cos(20*`MATH_2_PI) ); `` moved `pass → fail` under the lane: the source
  line's second pass used to carry expansion-text provenance and DROPPED the author's paste — but only when a usage
  shared the line (`$display( ``MATH_2_PI );` alone kept it), an accident, not a rule. `.6b.2`(i)'s stated rule is
  *"in a source line it stays for the parser to judge"*, §22.5.1 defines ``` `` ``` for macro text, and the two tools
  extend the form in two different ways (Verilator drops it, `MATH_2_PI`; Icarus STRINGIZES it, `"6.28"`). The row has
  NO answer key (`deferred:no_sv_key`; ivtest expects it to compile under Icarus's reading), so no verdict is asserted
  either way and the dark count is unchanged. Recorded as the strict reading, with both extensions named.
- **`t_preproc_strify_join.v` moved `pass → fail` for the honest reason**: its expected output is three bare string
  literals at file scope — a preprocessor test, not a design — and a bare `"foo-bar-qux"` is a parser refusal
  (measured) where the unrendered `` `"foo-bar-qux`" `` had been tolerated as a stray directive token. A
  `svpp_owned` row either way (NO VERDICT); the bytes now match its `.out`.

**ROUTED — owned, each with the measurement that found it:**

- **`.5`(k) — a formal inside a plain `"…"` string** (from `.5`(i), unchanged here): substituted here and in Icarus,
  kept verbatim in Verilator; §22.5.1 `:370-372` reads with Verilator. A census first.
- **sv2v's plain-quote closer** — `macro.sv:102` `` `define MACRO_F(t) $display(`"s t = `\`"t`\`""); `` closes a region
  with a bare `"`; sv2v's reference renders `"s t = \"foo\""`, Verilator refuses the file (*Unterminated string*), and
  the clause says *"the use of " always starts a string literal and must have a terminating ""*. NOT adopted: the region
  stays as written, the row is a pinned must-reject (`:142`) either way. Recorded here so the sv2v extension is a known
  reading, not a rediscovery; sv2v's own `macro_iv.sv` twin spells the standard's `` `\`"`" `` and renders.
- **The lane does not carry a Surelog `.sl` driver's `-D`** — `PreprocString.sl` passes `-DMEM_ROOT=test` and
  `pipeline_context_defines.tsv` has no row for it, so the lane reads `"`MEM_ROOT/rom.mem`message"` where Surelog's
  golden reads `"test/rom.mem`message"`. A `.4c.1` follow-up (the deriver reads `+define+` and `-D` in some driver
  shapes, not `.sl`); it changes no verdict today (the row is corroborated) and is named rather than left.

**Acceptance checklist (`TASK-ACCEPTANCE`):**

  - [x] **REPRODUCE / ISSUE** — `./rust/target/debug/parseability_probe --parse systemverilog stimuli/sv/subs/opentitan/hw/ip/otbn/dv/uvm/env/otbn_ssctrl_if.sv --profile sv_2017 --preprocess <the lane's context>` → `[origin=…/otbn_ssctrl_if.sv:15:13]`, the `{` holding the unrendered `` `"tb.dut…`" ``; `ast_pipeline --preprocess-systemverilog --output` on `rust/target/s5j/msg.sv` (the clause's example) → `"left side: `\"right side`\""` where the clause prints `"left side: \"right side\""`; `STRING-REGION-PROBE` rows `NO` before the fix (sv_macro3a/b, str_undef, strify_join, PreprocString, s1, s3).
  - [x] **ROOT CAUSE (WHY + WHERE)** — correctness family: `furthest_position=` / `[origin=…:15:13]` on the OpenTitan row; the expansion text beside `verilator -E -P` and `iverilog -E` on the same bytes (`STRING-REGION-PROBE:` before: PGEN≠both on every region row); for (j′) the preprocessor's own `E_SVPP_MISSING_MACRO_ARGUMENT … `thru invoked with 0 of 1 arguments` on `rust/target/s5j/thru.sv` line 4 — an ORDINARY line — on the pre-change binary. WHERE: `substitute_function_macro_body`'s region branch (rendered at substitution; its two-byte close scan), `expand_macros_in_text`'s `` `" `` pass-through, `parse_macro_invocation_arg_ranges`'s `if s < e || !args.is_empty()`. WHY: the clause's three things (§22.5.1 `:366-368`) and its empty-actual sentence (`:220-221`).
  - [x] **FIX** — engine tier (a preprocessor has no grammar): one renderer in the rescan for both paths, an escape-aware close finder, substitution emitting the region with its markers, `()` as one empty actual; the source-line second pass kept a source line. ZERO grammar / codegen / generated bytes.
  - [x] **ADDRESSED (verified)** — the OpenTitan row `otbn_ssctrl_if.sv` `[origin=…:15:13]` → `parse_full passed … [preprocessed]` (`fail → pass` in the lane); the clause's `msg` example → `"left side: \"right side\""` (test 3; iverilog `sv_macro2.v` and sv-tests `22.5.1--define-expansion_25.sv` now render it verbatim, POPULATION-DIFF); `STRING-REGION-PROBE` `pgen_eq_verilator` 4 → 8 of 12 (the four remaining `NO` rows each a recorded tool split); `t_pp_display.v` `refuse → pass` at the preprocess stage on (j′); `cargo test --lib sv_preprocessor` 83/83 with the nine oracle-golden tests; the lane's four `fail → pass` rows; the pipeline tuple `9136/2606/4594/3313/5 → 9140/2606/4590/3310/5` through `SV-CORPUS-DENOMINATOR`'s gate.
  - [x] **NO REGRESSION** — POPULATION-DIFF over the WHOLE corpus: 16 342 of 16 388 expansions BYTE-IDENTICAL, rc moved on 0, every one of the 46 movers a rendering, 0 changed lines with a literal left open; the lane's other 16 330 rows verdict-identical, the two `pass → fail` rows both NO VERDICT (`no_sv_key`, `svpp_owned`) with their readings recorded above; the raw tuple `7486/2606/6244/4039/6` unmoved; the bar unmoved at 5; pins (`stimuli/sv/run_adjudication_repros.py`): debug probe `checked=1214 armed=231 listed=607 preprocess_rows=13 failures=0` and the rebuilt release probe (the lowmem host recipe, installed at the provenance path) the same — 1210 / 605 / 11 before, the two new rows (`fixed_string_region_object_macro.sv` ACCEPT, `invalid_string_region_source_line.sv` REJECT — the pin runner refused a REJECT under class `control`, whose class IS the claim, so the negative control carries class `invalid` as the earlier keyword-region control does); the three RAW lanes (`stimuli/run_external_corpus.sh sv | sv2023 | sv2005`, release probe, into `PGEN_CORPUS_OUT_DIR`, verdict columns diffed): **all three IDENTICAL** — `sv_2017` 9 844 / 6 492 (16 336 rows), `sv_2023` 9 869 / 6 467, `verilog_2005` 2 313 / 293 (2 606 rows), 0 rows moved on any lane; `clippy_on_rust_change` ✅ on the formatted bytes (`GENERATED-CLIPPY-CORRECTNESS: ✅ POLICY-ONLY PASS`, 68 lints); no grammar / codegen / generated byte moved; `bash scripts/check_doctrines.sh` ALL PASS (run at commit).
  - [x] **LOCKSTEP** — the SV parser book chapter (*The string region*), the SV integration contract's *Front End* guarantees, the preprocessor contract's *Runtime Execution Stage*, the tracked probe + shapes + run + population diff under `docs/tasks/artifacts/svpp_expansion/string_region/`, the re-rendered re-adjudication / coverage artifacts and the live tuple, this tree, `docs/TASK_TREE.md`, `MEMORY.md`, `CHANGES.md`, `DEVELOPMENT_NOTES.md`. No release / schema bump.
  - promotion: declined for now — *a region is rendered where it lands, once, by the rescan* is stated in the code.

## `.5`(k) a formal inside a plain `"…"` string in macro text — ✅ done 2026-09-15 (`PGEN-SVPP-EXPANSION-0027`): the standard's opaque string by default, the two measured tool readings behind a named policy

**The clause, with its own worked example.** IEEE 1800-2017 §22.5.1 (`docs/systemverilog/2017/md/section-22-compiler-directives.md:370-372`):
*"the use of " always starts a string literal and must have a terminating ". Any characters embedded inside this string
literal, including `", become part of the string in the replaced macro text"* — and `:340-352`: `` `define H(x) "Hello, x" ``
… `` $display(`H(world)); `` … *"will print: … Hello, x"*. The `` `" `` form (`.5`(j)) exists because a plain string does
NOT substitute.

**The row.** `.5`(i)'s three-way probe, shape `s1` (verible `lint/testdata/macro_string_concatenation.sv`,
`` `define FOO(arg) "foo``arg``bar" ``): `"fooxbar"` here and in Icarus 13.0, `"foo``arg``bar"` verbatim in Verilator
5.050. sv-tests `22.5.1--define-expansion_24.sv` and ispras `22.05.01_07.sv` carry the clause's `H(x)` example verbatim
— parse-verdict rows, so the bytes had never been looked at.

**The census, before the ruling** (`docs/tasks/artifacts/svpp_expansion/plain_string/census.py`, full bodies with continuations joined,
every plain-string literal OUTSIDE a `` `" `` region, every whole-identifier formal inside it): **46 files, 63 macro
bodies, 87 sites** in nine suites; by shape, **66 PROSE / FORMAT** — a formal's NAME that happens to be a word of the
message: iverilog's `` `define check(value, expected, error) `` … `"  expected %0h, got %0h", expected, value` (12
files), Verilator's `` `check(got, exp) `` … `"got='h%x exp='h%x"` (20 files), OpenTitan `pwrmgr_base_vseq.sv`'s
`"in %0d fast clock cycles"` with a formal `cycles`, UVM's `` `UVM_TLM_*_IMP(…, delay) `` … `"called with 'null'
delay"` — every one of which the engine SUBSTITUTED, turning the message into its actual (`"  8'h5 %0h, got %0h"`);
and **6 PASTE-DELIMITED** — OpenTitan's `alert_handler_alert_accum_saturation_vseq.sv` (two tops × two lines)
`` "tb.dut.gen_classes[``i``].u_accu.u_prim_count.cnt_q[0]" ``, `mem_bkdr_util.sv`'s `` "addr 0x%0h not ``_DW``-bit
aligned" ``, verible's `` "foo``arg``bar" `` — where the pastes MARK the formal and the code, written for commercial
simulators, relies on the substitution. ⛔ So the lenient reading mangles 66 of the corpus's 72 sites to serve 6, and
the strict reading serves the 66 and un-substitutes 6 design-code paths that a Verilator run would un-substitute too.

**The ruling — the `.5`(c) precedent applied.** The standard is the default: a plain string in macro text is opaque
(`Hello, x`; Verilator's reading; the 66 messages un-mangled). The two deviations are named POLICIES, each a measured
population, on `SvPreprocessorConfig::plain_string_substitution` / `--sv-plain-string-substitution` on both binaries:
`paste_delimited` — only a formal DELIMITED by pastes on both sides is substituted, the pastes removed, a bare word in
the same string stays prose (OpenTitan's idiom, 5 sites; it never mangles a message); `all` — every formal substituted
and every paste joined (Icarus's reading, this engine's until now). A consumer parsing OpenTitan DV code names
`paste_delimited`; nobody is served by `all` except a reproduction of Icarus, which is why it is spelled out rather
than left as an unnamed default.

**What shipped** (`rust/src/sv_preprocessor.rs`, `rust/src/main.rs`, `rust/src/bin/parseability_probe.rs`; ZERO
grammar / codegen / generated bytes; 6 tests, 88/88; the `.5`(i) pin that had deliberately held the old bytes for the
verible shape rewritten to the ruling it deferred to):

- `PlainStringSubstitutionPolicy { Standard (default), PasteDelimited, All }` with `parse`; the policy threaded through
  the expansion family (`expand_macros_in_text`, `expand_usage_at`, `render_macro_string_region`, `join_paste_chain`,
  `expand_object_macro`, `expand_function_macro`, `substitute_function_macro_body`) from `state.config` at the four
  top-level calls — explicit, as `expanding` and `user_mask` are, not a global.
- `substitute_function_macro_body`: inside a plain string a formal is substituted only as the policy reads it, and a
  paste inside a plain string is the string's under `standard` (Verilator keeps it) and removed under the other two.
- Tests: `a_formal_inside_a_plain_string_is_the_strings_by_default` (the clause's three `$display` lines verbatim),
  `prose_that_names_a_formal_is_no_longer_mangled` (iverilog's `check`), `a_paste_delimited_formal_inside_a_plain_string_is_the_idiom_the_knob_admits`
  (OpenTitan's shape under all three), `the_verible_paste_string_shape_under_each_reading`,
  `the_policy_parses_its_three_spellings_and_refuses_others`, and the rewritten `a_paste_inside_a_plain_string_is_the_strings_by_default`.

**Measured** (rebuilt debug binaries; the post-`-0026` binaries snapshotted first, `rust/target/s5k/ast_pipeline.before`):

```text
(1) POPULATION-DIFF over the WHOLE corpus (population_diff.sh … ALL; 8-way; the context is now the LANE's own per-suite
    derivation — the .5 UVM+OpenTitan directories cannot resolve VeeR's generated snapshots/default/el2_param.vh, so
    under them VeeR refused identically on BOTH binaries and read as "identical"; stated, and fixed in the script):
      first cut (the escape leak, see below): files=16388 identical=15820 moved=568 — REJECTED, a bug in the change
      after the fix, .5 context:            files=16388 identical=16321 moved=67   (932 changed lines)
      after the fix, the lane's context:    files=16388 identical=16288 moved=100  (rc 0 on 16 108, rc 1 on 280)
    every changed line INSIDE a plain string: 1 539 lines in 99 files by a classifier that blanks every quoted literal
    and compares the rest; the one line it flags is verilator t_preproc_def09.v, whose `'"== b c'` was the OLD
    substitution of an actual into `'a b c'` inside a deliberately quote-mangling define
(2) PASTE-ORDER-PROBE: rows=14 pgen_eq_verilator=14 pgen_eq_iverilog=9 verilator_eq_iverilog=9 — was pgen_eq_verilator=8
    (shape s1 now agrees); STRING-REGION-PROBE unchanged since -0026 (rows=12 pgen_eq_verilator=8)
(3) the census: 46 files / 63 bodies / 87 sites (66 prose + 6 paste-delimited + the rest in `" regions or Verilator's
    four vendored UVM copies); iverilog `check` ×12 files, verilator `check`/`checkh` ×20, OpenTitan pwrmgr ×3, UVM ×2
(4) cargo test --lib sv_preprocessor: 89 passed / 0 failed (82 + 7); clippy_on_rust_change ✅ on the formatted bytes
(5) THE LANE, twice: run 1 (a platform-book build overlapping the library pass's first minute) files=16336 pass=12144
    fail=3915 — the library pass timed out its FIRST TWO packages, VeeR's el2_pkg and tb_top_pkg (0.40 s / 0.08 s alone,
    1.87 s in the tracked run), and their 58 dependants failed at el2_param.vh:182:22 with no facts to import; run 2,
    idle host: PIPELINE-LIBRARY Cores-VeeR-EL2=2/2 … published_packages=166 and files=16336 pass=12202 fail=3857
    pp_fail=277 timeout=0 — IDENTICAL to the tracked dump, 0 verdict movers, 0 origin-line movers (one furthest-byte
    shift inside sv2v macro.sv's failing line 142); re-adjudication unchanged; PIPELINE-LANE-CURRENCY OK; the tuple
    9140/2606/4590/3310/5 unchanged through check_sv_corpus_denominator.sh (no anchor move)
(6) pins (`stimuli/sv/run_adjudication_repros.py`): debug probe `checked=1214 armed=231 listed=607 preprocess_rows=13 failures=0`, and the rebuilt release probe (the lowmem host recipe, installed at the provenance path, WARMED first — its cold first launch had aborted the runner) the same; the three RAW lanes (`stimuli/run_external_corpus.sh sv | sv2023 | sv2005`, release probe, into `PGEN_CORPUS_OUT_DIR`, verdict columns diffed): all three IDENTICAL — `sv_2017` 9 844 / 6 492 (16 336 rows), `sv_2023` 9 869 / 6 467, `verilog_2005` 2 313 / 293 (2 606 rows), 0 rows moved and 0 timeout rows on any lane
```

⛔ **A FRESHLY BUILT OR COPIED BINARY'S FIRST LAUNCH ON THIS HOST COSTS 40–100 s, AND IT IS A MEASUREMENT HAZARD** (measured 2026-09-15, `.5`(k)): the lowmem release probe's first `--parser-fingerprint` exceeded the pin runner's 60 s (`subprocess.TimeoutExpired`), the SECOND first-launch attempt took **43.49 s**, the third launch **0.008 s**; a fresh COPY at the provenance path is a new inode and pays again (its first call landed inside the raw-lane run). It is the host's first-execution validation of an 83 MB binary, not project code — and it is what timed out VeeR's `el2_pkg` / `tb_top_pkg` (0.4 s / 0.08 s alone) at the START of the library pass right after a debug rebuild, costing 58 rows in the first lane run. RULE: after every build or copy, warm the binary once (`<probe> --parser-fingerprint`) BEFORE any timed measurement, and read a timeout on the first rows of a run as this before reading it as the parser.

**ROUTED — owned:** `scripts/report_family_status.sh` splits an index row's cells on EVERY `|` and does not honour the
escaped `\|` that `TABLE-ARITY-RATCHET` honours (and GFM defines), so a code span carrying `UVM_ALL_ON\|UVM_HEX` in this
tree's index cell cut the cell short of its `active` and the row read UNREADABLE — the derived count moved 6 → 7 and
`FAMILY-STATUS-CURRENCY` refused the commit. Two readers of the same table disagree about one escape; the index cell was
respelled without the pipe here, and the reader defect is `DONE-BAR-VISIBILITY.8`'s (the leaf that reconciles the
index's readers), named rather than left.

**Acceptance checklist (`TASK-ACCEPTANCE`):**

  - [x] **REPRODUCE / ISSUE** — `ast_pipeline --preprocess-systemverilog --output` on `rust/target/s5k/plain.sv` (`` `define P(arg) $display("foo arg bar", arg) `` …): `$display("foo val bar", val);` here, `$display("foo arg bar", val);` in Verilator 5.050 (`verilator -E -P`); the clause's own `H(world)` printing `Hello, world` here where the clause prints `Hello, x`; `PASTE-ORDER-PROBE` row `s1` `PGEN=V NO` before.
  - [x] **ROOT CAUSE (WHY + WHERE)** — correctness family: the expansion text beside `verilator -E -P` and `iverilog -E` on the same bytes (`PASTE-ORDER-PROBE:` / `STRING-REGION-PROBE:` rows, and the census's 66 mangled messages read from the expansions); WHERE: `substitute_function_macro_body`'s identifier branch substituted a formal wherever it stood — `in_plain_string` was tracked (since `.5`(i)) but consulted only for the paste — so a plain string was as transparent as body text; WHY: §22.5.1 `:370-372` makes the literal opaque and its `:340-352` example prints `Hello, x`.
  - [x] **FIX** — engine tier: the policy enum, the threaded parameter, the two in-string rules; the flag on both binaries; six tests. ZERO grammar / codegen / generated bytes.
  - [x] **ADDRESSED (verified)** — the clause's example `$display(`H(world));` → `$display("Hello, x");` (test 1, and sv-tests `22.5.1--define-expansion_24.sv` / ispras `22.05.01_07.sv` in the corpus diff: `"Hello, world"` → `"Hello, x"`); iverilog's `check` message `"  8'h5 %0h, got %0h"` → `"  expected %0h, got %0h"` (test 2; 12 corpus files); UVM's `"Field macro for data uses UVM_ALL_ON|UVM_HEX"` → `"… for ARG uses FLAG"` (the diff, 26 files); `PASTE-ORDER-PROBE pgen_eq_verilator 8 → 14`; OpenTitan's idiom under `paste_delimited` `"tb.gen[3].cnt i"` (test 3); 89/89.
  - [x] **NO REGRESSION** — the lane on an idle host IDENTICAL to the tracked dump (`pass=12202 fail=3857 pp_fail=277 timeout=0`, 0 verdict movers); the whole-corpus diff: 16 288 of 16 388 expansions byte-identical and every changed line of the 100 movers inside a plain string; the tuple `9140/2606/4590/3310/5` and the raw `7486/2606/6244/4039/6` unmoved; `STRING-REGION-PROBE` unchanged; pins (`stimuli/sv/run_adjudication_repros.py`): debug probe `checked=1214 armed=231 listed=607 preprocess_rows=13 failures=0`, and the rebuilt release probe (the lowmem host recipe, installed at the provenance path, WARMED first — its cold first launch had aborted the runner) the same; the three RAW lanes (`stimuli/run_external_corpus.sh sv | sv2023 | sv2005`, release probe, into `PGEN_CORPUS_OUT_DIR`, verdict columns diffed): **all three IDENTICAL** — `sv_2017` 9 844 / 6 492 (16 336 rows), `sv_2023` 9 869 / 6 467, `verilog_2005` 2 313 / 293 (2 606 rows), 0 rows moved and 0 timeout rows on any lane; `clippy_on_rust_change` ✅ on the formatted bytes (`GENERATED-CLIPPY-CORRECTNESS: ✅ POLICY-ONLY PASS`, 68 lints); no grammar / codegen / generated byte moved; `bash scripts/check_doctrines.sh` ALL PASS (run at commit).
  - [x] **LOCKSTEP** — the SV parser book chapter (*A plain string in macro text*, the flag), the platform book's CLI chapter (the flag), the SV integration contract's *Front End* guarantees (the policy and its default), the preprocessor contract's *Runtime Execution Stage*, the re-rendered artifacts and the live tuple as the lane decides, this tree, `docs/TASK_TREE.md`, `MEMORY.md`, `CHANGES.md`, `DEVELOPMENT_NOTES.md`. No release / schema bump.
  - promotion: declined for now — *the lenient reading serves the few and mangles the many: count both before ruling* is stated here and in `DEVELOPMENT_NOTES.md`.

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
  ⭐ **2026-09-24 — a FOURTH surface waits on this ruling (`SV-CORPUS-GRAD.24`(d), `PGEN-SV-CORPUS-GRAD-0415`).** `.24`(b) gave the preprocessor syntax-closure contract its identity block, so `sv_formal_exhaustive_closure_gate` runs to its surface for the first time since `1.0.243` — and reads `external_corpus_backed_proof_totals_drift=count=2` / `status_drift=count=4`: the external-corpus triage's `uvm_pkg` and `uvm_compat_pkg` cases fail on both profiles at exactly this line (`furthest_position=1016474`, the preprocessed `uvm_transaction.svh:443:38`), against a sidecar (`systemverilog_external_corpus_backed_proof_surface_v0.json`) pinned 14 / 14 on 2026-05-29 under the mis-lexed `local`. Not re-pinned: `all_cases_must_pass` is that proof's meaning. Until ruled, the SV done-bar's second leg (`.24`) stays open on it beside `.6d.3` and `.4`'s lane.
  ⭐⭐ **2026-09-30 — a FIFTH surface waits on this ruling, and it is the PUSH LANE (`PGEN-SVPP-EXPANSION-0028`).** The 300-commit push cadence's full CI mirror replays `sota-exit-gate`, whose required check `sv_parser_family_status_gate` runs `sv_formal_exhaustive_closure_gate`, whose external-corpus-backed proof surface (`rust/test_data/grammar_quality/systemverilog_external_corpus_backed_proof_surface_v0.json`, `closure_rules.no_parse_failures_allowed: true`) reads the two UVM triage cases as `parse_fail` in both profiles (`parse_pass_total 10 / parse_fail_total 4` against the tracked `14 / 0`, `furthest_position=1016474` = `uvm_transaction.svh:443`). Measured today on `50a97d00c`: the local aggregate (4.9 h) refused it, the bisect over the 50 grammar commits since the gate's last green run (2026-08-24) lands on `e0a324964` exactly as `.3`'s bisect did, the one-line repro `class P #(type T=int); endclass typedef P #(int) T1; class C; const local T1 events = new("events"); endclass` fails on the shipped parser and the interpreter alike while `const T1 …` and `local T1 …` pass, and the widened arm (`( assign ( class_new | constant_expression ) )?`, class_new first) parses every repro through the interpreter — built, measured, and WITHDRAWN before regeneration because it is this ruling. Until ruled: the tree's grammar stands, the mirror cannot be green, the push waits (313 unpushed at the moment of writing). The bounding ruling's own operative test (`feedback_sv_strict_lrm_compliance_default`: *does the change make the SV parser reject input the ecosystem actually writes?*) answers this one: the whole UVM library and everything built on it. Recommendation unchanged: (a), with the deviation record; the alternative that keeps the letter is (b), the `--sv-extensions` knob — either is a ruling, and either unblocks the lane the day it lands (a grammar release under the ceremony, or the sidecar re-baselined with the four rows owned by name).
  ✅✅ **RULED AND CLOSED 2026-10-01 — option (a), release `1.0.247` (`SV-CORPUS-GRAD.37`, `PGEN-SV-CORPUS-GRAD-0423`, ledger `SV-0132`).** Asked a third time on 2026-10-01 ("looks like you are stuck"), the director left the standing 2026-08-10 bounding ruling to decide it: the operative test comes first, and this rejection took the whole UVM library. The documented tool-extension arm is ON by default; the deviation record is the SV book's pipeline chapter and the changelog; the `--sv-extensions` knob of option (b) is not built. The five surfaces that waited: `.4`'s lane (UVM and its dependants), `.6d.3` (unblocked below), `SV-CORPUS-GRAD.24`(d), the ceremony's item 11 clause, and the push lane's mirror — all read the same arm now. Measured: the SV closure gate's external-corpus triage `parse_pass_total: 14` / `parse_fail_total: 0` (was 10 / 4), and under the pipeline `uvm.sv`, `uvm_pkg.sv` and three dependants move `fail` → `pass` (`PIPELINE-MOVERS … moved=5`). The mirror's `sota-exit-gate` is not green yet for two OTHER reasons the same run found one stage above the triage — `ENGINE-UNIVERSAL-SERVICES.119` and the SV register row (`DONE-BAR.5e`).

- ✅ **CLOSED, AND IT WAS ALREADY CLOSED WHEN THIS BULLET LAST READ "OWED" (corrected 2026-09-15 under a
  director challenge, `PGEN-SVPP-EXPANSION-0024`).** The question was: the preprocessor's 26 pre-existing unit
  tests write their scratch files to `std::env::temp_dir()`, off the repository volume, against the
  data-locality rule ([[project_north_star]]). **`.5`(e) fixed it** (`PGEN-SVPP-EXPANSION-0006`, `afed30507`) by
  making `create_temp_dir` a shim over `create_repo_scratch_dir`, so every test's scratch directory is under
  `rust/target/svpp_unit_tests/` on the repository volume. Re-derived at HEAD:
  `grep -c 'temp_dir()' rust/src/sv_preprocessor.rs` = **0** over 28 `create_temp_dir` call sites.
  ⛔ **The routing outlived the fix by eighteen commits** — the fix landed in the leaf that the routing named,
  and nothing walked back to the question that sent it there. ⇒ **a routed item is not closed by the fix; it is
  closed by someone re-reading the route**, which is why this bullet now carries the command that decides it.

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

## ROUTING EVIDENCE

Every finding this tree has sent OUT of it, with what was MEASURED to place it there — and what would
have to be true for the placement to be wrong. ⛔ An honest *"not checked outside this family"* is a legal
answer here; a silent routing is not.

### The off-volume test-scratch question → `.5`, and the route that outlived its own fix (2026-09-09 → corrected 2026-09-15)

- **What was routed.** `.2` found the preprocessor's 26 pre-existing unit tests writing scratch files to
  `std::env::temp_dir()`, off the repository volume, and routed the repair to `.5` as a housekeeping item.
- **Does it reproduce outside this family?** The question was never about a parser family: it is a
  DATA-LOCALITY breach in one Rust test module, and it was measured in that module only
  (`rust/src/sv_preprocessor.rs`). Other families' tests were NOT checked at the time and are not covered
  by this routing — stated rather than implied. (A repo-wide sweep for the same shape is not owed by this
  tree; if one is wanted it belongs with the data-locality policy owner, not here.)
- **What was measured to place it in `.5`?** `.5` was the leaf already editing that test module, so the
  repair was one edit in the file the leaf had open — placement by FILE, not by plausibility.
- **What would make the routing wrong, and was it checked?** It would be wrong if `.5` closed without doing
  the work. ⛔ **It did the work — and the route was never re-read, so the question stayed open for eighteen
  commits while the code was already correct.** Re-derived at HEAD under a director challenge
  (`PGEN-SVPP-EXPANSION-0024`): `grep -c 'temp_dir()' rust/src/sv_preprocessor.rs` = **0** over 28
  `create_temp_dir` call sites, which is a shim over `create_repo_scratch_dir` since `afed30507`
  (`.5`(e)). ⇒ **a routed item is not closed by the fix; it is closed by someone re-reading the route**, and
  every routing bullet in this tree now carries the command that decides it.

### The parser questions of `.6b.4` / `.12b.1` → `SV-CORPUS-GRAD` (2026-09-09)

- **Does it reproduce outside the family being routed to?** Yes, deliberately: each row was re-run on BOTH
  SV profiles and, where the construct is not SV-only, on `verilog_2005` — that is why eleven of them came
  back as the standard's own refusals rather than as SV grammar gaps, and two were re-routed AWAY from
  `SV-CORPUS-GRAD` (one to `.5`(i) here, one to a driver-define tie).
- **What was measured?** The probe's furthest TOKEN on the expanded text, not the mapped origin line —
  `.12b.1`'s whole finding is that the mapped line is the last MAPPED byte and the construct is the token.
- **What would make it wrong?** A row whose furthest token is inside a macro body (the preprocessor's, not
  the parser's). Checked per row; the two that were, were re-routed.

### `ENGINE-UNIVERSAL-SERVICES.107` / `SV-CORPUS-GRAD.3.62` — routed OUT of this tree's scope on purpose

- **Does it reproduce outside SystemVerilog?** **Yes, and that is the evidence for the placement**: the
  unterminated-block-comment defect is in the ENGINE's built-in layout skipper, and re-emitting all eleven
  generated artifacts moved **seven** of them. Measured per family (a file ending in `/*` versus the same
  file with the comment closed): `vhdl`, `return_annotation`, `rtl_frontend`, `rtl_const_expr` and `json`
  all move the same way as `systemverilog`. Routing it to the SV tree alone would have been the misroute.
- **What would make it wrong?** If the arm were SV-specific. It is not: `regex` is unaffected because it is
  `@whitespace_sensitive`, and `ebnf` / `scratch` / `semantic_annotation` are unaffected because they claim
  `"/*"` as their own token — four negatives that are properties, not coincidences.

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
| `2026-09-09` | `.5`(g)(h)(g′) | `rust/target/svpp_e1/verify_s5gh.sh` (`verify_s5gh_final2.log`): byte-identity ×4, census `rc_nonzero=73`, the four fixtures + the two Surelog timeouts refuse at the preprocess stage in 0 s, the (g′) reproducer expands; `cargo test --lib sv_preprocessor` 46/46; the whole lane (debug probe, 600 s): `files=16336 pass=11671 fail=4387 pp_fail=278 timeout=0`, 23 movers all explained, 0 `pass → fail`; clippy_on_rust_change ✅ (`CLIPPY_RC=0`) | ✅ |
| `2026-09-09` | `.6a` | `pipeline_readjudication.py` → `rows=6534 missing_lane_rows=0`, `--check` OK; `check_pipeline_lane_currency.sh` arms (1)(2)(3) OK, `--self-test` 5/5; the answer key untouched | ✅ |
| `2026-09-09` | `.6b.1` | 34 pins verified on the corpus rows (both SV profiles) and on minimal reproducers with controls; adjudicator to scratch → exactly 34 rows moved; coverage + provenance + live tuple re-derived, three corpus gates OK; re-adjudication `must_reject:match_under_expansion=404`, `accepts_invalid=2`, `refuted 223 → 189` | ✅ |
| `2026-09-09` | `.6b.2` | `verify_s5gh.sh` on the rebuilt debug binaries: byte-identity ×4, census 73, fixtures unchanged; the whole lane `pass 11671 → 11682`, 11 `fail → pass`, 0 the other way; re-adjudication `refuted 189 → 183`; clippy ✅ | ✅ |
| `2026-09-09` | `.6b.3` | 126 verilator identity-symbol files re-parsed with `VERILATOR` + `verilator` defined: 5 `fail → pass`, 24 `pass → fail` — predefines NOT adopted; 119 dispositions written with evidence, the instrument re-rendered (`refuted_awaiting_disposition=64`), `--check` OK, self-test 5/5 | ✅ |
| `2026-09-09` | `.6b.4` | 64 rows looked at with the toolbox (expansion + probe, the expanded text re-probed where the origin looked wrong); the table re-rendered → `refuted_awaiting_disposition=0`, `--check` OK, self-test 5/5 | ✅ |
| `2026-09-09` | `.6b.5` | rebuilt debug binaries: three `.5` expansions byte-identical, uvm_pkg 98 changed lines in 10 hunks ALL inside block comments (audited), census 73; the whole lane `pass 11682 → 11692`, 10 rows recognised, 0 the other way; re-adjudication `corroborated 1213`, `refuted_awaiting_disposition=0`; clippy ✅ | ✅ |
| `2026-09-09` | `.6c.1` | `corpus_verdict_coverage.py` → `PIPELINE: 8669/2606/5061/3779/30` (raw `7529/2606/6201/3994/9` unchanged); `check_sv_corpus_denominator.sh` OK with both tuples; `check_family_status.sh` OK | ✅ |
| `2026-09-09` | `.6c.2` | 11 reproducers reject / 11 controls accept on both profiles; 28 corpus rows reject under the pipeline on both; the adjudicator to scratch moves exactly 28 rows; `must_reject:match_under_expansion 404 → 433`; both tuples re-derived and anchored | ✅ |
| `2026-09-09` | `.4c.1` | `pipeline_context.py --self-test` 14/14, `--check` OK; the whole lane re-run on the unchanged debug probe (463 s, peak 9 955 MB): exactly 5 movers, all among the 48 files with defines; 2 reproducers reject / 2 controls accept on both profiles; the adjudicator moves exactly 3 rows; `must_reject:match_under_expansion 433 → 436`; both tuples re-derived and anchored; `PIPELINE-LANE-CURRENCY` OK + self-test 5/5 | ✅ |
| `2026-09-09` | `.4c.2` | `pipeline_context.py --self-test` 20/20, `--check` OK; the 4 rows pre-measured with the hand-given predecessor (rc 0 on both profiles); the whole lane re-run (468 s, peak 9 390 MB): exactly 3 movers, all `fail → pass`, all among the 17 files with predecessors, 0 the other way; the pipeline tuple re-derived and anchored, the raw one unmoved; `PIPELINE-LANE-CURRENCY` OK + self-test 5/5 | ✅ |
| `2026-09-09` | `.6d.1` | the mechanism measured on four scratch chains (export → import → pass; same-dir; named import); the builder's smoke run (3 suites) then the full pass (407 files, 166 published, 0 cycles); the whole lane re-run (689 s, peak 9 797 MB): 425 movers, all `fail → pass`, 0 the other way; the pipeline tuple re-derived and anchored, the raw one unmoved; `PIPELINE-LANE-CURRENCY` OK + self-test 5/5 | ✅ |
| `2026-09-09` | `.6d.2` | the library re-derived standalone with the membership table (226 s; verdicts identical to `-0019`); the re-adjudication re-read 2 059 members (0 / 89 / 1 923); the zero-accepted count verified from the dumps by a second script; coverage buckets the new classes; the tuple unmoved through the denominator gate; currency OK + self-test 5/5 | ✅ |
| `2026-09-09` | `.7` | every contract sentence read from its source (`parseability_probe.rs:502-505`, `origin_of_position`, `grep -o E_SVPP_`, a real `facts.json`); `mdbook build` ×3 clean; `check_sv_contract_currency.sh` OK; no dump or tuple moved | ✅ |
| `2026-09-15` | `.5`(k) | `cargo test --lib sv_preprocessor` 89/89 (7 new — the clause's example, iverilog's `check`, OpenTitan's idiom under all three policies, verible's shape, the policy's spellings, Verilator's `checkh` escape shape — and the `.5`(i) pin rewritten to the ruling it deferred to); the census 46 / 63 / 87 (66 prose, 6 paste-delimited); POPULATION-DIFF over the WHOLE corpus under the lane's context `files=16388 identical=16288 moved=100`, 1 539 changed lines all inside a plain string (the first cut's 568 movers were the escape leak — rejected as a bug in the change); PASTE-ORDER-PROBE `pgen_eq_verilator=14` of 14; the lane twice — run 2 (idle host) IDENTICAL to the tracked dump, 0 verdict movers, run 1's 58 VeeR failures the library pass's first two packages timing out under a book build; `PIPELINE-LANE-CURRENCY` OK; the tuple unmoved; pins (`stimuli/sv/run_adjudication_repros.py`): debug probe `checked=1214 armed=231 listed=607 preprocess_rows=13 failures=0`, and the rebuilt release probe (the lowmem host recipe, installed at the provenance path, WARMED first — its cold first launch had aborted the runner) the same; the three RAW lanes (`stimuli/run_external_corpus.sh sv \| sv2023 \| sv2005`, release probe, into `PGEN_CORPUS_OUT_DIR`, verdict columns diffed): **all three IDENTICAL** — `sv_2017` 9 844 / 6 492 (16 336 rows), `sv_2023` 9 869 / 6 467, `verilog_2005` 2 313 / 293 (2 606 rows), 0 rows moved and 0 timeout rows on any lane; `clippy_on_rust_change` ✅ | ✅ |
| `2026-09-15` | `.5`(j) | `cargo test --lib sv_preprocessor` 83/83 (9 new, each a Verilator / Icarus / sv2v / Surelog / LRM golden); POPULATION-DIFF by needle (region 483/463/20, empty invocation 107/100/7, paste 188/180/8) and over the WHOLE corpus `files=16388 identical=16342 moved=46`, rc moved 0, 0 literals left open; STRING-REGION-PROBE `rows=12 pgen_eq_verilator=8 pgen_eq_iverilog=7 verilator_eq_iverilog=6`; the lane (idle host) `files=16336 pass=12202 fail=3857 pp_fail=277 timeout=0` — six movers, four `fail → pass` (OpenTitan), two `pass → fail` (both NO VERDICT, recorded); re-adjudication OK, `PIPELINE-LANE-CURRENCY` OK; pipeline tuple `9136/2606/4594/3313/5 → 9140/2606/4590/3310/5`, raw unmoved; `check_sv_corpus_denominator.sh` OK; `clippy_on_rust_change` ✅; pins (`stimuli/sv/run_adjudication_repros.py`): debug probe `checked=1214 armed=231 listed=607 preprocess_rows=13 failures=0` and the rebuilt release probe (the lowmem host recipe, installed at the provenance path) the same — 1210 / 605 / 11 before, the two new rows (`fixed_string_region_object_macro.sv` ACCEPT, `invalid_string_region_source_line.sv` REJECT — the pin runner refused a REJECT under class `control`, whose class IS the claim, so the negative control carries class `invalid` as the earlier keyword-region control does); the three RAW lanes (`stimuli/run_external_corpus.sh sv \| sv2023 \| sv2005`, release probe, into `PGEN_CORPUS_OUT_DIR`, verdict columns diffed): **all three IDENTICAL** — `sv_2017` 9 844 / 6 492 (16 336 rows), `sv_2023` 9 869 / 6 467, `verilog_2005` 2 313 / 293 (2 606 rows), 0 rows moved on any lane | ✅ |
| `2026-09-15` | `.5`(i) | `cargo test --lib sv_preprocessor` 74/74 (5 new, each a Verilator / sv2v golden); POPULATION-DIFF over every paste-carrying corpus file + the four `.5` references: `files=188 identical=187 moved=1` (sv2v `macro_macro.sv`); PASTE-ORDER-PROBE (Verilator 5.050 + Icarus 13.0, run): `rows=14 pgen_eq_verilator=8 pgen_eq_iverilog=8 verilator_eq_iverilog=9`, every PGEN≠Verilator row a routed `` `" ``-region / plain-string item; the `.1` census `73` unchanged; the whole lane run twice — run 2 (idle host) `files=16336 pass=12200 fail=3859 pp_fail=277 timeout=0`, exactly ONE row moved (`fail → pass`), run 1's single 60 s timeout a load artefact on a byte-identical 1.3 MB expansion; re-adjudication `corroborated 1223 → 1224`; pipeline tuple `9135/2606/4595/3314/5 → 9136/2606/4594/3313/5`, raw unmoved; `PIPELINE-LANE-CURRENCY` OK; `check_sv_corpus_denominator.sh` OK; pins `checked=1210 … preprocess_rows=11 failures=0` on BOTH probes; the three RAW lanes (`stimuli/run_external_corpus.sh sv \| sv2023 \| sv2005`, the rebuilt release probe installed at the provenance path `rust/target/release/parseability_probe`, into `PGEN_CORPUS_OUT_DIR` and diffed on the verdict columns): **all three IDENTICAL** — `sv_2017` 9 844 / 6 492 (16 336 rows), `sv_2023` 9 869 / 6 467, `verilog_2005` 2 313 / 293 (2 606 rows), 0 rows moved on any lane — byte-inert by construction (`--parse` without `--preprocess` never enters the expansion) and measured anyway; `make -C rust SHELL=/opt/homebrew/bin/bash clippy_on_rust_change` on the FORMATTED bytes → `✅ clippy_on_rust_change completed.` (strict source lint, the generated-parser stage, `GENERATED-CLIPPY-CORRECTNESS: ✅ POLICY-ONLY PASS — 68 pinned lints, all still in clippy::correctness`; guard exit 0, peak 7 263 MB); `cargo test --lib sv_preprocessor` re-run on the same bytes: 74 passed / 0 failed | ✅ |
| `2026-09-14` | `.8` | `cargo test --lib sv_preprocessor` 69/69 (13 new) + `--lib sv_keyword_versions` 6/6; `python3 stimuli/sv/check_sv_keyword_versions.py --self-test` PASS (6 arms) and gate mode OK (`22-1=102 22-2=21 22-3=1 22-4=97 22-5=23 22-6=4 noconfig=10`, Σ == Annex B Table B.1 = 248); `bash scripts/check_doctrines.sh` ALL **41** PASS; `stimuli/run_pipeline_lane.sh all` `files=16336 pass=12202 fail=3857 pp_fail=277 timeout=0 crash=0` (before `12136/3923/277/0`) — 68 movers, **0 of them outside the 113-file §22.14 census**, 67 `fail → pass` / 1 `pass → fail`; `check_sv_corpus_denominator.sh` OK, pipeline tuple `9135/2606/4595/3312/8` (bar `9 → 8`); `run_adjudication_repros.py` `checked=1191 armed=231 listed=598 preprocess_rows=9 failures=0`; the three RAW lanes re-run into `PGEN_CORPUS_OUT_DIR` and diffed — all three IDENTICAL (`sv_2017` 9 847 / 6 489, `sv_2023` 9 872 / 6 464, `verilog_2005` 2 313 / 293); `clippy_on_rust_change` ✅ | ✅ |
## Commit log

| Leaf | Commit subject or reference | Notes |
| --- | --- | --- |
| (tree creation) | `PGEN-SVPP-EXPANSION-0001` | tree + decision record created; registered in `docs/TASK_TREE.md` Proposed Task Trees; sequenced after the locked program |
| `.1` | `PGEN-SVPP-EXPANSION-0002` | tree ACTIVATED on the director's 2026-09-09 steer; `.1` scoping re-measured (docs only); leaves re-planned `.2`–`.7`; decision record [[project_preprocess_parse_pipeline_is_the_sv_deliverable]] |
| `.5`(g)(h)(g′) | `PGEN-SVPP-EXPANSION-0008` | a missing / extra actual and a recursive macro refused at the preprocess stage (§22.5.1 `:224-226`, `:342-343`); the apostrophe-as-quote misreading of a formal list fixed — a SILENT mis-expansion the refusal exposed; ENGINE tier, `sv_preprocessor.rs` only |
| `.5`(k) | `PGEN-SVPP-EXPANSION-0027` | a plain `"…"` string in macro text is OPAQUE by default (§22.5.1 `:370-372`, its own `Hello, x`; Verilator's reading; 66 mangled messages un-mangled) with the two measured tool readings the named policies `paste_delimited` (OpenTitan's marked-formal idiom) / `all` (Icarus's) on `--sv-plain-string-substitution`; the `.5`(i) escape leak (`\n"` never closed a string) found by the whole-corpus diff and fixed; ENGINE tier, `sv_preprocessor.rs` + the two binaries' flag |
| `.5`(j) | `PGEN-SVPP-EXPANSION-0026` | the `` `" `` string region: §22.5.1's three things by ONE renderer in the rescan for both macro paths (an embedded usage expanded, an object macro's region rendered at all, `` `\`" `` a quote) plus §5.9's one-line literal and (j′) `()` as one empty actual; four OpenTitan rows `fail → pass`; the whole corpus diffed; the source-line region and paste kept for the parser (the provenance leak on a source line's second pass closed); ENGINE tier, `sv_preprocessor.rs` only |
| `.5`(i) | `PGEN-SVPP-EXPANSION-0025` | the ORDER around a paste — a usage that is a paste operand is expanded first, an undefined usage joined by a paste is the WHOLE constructed name, looked up in place with its arguments unexpanded (§22.5.1 *"allowing identifiers to be constructed"*); measured against Verilator 5.050 and Icarus 13.0 (run, not quoted — they split; sv2v's reference and the clause decide); the whole paste-carrying population diffed before/after, exactly one expansion moved; (j) and (k) routed with their probe rows; ENGINE tier, `sv_preprocessor.rs` only |
| `.8` | `PGEN-SVPP-EXPANSION-0022` | IEEE 1800-2017 §22.14 applied by the expansion stage — a region's non-keywords rendered as ESCAPED identifiers (§5.6.2), the per-version tables DERIVED from three in-repo standards and gate-held by the new doctrine `SV-KEYWORD-VERSIONS` (registry 40 → 41); the plan's *"an unknown specifier is an error"* MEASURED to move 7 Verilog-AMS rows `fail → pp_fail` and withdrawn — the grammar is the refusal authority; ENGINE + INSTRUMENT + DOCTRINE tier, ZERO grammar / codegen / generated bytes |
| `.6a` | `PGEN-SVPP-EXPANSION-0009` | the re-adjudication instrument + tracked report/table; doctrine arm (3); `.12b`/`.13d` pointed at the numbers; INSTRUMENT + DOCTRINE tier |
| `.6b.1` | `PGEN-SVPP-EXPANSION-0010` | 34 verilator rows pinned `must_reject` (A.2.1.3 / A.2.2.1 / A.2.4, 2023 §6.18); the raw key demotes them by design, the pipeline cashes them (`must_reject` population of the re-adjudication instrument, read from the pin tables); ceiling 173 → 207 with the argument; live tuple 7529/2606/6201/3994/9 |
| `.6b.2` | `PGEN-SVPP-EXPANSION-0011` | four engine items: the paste token in an object macro, `` `line `` consumed (well-formed only), `//` stripped from macro text, `(` on the next line; 11 rows recognised, 0 regressions; ENGINE tier |
| `.6b.3` | `PGEN-SVPP-EXPANSION-0012` | the tracked disposition table + the instrument's disposition input; the tool-identity predefine measured (5 / 24) and declined; 119 of 183 refuted rows dispositioned (103 / 9 / 7), 64 await |
| `.6b.4` | `PGEN-SVPP-EXPANSION-0013` | the last 64 refuted rows dispositioned (two new classes: `cross_file_define`, `key_question`); the §40.3.1 predefined macros and three more engine rules named for `.6b.5`; 21 parser questions and 27 key questions routed |
| `.6b.5` | `PGEN-SVPP-EXPANSION-0014` | four engine rules (a `$ident` token, block comments across lines and after an include, the §40.3.1 predefined macros, the §34 envelope); 10 rows recognised; the UVM reference expansion found wrong inside its comments and superseded |
| `.6c.1` | `PGEN-SVPP-EXPANSION-0015` | the verdict-coverage tuple read under the pipeline (`pipeline_coverage.tsv`), published beside the raw one through the denominator gate; the bar 9 → 30 as defect candidates, 1 140 rows adjudicated |
| `.6c.2` | `PGEN-SVPP-EXPANSION-0016` | 28 key-question rows pinned `must_reject` (clause + reproducer/control each; two edition contrasts); the raw key demotes them, the pipeline cashes them; tuples `7501/2606/6229/4022/9` and `8698/2606/5032/3750/30` |
| `.4c.1` | `PGEN-SVPP-EXPANSION-0017` | the drivers' defines derived per file (56 rows; computed / tie reported; C++ spans skipped); the lane re-run: 5 movers; 3 pins (a vendor-extension branch ×2, a `.3.56` row); tuples `7498/2606/6232/4025/9` and `8701/2606/5029/3746/30` |
| `.4c.2` | `PGEN-SVPP-EXPANSION-0018` | the harness's unit predecessors derived per file (44 rows on 17 files; `-f` lists, globs, `--siloed`, conflicts reported); the lane re-run: 3 movers, all `fail → pass`; a `.6b.4` pairing claim corrected forward; pipeline tuple `8704/2606/5026/3743/30`; `.4c` closed |
| `.6d.1` | `PGEN-SVPP-EXPANSION-0019` | the library pass: 407 package files in dependency order, 166 packages published, every row with `--lib-in`; 425 movers, all `fail → pass`; 164/168 rejecting OpenTitan packages import `uvm_pkg` (the (d) gate); six per-top register packages disagree; pipeline tuple `9129/2606/4601/3318/30` |
| `.6d.2` | `PGEN-SVPP-EXPANSION-0020` | the unit-membership reading (5 074 member rows from the source map; `accepted_in_unit` / kept / `unreached_in_unit`); measured 0 / 89 / 1 923 — idle behind the UVM gate; the tuple unmoved |
| `.7` | `PGEN-SVPP-EXPANSION-0021` | contract + book lockstep: the SV contract's front-end section, the preprocessor contract's runtime section, the books' notes, the user guide; the HTML mirrors rebuilt |

## Changelog

- `2026-06-08` — tree created (`proposed`, own-now/build-later) per the director directive; decision record
  [[project_svpp_expansion_stage_for_nexsim]] written; sequenced strictly after the locked program.
- `2026-09-09` — ACTIVATED by the director's steer (`PGEN-SVPP-EXPANSION-0002`); `.1` SCOPING done on
  measurement (the expansion engine exists since 2026-02-27; 14/14 curated composition; 98.1 % of UVM +
  OpenTitan preprocesses alone; six missing pieces named; three engine defects named); leaves `.2`–`.7`.
- `2026-09-09` — `.4c.1` done (`PGEN-SVPP-EXPANSION-0017`): the drivers' defines derived per file and applied by the
  lane; 5 movers; 3 pins; frontier `.4c.2` (the units' file lists), `.6d`, `.7`.
- `2026-09-09` — `.4c.2` done (`PGEN-SVPP-EXPANSION-0018`): the harness's unit predecessors derived per file and applied
  by the lane; 3 movers, all `fail → pass`; a `.6b.4` pairing claim corrected forward; `.4c` closed; frontier `.6d`, `.7`.
- `2026-09-09` — `.6d.1` done (`PGEN-SVPP-EXPANSION-0019`): the library pass over the design suites' packages; 425 movers, all
  `fail → pass`; the UVM gate measured (164/168); frontier `.6d.2` (unit membership), `.7`, the (d) ruling.
- `2026-09-09` — `.6d.2` done (`PGEN-SVPP-EXPANSION-0020`): the unit-membership reading, measured idle behind the UVM gate
  (0 accepted, 1 923 unreached); `.6d.3` (the UVM library) BLOCKED on (d); frontier `.7`.
- `2026-09-15` — `.5`(k) done (`PGEN-SVPP-EXPANSION-0027`): a plain string in macro text is opaque by default, the two tool
  readings named policies; no verdict moved, 66 messages un-mangled; the tree's remaining leaf `.6d.3` (the UVM library) stays
  BLOCKED on the director's (d) — the tree is at its handoff-ready stop again.
- `2026-09-15` — `.5`(j) done (`PGEN-SVPP-EXPANSION-0026`): the `` `" `` string region rendered once, in the rescan, for both
  paths, one line, its embedded usages expanded; `()` one empty actual; four OpenTitan rows `fail → pass`, the pipeline tuple
  `9140/2606/4590/3310/5`; frontier `.5`(k) (the plain string, a census first), then `.6d.3` still BLOCKED on the director's (d).
- `2026-09-15` — `.5`(i) done (`PGEN-SVPP-EXPANSION-0025`): the order around a paste — the sv2v `macro_macro.sv` row
  `fail → pass`, the pipeline tuple `9136/2606/4594/3313/5`; two divergences closed (a usage as a paste operand, a
  defined usage as the left operand), two more found by the three-way probe and owned: `.5`(j) the `` `" `` region,
  `.5`(k) the plain string; frontier `.5`(j), `.5`(k), then `.6d.3` (the UVM library) still BLOCKED on the director's (d).
- `2026-09-09` — `.7` done (`PGEN-SVPP-EXPANSION-0021`): the front end contracted and stated in the books; the tree at its
  handoff-ready stop — `.6d.3` (the UVM library) BLOCKED on the director's (d); the tracked HTML mirrors rebuilt.

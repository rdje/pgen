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
| ⏸ | `.6d.5` | `parked` (opened 2026-10-02 by `.6d.4`'s census; parked the same day by `.6d.1`'s own rule) | OpenTitan's PER-IP build defines: 6 of the 38 remaining DV packages stop at an unexpanded `` `RNG_BUS_WIDTH `` / `` `EN_GCM `` / `` `NUM_END_POINTS `` / `` `SEC_VOLATILE_RAW_UNLOCK_EN `` / `` `INSTR_EXEC `` / `` `SRAM_WORD_ADDR_WIDTH ``, each a `+define+NAME=VALUE` in the IP's `dv/*_sim_cfg.hjson` `build_opts`. MEASURED: no IP has a base config carrying the symbol — every one is VARIANT-ONLY with contradicting values (`entropy_src_rng4bits` 4 / `rng16bits` 16; `edn_edn0` 8 / `edn1` 1; `sram_ctrl_main` `SRAM_WORD_ADDR_WIDTH=15` `INSTR_EXEC=1` / `ret` 10, 0 / `exec_64kB` 14, 1; `aes_base_sim_cfg.hjson` carries `EN_GCM=0` at `:69,75` AND `EN_GCM=1` at `:81,87`; `lc_ctrl_*volatile_unlock_{enabled,disabled}` 1 / 0), and the value shapes the parse (`` `EN_GCM ? 10 : 0 ``, `` `NUM_END_POINTS] ``). `.6d.1`'s rule binds: a disagreement between variants is reported, not resolved by guessing which build the row meant — so NO row is written; the 6 packages stay reported as rejecting at a per-IP macro. What would unpark it: a director call for a per-variant reading of those rows (a lane dimension: variant × file), or a derivation of the IP's own variant list from its `*_sim_cfg.hjson` (`.1` declined reading build systems; this would reopen that measurement). Parked, not blocking: the 6 packages carry 846 − (the `.6d.6` conversions) members behind them, counted in `unreached_in_unit`. ➕ 2026-10-02 (`.5`(l)): a NINTH package joins — `core_ibex_test_pkg.sv`, which now stops at `` 32'h`BOOT_ADDR `` (`core_ibex_base_test.sv:142:34`, `furthest_position=77421`), ibex's own per-configuration `+define+BOOT_ADDR=…`. ➕ 2026-10-02 (`SV-CORPUS-GRAD.38`): a TENTH under the `sv_2023` reading only — `spi_device_env_pkg.sv` clears its 2023 parameter array there and stops at `` `SRAM_TYPE `` (`spi_device_ram_cfg_vseq.sv:13:26`). ➕ 2026-10-02 (`SV-CORPUS-GRAD.41`): an ELEVENTH, of a different kind — `riscv_instr_test_pkg.sv` stops at an unexpanded `` `SAMPLE `` whose definition lives in `riscv_instr_pkg.sv`'s own include, and that package is `pp_fail` on `riscv_core_setting.sv`, shipped once per riscv-dv TARGET (12 `target/*/` copies): a variant-only include, not a define |
| — | `.6d.6` | ✅ `done` (of its 25 remaining packages, `SV-CORPUS-GRAD.39` / release `1.0.248` admits the `local::` method-receiver shape behind 3 — one passes, `i2c_agent_pkg`; the other two stop at `.5`(l)) | `PGEN-SVPP-EXPANSION-0031`: OpenTitan's UNIT PREDECESSORS — the three macro headers its dvsim flow compiles before every DV file (`uvm_macros.svh`, `dv_macros.svh`, `prim_assert.sv`) as hand-curated suite rows (`HAND_UNITS`, `unit_file` rows in `pipeline_context.tsv`, applied as `--unit-file` in order); 15 of the 38 packages still rejecting after `.6d.4` had stopped at a raw `` `uvm_object_utils `` / `` `DV_CHECK_* `` / `` `ASSERT_I `` the package never includes itself; the lane re-run: `PIPELINE-MOVERS … moved=195` (all OpenTitan `fail → pass`, zero the other way), the library 264 → 273 packages (OpenTitan 355 → 368 of 396), members `accepted_in_unit` 999 → 1 287 / `unreached_in_unit` 846 → 566, the tuple under the pipeline `11053/2606/2677/1391/0`; 25 packages remain: 8 for `SV-CORPUS-GRAD.38`, 8 behind `.6d.5`, 9 grammar constructs for the SV lane |
| — | `.5`(l) | ✅ `done` | `PGEN-SVPP-EXPANSION-0032`: a `//` LINE COMMENT INSIDE A MACRO ACTUAL is not substituted (§22.5.1's *"shall not become part of the substituted text"*, read of the text that gets substituted) — `strip_line_comments_from_actual` where a function macro's actuals are bound, the scanner's own state machine, the provenance mask dropped byte for byte; RED-first unit test; the lane re-run `PIPELINE-MOVERS … moved=4` (all OpenTitan `fail → pass`: `spi_agent_pkg.sv`, `alert_esc_agent_pkg.sv`, both tops' `otp_ctrl.sv`), zero the other way, the tuple under the pipeline `11115/2606/2615/1329/0`; `core_ibex_test_pkg` passes its comment and stops at `` 32'h`BOOT_ADDR `` (→ `.6d.5`) |
| — | `.6d.7` | ✅ `done` | `PGEN-SVPP-EXPANSION-0033`: OpenTitan's FOURTH suite-wide unit predecessor, `hw/dv/sv/cip_lib/cip_macros.svh` (sourced to `cip_base_pkg.sv:24`). Both readings re-run: `PIPELINE-MOVERS[sv_2017] … moved=38` and `PIPELINE-MOVERS[sv_2023] … moved=38`, the same 38 rows, all OpenTitan `fail → pass` (37 `tb.sv` + `csrng_env_pkg.sv`), zero the other way; `opentitan=372/396` (2023: `375/396`); the tuple under the pipeline `11168/2606/2562/1276/0`; the census `2023_only_pass=40 2017_only_pass=0` unchanged |
| — | `.5`(m) | ✅ `done` | `PGEN-SVPP-EXPANSION-0034`: `` `include `` FOLLOWED BY A MACRO is expanded first and its result read as the filename (IEEE 1800-2017 §22.5.1: *"The `` `include `` directive can be followed by a macro, instead of a string literal"*) — `.5`(c)'s refusal RETRACTED. RED-first unit tests; both readings re-run: `pp_fail` 277 → 271 under each, 3 rows `pp_fail → pass` (the standard's own sv-tests case and two iverilog files), 3 `pp_fail → fail` at their next barrier, 2 refusals renamed to their real cause; zero `pass → fail`; the tuple under the pipeline `11168/2606/2562/1275/0` (adjudicated unmoved, dark −1) |
| — | `.5`(n) | ✅ `done` | `PGEN-SVPP-EXPANSION-0035`: a `` `line `` directive whose PARAMETERS are macro usages is expanded first and then CONSUMED (IEEE 1800-2023 §22.2: *"Macro expansion shall occur within a compiler directive"*; 1800-2017 §22.5.1: a defined macro *"can be used anywhere in the compilation unit where it is defined"*). One row `fail → pass` under EACH of the three readings (sv2v `test/lex/line.sv`), none the other way; a malformed result still passes through to the parser's own §22.12 production. Section below |
| — | `.5`(o) | ✅ `done` | `PGEN-SVPP-EXPANSION-0035`: a COMMENT between macro formals stays REFUSED, by the clause (§22.5.1, both editions: formals are *"separated by commas and optionally white space"*; §5.3's white space is spaces, tabs, newlines, formfeeds; the standard writes *"white space or a comment"* where it admits one) — the verdict is unchanged, the refusal now NAMES the clause. Three corpus files write it, all tool test suites (verilator `t_preproc.v:277`, iverilog `macro_with_args.v:17`, sv2v `macro_arg_comment.sv:1`); no design file does, so no opt-in tolerance is offered. Section below |
| — | `.5`(p) | ✅ `done` | `PGEN-SVPP-EXPANSION-0036`: THREE readings selected by the profile's EDITION, where IEEE 1800-2023's clause 22 says more than 1800-2017's — under `sv_2023` / `sv_2023_tolerant` a block comment in macro text is not substituted, a block comment carries a `` `define `` across its newlines, and a macro usage as the argument of `` `ifdef `` / `` `ifndef `` / `` `elsif `` / `` `undef `` is expanded first (§22.5.1, §22.2); under the 2017 reading the comment is substituted, a `` `define `` whose text ends inside a block comment is REFUSED naming the clause (before, its tail leaked into the source), and the argument is read as written. `moved=0` under each of the three readings, as predicted in writing before the chain; four edition-twin pins. Section below |
| — | `.6e` | ✅ `done` | `PGEN-SVPP-EXPANSION-0039`: the 310 rows the re-adjudication files as `rejects_own_file`, re-expanded under the lane's own context and classed by what the parser SAW — a derived, tracked census (`docs/tasks/artifacts/sv_corpus_grad/rejects_own_file_census/`, arm (4) of `PIPELINE-LANE-CURRENCY`): 47 are undefined build macros that reached the parser (context, not syntax: the parked `.6d.5`'s OpenTitan defines with their row count; VeeR's `common_defines.vh` → `.6d.8`), 154 the cross-file declaration barrier, 55 include fragments, 12 the 2023 text's or the tolerant profile's, 16 non-SystemVerilog tests, 26 ruled one by one (five ecosystem spellings no edition derives → `SV-CORPUS-GRAD.46`, parked; one preprocessor question → `.5`(r)). No verdict moved. Section below |
| — | `.6d.8` | ✅ `done` | `PGEN-SVPP-EXPANSION-0040`: VeeR's flow predecessors as the suite's hand-curated unit files (the `.6d.6` pattern): `tools/Makefile:167-169` `defines = $(BUILD_DIR)/common_defines.vh` + `el2_def.sv` + `el2_pdef.vh`, `:210` passes the first as the VCS build's first source; no VeeR file `include`s it by name, so every `RV_*` / `TEC_RV_ICG` use was a raw backtick at the parser. Three rows `fail → pass` under each of the three readings (`beh_lib.sv`, `axi_lsu_dma_bridge.sv`, `tb_top.sv`), the three named in writing first and the 82 passing rows pre-measured unmoved; the tuple under the pipeline `11176/2606/2554/1267/0`; the rejects-own-file census 310 → 307 rows, `raw_backtick` 47 → 44. Section below |
| — | `.6c.3` | ✅ `done` | `PGEN-SVPP-EXPANSION-0042`: the 94 sv-tests `:type: preprocessing` rows the raw key deferred to the preprocessor lane BY DESIGN are judged by their own header through the key's own function in its pipeline mode (`should_fail_because` → must_reject, absent → must_accept): 86 `svpp_owned:corroborated`, 4 refused as expected (`must_reject:refused_engine`, now ADJUDICATED), 2 rejected as expected, and TWO the pipeline ACCEPTS against a should-fail header — the standard's own split-string example and `` `define define `` — on the bar under the pipeline (tuple `11270/2606/2460/1261/2`) until `.5`(s)/(t) land. Section below |
| ⏳ | `.5`(s) | `open` (opened 2026-10-06 by `.6c.3`) | ENGINE: macro text split across a STRING LITERAL is accepted and glued at the use site — `` `define first_half "start of string `` / `` $display(`first_half end of string"); `` expands to one string. 1800-2017 §22.5.1 `:275-281` "shall not be split across … String literals", `:295-297` the standard's own illegal example (2023 `:307, :320-322`; 1364-2005 §19.3.1 `:172, :192, :214-215`). To do: the refusal naming the clause (an unterminated string literal in macro text), a corpus census of the shape first, RED-first tests, pins, the chain |
| ⏳ | `.5`(t) | `open` (opened 2026-10-06 by `.6c.3`) | ENGINE: a compiler directive redefined as a macro name is accepted — `` `define define "illegal" ``, `` `define include 1 `` both pass. 1800-2017 §22.5.1 `:139-140` "All compiler directives shall be considered predefined macro names; it shall be illegal to redefine a compiler directive as a macro name" (2023 `:152-153`; 1364-2005 §19.3.1 `:100-101`, `:229`). To do: the directive-name set from §22's own list (both editions; `__FILE__` / `__LINE__` included), the refusal, a corpus census, RED-first tests, pins, the chain |
| — | `.5`(r) | ✅ `done` (a RULING) | `PGEN-SVPP-EXPANSION-0041`: the comment-backslash continuation 1800-2023 §22.5.1 names outright (`docs/systemverilog/2023/md/section-22-compiler-directives.md:200-201`) already reads as licensed on both profiles (measured before any change); the Surelog `TestMacros/macros.v:5` line writes a TAB and spaces AFTER its backslash, which no edition licenses (2017 `docs/systemverilog/2017/md/section-22-compiler-directives.md:171-172` "preceded by a backslash", 2023 `docs/systemverilog/2023/md/section-22-compiler-directives.md:173-175` "immediately preceded", 1364-2005 `docs/verilog/2005/md/section-19-compiler-directives.md:107-108`); the corpus writes that shape at two sites, both tests (verilator's own `t_lint_bsspace_bad.v` is its lint-error test for it). Two pins, both SV profiles; no stage option; no row moved. Section below |
| — | `.4d` | ✅ `done` | `PGEN-SVPP-EXPANSION-0038`: the lane's refused-context rows grouped by what the context fails to supply — 168 black-parrot rows name BaseJump STL (not vendored; honestly absent), the rest top-specific files, build outputs and error tests, AND 25 rows name a file the tree holds: the derivation's include-candidate population was the SOURCE suffixes only (a `.vi` / `.vams` header held once was "absent"), and a tie of byte-identical copies was "ambiguous". Both fixed in `stimuli/sv/pipeline_context.py`; the fix's first cut was REFUSED by the shadow guard written beside it (a suite directory chosen for one name held two ambiguous ones) and redesigned per file. Five rows left `pp_fail` under each of the three readings — four to `pass` (two HardFloat `specialize.v`, iverilog `test_va_math.v` / `test_vams_math.v`), one to `fail` (the friscv testbench) — the five named in writing before the chain, none the other way; the 20 HardFloat `source` rows now stop at the honest three-way tie behind the resolved header. Section below |
| — | `.5`(q) | ✅ `done` | `PGEN-SVPP-EXPANSION-0037`: the lane's 47 `refused_engine` rows grouped by reason — twelve families, ten right (error tests by name, the standard's own "illegal" example lines, `should_fail` conformance tests) and TWO defects, both LRM-licensed: a GUARDED SELF-INCLUDE refused as an "include cycle" where §22.4 bounds include files by NESTING depth (the depth limit alone now refuses a true loop, naming the clause and the chain); an ESCAPED identifier refused as a macro name where Syntax 22-2's `text_macro_identifier` is an `identifier`, simple or escaped (one name-token helper at every reader of a name; the usage scanner admits the escaped spelling for a defined name). Three keyed rows `pp_fail → pass` under each of the three readings — sv2v `file_line.sv`, `include_self.sv`, iverilog `escaped_macro_name.v` — none the other way, as predicted in writing before the chain; the re-adjudication's landing classifier learned the new refusal's spelling. Section below |
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
- ⛔ **RETRACTED 2026-10-02 by `.5`(m) — the ruling below read Syntax 22-1 and not §22.5.1, which licenses the form in a sentence; kept as written, because it is what shipped for 23 days.**
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

## `.6d.6` the UNIT PREDECESSORS — ✅ done 2026-10-02 (`PGEN-SVPP-EXPANSION-0031`): the macro headers OpenTitan's flow compiles first, and 195 rows open

**What landed** (LANE + CONTEXT tier — `stimuli/sv/pipeline_context.py` (`HAND_UNITS`, a suite-level row kind `unit_file`, emitted by `rows_for`), `stimuli/sv/pipeline_context_args.sh` (suite `unit_file` rows → `--unit-file`, after the file's own `.4c.2` rows), the tracked `stimuli/sv/pipeline_context.tsv` (three rows), the lane's derived reports; ZERO Rust / grammar / answer-key / codegen bytes):

- **How it was found.** `.6d.4`'s census of the 38 packages still rejecting read ~15 origins at the END of a line whose next line is a macro call; the preprocessed text at `furthest_position` (`rust/target/sv_next/pp/*.pp.sv`, every one of the 38 re-preprocessed with its lane context) showed the call RAW — `` `uvm_object_utils(aes_nist_vectors) ``, `` `uvm_object_param_utils(scoreboard_queue#(SEQ_ITEM)) ``, `` `uvm_object_utils_begin(tl_agent_env_cfg) ``, `` `uvm_component_utils(xbar_base_test) ``, `` `DV_CHECK_FATAL(req.randomize() … ``, `` `DV_MUBI8_DIST(… ``, `` `gfn ``, `` `ASSERT_I(NumInstrMatch_A, … ``, `` `uvm_fatal("lc_env_pkg", … `` — in packages that `include` no macro header at all (`nist_vectors_pkg.sv`, `scoreboard_pkg.sv`, `xbar_test_pkg.sv`, `tl_agent_env_pkg.sv`: 0 includes of `uvm_macros` / `dv_macros` / `cip_macros`; the passing `otbn_env_pkg.sv` includes `uvm_macros.svh` itself). OpenTitan's dvsim builds ONE compilation unit: `{UVM_HOME}/src/uvm_pkg.sv` first (`hw/dv/tools/dvsim/dsim.hjson:14-15`; it includes `uvm_macros.svh`), `dv_utils_pkg.sv` (`dv_macros.svh`, `:11`) and the RTL (`prim_assert.sv`, guarded `PRIM_ASSERT_SV`) before any env or test package — §22.5.1 makes the macros unit-wide from there.
- **Measured first, by hand flags.** `parseability_probe … --preprocess <lane context> --unit-file uvm_macros.svh [--unit-file dv_macros.svh --unit-file prim_assert.sv] --lib-in …`: `nist_vectors_pkg.sv`, `xbar_test_pkg.sv`, `tl_agent_env_pkg.sv`, `pwrmgr_env_pkg.sv` (earlgrey) → `parse_full passed … [preprocessed]`; `scoreboard_pkg.sv` with `uvm_macros.svh` alone still rejected (`furthest_position=11071`) and passed with all three. Cost on `nist_vectors_pkg.sv`: `unit_predecessors=0` 0.127 s → `unit_predecessors=3` 0.492 s (+0.37 s per row).
- **The rows.** `HAND_UNITS["opentitan"]` = the three headers, each with its source (`dsim.hjson:14-15`; `dv_utils_pkg.sv:11`; `prim_assert.sv:9`); `rows_for` emits `(suite, "unit_file", path, "hand", source)` into the suite context; `context_args` prints `--unit-file` for them after the file's own `.4c.2` predecessors. `python3 stimuli/sv/pipeline_context.py --write` → `rows=78` (was 75); `--check` → `PIPELINE-CONTEXT: OK`. Through the emitter alone (no hand flags): the five packages above pass; `aes_env_pkg.sv` still rejects (`` `EN_GCM `` — `.6d.5`).
- **The whole lane re-run** (`wall_s=972`; the library pass 01:43 → ~02:15, slower than before because 13 more packages export and `ibex_icache_test_pkg` alone writes 35 MB of facts): `PIPELINE-LIBRARY: Cores-VeeR-EL2=2/2 black-parrot=3/7 opentitan=368/396 uvm-core=2/2 published_packages=273 unresolved_imports=44 cyclic_files=0 member_rows=2392` (was `opentitan=355/396 … published_packages=264 … member_rows=5130`); `PIPELINE-LANE: files=16336 pass=12696 fail=3363 pp_fail=277 timeout=0 crash=0` (was `pass=12501 fail=3558`); `PIPELINE-MOVERS rows_before=16336 rows_after=16336 moved=195 added=0 lost=0`, by suite and direction `{('opentitan', 'fail->pass'): 195}` — zero `pass → fail`, zero outside OpenTitan; `LIBRARY-MOVERS {('fail', 'pass'): 13}`, 25 OpenTitan package files still `fail`. `PIPELINE-READJUDICATION: … chained:accepted_in_unit=1429 chained:accepted_under_expansion=2356 chained:hollow=29 … chained:rejects_own_file=321 chained:unreached_in_unit=566` (was 1141 / 2161 / 39 / 514 / 846); OpenTitan members re-read 1 890 → 1 887, `accepted_in_unit` 999 → 1 287, kept 27 → 17, `unreached_in_unit` 846 → 566. The tuple under the pipeline `10570/2606/3160/1874/0` → `11053/2606/2677/1391/0` (483 rows adjudicated; the raw tuple `7485/2606/6245/3864/0` unmoved), the platform book's anchor moved with it. `IDENTITY-CARRIERS: OK (live=14 frozen=7 rows_equal=66 notes=7 not_evaluated=0)`, `corpus-key-integrity: OK`, the three `PIPELINE-LANE-CURRENCY` rows OK.
- **The membership table, explained.** `member_rows` 5 130 → 2 392 while the DISTINCT member files read 2 060 → 2 058: a package that `include`s `uvm_macros.svh` (158 package files do) or `dv_macros.svh` now meets the header's own guard (`UVM_MACROS_SVH`, `` `ifndef gfn ``, `PRIM_ASSERT_SV`) already satisfied by the predecessor, the include expands to nothing, and the header and its 16 `macros/*.svh` sub-includes leave that package's source map — they were never class members. The three lane rows that left the unit reading (`aes_nist_vectors_gcm_vseq.sv`, `aes_nist_vectors_vseq.sv`: `unreached_in_unit` → `accepted_under_expansion`, they now parse on their own; `riscv_instr_cov_test.sv`: → `rejects_own_file`, at `` `SAMPLE `` — a `riscv-dv` macro, its own header) and the 11 that gained one are listed by the diff of the two re-adjudication tables (`rust/target/svpp_6d6/`).
- **What remains, measured (25 packages).** 8 at a construct 1800-2023 admits (`parameter string LIST_OF_LEAFS[] = {` ×3, `LIST_OF_READ_SEED_FORCE_PATHS[] = {` ×3, `parameter bit[11:0] ALL_TPM_HW_REG_OFFSETS[] = {`, `->m_adc_push_pull_ev[channel];`) → `SV-CORPUS-GRAD.38`; 8 at a per-IP define (`` `RNG_BUS_WIDTH `` ×2, `` `EN_GCM `` ×2, `` `NUM_END_POINTS ``, `` `SEC_VOLATILE_RAW_UNLOCK_EN ``, `` `INSTR_EXEC ``, `` `SRAM_WORD_ADDR_WIDTH ``) → `.6d.5` (parked: variant-only values); 9 at a grammar construct past every macro: `randomize() with { … == local::x … }` through `` `DV_CHECK_RANDOMIZE_WITH_FATAL `` (×3: `alert_receiver_base_seq.sv:29`, `i2c_base_seq.sv:180`, `spi_host_seq.sv:22` — the parse stops before the `()` of `local::data_q.size()`), the same macro with a `// with {` line comment inside its stringized argument (`core_ibex_seq_lib.sv:112`), `pure virtual task automatic inject_fault();` (`sec_cm_base_if_proxy.sv:21`) and `extern function automatic logic [DataWidth-1:0] get_written_data();` (`tl_seq_item.sv:190`) — a lifetime in a method PROTOTYPE, which A.2.6 `function_prototype` / `task_prototype` do not spell, `` `DV_MUBI8_DIST `` inside a constraint block (`csrng_env_cfg.sv:105`), `cross cp_key0_in, cfg.vif.key0_in;` (`sysrst_ctrl_in_out_inverted_vseq.sv:49`), and `` `SAMPLE `` (`riscv_instr_cov_test.sv:128`, riscv-dv's own header, not in this unit). The SV lane's, each with its line.
- **Not done, stated.** No per-file unit rows, no fourth header (`cip_macros.svh` is included by `cip_base_pkg.sv` itself and expands there); RTL rows carry the three headers too and changed nothing (`moved=195`, all DV); the predecessors' own text is discarded by the stage, so no RTL row gains or loses a declaration.

**Acceptance checklist (`TASK-ACCEPTANCE`):**

  - [x] **REPRODUCE / ISSUE** — `rust/target/sv_next/pp/*.pp.sv`: 15 of the 38 rejecting packages preprocessed with their lane context stop at a RAW macro call (`` `uvm_object_utils(aes_nist_vectors) `` at `nist_vectors_pkg.sv:55`, `` `uvm_component_utils(xbar_base_test) `` at `xbar_base_test.sv:5`, `` `ASSERT_I(…) `` at `pwrmgr_env_cfg.sv:37`, …); `parseability_probe … --preprocess <context>` on `nist_vectors_pkg.sv` → rejected (`included_files=1`); the packages include no macro header (`grep -c '`include "(uvm_macros|dv_macros|cip_macros)'` → 0 on four of five).
  - [x] **ROOT CAUSE (WHY + WHERE)** — WHERE: `stimuli/sv/pipeline_context.py` (no suite-level unit predecessor existed; `.4c.2`'s unit rows are per file, derived from verilator / sv2v drivers) and `pipeline_context_args.sh`. WHY: OpenTitan's dvsim compiles ONE unit beginning with `uvm_pkg.sv` (`dsim.hjson:14-15`) and `dv_utils_pkg.sv`, so its DV packages rely on §22.5.1's unit-wide macros and include nothing; parsed one file at a time they meet the raw backtick — the signature is the probe's reading at the macro call (`scoreboard_pkg.sv` with `uvm_macros.svh` alone: `Parser did not consume full input at position 171 [furthest_position=11071, …]`; `nist_vectors_pkg.sv` bare: `included_files=1`, rejected) and `parse_full passed` the moment the three headers precede the file (`--unit-file`, by hand first, then through the emitter).
  - [x] **FIX** — CONTEXT tier (data, sourced): three hand-curated suite rows and the one row kind that carries them; no engine tier (`--unit-file` existed since `.2`), no grammar tier.
  - [x] **ADDRESSED (verified)** — the five probed packages pass through the emitter alone; the lane: `PIPELINE-MOVERS … moved=195`, all OpenTitan `fail → pass`; `PIPELINE-LIBRARY … opentitan=368/396 … published_packages=273`; members `accepted_in_unit` 999 → 1 287, `unreached_in_unit` 846 → 566; the tuple under the pipeline `11053/2606/2677/1391/0`; `pipeline_context.py --check` OK; `PIPELINE-LANE-CURRENCY` (1) CONTEXT OK.
  - [x] **NO REGRESSION** — `moved=195 added=0 lost=0`, zero `pass → fail`, zero movers outside OpenTitan (the headers precede every OpenTitan row, RTL included, and changed only DV packages and their members); the raw lanes untouched by construction; the raw tuple `7485/2606/6245/3864/0` unmoved; `IDENTITY-CARRIERS: OK`, `corpus-key-integrity: OK`; the membership table names the same files (2 060 → 2 058 distinct) and the three rows that left it are accounted for above; `bash -n` on the emitter; ZERO grammar / generated bytes — every generated artifact byte-identical (`GENERATED-REPRODUCIBILITY` tier 1), no Rust touched (clippy has nothing to read); `bash scripts/check_doctrines.sh` → `doctrines: ALL 45 enforced doctrines PASS.`.
  - [x] **LOCKSTEP** — this section + the row, the SV book's pipeline chapter (the context paragraph's unit sentence, the `.6d.6` paragraph beside `.6d.4`'s, the `.6d.3` paragraph's tail) + tracked HTML, the platform book's verdict-coverage anchor, the derived lane reports, the knowledge card `a-raw-backtick-at-the-furthest-position-is-a-context-defect` (new; the map regenerated), `docs/TASK_TREE.md`, `MEMORY.md`, `CHANGES.md`, `DEVELOPMENT_NOTES.md`.
  - promotion: the knowledge card above.

## `.5`(l) the LINE COMMENT in an ACTUAL — ✅ done 2026-10-02 (`PGEN-SVPP-EXPANSION-0032`): a `//` comment written inside a macro actual argument is not part of the substituted text

**What landed** (ENGINE tier, the SystemVerilog family's own component — `rust/src/families/systemverilog/preprocessor.rs`: `strip_line_comments_from_actual` and its one call where a function macro's actuals are bound; one unit test RED-first; the lane's derived reports; ZERO grammar / codegen / generated bytes):

- **How it was found.** Release `1.0.248` admitted the `local::` method receiver and ONE of the three OpenTitan packages behind it passed; `spi_agent_pkg.sv` and `alert_esc_agent_pkg.sv` still stopped — `furthest_position=729798` (origin `spi_host_dummy_seq.sv:14:21`) and `321648` (`alert_receiver_base_seq.sv:29:19`) — at a RAW `` `DV_CHECK_FATAL(req.randomize() with { … `` in their preprocessed text, with `diagnostics=0`. Both calls carry a `//` comment inside the macro's actual argument (`// TODO(#15721), dummy CSB without clock affects flash mode`; `// This agent do not support alert_receiver int_err`); the sibling `i2c_base_seq.sv:180` carries none and its package had passed. The one-call macro expands (`` `DV_CHECK_FATAL(a == b) `` through the real header, three ways), a default-argument macro with a brace-nested comma expands — the comment is the only difference.
- **The repro** (`rust/target/sv_next/mac2.sv`, `ast_pipeline --preprocess systemverilog`): `` `M(a == b // a comment, with a comma<LF>) `` → `if (!(a == b // a comment, with a comma)) $display("");` — the comma inside the comment correctly did NOT split the argument, and the comment's text was substituted onto the expansion's single line, commenting out `)) $display("")`. `preprocess summary: … diagnostics=0 warnings=0 errors=0`: silent.
- **WHY + WHERE.** `parse_macro_invocation_arg_ranges` tracks `in_line_comment` so that a delimiter inside a `//` comment does not delimit — and returns the actual as a byte RANGE of the caller's text, comment included. `expand_macros_in_text` binds `text[s..e]` to the formal (through `expand_function_macro`) and `substitute_function_macro_body` emits it; the expansion is one line, so everything after the `//` on that line is gone to the parser. IEEE 1800-2017 §22.5.1 (`docs/systemverilog/2017/txt/section-22-compiler-directives.txt:151-152`): "If a one-line comment … is included in the text, then the comment shall not become part of the substituted text" — written of the macro text, and an actual is text that is substituted; `.5`'s `strip_line_comments_from_macro_text` already applied it to definitions (sv2v `macro.sv`).
- **FIX.** `strip_line_comments_from_actual(actual, mask)`: the scanner's own state machine (string literals with escapes, block comments) run over the bound actual; a `//` outside a string and outside a block comment is dropped up to the line feed, which stays; the provenance mask (`.5`(h)) is dropped byte for byte with it. Block comments stay (white space to the parser), a `//` inside a string literal is the string's. One call, where the actuals and their masks are built from the ranges.
- **Measured, unit level.** `a_one_line_comment_in_a_macro_actual_is_not_substituted` RED on `16cb8e3a7`: `panicked at …/preprocessor.rs:6032:9: a line comment inside an actual must not reach the expansion: if (!(a == b // a comment, with a comma)) $display("");`, `test result: FAILED. 0 passed; 1 failed`; GREEN on the fix, with its four sibling assertions (an actual that BEGINS with a comment keeps what follows it; a block comment stays; a `//` inside a string stays). The full lib suite `cargo test --features "generated_parsers ebnf_dual_run" --lib` → `test result: ok. 1303 passed; 0 failed; 28 ignored`; `clippy_on_rust_change` rc 0.
- **Measured, the three packages** (`parseability_probe … --preprocess <lane context> --lib-in …`, the rebuilt debug probe): `spi_agent_pkg.sv` and `alert_esc_agent_pkg.sv` → `parse_full passed … [preprocessed]`; `core_ibex_test_pkg.sv` → past the comment, `furthest_position=77421`, origin `core_ibex_base_test.sv:142:34`, the text `32'⟂h`BOOT_ADDR` — a per-configuration build define, joined to `.6d.5`.
- **Measured, the pins and the whole lane** (the release probe rebuilt by the recipe and published first — parser fingerprint unchanged `9cc363beab63c228`, the binary new): `ADJUDICATION-REPROS: checked=1820 armed=225 listed=891 multi_profile_rows=694 preprocess_rows=13 failures=0`; `PIPELINE-LIBRARY: Cores-VeeR-EL2=2/2 black-parrot=3/7 opentitan=371/396 uvm-core=2/2 published_packages=276 unresolved_imports=44 cyclic_files=0 member_rows=2392` (was `opentitan=369/396 … published_packages=274`); `PIPELINE-LANE: files=16336 pass=12701 fail=3358 pp_fail=277 timeout=0 crash=0` (was `pass=12697 fail=3362`); `PIPELINE-MOVERS rows_before=16336 rows_after=16336 moved=4 added=0 lost=0`, by suite and direction `{('opentitan', 'fail->pass'): 4}` — the two packages and `hw/top_{darjeeling,earlgrey}/ip_autogen/otp_ctrl/rtl/otp_ctrl.sv` (RTL: the same comment-in-an-actual shape in an assertion macro), zero `pass → fail`, zero outside OpenTitan (`uvm_pkg.sv`, `uvm.sv` and every other preprocessed row unchanged under the new binary); `LIBRARY-MOVERS {('fail', 'pass'): 2}`, 22 OpenTitan package files still `fail`; `PIPELINE-READJUDICATION: … chained:accepted_in_unit=1486 … chained:rejects_own_file=316 chained:unreached_in_unit=512` (was 1444 / 320 / 552) — OpenTitan members `accepted_in_unit` 1 302 → 1 344, kept 16 → 14, `unreached_in_unit` 552 → 512; the tuple under the pipeline `11069/2606/2661/1375/0` → `11115/2606/2615/1329/0` (46 rows adjudicated; the raw tuple `7485/2606/6245/3864/0` unmoved), the platform book's anchor moved with it; `IDENTITY-CARRIERS: OK`, `corpus-key-integrity: OK`, the three `PIPELINE-LANE-CURRENCY` rows OK.
- **Bound, stated.** The SV formal umbrella's external-corpus triage was not re-run for this slice (76 min; the generated parser did not move): its two UVM units are lane rows too and stay `pass` under the same preprocessor; the next release ceremony runs the umbrella. The lane's wall clock read `wall_s=1499` against 961 on the previous run — one measurement on a shared host, not attributed.

**Acceptance checklist (`TASK-ACCEPTANCE`):**

  - [x] **REPRODUCE / ISSUE** — `spi_agent_pkg.sv` under the pipeline on `16cb8e3a7`: `Parser did not consume full input … [furthest_position=729798 …] [origin=…/spi_host_dummy_seq.sv:14:21]`, the preprocessed text at that byte a raw `` `DV_CHECK_FATAL(req.randomize() with { … ``, `diagnostics=0`; the minimal repro `` `M(a == b // a comment, with a comma<LF>) `` → `if (!(a == b // a comment, with a comma)) $display("");`.
  - [x] **ROOT CAUSE (WHY + WHERE)** — WHERE: `rust/src/families/systemverilog/preprocessor.rs`, `expand_macros_in_text` binding `text[s..e]` from `parse_macro_invocation_arg_ranges` (which tracks `in_line_comment` for delimiters and keeps the comment's bytes in the range). WHY: the bound actual carried the `//…` text, `substitute_function_macro_body` emitted it on the expansion's single line, and the rest of that line was a comment to the parser — the signature is the lane's `furthest_position=729798` at the raw call (and `furthest_position=321648` on `alert_esc_agent_pkg.sv`), with the unit test's verbatim expansion `if (!(a == b // a comment, with a comma)) $display("");` at `preprocessor.rs:6032`.
  - [x] **FIX** — ENGINE tier, the family's own component: one function and one call; no grammar tier (the parser was right to reject the unbalanced line), no directive (comments in actuals are not a grammar policy).
  - [x] **ADDRESSED (verified)** — the unit test GREEN; the two packages `parse_full passed`; the lane `PIPELINE-MOVERS … moved=4`, all `fail → pass`; `PIPELINE-LIBRARY … opentitan=371/396 … published_packages=276`; members `unreached_in_unit` 552 → 512; the tuple `11115/2606/2615/1329/0`.
  - [x] **NO REGRESSION** — `moved=4 added=0 lost=0`, zero `pass → fail` over 16 336 preprocessed rows; the pins `failures=0` on the rebuilt release probe (13 preprocess rows); the full lib suite `1303 passed; 0 failed` (every pre-existing preprocessor test green, `a_one_line_comment_in_macro_text_is_not_substituted` among them); `clippy_on_rust_change` rc 0; ZERO grammar / generated bytes — every generated artifact byte-identical (the parser fingerprint `9cc363beab63c228` unchanged); `bash scripts/check_doctrines.sh` → `doctrines: ALL 45 enforced doctrines PASS.` on the staged set..
  - [x] **LOCKSTEP** — this section + the row, `.6d.5`'s row (the ninth package), the SV book's pipeline chapter (the bullet beside the macro-text rule, the census sentence) + tracked HTML, the SV integration contract's expansion-guarantees paragraph, the platform book's verdict-coverage anchor, the derived lane reports, the revision register's `1.0.248` row back-filled (`PENDING` → `16cb8e3a`), `docs/TASK_TREE.md`, `MEMORY.md`, `CHANGES.md`, `DEVELOPMENT_NOTES.md`.
  - [x] **NEUTRALITY TIER** — family component: `rust/src/families/systemverilog/` names its own family by path; the engine moved no byte.
  - promotion: declined (see `DEVELOPMENT_NOTES.md`).

- **Rider, found at the commit gate.** `GRAMMAR-CERT-CURRENCY` refused the commit: `grammars/systemverilog.ebnf leads the published table by 14 commits (budget 10)` — the `1.0.247` and `1.0.248` ceremonies had not run item 9's certification re-splice. Re-derived under the guard (`elapsed=75s`): the block is byte-identical to the published one. The page's live sentence beside it had drifted (*46.3 % adjudicated, 274 known defects* against the instrument's `45.8 %` and a bar of `0`) and is corrected to point at the derived anchor, the KM card with it; the instrument gap (an identical re-derivation leaves nothing to stage) is routed as `GRAMMAR-CERT-STATUS.6`.

## `.5`(m) `` `include `` followed by a macro — ✅ done 2026-10-02 (`PGEN-SVPP-EXPANSION-0034`): the standard licenses it in a sentence; `.5`(c)'s refusal is retracted

**What landed** (ENGINE tier, the SystemVerilog family's own component — `rust/src/families/systemverilog/preprocessor.rs`: the `include` arm expands a macro that follows the directive before reading the filename, `parse_include_directive`'s refusal reworded; two unit tests RED-first, one replacing the test that pinned the wrong ruling; the derived reports of both lane readings; ZERO grammar / codegen / generated bytes):

- **How it was found.** Reading the lane's `refused_engine` rows by reason (2026-10-02): six say `include path must be quoted or angle-bracketed (… a macro-valued path is a tool extension PGEN does not admit)`. One of them is `sv-tests/tests/chapter-22/22.5.1--include-define-expansion.sv` — a test NAMED after the clause. The clause: IEEE 1800-2017 §22.5.1, *"The `` `include `` directive can be followed by a macro, instead of a string literal:"* with `` `define home(filename) `"/home/mydir/filename`" `` / `` `include `home(myfile) `` (`docs/systemverilog/2017/txt/section-22-compiler-directives.txt:341-343`; 1800-2023 `…/2023/txt/section-22-compiler-directives.txt:409-411`).
- ⛔ **WHY IT WAS REFUSED: a ruling of mine, `.5`(c), 2026-09-09.** It read §22.4's Syntax 22-1 (`"filename"` / `<filename>`), concluded *"a macro-valued path is a tool extension"*, put that sentence into the refusal users see and into a unit test, and cited the strict-LRM default for it. The sentence two pages later was not read. This is the bounding ruling's own failure class — *"non-LRM" is a citation, never an inference* — a fourth instance beside `q[a:$]`, `use #(…)` and `'{ }`, and the first one found in the preprocessor; it is added to that record's table.
- **WHERE.** `parse_include_directive(rest)` saw `` `F `` and bailed; nothing expanded `rest` first.
- **FIX.** In the `include` arm, when the text after the directive begins with a backtick, `expand_macros_in_text` runs over it (the same call every passthrough line gets) and the RESULT is read by `parse_include_directive` and by `include_directive_remainder`; a result that is neither `"filename"` nor `<filename>` — an unquoted name, an undefined macro — is still refused, now naming both clauses and no longer calling the form an extension.
- **Measured, unit level.** RED on `330601ba5` with the final tests: `test result: FAILED. 0 passed; 2 failed` — `…/top.sv:2: include path must be quoted or angle-bracketed (IEEE 1800-2017 §22.4 Syntax 22-1: … a macro-valued path is a tool extension PGEN does not admit); got '`F'` (`preprocessor.rs:5417`, `:5379`). GREEN on the fix: `test result: ok. 2 passed; 0 failed`. The first test also holds the two forms EQUAL: `` `include `home(inc.svh) // c `` and `` `include "inc.svh" // c `` produce one text. The full lib suite: `test result: ok. 1304 passed; 0 failed; 28 ignored`; `clippy_on_rust_change` rc 0.
- ⚠️ **Two false starts in the chain, both mine, both stopped before a build.** The first test asserted a trailing comment is dropped; it is carried to the next logical line for the literal form too (`.6b.5`(b)), so the assertion was wrong and GREEN failed with the fix working. Correcting it through a nested string literal mangled the Rust escapes and RED did not compile. The test text now lives in a verbatim file the patch reads.
- **Measured on both readings** (the rebuilt debug probe `1e08b5ceedba…`, release probe `81707b046685…`, parser fingerprint `9cc363beab63c228` unchanged; pins on the release probe `ADJUDICATION-REPROS: checked=1834 armed=225 listed=898 multi_profile_rows=701 preprocess_rows=13 failures=0`):
  - `sv_2017`: `PIPELINE-LANE: files=16336 pass=12742 fail=3323 pp_fail=271 timeout=0 crash=0 wall_s=989 … profile=sv_2017` (was `pass=12739 fail=3320 pp_fail=277`); library unchanged (`opentitan=372/396 … published_packages=277`). `PIPELINE-MOVERS[sv_2017] rows_before=16336 rows_after=16336 moved=6 added=0 lost=0`, `{('black-parrot', 'pp_fail->fail'): 2, ('iverilog', 'pp_fail->pass'): 2, ('sv-tests', 'pp_fail->pass'): 1, ('sv2v', 'pp_fail->fail'): 1}`; `LIBRARY-MOVERS[sv_2017] {}`.
  - `sv_2023`: `PIPELINE-LANE: files=16336 pass=12782 fail=3283 pp_fail=271 timeout=0 crash=0 wall_s=1076 … profile=sv_2023` (was `pass=12779 fail=3280 pp_fail=277`); `PIPELINE-MOVERS[sv_2023] … moved=6 added=0 lost=0`, the SAME six rows in the same directions; `LIBRARY-MOVERS[sv_2023] {}` (`opentitan=375/396 … published_packages=278`).
  - The six rows, named: `pp_fail → pass` — `sv-tests …/22.5.1--include-define-expansion.sv`, iverilog `ivltests/br_gh484.v`, `ivltests/pr3012758.v`; `pp_fail → fail` — black-parrot `bp_common_aviary_cfg_pkgdef.svh` and `bp_common_aviary_pkgdef.svh` (past `` `include `BP_CUSTOM_DEFINES_PATH `` they stop at `:408:7`, a raw `` `bp_aviary_define_override( `` — a macro of a header the context does not supply), sv2v `test/lex/line.sv` (→ `.5`(n)). Two refusals kept their status and changed their reason to the real one: Surelog `PreprocLine/dut.sv` (`include 'line.vh' was not found in search paths` — context) and verilator `t_preproc.v` (`:277`, → `.5`(o)). OpenTitan's `aes_tb_pkg.sv:39` is still refused: `` `REQUESTS_FILE `` is defined on the testbench's command line, so the macro expands to nothing here — the refusal now says a macro must expand to a filename.
  - The primary reading's re-adjudication: `PIPELINE-READJUDICATION: … chained:hollow=26 chained:refused_context=211 chained:refused_engine=12 chained:rejects_own_file=279 … hollow=1 … refused_engine=6 …` (was 25 / 210 / 15 / 278 / 0 / 7); `svpp_owned:refused_engine` 30 → 29. The verdict-coverage tuple under the pipeline `11168/2606/2562/1276/0` → `11168/2606/2562/1275/0`: adjudicated unmoved (no mover sits in an adjudicated class — the sv-tests row is `svpp_owned`, the iverilog rows are routed to the `verilog_2005` lane), dark −1 (the sv-tests row no longer fails); the raw tuple `7485/2606/6245/3864/0` unmoved; `check_sv_corpus_denominator.sh` refused the stale anchor, then OK.
  - The census of the two readings: `PIPELINE-SV2023-CENSUS: rows=16336 both_pass=12742 both_nonpass=3554 2023_only_pass=40 2017_only_pass=0 population_breaches=0 | library: files=407 2023_only_pass=3 2017_only_pass=0 published_2017=277 published_2023=278 | members: unreached_2017=498 reached_2023=51 lost_2023=0` — the differential unmoved (40 / 0 / 3 / 51), the expansion taking no profile. `IDENTITY-CARRIERS: OK (live=15 frozen=7 rows_equal=75 notes=7 not_evaluated=0)`; `corpus-key-integrity` rc 0; `PIPELINE-LANE-CURRENCY: OK`.
- ⛔ **A claim of mine corrected.** Before measuring I told the director this refusal "blocks black-parrot's common package". It does not: `bp_common_pkg.sv` is `pp_fail` on `bsg_defines.sv`, a vendored include absent from the tree, before and after. The two black-parrot rows that moved are the aviary headers.

**Acceptance checklist (`TASK-ACCEPTANCE`):**

  - [x] **REPRODUCE / ISSUE** — `22.5.1--include-define-expansion.sv` under the pipeline on `330601ba5`: `pp_fail`, `invalid `include directive at …:17: include path must be quoted or angle-bracketed (… a macro-valued path is a tool extension PGEN does not admit)`; the unit repro `` `define F "inc.svh" `` / `` `include `F `` refused the same way.
  - [x] **ROOT CAUSE (WHY + WHERE)** — WHERE: `rust/src/families/systemverilog/preprocessor.rs`, the `include` arm calling `parse_include_directive(rest)` on unexpanded text. WHY: `.5`(c) ruled the form out from Syntax 22-1 alone; §22.5.1 licenses it. Signature: the refusal text itself and the RED tests' panics at `preprocessor.rs:5417` / `:5379` (`[E_SVPP_…]`-class refusal without a code: `invalid `include directive`), reproduced on six lane rows (`pp_fail=277`).
  - [x] **FIX** — ENGINE tier, the family's own component: one expansion call in the `include` arm; no grammar tier (a directive, not a production), no engine-neutral tier (no second language has `` `include ``).
  - [x] **ADDRESSED (verified)** — the unit tests GREEN; the sv-tests case `parse_full passed`; `pp_fail` 277 → 271 under each reading; three rows pass.
  - [x] **NO REGRESSION** — zero `pass → fail` over 16 336 rows under EACH reading (every mover starts at `pp_fail`); the two-edition census `2017_only_pass=0 … lost_2023=0`; the pins `failures=0` on the rebuilt release probe (13 preprocess rows); the full lib suite `1304 passed; 0 failed`; `clippy_on_rust_change` rc 0; ZERO grammar / generated bytes — every generated artifact byte-identical (parser fingerprint `9cc363beab63c228` unchanged); `bash scripts/check_doctrines.sh` → `doctrines: ALL 45 enforced doctrines PASS.` on the staged set (first run refused `MEMORY-ARCH`: `MEMORY.md` 33 111 bytes over its 32 768 cap — the next-action line trimmed to its four newest entries, 19 805 bytes).
  - [x] **LOCKSTEP** — this section + the rows (`.5`(m) done, `.5`(n) and `.5`(o) opened, `.5`(c) annotated), the SV book's pipeline chapter (the refusal paragraph corrected; + tracked HTML), the SV integration contract's expansion-guarantees paragraph, the two decision records that repeated the ruling (`project_preprocess_parse_pipeline_is_the_sv_deliverable`, `feedback_sv_strict_lrm_compliance_default` — the fourth row of its table), the derived reports of both readings and the census, `docs/TASK_TREE.md`, `MEMORY.md`, `CHANGES.md`, `DEVELOPMENT_NOTES.md`.
  - [x] **NEUTRALITY TIER** — family component: `rust/src/families/systemverilog/` names its own family by path; the engine moved no byte.
  - promotion: declined (see `DEVELOPMENT_NOTES.md`).

## `.5`(n) `` `line `` with macro parameters, `.5`(o) a comment between macro formals — ✅ done 2026-10-05 (`PGEN-SVPP-EXPANSION-0035`): one is licensed and fixed, the other is refused by the clause and now says so

**What landed** (ENGINE tier, the SystemVerilog family's own component — `rust/src/families/systemverilog/preprocessor.rs`: the `line` arm reads its parameters AFTER macro expansion; two refusals of the formal-list parser reworded through one helper; three unit tests, two RED-first and one guard; three preprocess pins; the derived reports of all THREE lane readings; ZERO grammar / codegen / generated bytes):

- **Both items were asked of the LRM text first, in all three in-repo editions** (`.5`(c)'s lesson).
  - **(n) is LICENSED.** IEEE 1800-2023 §22.2: *"Macro expansion shall occur within a compiler directive. Directives are not recognized within comments or string literals."* (`docs/systemverilog/2023/txt/section-22-compiler-directives.txt:68-69`) — a sentence 1800-2017 does not have; 1800-2017 §22.5.1 covers the same ground generally: *"Once a text macro name has been defined, it can be used anywhere in the compilation unit where it is defined. There are no other scope restrictions once inside the compilation unit."* (`docs/systemverilog/2017/txt/section-22-compiler-directives.txt:227-228`). §22.12 says what the parameters shall BE — *"a positive integer"*, *"a string literal"*, a level of 0, 1 or 2 (`:713-718`) — which is what `` `line `new_line_num `foo(line) 1 `` is once its macros are substituted.
  - **(o) is NOT licensed, in any edition.** §22.5.1: *"The formal argument names shall be simple_identifiers, separated by commas and optionally white space."* (2017 `:143-144`; 2023 `:168-169`, in a paragraph the 2023 revision REWROTE — it now specifies the white space between the macro name and its parenthesis — and left this sentence as it was while adding comment rules for the macro TEXT only, `:175-179`). §5.3: *"White space shall contain the characters for spaces, tabs, newlines, and formfeeds"* (`docs/systemverilog/2017/txt/section-5-lexical-conventions.txt:37`). And the clause names a comment where it admits one — *"Only white space or a comment may appear on the same line as the `` `include `` compiler directive"* (2017 `:82`) — and forbids one where it means to — *"Comments are not allowed on the same line as a `` `line `` directive"* (`:704-705`). A search of clauses 5 and 22 of 1800-2017 / 1800-2023 and clauses 3 and 19 of 1364-2005 for a sentence equating a comment with white space finds none. A NEWLINE between formals is white space and stays accepted (`.5`(a)).
- **WHERE (n).** The `line` arm's guard called `parse_line_directive(rest)` on unexpanded text; a backtick made it `None`, the line fell to the unknown-directive arm, which EXPANDED it and passed it through — so the output carried a raw `` `line 200 "line.vh" 1 `` and `` `__LINE__ `` after it kept the old numbering. Inside a procedural block the parser has no production for it (`furthest_position=265` on sv2v `line.sv`; `111` on the pin).
- **FIX (n).** Before the directive match, when the directive is `line`, the region is active and the text holds a backtick, `expand_macros_in_text` runs over the parameters into a scratch event vector; the arm's guard and body read the expanded text, and the events are appended only when the directive is consumed. A result that is not a well-formed directive passes through exactly as before — expanded once, recorded once.
- **(o): no verdict moves; the message does.** `parse_macro_parameter_list` refused these shapes already, as `invalid macro parameter token '/*more'` or `invalid trailing text in macro parameter`. Both sites now go through `formal_list_comment_refusal`: *"a comment inside a macro formal argument list is not admitted (IEEE 1800-2017 / 1800-2023 §22.5.1: the formal argument names are "separated by commas and optionally white space"; §5.3: white space is spaces, tabs, newlines and formfeeds, and the standard writes "white space or a comment" where it admits one, §22.4); got '…'"*.
- ⛔ **A count of mine corrected by the lane.** I priced (o) at two corpus files from a one-line grep (verilator `t_preproc.v:277`, iverilog `macro_with_args.v:17`). The readings show THREE rows whose refusal changed to the new text: the third is sv2v `test/lex/macro_arg_comment.sv:1`, whose formal list opens on one line and carries its comments on the next twelve. The count is now the lane's own (the rows carrying the reason). All three are tool test suites probing the edge — verilator's says *"comments MIGHT not be legal"* — and no design file writes it, which is why no opt-in tolerance is proposed. The sv2v row is keyed `must_accept` by suite convention and stays `refused_engine` in the re-adjudication: that class does not tell a refusal BY RULING from an engine gap; its reason column now carries the citation.
- **Measured, unit level.** RED on `1e7e6ed9f` with the final tests: `test result: FAILED. 1 passed; 2 failed` — `a_line_directive_whose_parameters_are_macros_is_expanded_then_consumed` (`preprocessor.rs:6153`, *the directive is consumed*) and `a_comment_inside_a_macro_formal_list_is_refused_naming_the_clause` (`:6240`, the refusal read `invalid macro parameter list: invalid macro parameter token …`); the guard `a_line_directive_whose_macros_do_not_form_it_still_passes_through` green on both sides (it also counts `MacroExpand` events = 1, so the probe's expansion is not recorded twice). GREEN on the fix: `test result: ok. 3 passed; 0 failed`. The full lib suite `test result: ok. 1308 passed; 0 failed; 28 ignored` (was 1305; + 3); `clippy_on_rust_change` rc 0.
- **Measured on the three readings** (the rebuilt debug probe `5e75b758c5ae…`, release probe `8dafa3d397fd…`, parser fingerprint `faeb6be3ad122b08` unchanged; pins on the release probe `ADJUDICATION-REPROS: checked=1869 armed=231 listed=913 multi_profile_rows=710 preprocess_rows=16 tolerated_rows=6 superset_rows=713 failures=0`, was `checked=1862 listed=910 preprocess_rows=13`; `timeout_s=600 jobs=6`, every step under the guard, sequential):
  - `sv_2017`: `PIPELINE-LANE: files=16336 pass=12743 fail=3322 pp_fail=271 timeout=0 crash=0 wall_s=1074 … profile=sv_2017` (was `pass=12742 fail=3323`); `PIPELINE-LIBRARY: … opentitan=372/396 … published_packages=277` unchanged; `PIPELINE-MOVERS[sv_2017] rows_before=16336 rows_after=16336 moved=1 added=0 lost=0`, `{('sv2v', 'fail->pass'): 1}`; `LIBRARY-MOVERS[sv_2017] {}`.
  - `sv_2023`: `PIPELINE-LANE: files=16336 pass=12783 fail=3282 pp_fail=271 timeout=0 crash=0 wall_s=1051 … profile=sv_2023` (was `pass=12782`); `opentitan=375/396 … published_packages=278`; `PIPELINE-MOVERS[sv_2023] … moved=1 added=0 lost=0`, the SAME row; `LIBRARY-MOVERS[sv_2023] {}`.
  - `sv_2023_tolerant`: `PIPELINE-LANE: files=16336 pass=12798 fail=3267 pp_fail=271 timeout=0 crash=0 wall_s=1398 … profile=sv_2023_tolerant` (was `pass=12797`); `opentitan=381/396 … published_packages=282`; `PIPELINE-MOVERS[sv_2023_tolerant] … moved=1 added=0 lost=0`, the same row; `LIBRARY-MOVERS[sv_2023_tolerant] {}`.
  - The row: sv2v `test/lex/line.sv`, `fail → pass`. Three refusals keep `pp_fail` and change their reason to the clause-naming text (the three files of (o)). Surelog `PreprocLine/dut.sv` and verilator `t_lint_warn_line_bad.v`, the other two files that write `` `line `` with a macro parameter, do not move: the first is `pp_fail` on an include its context does not supply, the second passed already (its directives sit at description level, where the parser's own production read the passed-through line).
  - The primary reading's re-adjudication: `PIPELINE-READJUDICATION: rows=6911 … corroborated=1332 … refused_engine=6 …` — `corroborated` 1331 → 1332 and `hollow` 1 → 0 (the one row), every other class as it was. The verdict-coverage tuple under the pipeline `11168/2606/2562/1275/0` → `11169/2606/2561/1274/0` (the raw tuple unmoved; `check_sv_corpus_denominator.sh` refused the stale anchor, then OK).
  - The censuses: `PIPELINE-SV2023-CENSUS: rows=16336 both_pass=12743 both_nonpass=3553 2023_only_pass=40 2017_only_pass=0 population_breaches=0 | library: files=407 2023_only_pass=3 2017_only_pass=0 published_2017=277 published_2023=278 | members: unreached_2017=498 reached_2023=51 lost_2023=0`; `PIPELINE-TOLERANT-CENSUS: rows=16336 both_pass=12783 both_nonpass=3538 tolerant_only_pass=15 2023_only_pass=0 population_breaches=0 | library: files=407 tolerant_only_pass=6 2023_only_pass=0 published_2023=278 published_tolerant=282 | members: unreached_2017=498 reached_tolerant=315 lost_tolerant=0` — both differentials unmoved (40 / 0 and 15 / 0), the expansion taking no profile. `IDENTITY-CARRIERS: OK (live=16 frozen=7 rows_equal=84 notes=7 not_evaluated=0)`; `corpus-key-integrity` rc 0; `PIPELINE-LANE-CURRENCY: OK`.
- **Found on the way and ROUTED as `.5`(p):** three places where 1800-2023 says more than 1800-2017 and the preprocessor reads no edition — a block comment that opens in a `` `define ``'s text and continues past the newline, a block comment in macro text that 2023 says is not substituted, and an `` `ifdef `` whose argument is a macro usage — each with its scratch repro and corpus count in the row above.
- ⚠️ **A wrong estimate of mine.** I told the director the chain would take four and a half hours with readings of about twenty-five minutes; a reading is the library pass plus the lane, ≈ 50 minutes (the tolerant one 67), as `SV-CORPUS-GRAD.43`'s record already said. The chain ran 09:16 → 12:32.

**Acceptance checklist (`TASK-ACCEPTANCE`):**

  - [x] **REPRODUCE / ISSUE** — (n) sv2v `test/lex/line.sv` under the pipeline on `1e7e6ed9f`: `fail`, `furthest_position=265`; the scratch repro `rust/target/svpp_5n/n2.sv` shows the raw `` `line 200 "x.v" 1 `` in the expanded text with the following `` `__LINE__ `` still reading the file's own line; the pin text rejects at `furthest_position=111` on the 1.0.249 release probe under all three profiles. (o) `rust/target/svpp_5n/o2.sv` / `o3.sv`: `invalid macro parameter list: invalid macro parameter token '/*more'` / `'// Hello …'`.
  - [x] **ROOT CAUSE (WHY + WHERE)** — WHERE: `rust/src/families/systemverilog/preprocessor.rs`, the `line` arm's guard `parse_line_directive(rest)` on unexpanded text, and `parse_macro_parameter_list`'s two generic refusals. WHY (n): the arm predates any reading of macro usages inside a directive; 1800-2023 §22.2 requires the expansion. WHY (o): nothing was wrong with the verdict; the message cited no clause. Signature: the RED tests' panics at `preprocessor.rs:6153` / `:6240`, the lane row's `furthest_position=265`.
  - [x] **FIX** — ENGINE tier, the family's own component: one expansion ahead of the `line` arm; one refusal helper. No grammar tier (directives, not productions), no engine-neutral tier (no second language has `` `line `` or `` `define ``).
  - [x] **ADDRESSED (verified)** — the unit tests GREEN (the macro-spelled directive reads as ONE text with the literal form); `fixed_line_directive_macro_parameters.sv` ACCEPT, `invalid_macro_formal_list_comment.sv` REJECT, `control_macro_formal_list_newline.sv` ACCEPT through the pipeline; sv2v `line.sv` passes under each reading.
  - [x] **NO REGRESSION** — zero `pass → fail` over 16 336 rows under EACH of the three readings (`moved=1 added=0 lost=0`, the one mover `fail → pass`); both census differentials unmoved; the pins `failures=0` on the rebuilt release probe (16 preprocess rows, 910 pre-existing rows as keyed); the full lib suite `1308 passed; 0 failed`; `clippy_on_rust_change` rc 0; ZERO grammar / generated bytes — parser fingerprint `faeb6be3ad122b08` unchanged; `bash scripts/check_doctrines.sh` on the staged set (recorded in the commit).
  - [x] **LOCKSTEP** — this section + the rows (`.5`(n) and `.5`(o) done, `.5`(p) opened), three pins + `MANIFEST.tsv` (913 rows), the SV book's pipeline chapter (the `` `line `` bullet, a new bullet for the formal-list refusal; + tracked HTML) and its status page (the distance block), the SV integration contract's expansion-guarantees paragraph, the platform book's verdict-coverage anchor, the derived reports of the three readings and both censuses, `docs/TASK_TREE.md`, `MEMORY.md`, `CHANGES.md`, `DEVELOPMENT_NOTES.md`. Riding, named: `SV-CORPUS-GRAD.45` OPENED by director delegation, with the decision record's addendum.
  - [x] **NEUTRALITY TIER** — family component: `rust/src/families/systemverilog/` names its own family by path; the engine moved no byte.
  - promotion: declined (see `DEVELOPMENT_NOTES.md`).

## `.5`(p) three readings selected by the profile's EDITION — ✅ done 2026-10-06 (`PGEN-SVPP-EXPANSION-0036`): where IEEE 1800-2023 says more than 1800-2017, the preprocessor now reads the edition the text will be parsed under

**What landed** (ENGINE tier, the SystemVerilog family's own component — `rust/src/families/systemverilog/preprocessor.rs`: one edition predicate `reads_1800_2023` on the parse profile; the `` `define `` reader carries a block comment across its newlines under 2023 and refuses the same text under 2017; `parse_define_directive` drops block comments from the macro text under 2023; the `` `ifdef `` / `` `ifndef `` / `` `elsif `` / `` `undef `` arms expand an argument written as a macro usage under 2023; six unit tests RED-first; four edition-twin preprocess pins; the derived reports of all three lane readings; ZERO grammar / codegen / generated bytes):

- **The three sentences, each present in 1800-2023's clause 22 and absent from 1800-2017's.**
  - (i) §22.5.1: *"A newline character that is not contained in a triple-quoted string literal or in a block comment and is not preceded by a backslash shall end the macro text"* (`docs/systemverilog/2023/txt/section-22-compiler-directives.txt:150-151`) and *"If the macro text contains a multi-line block comment, the newline characters in the block comment are not required to be preceded by a backslash"* (`:177-179`). 1800-2017: *"The first newline character not preceded by a backslash shall end the macro text"* (`docs/systemverilog/2017/txt/section-22-compiler-directives.txt:135-136`), and the text *"shall not be split across the following lexical tokens: — Comments …"* (`:230-232`).
  - (ii) §22.5.1: *"If a one-line comment or block comment (see 5.4) is included in the text, then the comment shall not become part of the substituted text"* (2023 `:175-176`); 1800-2017 says it of one-line comments only (`:151-152`).
  - (iii) §22.2: *"Macro expansion shall occur within a compiler directive"* (2023 `:68`); 1800-2017's Syntax 22-6 takes `` `ifdef text_macro_identifier `` (`:379-380`) and has no such sentence.
- **Measured before the change** (`rust/target/svpp_5n/{bc1,bc2,ifdef_macro}.sv`, the 1.0.250 debug `ast_pipeline --preprocess`, both profiles): (i) `` `define X 1 /* multi `` / `line */ + 2` — `X` became `1 /* multi` and `line */ + 2` was emitted as SOURCE text: the comment's tail leaked with its opener consumed into the macro, under both profiles; (ii) `` `define Y(a) a /* gone? */ + 1 `` substituted the comment under both; (iii) `` `ifndef `NODS_INDIRECT(NODS_DEFINED) `` was read as written — the guarded text emitted, `diagnostics=2` — under both.
- **The design.** ONE switch, the edition behind `--profile` (`sv_2023` and `sv_2023_tolerant` read 1800-2023; every other profile and the unprofiled reading read 1800-2017 — the mapping §22.14's keyword tables already use), not a knob per sentence: a caller cannot assemble an edition no standard describes. Under 2023: (i) the `` `define `` reader joins the next physical line while the logical line ends inside a block comment (a `//` hides only the rest of its own physical line); (ii) each `/* … */` of the macro text outside a string, a macro string and a `//` comment is replaced by ONE space before the one-line-comment cut, so the tokens on its two sides stay two tokens; (iii) an argument beginning with a backtick is expanded and the directive reads what it yields — for `` `elsif `` only where the directive is actually evaluated. Under 2017: (i) is REFUSED, naming §22.5.1 and the profile whose edition licenses the form (*"… the text "shall not be split across" a comment; IEEE 1800-2023 §22.5.1 lets a block comment carry the text across its newlines — that reading is selected by --profile sv_2023"*); (ii) and (iii) stay as they were.
- ⛔ **Why 2017 refuses instead of continuing the comment as the tools do.** The strict reading of an edition is what its text says. The 2017 clause ends the macro text at the newline and forbids splitting a comment across it; continuing the comment is a tool reading. And the previous behaviour was neither: it emitted the tail as source. The refusal cannot regress a passing row — a census with the implementation's own lexing rules (strings, `//` to end of line, continuation and formal-list joins) finds exactly ONE corpus `` `define `` that leaves a block comment open, verilator `t_preproc.v:358`, in a file already `pp_fail` at `:277`. (A first census by regex said four sites; it misread iverilog's `macro_comment2.v`, whose comments all close on their line.)
- **Predictions, written with a clock before the chain started** (`rust/target/svpp_5p/prereg.md`, 2026-10-05 22:17, HEAD `49cc3f928`): RED `0 passed; 6 failed`, GREEN `6 passed`; lib suite 1314; pins 923 with the `sv2023` pins accepted and their `sv2017` twins rejected; `moved=0` under each of the three readings; under the two 2023-edition readings a row may keep its status and change its stop position; both census differentials and the verdict-coverage tuple unmoved. EVERY ONE HELD.
- **Measured, unit level.** RED on `49cc3f928` with the final tests: `test result: FAILED. 0 passed; 6 failed` (`preprocessor.rs:6256`, `:6234`, `:6315`, `:6335` among the panics). GREEN: `test result: ok. 6 passed; 0 failed`. The full lib suite `test result: ok. 1314 passed; 0 failed; 28 ignored` (was 1308; + 6); `clippy_on_rust_change` rc 0.
- **Measured on the three readings** (the rebuilt debug probe `ac5e11891d62…`, release probe `38a3dd15b727…`, parser fingerprint `a5b5b9dc5c8ee247` unchanged; pins on the release probe `ADJUDICATION-REPROS: checked=1885 armed=236 listed=923 multi_profile_rows=714 preprocess_rows=20 tolerated_rows=8 superset_rows=715 failures=0`, was `checked=1881 listed=919 preprocess_rows=16 superset_rows=713`; `timeout_s=600 jobs=6`, every step under the guard, sequential):
  - `sv_2017`: `PIPELINE-LANE: files=16336 pass=12743 fail=3322 pp_fail=271 timeout=0 crash=0 wall_s=971 … profile=sv_2017`; `moved=0`, and NO row's stop position or reason changed — the 2017 reading of the corpus is byte-for-byte the previous one.
  - `sv_2023`: `PIPELINE-LANE: files=16336 pass=12783 fail=3282 pp_fail=271 timeout=0 crash=0 wall_s=1375 … profile=sv_2023`; `moved=0`. TEN rows keep `fail` with a SMALLER stop position in the expanded text (block comments are no longer substituted): eight OpenTitan DV files (`tl_agent_pkg.sv` 201933 → 200355, `adc_ctrl_env_pkg.sv`, `aes_cov_if.sv`, `entropy_src_cov_if.sv`, `lc_ctrl_env_pkg.sv`, `spi_device_env_pkg.sv`, `sram_ctrl_env_pkg.sv`, `sysrst_ctrl_env_pkg.sv`), sv2v `macro.sv`, verilator `t_var_dotted2.v`. Nine map back to the IDENTICAL `file:line:col`; one, `aes_cov_if.sv:387`, keeps its file and line and its column moves 252 → 161 on a 64-character line — inside a macro expansion the column counts expanded characters, as the contract's source-map paragraph states.
  - `sv_2023_tolerant`: `PIPELINE-LANE: files=16336 pass=12801 fail=3264 pp_fail=271 timeout=0 crash=0 wall_s=1273 … profile=sv_2023_tolerant`; `PIPELINE-LIBRARY: … opentitan=382/396 … published_packages=283` unchanged; `PIPELINE-MOVERS[sv_2023_tolerant] rows_before=16336 rows_after=16336 moved=0 added=0 lost=0`; seven rows keep `fail` with a smaller stop position (the same files, less the three the tolerant profile passes), one origin column changed (the same `aes_cov_if.sv:387`); `LIBRARY-MOVERS {}` under each reading.
  - The primary reading's re-adjudication unchanged (`corroborated=1332 … refused_engine=6`), the verdict-coverage tuple under the pipeline `11169/2606/2561/1274/0` unchanged. `PIPELINE-SV2023-CENSUS: rows=16336 both_pass=12743 both_nonpass=3553 2023_only_pass=40 2017_only_pass=0 population_breaches=0 | library: files=407 2023_only_pass=3 2017_only_pass=0 published_2017=277 published_2023=278 | members: unreached_2017=498 reached_2023=51 lost_2023=0`; `PIPELINE-TOLERANT-CENSUS: rows=16336 both_pass=12783 both_nonpass=3535 tolerant_only_pass=18 2023_only_pass=0 population_breaches=0 | library: files=407 tolerant_only_pass=7 2023_only_pass=0 published_2023=278 published_tolerant=283 | members: unreached_2017=498 reached_tolerant=337 lost_tolerant=0` — both differentials unmoved (40 / 0 and 18 / 0). `IDENTITY-CARRIERS: OK (live=16 frozen=7 rows_equal=84 notes=7 not_evaluated=0)`; `corpus-key-integrity` rc 0; `PIPELINE-LANE-CURRENCY: OK`; `sv-corpus-denominator: OK`.
- **What the slice does NOT move.** verilator `t_preproc.v` — the one file that writes all three shapes — is still refused at `:277` (a comment between macro formals, `.5`(o)'s ruling) before any of them is reached. The slice is conformance to the 2023 text and the end of a silent-garbage path, priced honestly: no row passes because of it today.

**Acceptance checklist (`TASK-ACCEPTANCE`):**

  - [x] **REPRODUCE / ISSUE** — the three scratch repros above on the 1.0.250 debug `ast_pipeline --preprocess` under `sv_2017` and `sv_2023`: `line */ + 2` in the expanded text (i), `/* gone? */` substituted (ii), `wire indirect_failed;` emitted with `diagnostics=2` (iii); the two pin texts reject under all three profiles on the 1.0.250 release probe (`furthest_position=9`).
  - [x] **ROOT CAUSE (WHY + WHERE)** — WHERE: `rust/src/families/systemverilog/preprocessor.rs` — the `` `define `` logical-line loop (it joined on an unclosed formal list and on a backslash, never on an open block comment, and then let the generic block-comment carry EMIT the tail), `parse_define_directive` (one-line comments cut, block comments kept for every profile), the conditional arms (`split_conditional_argument(rest)` on unexpanded text). WHY: the stage read `parse_profile` for §22.14 keywords only; three sentences 1800-2023 added had no reader. Signature: the RED panics at `preprocessor.rs:6256` / `:6234` / `:6315` / `:6335`; the pins' `furthest_position=9` and the repros' `preprocess_failed`-free garbage.
  - [x] **FIX** — ENGINE tier, the family's own component; no grammar tier (directives and macro text, not productions), no engine-neutral tier (no second language has these directives).
  - [x] **ADDRESSED (verified)** — the six tests GREEN; `fixed_define_multi_line_block_comment_sv2023.sv` and `fixed_ifdef_macro_usage_argument_sv2023.sv` ACCEPT under `sv_2023` (and under the tolerant profile, by the superset sweep), their byte twins REJECT under `sv_2017`.
  - [x] **NO REGRESSION** — `moved=0` over 16 336 rows under EACH of the three readings, zero `pass → fail`; the `sv_2017` reading unchanged to the stop position; both census differentials unmoved; the pins `failures=0` on the rebuilt release probe (20 preprocess rows, the 919 pre-existing rows as keyed); the full lib suite `1314 passed; 0 failed`; `clippy_on_rust_change` rc 0; ZERO grammar / generated bytes — the generated parser byte-identical (fingerprint `a5b5b9dc5c8ee247` unchanged); `bash scripts/check_doctrines.sh` on the staged set (recorded in the commit).
  - [x] **LOCKSTEP** — this section + the row (`.5`(p) done), four pins + `MANIFEST.tsv` (923 rows), the SV book's pipeline chapter (the edition table; + tracked HTML), the SV integration contract's expansion-guarantees paragraph, `TOOLBOX.md` (diagnose under the profile you parse with), the derived reports of the three readings and both censuses, `docs/TASK_TREE.md`, `MEMORY.md`, `CHANGES.md`, `DEVELOPMENT_NOTES.md`.
  - [x] **NEUTRALITY TIER** — family component: `rust/src/families/systemverilog/` names its own family by path; the engine moved no byte.
  - promotion: declined (see `DEVELOPMENT_NOTES.md`).

## `.6c.3` the rows the raw key deferred to this lane by design — ✅ done 2026-10-06 (`PGEN-SVPP-EXPANSION-0042`): 94 sv-tests preprocessing tests judged by their own header, 92 testify, and two over-acceptances of the preprocessor surface

**What landed** (INSTRUMENT tier — `stimuli/sv/adjudicate_external_corpus.py` (`expect_sv_tests(…, lane="raw")`: one parameter; in `lane="pipeline"` a `:type: preprocessing` test is judged by the header rule every other type gets; the raw call path and the raw manifests UNCHANGED), `stimuli/sv/pipeline_readjudication.py` (`svpp_owned_expected`: the derivation for sv-tests rows via the adjudicator's function; the `svpp_owned_expected` population; verdicts `svpp_owned:corroborated` / `svpp_owned:refuted`; the legend), `stimuli/sv/corpus_verdict_coverage.py` (the two new classes; `must_reject:refused_engine` ADJUDICATED — a clause-cited preprocessor refusal of a must-reject row is the rejection the key expects; a must-reject row the raw lane DEFERRED that the pipeline accepts is the PIPELINE's bar row), the re-rendered re-adjudication, dark audits, verdict-coverage artifacts and the rejects-own-file census, the platform book's verdict-coverage anchor, the SV book's pipeline chapter (+ tracked HTML); ZERO Rust / grammar / generated bytes):

- **How it was found.** The pipeline tuple's NO VERDICT bucket carried `pipeline:svpp_owned:accepted_under_expansion = 110` — rows that PASS under the pipeline and count for nothing. Their raw class is `deferred:svpp_owned`, whose own definition reads *"conformance owned by the preprocessor lane, by design"* (`PERMANENT` in the coverage census). 88 of the 110 are sv-tests chapter-22 tests: the standard's own preprocessor conformance suite, each with a header that says what it expects.
- **The derivation is the key's own, not a copy.** `adj.expect_sv_tests` already reads the header for every other `:type:`; the pipeline mode applies the same rule to `preprocessing` — `should_fail_because` → must_reject, absent → must_accept — through ONE function, so the two lanes cannot drift apart. Other suites' svpp_owned rows (verilator 35, sv2v 25, verible 5, Surelog 1) have no derivation yet and keep their expectation-free verdicts; verilator's driver convention (`VerilatorIndex.expect`) is the candidate for `.6c.4`.
- **Measured** (no lane re-run; the dumps of `-0040`): `PIPELINE-READJUDICATION: … must_reject:accepts_invalid=2 must_reject:match_under_expansion=453 must_reject:refused_engine=4 … svpp_owned:accepted_under_expansion=22 svpp_owned:corroborated=86 … svpp_owned:refused_engine=25 svpp_owned:rejects_own_file=28` (was `match_under_expansion=451`, `accepted_under_expansion=110`, `refused_engine=29`, `rejects_own_file=30`, no `corroborated`, no `accepts_invalid`); the verdict-coverage tuple under the pipeline `11176/2606/2554/1267/0` → `11270/2606/2460/1261/2` (+94 adjudicated: 86 + 4 + 2 + 2; dark −6; **the bar under the pipeline 0 → 2**); the raw tuple `7485/2606/6245/3864/0` unmoved; the rejects-own-file census 307 → 305 rows (`raw_backtick` 44 → 42: `22.3--resetall_illegal.sv` and `22.5.1--define-expansion_18.sv` are now `must_reject:match_under_expansion`); `PIPELINE-LANE-CURRENCY: OK` (all four arms); `check_sv_corpus_denominator.sh` refused the stale anchor, then OK; the adjudicator's `--self-test` PASS.
- ⛔ **The two over-acceptances, both the standard's words** (`.5`(s) and `.5`(t) OPENED): (s) `22.5.1--define-expansion_21.sv` — `` `define first_half "start of string `` then `` $display(`first_half end of string"); `` — IS the LRM's own illegal example: 1800-2017 §22.5.1 *"The text specified for macro text shall not be split across the following lexical tokens: Comments, Numbers, String literals, Identifiers, Keywords, Operators"* and *"The following is illegal syntax because it is split across a string"* (`docs/systemverilog/2017/md/section-22-compiler-directives.md:275-281, :295-297`; 2023 `docs/systemverilog/2023/md/section-22-compiler-directives.md:307, :320-322`; 1364-2005 §19.3.1 `docs/verilog/2005/md/section-19-compiler-directives.md:172, :192, :214-215`). Measured: the preprocessor accepts the definition and the expansion reads `$display("start of string end of string");` — the macro text was split across a string literal and glued at the use site. (t) `22.5.1--define-expansion_23.sv` — `` `define define "illegal" ``: §22.5.1 *"All compiler directives shall be considered predefined macro names; it shall be illegal to redefine a compiler directive as a macro name"* (2017 `docs/systemverilog/2017/md/section-22-compiler-directives.md:139-140`; 2023 `docs/systemverilog/2023/md/section-22-compiler-directives.md:152-153`; 1364-2005 §19.3.1 `docs/verilog/2005/md/section-19-compiler-directives.md:100-101`, and `:229` *"Text macro names may not be the same as compiler directive keywords"*). Measured: `` `define define `` and `` `define include 1 `` both `parse_full passed`. Both are ENGINE items of the family's own preprocessor, each a refusal naming its clause, each to be censused over the corpus before the chain.
- **What this slice does NOT do.** It fixes neither defect (the bar under the pipeline honestly reads 2 until they land); it derives no expectation for the non-sv-tests svpp_owned rows; it moves no lane row.

**Acceptance checklist (`TASK-ACCEPTANCE`):**

  - [x] **REPRODUCE / ISSUE** — `docs/tasks/artifacts/sv_corpus_grad/verdict_coverage/pipeline_coverage.tsv` at `f2b63480f`: `NO VERDICT pipeline:svpp_owned:accepted_under_expansion 110`; 88 of them `sv-tests/tests/chapter-22/*` with a `:type: preprocessing` header.
  - [x] **ROOT CAUSE (WHY + WHERE)** — WHERE: `expect_sv_tests` returned `out_of_scope_with_cause` for `:type: preprocessing` in every caller; `verdict_of` had no judged form for the `svpp_owned` population. WHY: the raw key deferred the rows to a lane that did not yet exist; once it did, nothing carried the header rule across. Signature (ops family): `git ls-files stimuli/sv/pipeline_readjudication.py stimuli/sv/corpus_verdict_coverage.py` names the two consumers of the verdict vocabulary and `grep -n svpp_owned stimuli/sv/corpus_verdict_coverage.py` showed every svpp_owned class mapped NO VERDICT unconditionally; the two accepted should-fail tests reproduce by hand on the release probe (`parse_full passed` on both; the expansion of `_21` glues the string).
  - [x] **FIX** — INSTRUMENT tier: one parameter on the key's function, the derivation in the re-adjudication, the mapping in the coverage census; no engine, grammar or generated byte.
  - [x] **ADDRESSED (verified)** — `svpp_owned:corroborated=86`, `must_reject:refused_engine=4` and `must_reject:match_under_expansion` +2 ADJUDICATED; the two over-acceptances named as `must_reject:accepts_invalid` and on the pipeline's bar; the tuple `11270/2606/2460/1261/2`.
  - [x] **NO REGRESSION** — the raw manifests byte-identical (the raw call path unchanged; the adjudicator's `--self-test` PASS; `CORPUS-KEY-INTEGRITY` and `CORPUS-CENSUS-CURRENCY` green in the doctrines); every other re-adjudication class unchanged; no lane row moved; `PIPELINE-LANE-CURRENCY: OK`; ZERO Rust / grammar / generated bytes (parser fingerprint `a5b5b9dc5c8ee247` unchanged); `bash scripts/check_doctrines.sh` → `doctrines: ALL 46 enforced doctrines PASS`.
  - [x] **LOCKSTEP** — this section + the rows (`.6c.3` done; `.5`(s), `.5`(t) opened), the three instruments, the re-rendered derived reports, the verdict-coverage anchor, the SV book's pipeline chapter (+ tracked HTML), `docs/TASK_TREE.md`, `MEMORY.md`, `CHANGES.md`, `DEVELOPMENT_NOTES.md`.
  - [x] **NEUTRALITY TIER** — corpus instruments under `stimuli/sv/`; the engine moved no byte.
  - promotion: declined (see `DEVELOPMENT_NOTES.md`).

## `.5`(r) a `` `define `` line whose backslash is followed by white space — ✅ done 2026-10-06 as a RULING (`PGEN-SVPP-EXPANSION-0041`): the comment-backslash continuation the 2023 text names outright already works; what the Surelog line writes is a backslash that is NOT immediately before the newline, which no edition licenses and the corpus writes only in two tests

**What landed** (PINS tier — two preprocess pins under `stimuli/sv/adjudication_repros/` (`control_define_line_comment_backslash_continues.sv` ACCEPT on both SV profiles; `invalid_define_backslash_trailing_space.sv` REJECT on both) + `MANIFEST.tsv` (925 → 927), the SV book's pipeline chapter (one bullet; + tracked HTML); ZERO Rust / grammar / generated bytes; no lane row moved):

- **The question `.6e` left.** Surelog `tests/TestMacros/macros.v:5-6`: inside a `` `define ``, the line `// A Macro comment \<TAB>   ` and then `begin\`; the stage ended the macro at the comment line and `begin\` reached the parser. Two readings were possible — the comment swallowed the backslash, or the backslash was not at the end of the line — and the leaf was opened to read the editions before ruling.
- **The three editions.** 1800-2017 §22.5.1: *"the newline character shall be preceded by a backslash ( \\ ). The first newline character not preceded by a backslash shall end the macro text"* (`docs/systemverilog/2017/md/section-22-compiler-directives.md:171-172`) and *"If a one-line comment … is included in the text, then the comment shall not become part of the substituted text"* (`docs/systemverilog/2017/md/section-22-compiler-directives.md:187-188`); 1364-2005 §19.3.1 the same (`docs/verilog/2005/md/section-19-compiler-directives.md:107-108`, `:159-160`). 1800-2023 §22.5.1 says the comment case OUTRIGHT — *"If a one-line comment is followed by a backslash and newline character, the comment ends before the backslash and the macro text continues on the next line"* (`docs/systemverilog/2023/md/section-22-compiler-directives.md:200-201`) — and tightens the continuation to *"immediately preceded by a backslash"* (`docs/systemverilog/2023/md/section-22-compiler-directives.md:173`).
- **Measured on scratch repros, both profiles, before any change** (`rust/target/svpp_5r/`): a `// comment \` line with the backslash immediately before the newline — `parse_full passed` under `sv_2017` and `sv_2023`, the expansion shows the comment dropped and the text continued (the stage already reads what 2023 :176-177 says); the same line with a tab and spaces after the backslash — rejected under both, the macro text ends at that line and the continuation lines are emitted as source; a plain macro line with trailing white space after its backslash — the same. **So the Surelog row's barrier is the trailing white space, not the comment**, and the strict text of every edition says that newline is not preceded by a backslash.
- **The corpus, counted with an implementation-equivalent scanner** (every `` `define `` chain over 16 435 source files, a backslash followed by white space before the newline): **2 sites, both tests** — verilator `test_regress/t/t_lint_bsspace_bad.v:10`, verilator's own lint-error test for exactly this shape (`BSSPACE`), and the Surelog line. No design file writes it. No stage option is added; the strict reading stands and the Surelog row keeps its honest rejection.
- **Pinned both ways.** The control pin holds the licensed shape green on both SV profiles; the invalid pin holds the trailing-white-space twin red on both. `ADJUDICATION-REPROS: checked=1894 armed=236 listed=927 multi_profile_rows=718 preprocess_rows=24 tolerated_rows=8 superset_rows=719 failures=0` on the release probe.

**Acceptance checklist (`TASK-ACCEPTANCE`):**

  - [x] **REPRODUCE / ISSUE** — `rust/target/svpp_5r/comment_bs_ws.sv` on the release probe: `parse_full rejected … Parser did not consume full input`, the expansion carrying `$display(a); \` as source; `comment_bs.sv` (no trailing space) `parse_full passed`.
  - [x] **ROOT CAUSE (WHY + WHERE)** — WHERE: the `` `define `` reader's continuation test, a backslash immediately before the newline (the text of all three editions). WHY: the Surelog line's backslash is followed by a tab and three spaces; the newline is not preceded by a backslash, so the macro text ends — the strict reading, not a defect. Signature (correctness family): `furthest_position` at the leaked `begin\` line in the lane dump (`macros.v:6:1`); the scratch pair above isolates the white space as the only difference.
  - [x] **FIX** — none required (a RULING); PINS tier: the licensed shape and its twin pinned on both SV profiles.
  - [x] **ADDRESSED (verified)** — the two pins behave as keyed on the release probe (`failures=0` over 927 rows); the three editions' sentences and the two-site census recorded here and in the pipeline chapter.
  - [x] **NO REGRESSION** — no engine, grammar or generated byte moved (parser fingerprint `a5b5b9dc5c8ee247` unchanged); the 925 pre-existing pins as keyed; no lane row or tuple touched; `bash scripts/check_doctrines.sh` → `doctrines: ALL 46 enforced doctrines PASS`.
  - [x] **LOCKSTEP** — this section + the row (`.5`(r) done as a ruling), the two pins + `MANIFEST.tsv`, the SV book's pipeline chapter (+ tracked HTML), `docs/TASK_TREE.md`, `MEMORY.md`, `CHANGES.md`, `DEVELOPMENT_NOTES.md`.
  - [x] **NEUTRALITY TIER** — SystemVerilog's own pins; no engine byte.
  - promotion: declined (see `DEVELOPMENT_NOTES.md`).

## `.6d.8` VeeR's flow predecessors — ✅ done 2026-10-06 (`PGEN-SVPP-EXPANSION-0040`): the generated configuration header the Makefile compiles first, which no file includes, supplied as the suite's unit predecessor; three rows reach the parser's acceptance

**What landed** (CONTEXT tier — `stimuli/sv/pipeline_context.py` (`HAND_UNITS["Cores-VeeR-EL2"]`: three sourced rows, the Makefile's `defines` list in its order), `stimuli/sv/pipeline_context.tsv` (81 → 84 rows), the derived reports of the THREE lane readings, both censuses and the rejects-own-file census, the SV book's pipeline chapter (+ tracked HTML), the platform book's verdict-coverage anchor; ZERO Rust / grammar / generated bytes):

- **How it was found.** `.6e`'s census filed three VeeR rows under `raw_backtick`: `testbench/tb_top.sv:996` `` `RV_RESET_VEC ``, `testbench/axi_lsu_dma_bridge.sv:82` `` `RV_ICCM_BITS ``, `design/lib/beh_lib.sv:770` `` `TEC_RV_ICG ``. The macros are defined in `snapshots/default/common_defines.vh`, a GENERATED header (`veer -target=default -snapshot=default`, its own first line). No VeeR source file `include`s it (`grep -rl common_defines` over the suite's `.sv/.svh/.v/.vh`: none), so the include derivation cannot see it; the only text that names it is the flow: `tools/Makefile:167` `defines  = $(BUILD_DIR)/common_defines.vh`, `:168-169` `el2_def.sv`, `el2_pdef.vh`, and `:210` passes `${BUILD_DIR}/common_defines.vh` as the VCS build's first source after `+incdir+${RV_ROOT}/design/include`. The same shape as `.6d.6`'s OpenTitan predecessors — a file the BUILD puts in front of every unit — in a second suite.
- **The row.** The flow's `defines` list, in its order, as the suite's hand unit predecessors, each with its Makefile line as the source. The two others carry no directive — `el2_def.sv`'s include guard is commented out (`//`ifndef EL2_DEF_SV`), `el2_pdef.vh` is parameter text (0 defines) — and are listed for fidelity to the flow; measured with and without them, the verdicts are the same.
- **Pre-measured by hand, two-sided** (the release probe, each row's `context_args` + the three `--unit-file` flags): the 82 passing VeeR rows → rc 0, every one; the 20 non-pass rows → 3 rc 0 (`beh_lib.sv`, `axi_lsu_dma_bridge.sv`, `tb_top.sv`), 17 rc 1 unchanged — the three `el2_param.vh:182:22` stops (a typedef of `el2_pkg` inside an included header: the declaration barrier), the eleven `uvm/mem/hdl` rows (UVM classes, the barrier), the two `csr_equ` fragments, the two riscv-dv files.
- **Predictions, written with a clock before the chain** (`rust/target/svpp_6d8/prereg.md`): `moved=3 {('Cores-VeeR-EL2', 'fail->pass'): 3}` under EACH reading, the three rows named; zero `pass → fail`; `LIBRARY-MOVERS {}`; the re-adjudication `chained:rejects_own_file` 280 → 277, `chained:accepted_under_expansion` 2401 → 2404; the tuple `11173/2606/2557/1270/0` → `11176/2606/2554/1267/0`; both census differentials unmoved; the rejects-own-file census `rows` 310 → 307, `raw_backtick` 47 → 44. **EVERY ONE HELD** — including that arm (4) of `PIPELINE-LANE-CURRENCY` read `✗ FAIL` on the stale census after the readings and `OK` once the census step re-derived it (the chain's `pipeline_currency rc=1`, then `pipeline_currency2` OK): the arm doing its job on its first live run.
- **Measured on the three readings** (the `.5`(q) debug probe `91de33b51d32…`, no Rust byte moved; `timeout_s=600 jobs=6`, each under the guard, sequential):
  - `sv_2017`: `PIPELINE-LANE: files=16336 pass=12753 fail=3320 pp_fail=263 timeout=0 crash=0 wall_s=1255 … profile=sv_2017` (was `pass=12750 fail=3323`); `PIPELINE-LIBRARY: Cores-VeeR-EL2=2/2 … published_packages=277` unchanged.
  - `sv_2023`: `PIPELINE-LANE: files=16336 pass=12793 fail=3280 pp_fail=263 … wall_s=1278 … profile=sv_2023` (was `pass=12790`); `published_packages=278` unchanged.
  - `sv_2023_tolerant`: `PIPELINE-LANE: files=16336 pass=12811 fail=3262 pp_fail=263 … wall_s=1190 … profile=sv_2023_tolerant` (was `pass=12808`); `published_packages=283` unchanged.
  - `PIPELINE-MOVERS[sv_2017] rows_before=16336 rows_after=16336 moved=3 added=0 lost=0 {('Cores-VeeR-EL2', 'fail->pass'): 3}` — `design/lib/beh_lib.sv`, `testbench/axi_lsu_dma_bridge.sv`, `testbench/tb_top.sv`, THE THREE ROWS NAMED BEFORE THE MEASUREMENT; `[sv_2023]` and `[sv_2023_tolerant]` the SAME three; ZERO `pass → fail` under any reading; `LIBRARY-MOVERS {}` under each; `PP-FAIL-REASON-MOVERS rows=0`.
  - The primary reading's re-adjudication: `chained:rejects_own_file` 280 → 277, `chained:accepted_under_expansion` 2401 → 2404, every other class as it was; the verdict-coverage tuple under the pipeline `11173/2606/2557/1270/0` → `11176/2606/2554/1267/0` (the raw tuple `7485/2606/6245/3864/0` unmoved; `check_sv_corpus_denominator.sh` refused the stale anchor, then OK). The rejects-own-file census re-derived: `rows=307 raw_backtick=44 … residue=26` (was 310 / 47; every other class unchanged — the three VeeR macros left the raw-backtick class and nothing else moved).
  - `PIPELINE-SV2023-CENSUS: rows=16336 both_pass=12753 both_nonpass=3543 2023_only_pass=40 2017_only_pass=0 population_breaches=0 | library: files=407 2023_only_pass=3 2017_only_pass=0 published_2017=277 published_2023=278 | members: unreached_2017=498 reached_2023=51 lost_2023=0`; `PIPELINE-TOLERANT-CENSUS: rows=16336 both_pass=12793 both_nonpass=3525 tolerant_only_pass=18 2023_only_pass=0 population_breaches=0 | library: files=407 tolerant_only_pass=7 2023_only_pass=0 published_2023=278 published_tolerant=283 | members: unreached_2017=498 reached_tolerant=337 lost_tolerant=0` — both differentials unmoved.
  - `IDENTITY-CARRIERS: OK (live=16 frozen=7 rows_equal=84 notes=7 not_evaluated=0)`; `corpus-key-integrity` rc 0; `PIPELINE-LANE-CURRENCY: OK` (all four arms, after the census re-derivation); the chain 08:51 → 11:54 under the guard, sequential.
- **What this slice does NOT do.** The 17 VeeR rows that still reject are the declaration barrier (`el2_pkg`'s typedefs read through `el2_param.vh`; UVM classes) and two fragments — the library's business, not the context's. The snapshot is the ONE configuration the corpus carries (`default`); VeeR's other targets generate other headers, which the lane cannot read.

**Acceptance checklist (`TASK-ACCEPTANCE`):**

  - [x] **REPRODUCE / ISSUE** — `pipeline_results.tsv` at `9ddbe7b51`: `Cores-VeeR-EL2 fail testbench/tb_top.sv 24800 …tb_top.sv:996:25`; the expanded text at 24800 begins `` `RV_RESET_VEC; `` (the `.6e` census, `raw_backtick`).
  - [x] **ROOT CAUSE (WHY + WHERE)** — WHERE: `HAND_UNITS` carried no VeeR row; the include derivation reads `` `include `` lines and the header is included by none. WHY: a build-command predecessor leaves no trace in the sources — only the Makefile names it. Signature (correctness family): the probe by hand on `tb_top.sv` — without the unit file `parse_full rejected … furthest_position=24800`; with `--unit-file …/common_defines.vh` `parse_full passed`; `grep -rl common_defines` over the suite's source files: none.
  - [x] **FIX** — CONTEXT tier: three sourced `HAND_UNITS` rows, derived into the tracked context by `--write`; no engine, grammar or generated byte.
  - [x] **ADDRESSED (verified)** — the three rows `fail → pass` under EACH of the three readings, the three named in writing before the chain; the census's `raw_backtick` 47 → 44, the three VeeR macros gone from it.
  - [x] **NO REGRESSION** — zero `pass → fail` over 16 336 rows under EACH of the three readings (`moved=3`, all `fail → pass`); the 82 passing VeeR rows pre-measured rc 0 with the predecessors; `LIBRARY-MOVERS {}` under each; both census differentials unmoved (`2023_only_pass=40`, `tolerant_only_pass=18`, `lost_*=0`); `IDENTITY-CARRIERS: OK`; `PIPELINE-LANE-CURRENCY: OK`; `pipeline_context.py --self-test` 27 arms unchanged; ZERO Rust / grammar / generated bytes — every generated artifact byte-identical (parser fingerprint `a5b5b9dc5c8ee247` unchanged); `bash scripts/check_doctrines.sh` → `doctrines: ALL 46 enforced doctrines PASS`.
  - [x] **LOCKSTEP** — this section + the row (`.6d.8` done), the context source and its tracked table, the derived reports of the three readings, both censuses and the rejects-own-file census, the SV book's pipeline chapter (the predecessors sentence and the census paragraph; + tracked HTML), the platform book's verdict-coverage anchor, `docs/TASK_TREE.md`, `MEMORY.md`, `CHANGES.md`, `DEVELOPMENT_NOTES.md`.
  - [x] **NEUTRALITY TIER** — a corpus instrument under `stimuli/sv/`; the engine moved no byte.
  - promotion: declined (see `DEVELOPMENT_NOTES.md`).

## `.6e` the rows the parser rejects in their own file, classed by what it actually saw — ✅ done 2026-10-06 (`PGEN-SVPP-EXPANSION-0039`): 310 "parser rejections", of which 47 are missing build defines, 55 are include fragments, 154 the cross-file declaration barrier, 12 the newer text's or the tolerant profile's, 16 tests that are not SystemVerilog, and 26 adjudicated one by one

**What landed** (INSTRUMENT tier — `docs/tasks/artifacts/sv_corpus_grad/rejects_own_file_census/census.py` (the census: every `rejects_own_file` row of the re-adjudication re-expanded under the lane's own derived context, classed MECHANICALLY by the token at the furthest position in the EXPANDED text and by the three readings' dumps; 23 self-test arms; `--check` re-hashes its identity block, `--check --full` re-derives) + its tracked `census.md` / `census.tsv`; `scripts/check_pipeline_lane_currency.sh` (arm (4): the census's identity names the table and dumps on disk; self-test 5 → 6 arms, the sixth a table edit only arm (4) can see); `scripts/check_doctrines.sh` (the registry text), `DOCTRINE_ENFORCEMENT.md`, `docs/book/src/the-enforced-doctrines.md` (re-derived), `TOOLBOX.md` (the lane section), the SV book's pipeline chapter (the residue paragraph replaced by the census reading; + tracked HTML); ZERO Rust / grammar / generated bytes):

- **How it was found.** The resume pointer named `chained:rejects_own_file=280` + `svpp_owned:rejects_own_file=30` as the next class to census, "grouped by stop construct". The re-adjudication's `construct` column reads the SOURCE line the stop maps back to; it classed 143 rows as a base-class header and 138 as `other`. The source line is not what the parser saw: the parser saw the EXPANSION. Re-expanding the 138 `other` rows with `ast_pipeline --preprocess systemverilog` under each file's `context_args` and reading the token at `furthest_position` sorted them in one pass — and 29 of them began with a backtick.
- **The classes, in the order the census applies them** (first match wins; each mechanical, each with a self-test arm): `raw_backtick` (the token at the stop carries a backtick — `` `X `` or `` 32'h`X `` glued: an undefined macro reached the parser, the card `a-raw-backtick-at-the-furthest-position-is-a-context-defect`); `base_class`; `edition_2023` (the row passes under `sv_2023`); `tolerated_spelling` (passes under `sv_2023_tolerant` only); `edition_2023_moves` / `tolerated_moves` (the later reading's `furthest_position` is GREATER than the earlier one's: the construct at THIS stop is the newer text's or the profile's, a later barrier remains); `error_test` (the answer key's intended-invalid name pattern); `preprocess_only` (verilator: the same-stem driver passes `-E`); `declared_fragment` (verible's `// verilog_syntax: parse-as-module-body`); `fragment` (the file is the `include` target of another corpus source file); `undeclared_type` (an identifier that follows a non-keyword identifier, with a qualifier and an optional `pkg::` / `#(…)`: a declaration whose TYPE the parser never saw); `residue`.
- **The reading of 2026-10-06** (`REJECTS-OWN-FILE-CENSUS: rows=310 raw_backtick=47 base_class=143 edition_2023=4 tolerated_spelling=5 edition_2023_moves=3 tolerated_moves=0 error_test=6 preprocess_only=5 declared_fragment=5 fragment=55 undeclared_type=11 residue=26`):
  - **`raw_backtick` 47** — by macro: OpenTitan's per-IP build defines (`` `EN_MASKING `` ×6, `` `RNG_BUS_WIDTH `` ×4, `` `NUM_END_POINTS `` ×2, `` `VERILATOR_TEST_STATUS_ADDR `` ×3, `` `MEM_BKDR_UTIL_FILE_OP `` ×2, `` `EN_GCM ``, `` `SEC_VOLATILE_RAW_UNLOCK_EN ``, `` `SRAM_TYPE ``, `` `INSTR_EXEC ``, `` `DM_ADDR ``, `` `INOUT_AI `` / `` `INOUT_AO ``, `` `DATA_LENGTH ``, `` `NUM_REQUESTS ``), every one a `+define+` in the IP's `dv/*_sim_cfg.hjson` `build_opts` with CONTRADICTING values across build modes — **the parked `.6d.5`, now with its row count**; VeeR's `` `RV_RESET_VEC `` / `` `RV_ICCM_BITS `` / `` `TEC_RV_ICG `` — defined in `snapshots/default/common_defines.vh`, which `tools/Makefile:167,210` compiles FIRST (`defines = $(BUILD_DIR)/common_defines.vh`; the flow's first source argument) — **`.6d.8` OPENED**, and pre-measured: `tb_top.sv` with `--unit-file …/common_defines.vh` reads `parse_full passed`; two BlackParrot `pkgdef.svh` chains behind the unvendored BaseJump STL (`.4d`); friscv's `` `SB `` (its own `friscv_h.sv`, not included by `friscv_memfy_h.sv`); three tool-specific or error-test spellings (`` `resetall `` where it is illegal, `` `error ``, `` `NDEFINED ``); two `` `` `` pastes in plain text (friscv's testbenches: `$sformat(tcname, "%s", ``TCNAME)` — §22.5.1 defines `` `` `` inside macro text only; a tool extension, recorded, not admitted).
  - **`base_class` 143** (126 of them include fragments of their package) and **`undeclared_type` 11** — the cross-file DECLARATION barrier (`SV-PARSE-STRICT.2`'s store gate), in the class-header and the data-declaration position; the library flags exist for it, and the membership reading already credits a member whose unit passes.
  - **`fragment` 55** — `riscv_core_setting.sv` ×13 (a riscv-dv target file `include`d by `riscv_instr_pkg.sv`), the `xbar` autogen pair ×19 (`xbar_env_pkg__params.sv`, `tb__xbar_connect.sv`), VeeR's `el2_dec_csr_equ_*.svh` ×8, and the rest: files whose only legitimate reading is through their includer.
  - **`edition_2023` 4 + `edition_2023_moves` 3** — a parameter's unsized `[]` (`parameter string LIST[] = {…}`: 1800-2023 A.2.4 `param_assignment ::= parameter_identifier { variable_dimension } …`, 1800-2017 `{ unpacked_dimension }` — `docs/systemverilog/2023/txt/section-6-data-types.txt:2062-2064` vs `docs/systemverilog/2017/txt/section-65536-216-bits.txt:871-873`; the grammar already carries both, `param_assignment_sv_2017` / `_sv_2023`), `` `begin_keywords "1800-2023" `` ×2, `t_preproc_preproczero_bad.v`. **`tolerated_spelling` 5** — the `flash_ctrl_env_pkg` trio and the two `core_ibex` coverage interfaces, the profile's dotted cross items.
  - **`error_test` 6, `preprocess_only` 5, `declared_fragment` 5** — not SystemVerilog to parse, by the suite's own convention.
  - **`residue` 26, adjudicated by hand**, each against the in-repo text:

    | rows | files | the stop | the ruling |
    |---:|---|---|---|
    | 2 | Surelog `ElabSysCall/dut.sv`, ibex `ibex_top_tracing.sv:123` | `$fatal("…")` | NOT derivable in 1800-2017: §20.10 `fatal_message_task ::= $fatal [ ( finish_number [ , list_of_arguments ] ) ]`, `finish_number ::= 0 \| 1 \| 2` (`section-20-…:924`) — the first argument is the finish number, a string is not one. Written by ibex's RTL wrapper, which its flows compile; which tools accept it is not measured here. → `SV-CORPUS-GRAD.46` candidate |
    | 2 | Surelog `GenerateAssigns/top.v`, `UnitTest/top.v` | `a ~& b` | invalid: `~&` is a unary reduction operator (§11.3), never binary — honest |
    | 1 | Surelog `SimpleInterface/simple_if.sv` | `modport` inside a `generate` | NOT derivable: `modport_declaration` is a `non_port_interface_item`, and a generate block's items are `interface_or_generate_item` (A.1.6 / A.4.2) — honest |
    | 1 | Surelog `TestMacros/macros.v` | a `` `define `` whose `// comment \` line ends in a backslash | the stage ends the macro at the comment line; the next line (`begin\`) reaches the parser as source. §22.5.1 says the text continues where "the newline is preceded by a backslash" and that a one-line comment "shall not become part of the substituted text" — the two sentences meet on this line. → `.5`(r) OPENED (parked): read the three editions and measure the tools before ruling |
    | 1 | Surelog `UnitForeach/top2.v` | `uvm_sequencer_base::do_print (` | an out-of-class method of a class the row never saw — the declaration barrier in `::` position (the shape test reads data declarations only) |
    | 1 | friscv `rtl/friscv_alu.sv:195` | `{01'b0, …}` | NOT derivable: §5.7.1 `size ::= non_zero_unsigned_number`, which begins with a non-zero digit (A.8.7) — a leading zero in a size. Written by friscv's RTL, which its own flow simulates; not measured here. → `.46` candidate |
    | 2 | OpenTitan `keccak_2share_fpv.sv:46`, verilator `t_sv_cpu_code/cpu.sv:225` | `mubi4_t sel_mux;`, `mPreAdrDecode_resp busproperty;` | the declaration barrier, read as an instantiation head by the parser, so the stop is at `;` and the shape test missed it — `undeclared_type` by hand |
    | 1 | OpenTitan `kmac_reduced_tb.sv:52` | `.EntropyWidth(EntropyWidth), )` | a trailing comma in a parameter assignment list — not derivable (A.4.1.1); pre_dv code — honest |
    | 2 | OpenTitan `otbn_top_sim.sv:168`, ibex `ibex_simple_system_cosim_checker_bind.sv:7` | `#(.ImemAddrWidth, .DmemAddrWidth)` | NOT derivable: A.4.1.1 `named_parameter_assignment ::= . parameter_identifier ( [ param_expression ] )` — the parentheses are required (the implicit `.name` form is a PORT connection's, §23.3.2.3). Both files are verilator testbenches, so verilator compiles the form; other tools not measured here. → `.46` candidate |
    | 1 | OpenTitan `otbn_mask_accelerator_sca_wrapper.sv:50` | `ctr_q <= '0; done_q <= 1'b1; else` | an `if` arm of two statements without `begin`/`end` — invalid; pre_sca code — honest |
    | 6 | OpenTitan `spi_device/pre_dv/{program,tb}/*.sv` | `static task automatic wait_trans();` in a `program` / `module` | NOT derivable: `task_declaration ::= task [ lifetime ] task_body_declaration` — the lifetime FOLLOWS the keyword; `static` before it is a `class_item_qualifier`, legal only inside a class (A.1.9). Written by OpenTitan's pre_dv programs; which tools accept it is not measured here. → `.46` candidate |
    | 1 | ibex `syn/rtl/latch_map.v:6` | `module $_DLATCH_P_` | a Yosys cell name; `$` cannot begin an identifier (§5.6) — honest |
    | 1 | slang `tests/unittests/data/file_uses_defn.svh` | a string at top level | unit-test data (`` `include `` + a macro yielding a string) — not SystemVerilog |
    | 1 | verilator `t_langext_order_sub.v:11` | `input do` | `do` is reserved in every SystemVerilog edition; the test selects Verilog-2001 per file (`+1364-2001ext+`), which the lane does not — honest under the profile |
    | 1 | verilator `t_preproc_persist2.v` | `Inside "…".` | the `-E` sibling of `t_preproc_persist.v` with no driver of its own — not SystemVerilog |
    | 2 | verilator `uvm/uvm_pkg_all_v2017_1_0_{dpi,nodpi}.svh:18004` | `#setting.offset;` | NOT derivable: `delay_value ::= unsigned_number \| real_number \| ps_identifier \| time_literal \| 1step` — a member select needs `#(…)` (A.6.5). UVM 1.0's spelling; the 2020.3.1 library does not write it (`grep` finds none). → `.46` candidate, lowest |

- **Measured, instrument level.** `REJECTS-OWN-FILE-CENSUS-SELF-TEST: checks=23 failed=0`; the census derived in 22 s (310 expansions); `--check` (identity) OK; `--check --full` (re-derivation) OK; `PIPELINE-LANE-CURRENCY-SELF-TEST: arms=6 failed=0` (arm F: a trailing line appended to the re-adjudication table — invisible to arm (3)'s render, which the table equals — is refused by arm (4) as a moved input); `check_pipeline_lane_currency.sh` → `(4) REJECTS-OWN-FILE CENSUS — … OK — identity holds (inputs re-hashed)`, `PIPELINE-LANE-CURRENCY: OK`.
- ⛔ **A detail the census corrected in itself.** Its first token reader skipped spaces and tabs only, so a stop whose expansion reads `\n` and then the backtick (nine rows: the parser's furthest position sits at the end of the last token it accepted, before the trivia) was `<end of line>` — 36 raw-backtick rows, not 45; and a backtick glued to a based prefix (`32'h`DM_ADDR`) hid behind the `h`. The reader now skips all white space and the backtick test reads the whole token run: 47. The first census by eye had said 29.
- **What this slice does NOT do.** It changes no verdict, no lane row, no tuple — it classes rows the re-adjudication already carries. The 47 context rows stay `rejects_own_file` in the re-adjudication table (its verdict vocabulary is the coverage tuple's; renaming a class there is the denominator leaf's decision, and the census is the reading that would justify it). It admits nothing to the tolerant profile: the five candidates are recorded with their clause for `SV-CORPUS-GRAD.46`, parked, not SV-blocking.

**Acceptance checklist (`TASK-ACCEPTANCE`):**

  - [x] **REPRODUCE / ISSUE** — `stimuli/sv/characterization/pipeline_readjudication.tsv` at `3250f96c4`: 310 rows with a verdict ending `rejects_own_file`, 138 of them `construct=other`; by hand, `ast_pipeline stimuli/sv/subs/opentitan/hw/ip/aes/dv/tb/tb.sv --preprocess systemverilog --output … --unit-file … --include-dir …` and the text at `furthest_position=12122` begins `` `EN_MASKING `` — a parser "rejection" of an undefined macro.
  - [x] **ROOT CAUSE (WHY + WHERE)** — WHERE: `pipeline_readjudication.py` classes a failing row by `pv.origin_bucket(line)` over the SOURCE line at the mapped origin. WHY: the parser read the EXPANSION; a stop on a macro use site maps back to a line whose text says nothing about what stood there after expansion, so a missing define, a fragment and a parser defect all read `other`. Signature (correctness family): the probe's `furthest_position=12122` on `aes/dv/tb/tb.sv` and the expanded text at that offset, `` `EN_MASKING   ), ``; the parked `.6d.5` had measured the same macros as contradicting build-mode defines.
  - [x] **FIX** — INSTRUMENT tier: a derived, tracked census with an identity block, a self-test, and a currency arm; no engine, grammar or generated byte.
  - [x] **ADDRESSED (verified)** — the 310 rows carry a class each; 47 context rows are named by macro; the 26 residue rows are ruled one by one with the clause; `--check --full` re-derives the tracked census byte-identical.
  - [x] **NO REGRESSION** — no lane row, verdict or tuple moved (`stimuli/sv/characterization/*` untouched; `PIPELINE-LANE-CURRENCY: OK` on all four arms; the verdict-coverage tuple `11173/2606/2557/1270/0` as `.4d` left it); the currency self-test's five prior arms unchanged; ZERO Rust / grammar / generated bytes — every generated artifact byte-identical (parser fingerprint `a5b5b9dc5c8ee247` unchanged); `bash scripts/check_doctrines.sh` → `doctrines: ALL 46 enforced doctrines PASS`.
  - [x] **LOCKSTEP** — this section + the rows (`.6e` done, `.6d.8` opened, `.5`(r) opened parked), the census and its tracked artifacts, the currency script and the doctrine's registry text, `DOCTRINE_ENFORCEMENT.md`, the enforced-doctrines page (re-derived), `TOOLBOX.md`, the SV book's pipeline chapter (+ tracked HTML), `SV-CORPUS-GRAD.46` (opened, parked), `docs/TASK_TREE.md`, `MEMORY.md`, `CHANGES.md`, `DEVELOPMENT_NOTES.md`.
  - [x] **NEUTRALITY TIER** — a corpus instrument under `docs/tasks/artifacts/sv_corpus_grad/`; the engine moved no byte.
  - promotion: declined (see `DEVELOPMENT_NOTES.md`).

## `.4d` the context derivation's candidate population, identical copies, and the shadow guard — ✅ done 2026-10-06 (`PGEN-SVPP-EXPANSION-0038`): the lane's refused-context rows grouped by what the context fails to supply; two defects of the DERIVATION, and a hazard the fix itself created and the new guard caught

**What landed** (INSTRUMENT tier — `stimuli/sv/pipeline_context.py` (the include-candidate population is EVERY file of the tree; a tie of byte-identical copies resolves per file; the SHADOW guard: a suite directory that holds an ambiguous name REFUSES the derivation, exit 4, and a per-file directory in that position is WITHHELD for that includer; seven self-test arms, one a red control), the re-derived `stimuli/sv/pipeline_context.tsv` (80 → 81 rows) and `pipeline_context_files.tsv` (41 → 42), the derived reports of the THREE lane readings and both censuses, the SV book's pipeline chapter (+ tracked HTML), the SV integration contract's context paragraph, the platform book's verdict-coverage anchor; ZERO Rust / grammar / generated bytes):

- **The census** (the primary reading's `pipeline_results.tsv` at `22a2f044b`, every `pp_fail` row — the `refused_context` rows of the re-adjudication carry no reason column, the lane's dump does): **268 refusals.** 221 are `failed to resolve include` over 24 names: **168 black-parrot rows name `bsg_defines.sv` / `bsg_noc_links.svh` / `bsg_cache.svh` — BaseJump STL, a library the corpus does not vendor; honestly absent on the volume, and left so.** 7 OpenTitan rows name a top-specific file (`riscv_core_setting.sv`, 12 copies, 11 texts; the `xbar` autogen pair, 11 texts each; three `*_base_vseq.sv` from `ip_templates`, 2 texts each) and 4 a build output (`../build/psgen-jg.sv`, `sail.sv`, `formal_tb_frag.svh`); 8 friscv rows name `HardFloat_specialize.vi` (3 copies, 3 texts, equidistant from `source/*.v`: the build picks the target); the rest are error tests by name (`nonexistent.svh`, `does_not_exist.sv`, `this_file_is_not_found.vh`, `non-existant.v`), a generated header (`portlist.vh`), a tool's own include tree (`verilated.v`, `dpi/uvm_dpi.svh`), a sibling repository (`svlogger.sv`) and one Surelog snippet (`line.vh`). 13 are unbalanced conditionals — twelve error tests by name and Surelog `PreProcMacroFunc/dut.sv`, a snippet with a stray `` `endif `` at `:7`. 34 are macro-arity / recursion / formal-list error tests, the four include loops of `.5`(q), and two unquoted include paths. **And 25 rows name a file the tree HOLDS**: `HardFloat_consts.vi` (22 friscv rows, ONE copy, `dep/hardfloat/source/`), `constants.vams` (2 iverilog rows, ONE copy, the suite root), and `svut_h.sv` (1 friscv row, SIX copies, every one the same bytes).
- **WHERE (i).** `derive()` built the candidate population from `source_files(base)` — `.sv` / `.svh` / `.v` / `.vh` only — so a header with any other suffix was reported `absent` while the tree held exactly one copy. IEEE 1800-2017 §22.4 constrains neither the filename nor its suffix (`docs/systemverilog/2017/txt/section-22-compiler-directives.txt:82-103`: a quoted or bracketed *filename*); Berkeley HardFloat writes `.vi`, iverilog's `ivtest` writes `.vams` and `.vi`, VeeR writes a `.h`. **FIX:** `all_files(base)` — every regular file — is the candidate population; the INCLUDER population stays the source files (a non-source header's own includes are not derived; the four headers this slice resolves include nothing, measured by `grep`). The derivation's own `--suite` reports had printed the defect for weeks: `absent HardFloat_consts.vi (x22)`, `absent constants.vams (x2)`, `absent ivltests/br_gh782a.vi (x4)`.
- **WHERE (ii).** A name with several candidate directories equidistant from the includer was a tie, reported ambiguous, even when every copy was the same bytes — `svut_h.sv` six times under `friscv/test/*/`, OpenTitan's `*_vseq_list.sv` under its tops (2–4 copies, one text). The choice cannot change the expansion, so there is nothing to guess. **FIX:** `identical_text()` — a tie whose copies are one text resolves PER FILE (a `.4b` row: the nearest copy, the first on a tie) and is reported `identical NAME: N copies, one text`.
- ⛔ **The hazard the fix created, and the guard that caught it.** The first cut of (ii) emitted the chosen directory SUITE-WIDE. A new guard written for the same slice — a directory emitted for one name that also holds an AMBIGUOUS name would let the preprocessor resolve that name silently, a guess by the back door — REFUSED the real tree on its first run: OpenTitan's `top_darjeeling/ip_autogen/pwrmgr/dv/env/seq_lib`, chosen for `pwrmgr_vseq_list.sv` (4 copies, one text), also holds `pwrmgr_base_vseq.sv` and `pwrmgr_smoke_vseq.sv` (3 copies, 2 texts). Redesigned as above (per file, never suite-wide). The per-file form of the same hazard then fired once: `hw/ip_templates/pwrmgr/dv/env/seq_lib/pwrmgr_vseq_list.sv` includes vseqs that are one text across the tops AND `pwrmgr_base_vseq.sv`, which differs per top — the directory for the former would hand it the latter. That per-file directory is WITHHELD for that includer, reported (`withheld … the file keeps its refusal`), and the template keeps its honest refusal. A suite directory in that position REFUSES the whole derivation (exit 4; zero today, the self-test's red control exercises it). The shadow census over the tracked table BEFORE this slice: 0 pairs — the hazard was latent, not live.
- **Predictions, written with a clock before the derivation was changed** (`rust/target/svpp_4d/prereg.md`, 2026-10-06 05:18, HEAD `22a2f044b`), and the deviations recorded there before the lane ran: 27 self-test arms, not 24 (a control arm, the red control and the withholding arm were added while writing); the `svut_h.sv` row is a per-file row, not a suite directory (the redesign above); VeeR's existing directory row gains `pic_map_auto.h` and iverilog's `ivtest` row gains `br_gh782a.vi`, both already resolved through directories emitted for other names (no lane row can move for them). The lane prediction: under EACH reading `moved=5` — `pp_fail → pass` ×4 (friscv `source/8086-SSE/HardFloat_specialize.v`, `source/ARM-VFPv2/HardFloat_specialize.v`; iverilog `ivltests/test_va_math.v`, `test_vams_math.v`, each pre-measured with the probe and the directory by hand), `pp_fail → fail` ×1 (friscv `test/common/friscv_testbench.sv`, pre-measured `furthest_position=30940`); ZERO `pass → fail`; ~20 friscv rows keep `pp_fail` with the reason moving from `HardFloat_consts.vi` to `HardFloat_specialize.vi`; `LIBRARY-MOVERS {}`; both census differentials unmoved. **EVERY ONE HELD.**
- **Measured, instrument level.** `PIPELINE-CONTEXT-SELF-TEST: checks=27 failed=0` (was 20); `--write` rc 0 after the redesign (rc 4, REFUSED, on the first cut — the guard's first real catch); `--check`: `PIPELINE-CONTEXT: OK — the tracked context equals a fresh derivation`. The suite lines: `friscv … include_dirs=1 per_file_rows=1 … ambiguous_nonlocal_names=1 absent_names=1 identical_names=1 shadowed=0 withheld=0` (was `include_dirs=0 per_file_rows=0 ambiguous=1 absent=3`: `HardFloat_specialize.vi` is now ambiguous, as it is, and `svlogger.sv` the one absent); `iverilog … include_dirs=2 … absent_names=1` (was 1 / 3); `opentitan … per_file_rows=24 names_resolved_per_file=12 ambiguous_nonlocal_names=17 absent_names=4 identical_names=7 shadowed=0 withheld=1` (was 24 / 10 / 19 / 4); `Surelog … identical_names=1` (`bar.vh`, already per-file). The tracked rows: `+friscv include_dir stimuli/sv/subs/friscv/dep/hardfloat/source 22 HardFloat_consts.vi`, `+iverilog include_dir stimuli/sv/subs/iverilog 2 constants.vams`, `+friscv … friscv_testbench.sv stimuli/sv/subs/friscv/test/apps svut_h.sv` (per file).
- **Measured on the three readings** (the debug probe `91de33b51d32…` of `.5`(q), no Rust byte moved; `timeout_s=600 jobs=6`, each under the guard, sequential):
  - `sv_2017`: `PIPELINE-LANE: files=16336 pass=12750 fail=3323 pp_fail=263 timeout=0 crash=0 wall_s=1017 … profile=sv_2017` (was `pass=12746 fail=3322 pp_fail=268`); `PIPELINE-LIBRARY: Cores-VeeR-EL2=2/2 black-parrot=3/7 opentitan=372/396 uvm-core=2/2 published_packages=277 unresolved_imports=44 cyclic_files=0 member_rows=2392` unchanged.
  - `sv_2023`: `PIPELINE-LANE: files=16336 pass=12790 fail=3283 pp_fail=263 … wall_s=1012 … profile=sv_2023` (was `pass=12786 fail=3282 pp_fail=268`); `published_packages=278` unchanged.
  - `sv_2023_tolerant`: `PIPELINE-LANE: files=16336 pass=12808 fail=3265 pp_fail=263 … wall_s=1220 … profile=sv_2023_tolerant` (was `pass=12804 fail=3264 pp_fail=268`); `published_packages=283` unchanged.
  - `PIPELINE-MOVERS[sv_2017] rows_before=16336 rows_after=16336 moved=5 added=0 lost=0 {('friscv', 'pp_fail->pass'): 2, ('friscv', 'pp_fail->fail'): 1, ('iverilog', 'pp_fail->pass'): 2}` — `source/8086-SSE/HardFloat_specialize.v`, `source/ARM-VFPv2/HardFloat_specialize.v`, `test_va_math.v`, `test_vams_math.v` to `pass`; `test/common/friscv_testbench.sv` to `fail` — THE FIVE ROWS NAMED BEFORE THE MEASUREMENT; `[sv_2023]` and `[sv_2023_tolerant]` the SAME five; ZERO `pass → fail` under any reading; `LIBRARY-MOVERS {}` under each. The reason movers: 20 friscv rows keep `pp_fail` with `HardFloat_consts.vi` replaced by `HardFloat_specialize.vi` in the refusal — the honest tie behind the resolved header, exactly the 20 predicted.
  - The primary reading's re-adjudication: `chained:refused_context` 211 → 208, `chained:accepted_under_expansion` 2399 → 2401 (the two HardFloat files), `chained:rejects_own_file` 279 → 280 (the friscv testbench, now a parser stop at `furthest_position=30940` for the leaf that reads that class); every other class as it was (`corroborated=1334 refused_engine=4 refuted_awaiting_disposition=1`); the verdict-coverage tuple under the pipeline `11171/2606/2559/1272/0` → `11173/2606/2557/1270/0` (the two iverilog rows are keyed rows outside the re-adjudication population; the raw tuple `7485/2606/6245/3864/0` unmoved; `check_sv_corpus_denominator.sh` refused the stale anchor, then OK after the anchor moved).
  - `PIPELINE-SV2023-CENSUS: rows=16336 both_pass=12750 both_nonpass=3546 2023_only_pass=40 2017_only_pass=0 population_breaches=0 | library: files=407 2023_only_pass=3 2017_only_pass=0 published_2017=277 published_2023=278 | members: unreached_2017=498 reached_2023=51 lost_2023=0`; `PIPELINE-TOLERANT-CENSUS: rows=16336 both_pass=12790 both_nonpass=3528 tolerant_only_pass=18 2023_only_pass=0 population_breaches=0 | library: files=407 tolerant_only_pass=7 2023_only_pass=0 published_2023=278 published_tolerant=283 | members: unreached_2017=498 reached_tolerant=337 lost_tolerant=0` — both differentials unmoved.
  - `IDENTITY-CARRIERS: OK (live=16 frozen=7 rows_equal=84 notes=7 not_evaluated=0)`; `corpus-key-integrity` rc 0; `PIPELINE-LANE-CURRENCY: OK`; the chain 05:23 → 08:06 under the guard, sequential.
- **What this slice does NOT do, and says so.** BaseJump STL is not vendored: the 168 black-parrot rows stay `refused_context` until a corpus decision adds it (a submodule pin is not this leaf's to change). The 20 friscv `source/*.v` and `test/source/*.v` rows now stop at `HardFloat_specialize.vi`, an honest three-way tie the build resolves with a target directory — the derivation reports it and does not pick. A non-source header's own `` `include `` lines are not scanned (bound stated above). The re-adjudication's `refused_context` class still carries no reason column; the lane's dump does, and this census read it there.

**Acceptance checklist (`TASK-ACCEPTANCE`):**

  - [x] **REPRODUCE / ISSUE** — the census above; `python3 stimuli/sv/pipeline_context.py --suite friscv` on `22a2f044b` prints `absent HardFloat_consts.vi (x22)` while `ls stimuli/sv/subs/friscv/dep/hardfloat/source/HardFloat_consts.vi` exists; `--suite friscv` prints `ambiguous svut_h.sv: …` over six copies whose sha256 is one.
  - [x] **ROOT CAUSE (WHY + WHERE)** — WHERE: `derive()`'s candidate set built from `source_files(base)`; the tie rule with no reading of the candidates' bytes. WHY: the includer population's suffix filter was reused for the candidate population, a different question; a tie was judged by path count, not by content. Signature (correctness family, the probe by hand on the release probe): without the directory `preprocess_failed … failed to resolve include 'svut_h.sv'`; with `--include-dir stimuli/sv/subs/friscv/test/apps` the same file reads `parse_full rejected … furthest_position=30940` — the CONTEXT, not the engine, was the barrier; the two HardFloat `specialize.v` files and the two `.vams` tests read `parse_full passed` with the directory the fixed derivation emits. The hazard the fix created (a suite-wide directory holding an ambiguous name) had no reader at all — the guard is new.
  - [x] **FIX** — INSTRUMENT tier (the derivation and its self-test); no engine, grammar or generated byte.
  - [x] **ADDRESSED (verified)** — the derivation reports both names resolved; the four rows whose header the tree holds `pp_fail → pass` and the testbench `pp_fail → fail` under EACH of the three readings, the five rows named in writing before the chain; the 20 HardFloat rows now stop at the honest tie behind the resolved header.
  - [x] **NO REGRESSION** — zero `pass → fail` over 16 336 rows under EACH of the three readings (`moved=5`, every one out of `pp_fail`); `LIBRARY-MOVERS {}` under each; both census differentials unmoved (`2023_only_pass=40`, `tolerant_only_pass=18`, `lost_2023=0`, `lost_tolerant=0`); `IDENTITY-CARRIERS: OK (live=16 frozen=7 rows_equal=84 notes=7 not_evaluated=0)`; `corpus-key-integrity` rc 0; `PIPELINE-LANE-CURRENCY: OK`; the withheld OpenTitan template keeps its refusal; the self-test's 20 prior arms unchanged; the shadow guard's red control fires (exit 4) and the real tree reads `shadowed=0` on every suite; ZERO Rust / grammar / generated bytes — every generated artifact byte-identical (the probe's parser fingerprint `a5b5b9dc5c8ee247` unchanged since `1.0.250`); `bash scripts/check_doctrines.sh` → `doctrines: ALL 46 enforced doctrines PASS`.
  - [x] **LOCKSTEP** — this section + the row (`.4d` done), the derivation and its tracked tables, the SV book's pipeline chapter (the derivation paragraph and the refused-context census; + tracked HTML), the SV integration contract's context paragraph, the derived reports of the three readings and both censuses, the platform book's verdict-coverage anchor, `docs/TASK_TREE.md`, `MEMORY.md`, `CHANGES.md`, `DEVELOPMENT_NOTES.md`.
  - [x] **NEUTRALITY TIER** — a corpus instrument under `stimuli/sv/`; the engine moved no byte.
  - promotion: declined (see `DEVELOPMENT_NOTES.md`).

## `.5`(q) the lane's refused rows, grouped by reason — ✅ done 2026-10-06 (`PGEN-SVPP-EXPANSION-0037`): forty-seven refusals, twelve reasons, ten of them right; the two that were wrong are a bound keyed on the wrong property and an identifier class the stage did not read

**What landed** (ENGINE tier, the SystemVerilog family's own component — `rust/src/families/systemverilog/preprocessor.rs`: the include bound is the DEPTH limit alone, its refusal citing §22.4 and showing the chain's last steps (the "include cycle" refusal deleted); one macro-name token helper, `parse_macro_name_token`, used by `` `define ``, `` `undef ``, the conditional arms and the formal-list reader, and the usage scanner's escaped-name arm; six unit tests RED-first; two preprocess pins; the derived reports of all three lane readings; ZERO grammar / codegen / generated bytes):

- **The census** (`stimuli/sv/characterization/pipeline_readjudication.tsv`, the primary reading's `refused_engine` + `chained:refused_engine` + `svpp_owned:refused_engine` rows, normalised by stripping paths, numbers and quoted text): **47 rows, 27 spellings, 12 reason families.** Ten are RIGHT: `unterminated conditional block` ×5 and `` `elsif `` / `` `else `` / `` `endif `` without a match ×7 (sv2v `test/error/*`, verilator `*_bad`, and Surelog `PreProcMacroFunc/dut.sv:7` — a stray `` `endif `` with no `` `ifdef `` in the file, keyed `must_accept` on a golden log that tolerates it: spec outranks the golden); `E_SVPP_MISSING_MACRO_ARGUMENT` ×13 (ten Surelog unit rows on `mode.vh:36`, which copies the standard's own example INCLUDING the lines it marks *"// illegal, only one argument"*; sv-tests `22.5.1--define-expansion_6` / `_7` / `_12`, each `:should_fail_because`; sv2v `error/macro_underapplied.sv`); `E_SVPP_EXTRA_MACRO_ARGUMENT` ×4 (`_8` `:should_fail`, three `*_bad` / `error/*`); `E_SVPP_RECURSIVE_MACRO` ×3 (verilator `*_bad`); the formal-list refusals ×5 (`.5`(o)'s comment ×2, sv2v `error/macro_arg_bad_*` ×2, verilator `t_preproc_eof2_bad.v`); the `` `include `` refusals ×3 (OpenTitan `aes_tb_pkg.sv:39` — `` `REQUESTS_FILE `` comes from the testbench's command line: context; sv2v `error/include_apos.sv`, `error/include_filename_eof.sv`). Two are WRONG:
  - **(i) `include cycle detected` ×6** — slang `infinite_chain.svh`, sv2v `error/include_loop_1.sv` / `_2.sv`, verilator `t_preproc_inc_recurse_bad.v` (true loops), AND sv2v `test/lex/file_line.sv` and `test/lex/include_self.sv`, keyed `must_accept`: files that include THEMSELVES under a guard (`` `ifndef INCLUDED `` / `` `define INCLUDED `` / `` `include "file_line.sv" `` / `` `else `` …; `include_self.sv` nests five guarded levels). IEEE 1800-2017 §22.4: *"A file included in the source using the `include compiler directive may contain other `include compiler directives. The number of nesting levels for include files shall be finite. Implementations may limit the maximum number of levels to which include files can be nested, but the limit shall be at least 15."* (`docs/systemverilog/2017/txt/section-22-compiler-directives.txt:96-98`; 1800-2023 `:107-109`; 1364-2005 `…/verilog/2005/txt/section-19-compiler-directives.txt:397-398`). The bound the standard names is NESTING; the stage refused a file's REAPPEARANCE on the include stack — a stronger property than the clause's, which a guarded self-include violates while nesting once. The stage's limit is 64 levels (≥ 15).
  - **(ii) `missing macro name in `define` ×1** — iverilog `ivltests/escaped_macro_name.v:2`, keyed `must_accept`: `` `define \escaped "escaped name" `` and five more escaped names, two containing a backtick and one a quote (`` \`name ``, `` \` ``, `` \`\`" ``), used as `` `escaped ``, `` `\escaped ``, `` `\simple ``, `` `\`name ``, `` `\quote (text) ``. 1800-2017 §22.5.1 Syntax 22-2: `text_macro_identifier ::= identifier` (`:131-132`), and A.9.3's identifier is simple OR escaped; 1800-2023 §22.5.1 says it outright — *"The macro name may be either a simple_identifier or an escaped_identifier (see 5.6)"* (`docs/systemverilog/2023/txt/section-22-compiler-directives.txt:130`), *"If the macro name is an escaped_identifier, only the single white space character terminating the identifier name (see 5.6.1) shall separate the opening parenthesis from the macro name"* (`:166-167`), *"If the text macro name is an escaped identifier, then white space shall be required"* (`:223`), and its example `` `define \M@CRO3 (a=5, b=0, c="C") … `` / `` `\M@CRO3 ( 1 ) `` (`:268-275`). §5.6.1: *"an escaped identifier \cpu3 is treated the same as a nonescaped identifier cpu3"*.
- **WHERE (i).** `preprocess_include_file`: `if state.include_stack.contains(&canonical_path) { bail!("include cycle detected: …") }` ahead of the depth check (`depth > max_include_depth`, 64). **FIX:** the cycle refusal is deleted; the depth refusal is reworded to cite §22.4 and to print the chain's last five steps, so a true loop still reads as what it is.
- **WHERE (ii).** `parse_define_directive`, `define_formal_list_unclosed`, the `` `undef `` arm and `split_conditional_argument` all read the name with `parse_identifier_token`, which admits `[A-Za-z_]`; the usage scanner (`expand_macros_in_text`) required `is_ident_start` after the backtick. **FIX:** `parse_macro_name_token` — a backslash and the printable characters up to white space, the name returned WITHOUT the backslash (so `\escaped` and `escaped` are one key), exactly ONE terminating white-space character consumed (so a `(` right after it opens the formal list); used at all four sites. The usage scanner gains an escaped arm: a backtick, a backslash, the run up to white space — taken as a usage ONLY where that run is a defined name, so the escaped-quote token `` `\`" `` of a macro string keeps its meaning wherever nobody defined a macro by that name. (The multi-line-actuals scanner `has_unclosed_function_macro_invocation` keeps the simple-name reading: an escaped-name function macro whose actuals continue on the next line is a bound stated, not built.)
- **Predictions, written with a clock before the chain** (`rust/target/svpp_5q/prereg.md`, 2026-10-06 01:45, HEAD `a9356e9a8`): RED `0 passed; 6 failed`, GREEN `6 passed`; lib suite 1320; pins 925, both new pins ACCEPT; under each reading the three keyed rows LEAVE `pp_fail` (`file_line.sv`, `include_self.sv`, `escaped_macro_name.v`), the four true loops keep `pp_fail` with the depth refusal, `t_preproc.v` keeps `pp_fail` at `:277`, zero `pass → fail`. **EVERY ONE HELD.**
- **Measured, unit level.** RED on `a9356e9a8`: `test result: FAILED. 0 passed; 6 failed` (`preprocessor.rs:6709`, `:6729`, `:6662`, `:6686` among the panics). GREEN: `test result: ok. 6 passed; 0 failed`. The full lib suite `test result: ok. 1320 passed; 0 failed; 28 ignored` (was 1314; + 6); `clippy_on_rust_change` rc 0.
- **Measured on the three readings** (the rebuilt debug probe `91de33b51d32…`, release probe `1d57068057d9…`, parser fingerprint `a5b5b9dc5c8ee247` unchanged; pins on the release probe `ADJUDICATION-REPROS: checked=1890 armed=236 listed=925 multi_profile_rows=716 preprocess_rows=22 tolerated_rows=8 superset_rows=717 failures=0`, was `checked=1885 listed=923 preprocess_rows=20`; `timeout_s=600 jobs=6`, every step under the guard, sequential):
  - `sv_2017`: `PIPELINE-LANE: files=16336 pass=12746 fail=3322 pp_fail=268 timeout=0 crash=0 wall_s=1181 … profile=sv_2017` (was `pass=12743 … pp_fail=271`); `PIPELINE-MOVERS[sv_2017] rows_before=16336 rows_after=16336 moved=3 added=0 lost=0 {('iverilog', 'pp_fail->pass'): 1, ('sv2v', 'pp_fail->pass'): 2}` — `escaped_macro_name.v`, `file_line.sv`, `include_self.sv`; four rows keep `pp_fail` with the refusal reworded (`include nesting deeper than 64 levels while entering '…' (IEEE 1800-2017 §22.4: …); the chain's last steps: …`): slang `infinite_chain.svh`, sv2v `error/include_loop_1.sv` / `_2.sv`, verilator `t_preproc_inc_recurse_bad.v`; `LIBRARY-MOVERS[sv_2017] {}`.
  - `sv_2023`: `PIPELINE-LANE: files=16336 pass=12786 fail=3282 pp_fail=268 … wall_s=1340 … profile=sv_2023` (was `pass=12783`); `moved=3`, the SAME three rows; `LIBRARY-MOVERS {}`.
  - `sv_2023_tolerant`: `PIPELINE-LANE: files=16336 pass=12804 fail=3264 pp_fail=268 … wall_s=1102 … profile=sv_2023_tolerant` (was `pass=12801`); `moved=3`, the same three; `PIPELINE-LIBRARY: … opentitan=382/396 … published_packages=283` unchanged; `LIBRARY-MOVERS {}`.
  - The primary reading's re-adjudication: `corroborated` 1332 → 1334 (`file_line.sv`, `escaped_macro_name.v`), `refused_engine` 6 → 4; the verdict-coverage tuple under the pipeline `11169/2606/2561/1274/0` → `11171/2606/2559/1272/0` (the raw tuple unmoved; `check_sv_corpus_denominator.sh` refused the stale anchor, then OK). ⛔ **An instrument detail the new message exposed:** the re-adjudication's `landing_of` keys a `pp_fail` row's landing on the refusal's SPELLING (`E_SVPP_|invalid `|without matching|cycle detected|unterminated`), so the chain's first render filed the four depth refusals as `refused_context` (`chained:refused_context` 211 → 212, `refused_engine` 6 → 4 → 3). The pattern gains `nesting deeper` (the old spelling kept for frozen dumps) and the reports are re-rendered: `chained:refused_context=211 chained:refused_engine=12 … refused_engine=4 … svpp_owned:refused_engine=29` — the four loops are engine refusals again. `PIPELINE-LANE-CURRENCY: OK` after the re-render.
  - `PIPELINE-SV2023-CENSUS: rows=16336 both_pass=12746 both_nonpass=3550 2023_only_pass=40 2017_only_pass=0 population_breaches=0 | library: files=407 2023_only_pass=3 2017_only_pass=0 published_2017=277 published_2023=278 | members: unreached_2017=498 reached_2023=51 lost_2023=0`; `PIPELINE-TOLERANT-CENSUS: rows=16336 both_pass=12786 both_nonpass=3532 tolerant_only_pass=18 2023_only_pass=0 population_breaches=0 | library: files=407 tolerant_only_pass=7 2023_only_pass=0 published_2023=278 published_tolerant=283 | members: unreached_2017=498 reached_tolerant=337 lost_tolerant=0` — both differentials unmoved. `IDENTITY-CARRIERS: OK (live=16 frozen=7 rows_equal=84 notes=7 not_evaluated=0)`; `corpus-key-integrity` rc 0; `sv-corpus-denominator: OK` after the anchor moved.
- **The twelve families, filed.** The ten right refusals stay as they are; their rows keep `refused_engine` in the re-adjudication, which does not tell a refusal BY RULING from an engine gap (its reason column does). The Surelog `PreProcMacroFunc` and the OpenTitan `aes_tb_pkg` rows are context / key matters, named here and not re-keyed in this slice.

**Acceptance checklist (`TASK-ACCEPTANCE`):**

  - [x] **REPRODUCE / ISSUE** — the census above; the pin texts on the 1.0.250 release probe: `preprocess_failed … include cycle detected: …/self.sv -> …/self.sv` and `preprocess_failed … missing macro name in `define` under both SV profiles.
  - [x] **ROOT CAUSE (WHY + WHERE)** — WHERE: `preprocess_include_file`'s `include_stack.contains` refusal; `parse_identifier_token` at the four name-reading sites and `is_ident_start` after the backtick in `expand_macros_in_text`. WHY: a bound keyed on a stronger property than §22.4's (reappearance, not depth); an identifier class (A.9.3's `escaped_identifier`) the stage never read. Signature: the pins' `preprocess_failed` lines, the RED panics at `preprocessor.rs:6709` / `:6729` / `:6662` / `:6686`.
  - [x] **FIX** — ENGINE tier, the family's own component; no grammar tier (directives), no engine-neutral tier.
  - [x] **ADDRESSED (verified)** — the six tests GREEN; `fixed_guarded_self_include.sv` ACCEPT on three profiles and `fixed_escaped_identifier_macro_name.sv` ACCEPT on both SV profiles; the three keyed rows `pp_fail → pass` under each of the three readings, two of them `corroborated` in the re-adjudication.
  - [x] **NO REGRESSION** — zero `pass → fail` over 16 336 rows under EACH of the three readings (`moved=3`, all `pp_fail → pass`); the four true loops still refused, now naming §22.4; both census differentials unmoved; the pins `failures=0` on the rebuilt release probe (22 preprocess rows, the 923 pre-existing rows as keyed); the full lib suite `1320 passed; 0 failed`; `clippy_on_rust_change` rc 0; ZERO grammar / generated bytes — the generated parser byte-identical (fingerprint `a5b5b9dc5c8ee247` unchanged); `bash scripts/check_doctrines.sh` on the staged set (recorded in the commit).
  - [x] **LOCKSTEP** — this section + the row (`.5`(q) done), two pins + `MANIFEST.tsv` (925 rows), the SV book's pipeline chapter (two bullets; + tracked HTML), the SV integration contract's expansion-guarantees paragraph, the derived reports of the three readings and both censuses, `docs/TASK_TREE.md`, `MEMORY.md`, `CHANGES.md`, `DEVELOPMENT_NOTES.md`.
  - [x] **NEUTRALITY TIER** — family component: `rust/src/families/systemverilog/` names its own family by path; the engine moved no byte.
  - promotion: declined (see `DEVELOPMENT_NOTES.md`).

## `.6d.7` OpenTitan's fourth unit predecessor, `cip_macros.svh` — ✅ done 2026-10-02 (`PGEN-SVPP-EXPANSION-0033`): the CIP macro header the build makes unit-wide, measured under BOTH readings

**What landed** (CONTEXT tier — `stimuli/sv/pipeline_context.py` (`HAND_UNITS`: one sourced row), `stimuli/sv/pipeline_context.tsv` (rows=79), the derived reports of both readings; ZERO Rust / grammar / generated bytes):

- **How it was found.** `SV-CORPUS-GRAD.41` read `csrng_env_pkg.sv`'s stop at `furthest_position=22550` in the expanded text: a raw `` `DV_MUBI8_DIST(otp_en_cs_sw_app_read, … `` (`csrng_env_cfg.sv:105:39`). The macro is defined at `hw/dv/sv/cip_lib/cip_macros.svh:114`; the header is guarded (`__CIP_MACROS_SVH__`), macro-only (13 `` `define ``s), and `cip_base_pkg.sv:24` includes it INSIDE the package every CIP env imports (`cip_lib.core:23` lists it as an include file). OpenTitan's build compiles `cip_base_pkg` before any env or testbench, and §22.5.1 makes its macros unit-wide from there; a per-file parse never met them.
- **Measured by hand flag first, then through the emitter.** The lane's own probe line + `--unit-file …/cip_macros.svh` on `csrng_env_pkg.sv` → `parse_full passed`; after the `HAND_UNITS` row, `pipeline_context.py --check` → `PIPELINE-CONTEXT: OK`, `context_args` carries four unit files and the package passes.
- **Measured on both readings** (probe `fb60a7ff0dbb448d…`, parser fingerprint `9cc363beab63c228`, `timeout_s=600 jobs=6`; 14:13:23 → 15:58:53 under the guard):
  - `sv_2017`: `PIPELINE-LIBRARY: Cores-VeeR-EL2=2/2 black-parrot=3/7 opentitan=372/396 uvm-core=2/2 published_packages=277 unresolved_imports=44 cyclic_files=0 member_rows=2392` (was `371/396`, 276); `PIPELINE-LANE: files=16336 pass=12739 fail=3320 pp_fail=277 timeout=0 crash=0 wall_s=1064 … profile=sv_2017` (was `pass=12701`); `PIPELINE-MOVERS[sv_2017] rows_before=16336 rows_after=16336 moved=38 added=0 lost=0`, `{('opentitan', 'fail->pass'): 38}` — 37 testbench tops (`tb.sv`) and `csrng_env_pkg.sv`; zero `pass → fail`, zero outside OpenTitan; `LIBRARY-MOVERS[sv_2017] {('fail', 'pass'): 1} fail rows now: 21`.
  - `sv_2023`: `opentitan=375/396 … published_packages=278`; `PIPELINE-LANE: files=16336 pass=12779 fail=3280 pp_fail=277 … profile=sv_2023` (was `pass=12741`); `PIPELINE-MOVERS[sv_2023] … moved=38 added=0 lost=0`, the SAME 38 rows; `LIBRARY-MOVERS[sv_2023] {('fail', 'pass'): 1} fail rows now: 18`.
  - The census of the two: `PIPELINE-SV2023-CENSUS: rows=16336 both_pass=12739 both_nonpass=3557 2023_only_pass=40 2017_only_pass=0 population_breaches=0 | library: files=407 2023_only_pass=3 2017_only_pass=0 published_2017=277 published_2023=278 | members: unreached_2017=498 reached_2023=51 lost_2023=0` — the differential itself did not move (40 / 0 / 3 / 51), as a context row that both editions read alike should leave it.
  - The primary reading's re-adjudication: `PIPELINE-READJUDICATION: … chained:accepted_in_unit=1501 chained:accepted_under_expansion=2399 chained:hollow=25 … chained:rejects_own_file=278 chained:unreached_in_unit=498` (was 1486 / 2361 / 26 / 316 / 512); OpenTitan members `accepted_in_unit` 1 344 → 1 359, `unreached_in_unit` 512 → 498; the tuple under the pipeline `11115/2606/2615/1329/0` → `11168/2606/2562/1276/0` (53 rows adjudicated: the 38 + 15 members; the raw tuple `7485/2606/6245/3864/0` unmoved; `check_sv_corpus_denominator.sh` refused the stale anchor, then OK).
- **What advanced without passing.** Five testbench tops moved their stop past the CIP macros to a per-IP build define — `aes/dv/tb/tb.sv:48:20` and `kmac/dv/tb.sv:50:16` (`` `EN_MASKING ``), `lc_ctrl/dv/tb.sv:139:29` (`` `SEC_VOLATILE_RAW_UNLOCK_EN ``), `spi_device/dv/tb/tb.sv:65:15` (`` `SRAM_TYPE ``), `sram_ctrl/dv/tb.sv:72:16` (`` `INSTR_EXEC ``) — the parked `.6d.5`'s population, now with its testbenches named.
- **What remains, counted.** OpenTitan package files still `fail`: 21 under `sv_2017`, 18 under `sv_2023` (10 per-IP defines, 1 per-target include, 7 tool tolerances — `SV-CORPUS-GRAD.43`, the opt-in profile the director ordered today).

**Acceptance checklist (`TASK-ACCEPTANCE`):**

  - [x] **REPRODUCE / ISSUE** — `csrng_env_pkg.sv` under the pipeline on `51c7ea454`: `fail`, `furthest_position=22550`, origin `csrng_env_cfg.sv:105:39`, the expanded text there a raw `` `DV_MUBI8_DIST( ``.
  - [x] **ROOT CAUSE (WHY + WHERE)** — WHERE: `stimuli/sv/pipeline_context.py` `HAND_UNITS["opentitan"]` carried three build-wide headers and not the CIP one. WHY: `cip_macros.svh` is included only inside `cip_base_pkg.sv:24`, so no per-file include of a CIP env or testbench reaches it, while the build's single unit does — signature `furthest_position=22550` at a raw backtick with `diagnostics=0` (the card `a-raw-backtick-at-the-furthest-position-is-a-context-defect`).
  - [x] **FIX** — CONTEXT tier: one `HAND_UNITS` row with its source; no preprocessor, grammar or engine byte.
  - [x] **ADDRESSED (verified)** — `csrng_env_pkg.sv` → `parse_full passed` through the emitter; `moved=38` under each reading; `opentitan=372/396` / `375/396`; the tuple `11168/2606/2562/1276/0`.
  - [x] **NO REGRESSION** — `moved=38 added=0 lost=0` with zero `pass → fail` over 16 336 rows under EACH reading; the census `2017_only_pass=0 … lost_2023=0`; `IDENTITY-CARRIERS: OK (live=15 frozen=7 rows_equal=75 notes=7 not_evaluated=0)`; `corpus-key-integrity` rc 0; `PIPELINE-LANE-CURRENCY: OK`; ZERO Rust / grammar / generated bytes — every generated artifact byte-identical (parser fingerprint `9cc363beab63c228` unchanged); `bash scripts/check_doctrines.sh` → `doctrines: ALL 45 enforced doctrines PASS.` on the staged set.
  - [x] **LOCKSTEP** — this section + the row, the derived reports of both readings (`pipeline_characterization.md`, `pipeline_readjudication.{md,tsv}`, the verdict-coverage artifacts, the dark audits, the censuses, `pipeline_sv2023_lane/census.{md,tsv}`), the platform book's verdict-coverage anchor, the SV book's pipeline chapter (+ tracked HTML), `docs/TASK_TREE.md`, `MEMORY.md`, `CHANGES.md`, `DEVELOPMENT_NOTES.md`; riding in the same commit, named: the director's tolerance-profile decision record (+ `INDEX.md`, `KNOWLEDGE_MAP.md`), `SV-CORPUS-GRAD.43` opened and `.42` retracted.
  - [x] **NEUTRALITY TIER** — a lane context table under `stimuli/sv/`: SystemVerilog's own measurement context, no engine byte.
  - promotion: declined (see `DEVELOPMENT_NOTES.md`).

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

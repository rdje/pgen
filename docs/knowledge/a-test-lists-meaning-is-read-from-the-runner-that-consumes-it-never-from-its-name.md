---
id: a-test-lists-meaning-is-read-from-the-runner-that-consumes-it-never-from-its-name
title: A test list's meaning is read from the runner that consumes it, never from its name — three of ivtest's "other-target" lists are read by the same regression runner as the two the answer key trusted, and 548 corpus rows sat in no-verdict because nobody had opened the runner
answers:
  - "why did 596 iverilog corpus rows carry no SV answer key"
  - "which ivtest lists does the SystemVerilog corpus adjudicator read as answer keys"
  - "what does a regress-vlog95.list CE mean for parse-level validity"
  - "what does regress-fsv.list imply about the dialect of its entries"
  - "how are blif.list and fpga_reg.list entries keyed"
  - "what does an unknown test type like noraml mean to the ivtest runners"
  - "how is a continuation source or an include target of an ivtest entry keyed"
  - "why are the permanent-deferral ceilings per lane"
tags: [sv-corpus, answer-key, ivtest, adjudicator, no-verdict, suite-convention, provenance]
date: 2026-10-08
status: current
evidence: "SV-CORPUS-GRAD.13e.13 (`PGEN-SV-CORPUS-GRAD-0434`). The adjudicator's `IvtestIndex` loaded two of ivtest's eleven list files and keyed every other ivtest file `deferred:no_sv_key` (*other-target list / multi-file companion / unlisted*). Read from the vendored runners: `vvp_reg.pl:52-59` reads `regress-ivl1`, `regress-vhdl` and `regress-synth` with the SAME `read_regression_list` (perl-lib/RegressionList.pm) and the same type semantics as `regress-vlg` / `regress-sv`; `regress-fsv` only under `--force-sv`, which `RegressionList.pm:180-182` turns into `-g2023`; `vlog95_reg.pl` compiles the ORIGINAL file first (so `normal`/`CO`/`EF`/`RE`/`CN`/`TE` all mean the front end accepted it, while a `CE` may be the Verilog-95 code generator refusing); `vhdl_reg.pl` and `vpi_reg.pl` likewise; `blif_reg.py` compiles both the DUT and its `_tb` under `-g2009`; `fpga_reg.sh` compiles test and testbench with no `-g`. An unknown type (`noraml`, an upstream typo) runs as normal because the Perl runners test only the special types. The lane is the ENTRY's generation (its flag, else `compiler.h GN_DEFAULT`), never the list's name — `regress-vhdl.list` carries 132 explicit `-g2005-sv` entries. Result on the tracked manifests, every moved row attributed to its list and type: 548 of the 596 rows left no-verdict (337 → the verilog_2005 lane, 196 → match, 8 NI, 4 `include fragments, 2 explained, 1 stage-unresolved); the 14 that surfaced as unexplained all stop on Icarus-only text (binary `~&`/`~|`, `reg bool`, `reg real`, `wire real` ports, `+=` in a 1364-2005 `for`, a module-scope `$attribute`) and are pinned must_reject with their clauses and reproducers; bar 0 on all three lanes; the tuple `7485/2606/6245/3864/0` → `7683/2943/5710/3821/0`, under the pipeline `11259/2606/2471/1295/0` → `11456/2943/1937/1253/0`."
reverify: "python3 stimuli/sv/adjudicate_external_corpus.py --self-test (the ivtest-lists arms: 31 ok); bash scripts/check_sv_corpus_denominator.sh (expect the tuple above or later); bash scripts/check_corpus_key_integrity.sh (the provenance residual within its ceiling with the .13e.13 vocabulary); python3 stimuli/sv/run_adjudication_repros.py (failures=0)"
---

**A list file is data; what it means is in the program that reads it.** The answer key trusted
`regress-sv.list` and `regress-vlg.list` because the first leaf that read ivtest opened those two.
The other nine were filed under *other-target list* by their names, and the name was wrong three
times over: three of them are read by the same runner, with the same reader and the same types.

- **Open the runner, cite the line.** Every rule the adjudicator now applies names the runner line
  that justifies it (`vvp_reg.pl:52-59`, `RegressionList.pm:180-182`, `blif_reg.py`, `fpga_reg.sh:78,105`),
  so the next reader checks a line instead of re-deriving a convention.
- **The dialect is the entry's, not the list's.** An entry carries its own `-g` flag or inherits the
  compiler's default; a list named `vhdl` carried 132 SystemVerilog entries.
- **A stage-ambiguous failure is a stage-ambiguous failure.** A `CE` in an output lane may be the
  code generator's; it is `negative_stage_triage`, never a parser verdict, until a golden or a pin
  decides — the `.13e` warning, kept.
- **Everything the lists newly key that the parser rejects goes on the bar first.** Fourteen rows
  did; each was then read at its furthest position and pinned by the clause that refuses it, with a
  reproducer the oracle re-runs — the rulings `.13e.2`/`.13e.3` had already made, applied to the rows
  that had been invisible.
- **Bound what the new keys add to a permanent class.** Twelve NI rows joined `ni_unimplemented`
  across two lanes; the ceilings rose by exactly those rows, in the same commit, naming the argument.

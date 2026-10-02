---
id: a-raw-backtick-at-the-furthest-position-is-a-context-defect
title: A raw backtick at the furthest position is a context defect — the macro was defined by something the flow compiled earlier, and the lane must say what that was
answers:
  - "why does the parser reject at a `uvm_object_utils or `DV_CHECK macro call"
  - "why does an OpenTitan DV package fail at a macro it never includes"
  - "which files does OpenTitan's dvsim compile before a DV package"
  - "how do I add a suite-wide unit predecessor to the pipeline lane"
  - "why did the lane's origin point at the end of the previous line"
  - "why did pipeline_library_members.tsv shrink after adding unit predecessors"
tags: [preprocessor, context, unit-file, HAND_UNITS, opentitan, dvsim, pipeline-lane, furthest_position]
date: 2026-10-02
status: current
evidence: "SVPP-EXPANSION.6d.6 (PGEN-SVPP-EXPANSION-0031): 15 of the 38 OpenTitan DV packages still rejecting after .6d.4 stopped, in their preprocessed text, at a raw `uvm_object_utils / `uvm_component_utils / `DV_CHECK_FATAL / `ASSERT_I in packages that include no macro header; OpenTitan's dvsim compiles one unit beginning with uvm_pkg.sv (hw/dv/tools/dvsim/dsim.hjson:14-15) and dv_utils_pkg.sv before any DV package; with uvm_macros.svh, dv_macros.svh and prim_assert.sv as hand-curated suite unit predecessors the lane moved 195 OpenTitan rows fail -> pass and none the other way"
reverify: "f=stimuli/sv/subs/opentitan/hw/ip/aes/dv/test_vectors/nist_vectors_pkg.sv; args=$(bash -c \"source stimuli/sv/pipeline_context_args.sh; context_args opentitan $f\" | tr '\\t' ' '); rust/target/debug/parseability_probe --parse systemverilog $f --profile sv_2017 --preprocess $args --lib-in stimuli/sv/characterization/pipeline_library 2>&1 | tail -1   # expect parse_full passed … [preprocessed]; the emitter carries three --unit-file flags"
---

The lane reports a rejection by the last byte the source map could attribute. When the failing
token is a macro call on the NEXT line, that byte is the end of the previous line, and the origin
reads like a grammar problem with whatever statement sits there. Re-preprocess the file with its
lane context and read the text at `furthest_position`: a backtick there means the preprocessor never
expanded the name, and a name nobody expanded is a context question before it is a grammar one.

- **Where the definition came from.** A project's flow compiles files in an order, and IEEE 1800-2017
  §22.5.1 makes a macro unit-wide once defined. OpenTitan's dvsim starts every DV build with
  `uvm_pkg.sv` (which includes `uvm_macros.svh`) and compiles `dv_utils_pkg.sv` (`dv_macros.svh`) and the
  RTL (`prim_assert.sv`) before any env or test package, so those packages use `` `uvm_object_utils ``,
  `` `DV_CHECK_* ``, `` `gfn `` and `` `ASSERT_I `` without including anything. Parsed one file at a time
  they meet the raw backtick.
- **Say it as context, with the source.** `stimuli/sv/pipeline_context.py` carries `HAND_UNITS`: per
  suite, the files the flow compiles before every row, each with the line that proves it. `--write`
  emits `unit_file` rows into `stimuli/sv/pipeline_context.tsv`; `pipeline_context_args.sh` turns them
  into `--unit-file` flags, in order, after the file's own `.4c.2` predecessors. Do not widen the
  grammar for the raw backtick, and do not hand the file an `include` it does not have.
- **Measure by hand first, then through the emitter, then on the lane.** A `--unit-file` flag typed by
  hand on one package costs seconds and either passes or names the next barrier; the emitter proves the
  row reaches the lane; the lane's movers (`PIPELINE-MOVERS … by suite and direction`) prove nothing
  moved the other way.
- **The membership table may shrink.** A header a predecessor already brought into the unit meets its
  own include guard inside the package, expands to nothing, and leaves the package's source map with
  its sub-includes. `member_rows` can halve while the distinct member files stay the same — count both,
  and diff the rows that left by name before calling it a loss.

Related: [[a-generator-change-is-accepted-on-every-familys-samples]],
[[a-filtered-acceptance-run-is-not-the-module]].

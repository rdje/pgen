---
id: a-fragment-is-read-as-its-includer-compiles-it
title: A fragment is read as its includer compiles it — the chain of prefixes, closed at the cut, materialized by the lane (`SVPP-EXPANSION.4c.6`)
answers:
  - "why did an included .svh / .sv file fail to preprocess on its own row when its includer defines the macro"
  - "what is pipeline_context_fragments.tsv and what do order, line and close mean"
  - "the probe refuses a unit file with an unterminated ifndef — how does the lane hand a fragment its includer's prefix"
  - "a header is included by many files — why is it a tie and not applied"
  - "where do the materialized prefixes live and why are they not tracked"
tags: [systemverilog, pipeline, context, include, derivation, preprocessor]
date: 2026-10-17
provenance: SVPP-EXPANSION.4c.6 (PGEN-SVPP-EXPANSION-0051); SVPP-EXPANSION.4b (the include resolution it reuses), .4c.2 / .4c.5 (the unit predecessors), SV-CORPUS-GRAD.58a (the rom.sv row that asked for it)
reverify: "python3 stimuli/sv/pipeline_context.py --self-test   # PIPELINE-CONTEXT-SELF-TEST: checks=44 failed=0; source stimuli/sv/pipeline_context_args.sh && context_args verilator stimuli/sv/subs/verilator/test_regress/t/t_sv_cpu_code/rom.sv   # a --unit-file under rust/target/pipeline_prefix/ ending in `endif"
---

# A fragment is read as its includer compiles it

**The rule.** A file another file `` `include ``s has no context of its own: the macro state it is compiled in is
its includer's text before the `include` line, under the includer's own context. The derived context now carries that
per fragment (`stimuli/sv/pipeline_context_fragments.tsv`: `suite relpath order includer line close`) — the chain of
includers up to the root (order 1 = the root; the deepest chain on the tree is 3), each with the line of its `include`
and the number of conditional regions open there — and the lane (`pipeline_context_args.sh`, `_prefix_of`) materializes
each prefix under `rust/target/pipeline_prefix/` as a unit file: `head -n (line-1)` of the includer plus `close` lines of
`` `endif ``, written atomically; its definitions carry over, its text is discarded, the fragment's own guard is untouched.
The root includer's own units, include dirs and defines come first; each includer's directory is an include dir so a
prefix's relative includes resolve.

**Why the cut is closed, not refused.** The header-guard idiom puts nearly every include inside an `ifndef`; the first
derivation reported 170 fragments as "inside a conditional region" and would have left UVM's package members and
verilator's `rom.sv` unread. One synthetic pair through the probe settled it: a unit file whose `ifndef` is left open
preprocesses to rc 3 (`unterminated`); the same prefix closed, rc 0 — and closing preserves the macro state at the cut
exactly. A derived refusal is still a refusal; it was measured and replaced by a column.

**What the includer cannot give, the row keeps.** A prefix that itself refuses to preprocess — the includer's own
context gap, a variant-only define or a library the corpus does not hold — is not the fragment's refusal: the lane
re-reads the row standalone (`PGEN_PIPELINE_NO_FRAGMENTS=1`, the reading every row had before this leaf) and records
the includer's refusal beside the dump (`rust/target/pipeline_lane/fragment_fallbacks<suffix>.tsv`, `fragment_fallbacks=`
in the lane line). The first chain had inherited 329 such gaps as refusals of the fragments and was stopped. And the
prefix goes AFTER the suite's flow predecessors, where the compiler puts it: emitted before OpenTitan's UVM predecessor
it refused a good `uvm_object_utils` call as incomplete.

**What is reported.** A fragment several files include has several prefixes — a tie, never applied (83 on the tree,
`dv_macros.svh` by 222 files among them); a cycle or a chain past eight levels (4); a file declaring a `package`, a unit
of its own (2); a file its own harness compiles as a top, which keeps its own reading (4 — `t_mailbox.v`, whose own
driver's `MAILBOX_T` is not the one `t_mailbox_class.v` includes it under). An include name `.4b` cannot resolve to one candidate yields no row. The prefixes are never tracked:
they are a derivation of the tracked table at lane time.

**Measured.** 3,611 rows on 2,292 files; `rom.sv` under `program_h.sv`'s 32-line prefix (one `endif`): rc 0, as
`SV-CORPUS-GRAD.58a` said it would. Pre-registered: 246 `pp_fail` fragment rows (the macro barrier), 1,915 `fail` (the
class-base barrier, the library's), 109 `pass`; the lane re-run under three readings, **143 movers on each of the three readings, all fragment files, all `pp_fail` at HEAD, none moved down** — 138 `pp_fail → fail` (the macro barrier removed, the declaration barrier exposed: 113 at a class header) and 5 `pp_fail → pass` (`rom.sv` among them); 279 rows per reading fell back to their standalone reading on their includer's own gap, recorded.

**Related.** [[an-option-file-is-the-command-line-read-it-as-the-tool-does]] (the unit predecessors the same lane reads);
[[a-delay-on-a-member-select-needs-parentheses]] (the `rom.sv` row).

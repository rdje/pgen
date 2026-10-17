---
id: a-file-no-unit-compiles-has-no-unit-to-be-valid-in
title: A file no unit compiles has no unit to be valid in — the design convention's premise derived per row from the suite's own compile declarations (`SV-CORPUS-GRAD.59`)
answers:
  - "why are OpenTitan's hw/ip_templates files deferred:no_unit instead of chained_must_accept"
  - "what is the sixth permanent no-verdict class and how is it bounded"
  - "why did the pipeline's adjudicated count drop on 2026-10-17"
  - "how does the key decide a core's include file is dead, and why are aes's data tables not"
  - "why are ibex's and riscv-dv's vendored DV files still chained_must_accept"
tags: [systemverilog, answer-key, corpus, opentitan, permanent-deferral, derivation]
date: 2026-10-17
provenance: SV-CORPUS-GRAD.59 (PGEN-SV-CORPUS-GRAD-0455); SV-CORPUS-GRAD.57 (the convention carried by the key); SV-CORPUS-GRAD.13f / .13c.1 (the permanent classes and the never-dark rule); SVPP-EXPANSION.6d.9 (the measurement that found the rows)
reverify: "python3 stimuli/sv/adjudicate_external_corpus.py --help > /dev/null   # the no-unit ground-truth check runs on every invocation; grep -c 'deferred:no_unit' stimuli/sv/characterization/adjudication_manifest.tsv   # 205"
---

# A file no unit compiles has no unit to be valid in

**The premise.** The answer key keys every file of a design corpus `chained_must_accept` — *a shipping design's file is
valid as part of its unit* — and the pipeline reading cashes that expectation when the file passes under its derived
context or through a unit that includes it. The convention presumes a unit. OpenTitan's `hw/ip_templates/` holds the
Mako templates ipgen renders into `hw/top_*/ip_autogen/<ip>/`: the rendered copies carry real `.core` files and are the
unit members; the copies under the templates directory have only `.core.tpl` templates as compile declarations and are
compiled in place by nothing. A core can also list an include file (`is_include_file: true`) that no `include` of the
tree names. For such rows the convention has no subject.

**The derivation.** `OpenTitanNoUnitIndex`, built from the tree on every run: (a) a top-level directory under
`hw/ip_templates/` with at least one `.core.tpl` below it and no `.core` at any depth; (b) an include-only core entry
that no literal `include` operand of the suite names by basename, that no core lists as a source, and whose listing core
holds no source including through a macro-built name — `aes_tb_pkg.sv` includes its data tables as `` `include
`REQUESTS_FILE ``, so without that guard five live files would read dead. Such a row is `deferred:no_unit`: the sixth
PERMANENT class (ceiling 205 with its argument; 203 template copies, the two `usb20_*_driver.sv` — the one template package `.60` pinned keeps its clause, read before the unit question), never dark — a file
that was never a compile input is not a parse nobody looked at — and not re-read under the pipeline.

**What the measurement refused.** The route's 57 rows were a join of package expansions. A derivation from fusesoc cores
alone classed 328 rows, 151 of them ibex's and riscv-dv's vendored DV — compiled by their own `.f` lists and Makefiles,
which the index does not read. The class reads only what the tree decides without a flow; every other design file keeps
the convention, and a wider reading is a context leaf per declaration form. Under the pipeline the adjudicated count
fell by 77: template copies the lane had accepted under their derived context and the convention credited.

**Measured.** 11662/2943/1731/1226/0 → 11585/2943/1808/1098/0 under the pipeline; the raw tuple 7733/2943/5660/3812/0 → 7733/2943/5660/3640/0; burnable 1,266 → 1,138 (permanent 512 → 717).

**Related.** [[a-members-barrier-is-its-units-context-not-a-missing-fact-export]] (the measurement that found the rows);
[[an-accept-is-not-a-verdict-until-the-key-expected-it]] (the key side of the same discipline).

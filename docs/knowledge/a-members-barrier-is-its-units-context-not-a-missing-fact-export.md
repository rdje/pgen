---
id: a-members-barrier-is-its-units-context-not-a-missing-fact-export
title: A member's barrier is its unit's context, not a missing fact export — 224 "unit-fact" rows measured into four populations, none a missing export (`SVPP-EXPANSION.6d.9`)
answers:
  - "why do OpenTitan DV class files reject on a base class the suite's own package declares"
  - "does the library pass carry a package's facts to a file the package includes"
  - "what became of SVPP-EXPANSION.6d.9 and the 223 rows -0452 named for it"
  - "how many rows would a parse-level unit reading of the .4c.2 units move"
  - "which OpenTitan units fail on their own text and what does the LRM say"
tags: [systemverilog, pipeline, library, unit, measurement, answer-key, opentitan]
date: 2026-10-17
provenance: SVPP-EXPANSION.6d.9 (PGEN-SVPP-EXPANSION-0052); SVPP-EXPANSION.6d.1 / .6d.2 (the library pass and the unit-membership reading); SV-CORPUS-GRAD.57c / PGEN-SV-CORPUS-GRAD-0452 (the join that named the 223); SV-CORPUS-GRAD.59 / .60 (the routes)
reverify: "grep -c 'export_to_library\\|import_from_library' grammars/systemverilog.ebnf   # the two sites: package_declaration, package_import_item; cut -f2 stimuli/sv/characterization/pipeline_library.tsv | sort | uniq -c   # 379 pass / 18 pp_fail / 10 fail"
---

# A member's barrier is its unit's context, not a missing fact export

**The mechanism.** The library pass (`SVPP-EXPANSION.6d.1`) parses every package of the design suites in dependency
order and exports its facts; a file receives them only through its OWN `import` — `@export_to_library` sits on
`package_declaration` and `@import_from_library` on `package_import_item`, nowhere else. A class file a package
`include`s has no `import` of its own: read standalone it has nothing, by design, and its reading is its unit's
(`.6d.2`: a member's verdict is the verdict of the unit whose expansion includes it). So "the library pass did not
carry the fact to this file" is never the finding; the finding is why the file is not read through its unit.

**The measurement.** Each of the 224 OpenTitan `chained_must_accept` rows rejected in their own file on a name another
suite file declares was traced up the `include` graph — an include resolving to the same-directory candidate first,
as the flow's `+incdir` does — to the packages that compile it, and joined with the library pass's status and the
members dump: 160 sit behind a package whose expansion the lane REFUSED (100 at a per-IP build macro such as
`SRAM_TYPE` or `EN_GCM` — the per-variant reading `.4c.4` is parked for; 60 at a generated include the tree does not
hold, `riscv_core_setting.sv` and `xbar_env_pkg__params.sv`); 57 are files the flow compiles NOWHERE (template copies
under `hw/ip_templates/` whose rendered siblings are the members, include files their `.core` lists and nothing
includes, a vendored test) — the design convention's "valid as part of its unit" presumes a unit, so these are the
answer key's (`SV-CORPUS-GRAD.59`); 7 are in units that FAIL on their own text, four shapes read against both LRM
editions (`SV-CORPUS-GRAD.60`): `[]` in a parameter's dimension is 2017-illegal and 2023-legal (an edition contrast
the sv_2023 reading already cashes), an event trigger on a select has no derivation in either edition yet the
grammar's 2023 variant accepts it (to settle), a hierarchical `cross` item and a lifetime in a method prototype are
derived in neither edition (pin candidates).

**What is left.** Six rows whose demanded name a predecessor of the SAME derived unit declares: verilator's
`t_sv_cpu` three, `t_hier_block_import_subsub.v`, Surelog's `AllBinding/dut.sv`, VeeR's `dccm_memtest.sv`. The
faithful reading — the unit's files parsed as ONE compilation unit under the top's context, the members read through
it as package members are — needs no engine or grammar byte and costs a lane pass plus three readings; it is designed,
priced and parked as the director's call.

**Related.** [[the-store-gate-explanation-holds-under-the-pipeline-too]] (the arm that reads the in-unit question);
[[declared-nowhere-means-nowhere-in-the-row-s-unit]] (the Surelog units, read the same day);
[[a-fragment-is-read-as-its-includer-compiles-it]] (the include fragments, the other half of a unit's context).

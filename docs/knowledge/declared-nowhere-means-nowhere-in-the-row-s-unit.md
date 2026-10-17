---
id: declared-nowhere-means-nowhere-in-the-row-s-unit
title: "Declared nowhere" means nowhere in the row's UNIT — the suite is not the unit, and a name another Surelog test declares is the store gate's refusal, not a cross-file dependency (`SV-CORPUS-GRAD.57c`)
answers:
  - "a Surelog row rejects on a class another test's dut.sv declares — is that a chaining barrier or the store gate"
  - "what is FACT-GATED-OUT-OF-UNIT in the dark chained-only audit"
  - "how does audit_dark_chained_only.py know a Surelog row's compilation unit"
  - "why did .57c find four rows when -0452 said three"
  - "why does a non-Surelog suite keep FACT-GATED-CROSS-FILE for the same shape"
tags: [systemverilog, pipeline, re-adjudication, store-gate, answer-key, surelog, measurement]
date: 2026-10-17
provenance: SV-CORPUS-GRAD.57c (PGEN-SV-CORPUS-GRAD-0453); SV-CORPUS-GRAD.57b (the arm it widens); SV-CORPUS-GRAD.13c / .13c.2 (the audit and the fixed-point proof); SV-CORPUS-GRAD.8b.1 / .57 (SurelogIndex's unit definition); PGEN-SV-CORPUS-GRAD-0452 (the hand join this measurement corrects)
reverify: "python3 stimuli/sv/audit_dark_chained_only.py --self-test   # DARK-CHAINED-AUDIT-SELF-TEST: checks=11 failed=0; grep -c FACT-GATED-OUT-OF-UNIT docs/tasks/artifacts/sv_corpus_grad/dark_chained_only/audit.tsv   # 4"
---

# "Declared nowhere" means nowhere in the row's unit

**The rule.** The `.13c` dark audit sorts a `deferred:chained_only` row by what the parser DEMANDED at the failure:
a missing type fact for a name this file declares, a name another file of the suite declares (`FACT-GATED-CROSS-FILE`,
the `.6d` barrier chaining would lift), a name a vendored library in another suite declares (`CROSS-LIBRARY`), or a name
nothing declares (`UNDECLARED-ANYWHERE`, a worklist bucket). The suite is the right scope for *defect or dependency*. It
is the wrong scope for the store-gate question — *could this row's compile have seen the declaration?* — whose scope is
the row's COMPILATION UNIT. Since `.57c` the audit reads a Surelog row's unit off the tree exactly as the answer key
defines it (`SurelogIndex`: a test unit is a directory under `tests/` directly containing a `.sl` driver — the nearest
such directory above the file), indexes that directory's own declarations with the suite index's over-inclusive reader,
and files a demanded name the suite declares but no unit file does as **`FACT-GATED-OUT-OF-UNIT`**: a worklist bucket,
so the `.13c.2` fixed-point search runs the same proof on it, and a stratum `.57b`'s `store_gate_explanation` accepts
beside `UNDECLARED-ANYWHERE` — a must-accept row it proves is `chained:explained_store_gate`, never a pass.

**The conservative order.** A demanded name a file of the row's own unit declares wins over one declared only outside
it (the row keeps `FACT-GATED-CROSS-FILE`, naming the unit's file); a false in-unit entry keeps a row OUT of the new
stratum. A suite with no unit notion here — every design corpus, whose unit is the library design's members dump — keeps
the `.13c` reading unchanged, which is why the 223 OpenTitan rows of `-0452`'s measurement stay `SVPP-EXPANSION.6d.9`'s.

**Measured.** Four Surelog rows: `ClassFuncProto/top.v:30` (`REQ`, a type parameter of `ClassExtendParam/dut.sv`),
`SimpleClass2/top.v:13` (`uvm_component`, `SimpleClass1/top.v`'s class), `UnitForeach/top2.v:5`
(`uvm_sequencer_base::do_print`, `ClassExtendParam/dut.sv`'s class, declared by neither file of its two-source unit),
`SimpleClass1/top_1.v:8` (`uvm_void`, `ClassVirtual/dut.sv`'s). Three are keyed `chained_must_accept` and are now
explained (`REQ,uvm_reg` / `uvm_component` / `uvm_printer,uvm_sequencer_base`); `top_1.v` is `chained_only` — its driver does not name it — so the key holds no
expectation and the row stays dark by the key, with its proof recorded. `-0452` had published THREE from a hand join of
the audit's `fact_names` against the members dump, which dropped `UnitForeach/top2.v`; the number that stands is the
owning instrument's. 11653/2943/1740/1235/0 → 11656/2943/1737/1232/0; burnable 1,275 → 1,272.

**Related.** [[the-store-gate-explanation-holds-under-the-pipeline-too]] (the arm this widens);
[[a-rejection-the-design-demands-is-explained-only-by-a-parse-that-proves-it]] (the raw lane's class).

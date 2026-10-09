---
id: a-row-finds-the-defect-the-clause-bounds-the-fix
title: A row finds the defect; the clause bounds the fix — before a conformance twin is built, every form the clause excludes is probed on the interpreter, both bounds are pinned, and a sibling rule's over-acceptance is routed, not folded in
answers:
  - "three corpus rows exposed an over-acceptance under verilog_2005 — do I fix those three shapes"
  - "how was the parameter port list's verilog_2005 twin bounded before the build"
  - "why does an invalid pin under verilog_2005 have a control with the SAME text under sv_2017"
  - "I found a second over-acceptance in a neighbouring rule while fixing one — does it ride the same release"
  - "what does the interpreter pre-measurement of a profile twin consist of"
  - "why was `#()` refused under verilog_2005 when no corpus row ever carried it"
  - "the pins pre-screen moved a control my pre-registration did not name — what is it"
tags: [systemverilog, verilog-2005, profiles, grammar-twin, pins, release-ceremony, pre-registration, lrm]
date: 2026-10-08
provenance: SV-CORPUS-GRAD.48 (PGEN-SV-CORPUS-GRAD-0436, release 1.0.252); the .3.66 twin pattern; a-routed-finding-describes-its-witness-not-its-construct
reverify: "python3 docs/tasks/artifacts/sv_corpus_grad/prescreen/prescreen_pins.py (the interpreter pre-screen); grep -n 'invalid_v2005_empty_parameter_port_list\|control_sv_empty_parameter_port_list' stimuli/sv/adjudication_repros/MANIFEST.tsv"
---

# A row finds the defect; the clause bounds the fix

**The shape.** `SV-CORPUS-GRAD.13e.14` surfaced three ivtest rows (`parameter_omit1/2/3.v`) that PGEN accepted
under `verilog_2005` and IEEE 1364-2005 A.1.3 / A.2.1.1 do not derive: a parameter port declaration without
`parameter`. The three rows are the WITNESS. The CONSTRUCT is the clause's whole production —
`module_parameter_port_list ::= # ( parameter_declaration { , parameter_declaration } )` — and the clause excludes
more than the three rows show: an EMPTY list (`#()`), and a `localparam` inside the list (1364-2005 derives
`local_parameter_declaration` only as a module or generate item). Neither form sat on any corpus row; both were
accepted by the shipped grammar; both are refused by the vendored iverilog under `-g2005`. A fix written to the
three rows would have left the rule over-accepting on the day it shipped as "fixed".

**What the leaf did instead, before a byte was generated.**

1. **Read the clause, then write the twin to the clause** — `parameter_port_list_v2005` admits `parameter`-led
   declarations, at least one, and nothing else. The twin's body is the production, not the three rows' negation.
2. **Probe every form the clause excludes, and every form it admits, on the interpreter** (`ast_pipeline <grammar>
   --grammar-profile verilog_2005 --interpret-parse <file>`): the three row shapes, `#()`, `localparam`, `parameter
   type`, and the five legal A.2.1.1 forms (`signed [3:0]`, `integer`, `real`, `realtime`, `time`) in one list —
   HEAD grammar against the candidate, and the vendored tool under the same edition beside them.
3. **Pin BOTH bounds, as edition pairs.** The lower bound: `invalid_v2005_empty_parameter_port_list.sv` and
   `invalid_v2005_localparam_in_parameter_port_list.sv` (REJECT, `verilog_2005`) each with a control of the SAME
   TEXT under `sv_2017` / `sv_2023` (`control_sv_*.sv`, ACCEPT) — so the oracle holds that the refusal is the
   edition's, not the parser's. The upper bound: `control_v2005_parameter_port_forms.sv`, every legal form in one
   list, ACCEPT on every profile with its typed AST byte-identical to the previous release's.
4. **Re-read every control under the edition, by instrument.** The pins pre-screen over the whole manifest moved
   one arm the pre-registration had not named: a `.3.51` control (`module ModuleA #(P=1) (inp);`) listed on
   `verilog_2005` with an IEEE 1800 citation only — it had parsed there through the defect being fixed, and iverilog
   `-g2005` refuses it with the three rows' message. It was narrowed to the IEEE 1800 profiles: a control is a claim
   about an edition, and the clause that bounds the fix bounds the answer key too.
5. **Route a sibling's over-acceptance; do not fold it in.** Probing `#()` as a PORT list led to `mod #() m1(…)`,
   an empty parameter VALUE assignment on an instance — a different rule (`parameter_value_assignment_sv_2017`), the
   same cause class, and here the vendored tool accepts what the clause (A.4.1.1) refuses. It became `.50` with a
   WATCHED `accepts_invalid` pin and a control, and the release under way kept its pre-registered scope.

**Why it matters.** A twin written to its witnesses is a patch; a twin written to its clause is a conformance rule.
The difference is invisible to the corpus (no row carried `#()` or `localparam`) and visible only to the bounds you
chose to pin. The interpreter makes the whole probe cost minutes, before the build; the pins make the bound hold
after it. See [a-routed-finding-describes-its-witness-not-its-construct](a-routed-finding-describes-its-witness-not-its-construct.md)
for the general rule this applies, and `docs/tasks/SV-CORPUS-GRAD.md` `.48` for the measurements.

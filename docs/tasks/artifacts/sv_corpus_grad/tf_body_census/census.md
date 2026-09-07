# Function and task bodies the corpus writes — census at release `1.0.217` (`SV-CORPUS-GRAD.3.44`, 2026-09-07)

Population: the **4 811** SV-lane files (of 16 336) whose text carries the `function` or `task` token — selected with
`grep -lE '\b(function|task)\b'` over the lane's file list — and, for the `verilog_2005` lane, its own list so filtered
(**396** of 2 606). Instrument: `../ast_field_census/ast_field_census.py` on the **1.0.217 release probe**
(`rust/target/s344/probe_head` = `rust/target/release/parseability_probe` `52d1ff8fecf06519…`, generated-parser identity
`07936ecb0b19912f…`), with the NEW `--select-keys` selector: every node whose key set is exactly `{lifetime, body}` —
the `function_declaration_sv_2017` / `task_declaration_sv_2017` return — under `sv_2017` and `verilog_2005`, and
`{dynamic_override, lifetime, body}` under `sv_2023` (the 2023 declarations carry the override slot), field `body`.
No fallback regex: a rejected file has no tree and cannot move.

```
AST-FIELD-CENSUS: keys=body,lifetime field=body profile=sv_2017 files=4811 accepted=1958 rejected=2853 timeout=0 nodump=0 nodes=3411 distinct=2968 fallback_rows=0
AST-FIELD-CENSUS: keys=body,dynamic_override,lifetime field=body profile=sv_2023 files=4811 accepted=1966 rejected=2845 timeout=0 nodump=0 nodes=3521 distinct=3013 fallback_rows=0
AST-FIELD-CENSUS: keys=body,lifetime field=body profile=verilog_2005 files=396 accepted=352 rejected=44 timeout=0 nodump=0 nodes=470 distinct=438 fallback_rows=0
```

Classified by `classify.py` (verbatim; the file lists are `rust/target/s344/tf_files_all.txt` and `tf_files_v2005.txt`):

```
TF-BODY-CLASSIFY: rust/target/s344/tf_census_head_sv_2017.tsv rows=6549 parser=3411 function=2235 task=1176 typed=1200 RAW=2211 unclassified=0
     559  function:alt1:typed
    1646  function:alt2:RAW
       2  function:alt3:RAW
      28  function:alt4:RAW
     641  task:alt1:typed
     513  task:alt2:RAW
      22  task:alt4:RAW
PREDICTION accepted files carrying a RAW body (typed AST must move when the alternatives are typed): 1060
PREDICTION accepted files whose bodies are all typed (typed AST must be byte-identical): 898

TF-BODY-CLASSIFY: rust/target/s344/tf_census_head_sv_2023.tsv rows=6653 parser=3521 function=2341 task=1180 typed=1300 RAW=2221 unclassified=0
     659  function:alt1:typed
    1652  function:alt2:RAW
       2  function:alt3:RAW
      28  function:alt4:RAW
     641  task:alt1:typed
     517  task:alt2:RAW
      22  task:alt4:RAW
PREDICTION accepted files carrying a RAW body (typed AST must move when the alternatives are typed): 1064
PREDICTION accepted files whose bodies are all typed (typed AST must be byte-identical): 902

TF-BODY-CLASSIFY: rust/target/s344/tf_census_head_verilog_2005.tsv rows=555 parser=470 function=231 task=239 typed=379 RAW=91 unclassified=0
     153  function:alt1:typed
      78  function:alt2:RAW
     226  task:alt1:typed
      13  task:alt2:RAW
PREDICTION accepted files carrying a RAW body (typed AST must move when the alternatives are typed): 40
PREDICTION accepted files whose bodies are all typed (typed AST must be byte-identical): 312
```

**Reading.** On `sv_2017` the parser wrote **3 411** function / task bodies in the 1 958 accepting files, and **2 211 of
them (64.8 %) were the raw `Sequence` envelope** — 1 646 ported functions and 513 ported tasks (alternative 2, the most
common function in any corpus), 2 + 28 out-of-block functions and 22 out-of-block tasks (alternatives 3–4); only the
1 200 non-ported in-block bodies (alternative 1) were typed. `unclassified=0` on every lane: every envelope decoded to one
of the eight shapes. The corpus writes **no** body of `task:alt3` (an out-of-block, non-ported task) on any lane — that
alternative's typed shape is witnessed only by the pin `control_tf_body_task_out_of_block_class.sv` and the AST-shape
contract sample `task_body_out_of_block_interface`.

**PREDICTION (pre-registered before a byte was generated).** The typed AST of exactly **1 060** accepting files must move
on `sv_2017`, **1 064** on `sv_2023`, **40** on the `verilog_2005` lane — the files carrying a RAW body, by name in
`rust/target/s344/tf_classify_head_<profile>.txt` — and every other accepted file (898 / 902 / 312 in the population, and
every accepting file outside it) must be byte-identical; **no verdict moves** on any lane (a return template changes what
the parser EMITS, never what it accepts). By suite, the `sv_2017` movers:

| suite | files whose typed AST must move |
|---|---|
| `verilator` | 390 |
| `iverilog` | 255 |
| `sv-tests` | 103 |
| `ispras-sv-tests` | 96 |
| `Surelog` | 87 |
| `opentitan` | 71 |
| `verible` | 33 |
| `sv2v` | 12 |
| `uvm-core-2020.3.1` | 5 |
| `slang` | 3 |
| `Cores-VeeR-EL2` | 2 |
| `friscv` | 2 |
| `scr1` | 1 |

## The class, censused: partial typing across the whole grammar

`partial_typing_census.py` over the frontend's `raw_ast` of the 1.0.217 grammar (`rust/target/s344/raw_head.json`) and of
this release's (`raw_new.json`), verbatim:

```
PARTIAL-TYPING-CENSUS: grammar=rust/target/s344/raw_head.json rules=1610 multi_alternative=558 fully_typed=490 untyped=28 PARTIAL=40 partial_untyped_alternatives=70 passthrough=45 RAW_ENVELOPE=25
PARTIAL-TYPING-CENSUS: grammar=rust/target/s344/raw_new.json rules=1610 multi_alternative=558 fully_typed=492 untyped=28 PARTIAL=38 partial_untyped_alternatives=64 passthrough=45 RAW_ENVELOPE=19
```

The two lists differ by exactly `function_body_declaration` and `task_body_declaration` (the set difference, both directions).
**The residue — every PARTIAL rule of release 1.0.218 still carrying a `RAW_ENVELOPE` alternative — is owned by
`SV-CORPUS-GRAD.3.45`** (19 alternatives in 8 rules; the 45 `passthrough` alternatives are the profile-split
`_sv_only` / `_v2005` helper pattern, typed one level down by design):

| rule | alternatives | untyped (index:bucket) |
|---|---|---|
| `assignment_pattern` | 5 | 2:RAW_ENVELOPE,3:RAW_ENVELOPE,4:RAW_ENVELOPE |
| `block_data_type` | 17 | 2:RAW_ENVELOPE,3:passthrough,4:RAW_ENVELOPE,5:RAW_ENVELOPE,6:passthrough,7:passthrough,8:RAW_ENVELOPE,9:passthrough,10:passthrough,11:passthrough,12:passthrough,13:passthrough,14:passthrough,15:passthrough,16:passthrough,17:passthrough |
| `case_statement` | 3 | 2:RAW_ENVELOPE,3:RAW_ENVELOPE |
| `clocking_declaration` | 2 | 2:RAW_ENVELOPE |
| `clocking_event_sv_2017` | 2 | 2:RAW_ENVELOPE |
| `list_of_path_delay_expressions` | 7 | 2:RAW_ENVELOPE,3:RAW_ENVELOPE,4:RAW_ENVELOPE,5:RAW_ENVELOPE,6:RAW_ENVELOPE,7:RAW_ENVELOPE |
| `net_type_declaration_sv_2017` | 2 | 2:RAW_ENVELOPE |
| `property_case_item` | 2 | 2:RAW_ENVELOPE |

# DPI import function RESULT types the corpus writes — census at release `1.0.215` (`SV-CORPUS-GRAD.3.43`, 2026-09-06)

Population: the **172** corpus files (of 16 335 on the SV lane) whose text carries a DPI spec string `"DPI` — selected
with `grep -l -F '"DPI'`, checked against the 49 verilator `t_dpi_*` files (48 selected; `t_dpi_accessors.v` keeps its
imports in an included `.vh`). Instrument: `../ast_field_census/ast_field_census.py` on the **1.0.215 release probe**
(`rust/target/release/parseability_probe`, generated-parser identity `8c7f336ca851f7a2…`), field
`proto.body.body.return_type` of every `import_function` node; rejected files scanned by the fallback regex.

```
AST-FIELD-CENSUS: kind=import_function field=proto.body.body.return_type profile=sv_2017 files=172 accepted=95 rejected=77 timeout=0 nodump=0 nodes=297 distinct=73 fallback_rows=1001
AST-FIELD-CENSUS: kind=import_function field=proto.body.body.return_type profile=sv_2023 files=172 accepted=95 rejected=77 timeout=0 nodump=0 nodes=297 distinct=73 fallback_rows=1001
```

The two SV profiles read the same 1 298 result slots (297 by the parser in the 95 accepting files, 1 001 by the regex
in the 77 files the parser rejects for other reasons — the fallback admits an escaped identifier as the C alias too). Classified against §35.5.5 (`classify.py`, verbatim):

```
DPI-RESULT-CLASSIFY: <census.tsv> rows=1298 parser=297 regex=1001 legal_keyword=1210 named=64 ILLEGAL=24 unclassified=0
     885  legal_keyword  void
     191  legal_keyword  int
      46  named          known_unscoped_data_type
      41  legal_keyword  chandle
      34  legal_keyword  bit
      28  legal_keyword  string
      11  ILLEGAL        logic [packed x1]
       7  legal_keyword  longint
       6  legal_keyword  byte
       5  ILLEGAL        bit [packed x1]
       5  legal_keyword  int unsigned
       4  legal_keyword  real
       2  legal_keyword  shortint
       2  legal_keyword  shortint unsigned
       2  legal_keyword  shortreal
       1  ILLEGAL        bit [ 0:0]  (packed)
       1  ILLEGAL        bit [16-1:0]  (packed)
       1  ILLEGAL        bit [17-1:0]  (packed)
       1  ILLEGAL        bit [31:0]  (packed)
       1  ILLEGAL        bit [32-1:0]  (packed)
       1  ILLEGAL        bit [8-1:0]  (packed)
       1  ILLEGAL        bit [9-1:0]  (packed)
       1  ILLEGAL        integer
       1  legal_keyword  byte unsigned
       1  legal_keyword  logic
       1  legal_keyword  longint unsigned
       1  named          bit_t
       1  named          byte_t
       1  named          byte_unsigned_t
       1  named          chandle_t
       1  named          int_t
       1  named          int_unsigned_t
       1  named          logic_t
       1  named          longint_t
       1  named          longint_unsigned_t
       1  named          real_t
       1  named          shortint_t
       1  named          shortint_unsigned_t
       1  named          shortreal_t
       1  named          string_t
       1  named          struct_2_state_1
       1  named          struct_2_state_32
       1  named          union_2_state_1
       1  named          union_2_state_32
  ILLEGAL	accepted	stimuli/sv/subs/sv2v/test/nosim/dpi.sv	bit [packed x1]
  ILLEGAL	rejected	stimuli/sv/subs/verilator/test_regress/t/t_dpi_import.v	bit [8-1:0]  (packed)
  ILLEGAL	rejected	stimuli/sv/subs/verilator/test_regress/t/t_dpi_import.v	bit [9-1:0]  (packed)
  ILLEGAL	rejected	stimuli/sv/subs/verilator/test_regress/t/t_dpi_import.v	bit [16-1:0]  (packed)
  ILLEGAL	rejected	stimuli/sv/subs/verilator/test_regress/t/t_dpi_import.v	bit [17-1:0]  (packed)
  ILLEGAL	rejected	stimuli/sv/subs/verilator/test_regress/t/t_dpi_import.v	bit [32-1:0]  (packed)
  ILLEGAL	rejected	stimuli/sv/subs/verilator/test_regress/t/t_dpi_result_type.v	bit [ 0:0]  (packed)
  ILLEGAL	rejected	stimuli/sv/subs/verilator/test_regress/t/t_dpi_result_type.v	bit [31:0]  (packed)
  ILLEGAL	accepted	stimuli/sv/subs/verilator/test_regress/t/t_dpi_result_type_bad.v	bit [packed x1]
  ILLEGAL	accepted	stimuli/sv/subs/verilator/test_regress/t/t_dpi_result_type_bad.v	bit [packed x1]
  ILLEGAL	accepted	stimuli/sv/subs/verilator/test_regress/t/t_dpi_result_type_bad.v	bit [packed x1]
  ILLEGAL	accepted	stimuli/sv/subs/verilator/test_regress/t/t_dpi_result_type_bad.v	bit [packed x1]
  ILLEGAL	accepted	stimuli/sv/subs/verilator/test_regress/t/t_dpi_result_type_bad.v	integer
  ILLEGAL	accepted	stimuli/sv/subs/verilator/test_regress/t/t_dpi_result_type_bad.v	logic [packed x1]
  ILLEGAL	accepted	stimuli/sv/subs/verilator/test_regress/t/t_dpi_result_type_bad.v	logic [packed x1]
  ILLEGAL	accepted	stimuli/sv/subs/verilator/test_regress/t/t_dpi_result_type_bad.v	logic [packed x1]
  ILLEGAL	accepted	stimuli/sv/subs/verilator/test_regress/t/t_dpi_result_type_bad.v	logic [packed x1]
  ILLEGAL	accepted	stimuli/sv/subs/verilator/test_regress/t/t_dpi_result_type_bad.v	logic [packed x1]
  ILLEGAL	accepted	stimuli/sv/subs/verilator/test_regress/t/t_dpi_result_type_bad.v	logic [packed x1]
  ILLEGAL	accepted	stimuli/sv/subs/verilator/test_regress/t/t_dpi_result_type_bad.v	logic [packed x1]
  ILLEGAL	accepted	stimuli/sv/subs/verilator/test_regress/t/t_dpi_result_type_bad.v	logic [packed x1]
  ILLEGAL	accepted	stimuli/sv/subs/verilator/test_regress/t/t_dpi_result_type_bad.v	logic [packed x1]
  ILLEGAL	accepted	stimuli/sv/subs/verilator/test_regress/t/t_dpi_result_type_bad.v	logic [packed x1]
  ILLEGAL	accepted	stimuli/sv/subs/verilator/test_regress/t/t_dpi_result_type_bad.v	logic [packed x1]
```

## What the census decides

- **Legal keyword types dominate**: `void` 885, `int` 191, `chandle` 41, `bit` 34, `string` 28, `longint` 7, `byte` 6, `real` 4, `shortint` 2, `shortreal` 2, `logic` 1, and 9 signed / unsigned atom forms — the eleven types §35.5.5 lists, with signing on the atom types (admitted: the
  subclause names types and the word "scalar" and is silent on signing). Total legal_keyword 1210.
- **Named types** (64): 46 typedef names read by the parser in accepting files and 18 in the regex rows of verilator's
  `t_dpi_result_type*.v`, which type their results through `typedef struct packed … struct_2_state_32;` on purpose — a typedef is
  not seen through by a grammar, so a named type without packed dimensions is admitted (what it resolves to is semantic).
- **ILLEGAL** (24): packed `bit` / `logic` vectors and one `integer`, in exactly FOUR files — two ACCEPTING on 1.0.215
  and two already rejected for other reasons:
  - `stimuli/sv/subs/sv2v/test/nosim/dpi.sv:35` `import "DPI-C" function bit [15:0] getStimulus(input integer x);` —
    ACCEPTING; keyed `must_accept` (sv2v suite contract) → **re-keyed `must_reject`** (`EXTRA_PINNED`, `.3.43`). ⭐ The
    line is the LRM's OWN informative example at §35.5.4 (`section-35-direct-programming-interface.md:476`; 1800-2023
    `:489` repeats it unchanged) — it contradicts the normative §35.5.5 one paragraph later and Annex A footnote 21;
    the normative text outranks the example.
  - `stimuli/sv/subs/verilator/test_regress/t/t_dpi_result_type_bad.v:80-83,104,107-117` — sixteen results (`bit
    [32:0]…[127:0]`, `integer`, `logic [0:0]…[127:0]`); ACCEPTING; keyed `must_accept` ("post-parse tool error —
    syntax valid") → **re-keyed `must_reject`**: footnote 21 makes the restriction grammar.
  - `stimuli/sv/subs/verilator/test_regress/t/t_dpi_import.v:37-41` — five `bit [8-1:0]…[32-1:0]` results,
    UNCONDITIONAL; already `fail` (byte 551, the `ifdef VCS/NC/VERILATOR` preamble, svpp) → **re-keyed `must_reject`**
    so the row cannot become a false `rejects_valid` when the svpp divergence resolves.
  - `stimuli/sv/subs/verilator/test_regress/t/t_dpi_result_type.v:114-115` — `bit [ 0:0]` / `bit [31:0]` under
    `ifndef NO_ARRAY`; already `fail` (svpp macro use); NOT re-keyed — the ruling depends on the preprocessing
    configuration; owner: that row's svpp-divergence resolution.
- **Pre-registered lane effect of release 1.0.216**: `sv_2017` and `sv_2023` each exactly **two rows `pass → fail`**
  (the sv2v row and `t_dpi_result_type_bad.v`), **0** `fail → pass`; `verilog_2005` unmoved (DPI is `@profiles`-gated
  out of it). Verilator accepts 2-state vector results of ≤ 32 bits as an extension (`svBitVecVal`); its own `_bad`
  test rejects the > 32-bit and 4-state ones at elaboration. Spec outranks tool.
- **Absent from the corpus** (refused by the grammar all the same, each with a pin): `reg`, `time`, `realtime`, enum /
  struct / union literals, `event`, virtual interfaces, parameterised class types, a typedef with packed dimensions,
  an open `bit []` result, `type(...)` references (admitted; 0 uses), class-scoped type identifiers (admitted; 0 uses).

## The same census on the release-1.0.216 probe (the final bytes)

```
AST-FIELD-CENSUS: kind=import_function field=proto.body.body.return_type profile=sv_2017 files=172 accepted=93 rejected=79 timeout=0 nodump=0 nodes=223 distinct=11 fallback_rows=1075
DPI-RESULT-CLASSIFY: <census_final.tsv> rows=1298 parser=223 regex=1075 legal_keyword=1210 named=64 ILLEGAL=24 unclassified=0
```

Conserved to the row: 297 + 1 001 = 223 + 1 075 = **1 298**, `legal_keyword=1210 named=64 ILLEGAL=24 unclassified=0` both times. The two accepting
files with illegal results (74 parser rows) moved to the regex-scanned rejected side; every ILLEGAL row now sits in a REJECTED file, and
`distinct` fell 73 → 11 because verilator's typedef'd result names all lived in `t_dpi_result_type_bad.v`. `sv_2023` identical.

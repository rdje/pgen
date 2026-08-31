# The `verilog_2005` rejects-valid census — keyed by the construct that stops the parse

> Produced by `stimuli/sv/v2005_rejects_valid_census.py` from the tracked v2005
> adjudication manifest and the `positions_v2005.tsv` sidecar. Deterministic and
> content-derived: re-run it and diff.

⛔ **This is the population `SV-CORPUS-GRAD.13k` is about** — every row the
`verilog_2005` answer key calls `must_accept` and the parser rejects. `.13k` opened on
three of them and said *"three is a sample"*. It is, and the sample was **not**
representative: the class is TWO defects, not one.

## The split

| edition verdict | rows | what it means |
|---|---|---|
| `1800_only` | 19 | Annex A of IEEE 1364-2005 cannot derive the text ⇒ **the ANSWER KEY is wrong** and the parser is right |
| `1364_legal` | 20 | Annex A **does** define it ⇒ **the `verilog_2005` PROFILE is under-accepting** — a parser defect, a different lane |
| `preproc` | 5 | a compiler-directive question, owned by the svpp lane |
| `UNCLASSIFIED` | 9 | no probe matched; adjudicate by hand and add one |
| **total** | **53** | |

## By construct

| construct | rows | edition | in-repo citation that settles it |
|---|---|---|---|
| `inc_dec_operator` | 11 | `1800_only` | IEEE 1800 A.6.2 inc_or_dec_operator; grep for '++' over 1364-2005 Annex A = 0 |
| `(unclassified)` | 9 | `UNCLASSIFIED` | no probe matched — adjudicate by hand and add one |
| `net_type_tri01` | 9 | `1364_legal` | IEEE 1364-2005 Annex A A.2.2.1:219 net_type ::= ... \| tri0 \| tri1 |
| `logic_keyword` | 5 | `1800_only` | IEEE 1800 A.2.2.1 integer_vector_type; grep '\blogic\b' over 1364-2005 Annex A = 0 |
| `macro_directive` | 5 | `preproc` | IEEE 1364-2005 clause 19 compiler directives; owned by the svpp lane |
| `udp_table_entry` | 5 | `1364_legal` | IEEE 1364-2005 Annex A A.5.3:594 udp_body ::= combinational_body \| sequential_body |
| `assignment_operator` | 3 | `1800_only` | IEEE 1800 A.6.2 assignment_operator; 1364-2005 A.6.2 is 'variable_lvalue = expression' |
| `gate_strength` | 3 | `1364_legal` | IEEE 1364-2005 Annex A A.3.1 pulldown/pullup + A.2.2.2 drive_strength |
| `specify_path_delay` | 3 | `1364_legal` | IEEE 1364-2005 Annex A A.7.4 path_delay_value / A.7.2 specify_block |

## Every row

| suite | file | line | construct | edition | text at the stop point |
|---|---|---|---|---|---|
| ispras-sv-tests | `ieee-1364-2005/test_04_03_02_1.v` | 8 | `net_type_tri01` | `1364_legal` | `tor net accessibility\n\n// ! TYPE: POSITIVE\n\nmodule t` |
| ispras-sv-tests | `ieee-1364-2005/test_08_04_00_1.v` | 28 | `udp_table_entry` | `1364_legal` | `+\n    // obtain output on rising edge of clock\n       ` |
| ispras-sv-tests | `ieee-1364-2005/test_08_06_00_1.v` | 15 | `udp_table_entry` | `1364_legal` | `q+\n    // obtain output on rising edge of clock\n    (0` |
| ispras-sv-tests | `ieee-1364-2005/test_08_07_00_1.v` | 18 | `logic_keyword` | `1800_only` | `// clock  jk pc state output/next state\n      ?     ?? ` |
| ispras-sv-tests | `ieee-1364-2005/test_15_04_00_1.v` | 16 | `(unclassified)` | `UNCLASSIFIED` | `or example, the construct\n    $width(posedge clr, tWidt` |
| ispras-sv-tests | `ieee-1364-2005/test_19_11_00_1.v` | 15 | `(unclassified)` | `UNCLASSIFIED` | `Std 1364-2001 Verilog keywords\nmodule m2;\n  wire [63:0` |
| iverilog | `ivtest/ivltests/br_gh1178a.v` | 16 | `logic_keyword` | `1800_only` | `cated, ({2{bus}}));\nendmodule\n\n\nmodule tb;\n  logic ` |
| iverilog | `ivtest/ivltests/br_gh1178b.v` | 15 | `logic_keyword` | `1800_only` | `ed, ({bus, bus}));\nendmodule\n\n\nmodule tb;\n  logic p` |
| iverilog | `ivtest/ivltests/br_gh1178c.v` | 17 | `logic_keyword` | `1800_only` | `ated, ({2{bus}}));\nendmodule\n\n\nmodule tb;\n  logic p` |
| iverilog | `ivtest/ivltests/br_gh1182.v` | 3 | `logic_keyword` | `1800_only` | `module top;\n  initial begin\n    logic [7:0] data;\n   ` |
| iverilog | `ivtest/ivltests/br_gh356a.v` | 15 | `net_type_tri01` | `1364_legal` | `ecify\n\nendmodule\n\nmodule test();\n\nreg     EN;\nreg` |
| iverilog | `ivtest/ivltests/br_gh531.v` | 39 | `assignment_operator` | `1800_only` | `[2:1] tmp;\n  begin\n    tmp[3:0]  = 4'b0000;\n    tmp[3` |
| iverilog | `ivtest/ivltests/br_gh782c.v` | 33 | `gate_strength` | `1364_legal` | ``comment */ // comment\n\n/* comment */`unconnected_drive`` |
| iverilog | `ivtest/ivltests/br_gh782d.v` | 9 | `macro_directive` | `preproc` | ``efine /* comment */ // comment\n\n/* comment */`resetall`` |
| iverilog | `ivtest/ivltests/br_gh782e.v` | 4 | `macro_directive` | `preproc` | ``/ `begin_keywords /* comment */ // comment\n/* comment *`` |
| iverilog | `ivtest/ivltests/br_gh782f.v` | 4 | `macro_directive` | `preproc` | `` `default_nettype /* comment */ // comment\n/* comment */ `` |
| iverilog | `ivtest/ivltests/macro_redefinition.v` | 1 | `macro_directive` | `preproc` | ``module test();\n\n`define MACRO 1\n`define MACRO 1\n`def`` |
| iverilog | `ivtest/ivltests/macro_replacement.v` | 1 | `macro_directive` | `preproc` | ``module test();\n\n`define MACRO 1\n`define MACRO 1\n`def`` |
| iverilog | `ivtest/ivltests/module_output_port_var2.v` | 4 | `(unclassified)` | `UNCLASSIFIED` | `d\n// as variables and be assigned a value.\n\nmodule te` |
| iverilog | `ivtest/ivltests/pr1032.v` | 12 | `net_type_tri01` | `1364_legal` | `come\n   // from the strength contained in the r-value.\` |
| iverilog | `ivtest/ivltests/pr1520314.v` | 16 | `(unclassified)` | `UNCLASSIFIED` | `fy\n $setuphold(posedge A &&& B, BL[0], 0, 0, C,,,D, BL_` |
| iverilog | `ivtest/ivltests/pr1664684.v` | 17 | `net_type_tri01` | `1364_legal` | `[31:0] up14;\n   wire [31:0] rdo;\n   reg        rm, cpe` |
| iverilog | `ivtest/ivltests/pr1716276.v` | 25 | `(unclassified)` | `UNCLASSIFIED` | `defined macro\n   // expanding to null\n\n    param_test` |
| iverilog | `ivtest/ivltests/pr1867332.v` | 10 | `specify_path_delay` | `1364_legal` | `cify specparam\n   tps = 0.0;\n   $setup(posedge CSB, ed` |
| iverilog | `ivtest/ivltests/pr1877743.v` | 84 | `specify_path_delay` | `1364_legal` | `\n  assign {co, s} = a + b + ci;\n\n  specify\n    (a, b` |
| iverilog | `ivtest/ivltests/pr2834340.v` | 6 | `gate_strength` | `1364_legal` | `module test;\n\nreg c1reg,c2reg;\npulldown (weak0) pd1 (` |
| iverilog | `ivtest/ivltests/pr2834340b.v` | 14 | `gate_strength` | `1364_legal` | `the arm.\n  pulldown pd1 (rla);\n  pullup (weak1) pu1 (r` |
| iverilog | `ivtest/ivltests/pr3194155.v` | 10 | `(unclassified)` | `UNCLASSIFIED` | `, x);\nendmodule\n\nmodule tb;\nwire y;\nwire z;\nfoo #1` |
| iverilog | `ivtest/ivltests/pr841.v` | 22 | `net_type_tri01` | `1364_legal` | `(clk);\n\nendmodule\n\n\nmodule pll (inclk0);\ninput  in` |
| iverilog | `ivtest/ivltests/random.v` | 6 | `inc_dec_operator` | `1800_only` | `st;\n\ninteger i;\n\ninitial begin\n  for (i = 0; i < 10` |
| iverilog | `ivtest/ivltests/specify_01.v` | 13 | `specify_path_delay` | `1364_legal` | `am tR_clk_q = 100,\n            tF_clk_q = 150;\n\n  (cl` |
| iverilog | `ivtest/ivltests/string12.v` | 4 | `(unclassified)` | `UNCLASSIFIED` | `module top;\n\n  initial begin\n    if ("this matches" =` |
| iverilog | `ivtest/ivltests/tri0.v` | 28 | `net_type_tri01` | `1364_legal` | `iver\n * otherwise.\n */\nmodule main;\n\n   reg enable,` |
| iverilog | `ivtest/ivltests/tri0b.v` | 28 | `net_type_tri01` | `1364_legal` | `iver\n * otherwise.\n */\nmodule main;\n\n   reg enable,` |
| iverilog | `ivtest/ivltests/tri1.v` | 28 | `net_type_tri01` | `1364_legal` | `iver\n * otherwise.\n */\nmodule main;\n\n   reg enable,` |
| iverilog | `ivtest/ivltests/tri3.v` | 1 | `net_type_tri01` | `1364_legal` | `module test();\n\ntri1 a;\ntri0 b;\n\nassign (pull1,pull` |
| iverilog | `ivtest/ivltests/udp_bx.v` | 11 | `udp_table_entry` | `1364_legal` | `1  : ? : -;\n       b   ?   0  : ? : 0;\n       ?   ?  (` |
| iverilog | `ivtest/ivltests/udp_dff.v` | 80 | `udp_table_entry` | `1364_legal` | `otifier;\n   table\n      // C D notifier : Q : Q+\n    ` |
| iverilog | `ivtest/ivltests/udp_dff_std.v` | 82 | `udp_table_entry` | `1364_legal` | `otifier;\n   table\n      // C D notifier : Q : Q+\n    ` |
| iverilog | `ivtest/ivltests/z1.v` | 11 | `(unclassified)` | `UNCLASSIFIED` | `parameter Q = 2;\nendmodule\n\nmodule foo;\n  bar #345 b` |
| iverilog | `ivtest/ivltests/z2.v` | 11 | `(unclassified)` | `UNCLASSIFIED` | `baz #(P+1) baz1();\nendmodule\n\nmodule foo;\n  bar #3 b` |
| sv2v | `test/core/array_in_package.v` | 5 | `inc_dec_operator` | `1800_only` | `n : foo\n        integer i, j;\n        for (i = 0; i < ` |
| sv2v | `test/core/data_lifetime.v` | 6 | `inc_dec_operator` | `1800_only` | `;\n        lcg = x;\n        for (temp = 0; temp < 3; te` |
| sv2v | `test/core/do_while.v` | 6 | `inc_dec_operator` | `1800_only` | `) begin\n            $display("hi %0d", x);\n           ` |
| sv2v | `test/core/for_decl.v` | 50 | `inc_dec_operator` | `1800_only` | `ger j, k;\n    initial begin\n        for (j = 0; j < 4;` |
| sv2v | `test/core/inside_expr.v` | 13 | `inc_dec_operator` | `1800_only` | `: foo\n        reg [1:0] a;\n        for (a = 0; a < 3; ` |
| sv2v | `test/core/interface_generate.v` | 19 | `assignment_operator` | `1800_only` | `r = 0;\n    always @(posedge clock) begin\n        count` |
| sv2v | `test/core/jump.v` | 51 | `inc_dec_operator` | `1800_only` | `: loop_y\n        integer i;\n        for (i = 0; i < 10` |
| sv2v | `test/core/multipack_port.v` | 9 | `inc_dec_operator` | `1800_only` | `integer i, j;\n        for (i = (CHUNKS-1); i >= 0; i--)` |
| sv2v | `test/core/packed_array.v` | 5 | `inc_dec_operator` | `1800_only` | `e\n        integer i, j, k;\n        for (i = 0; i <= 1;` |
| sv2v | `test/core/part_select_bits.v` | 8 | `inc_dec_operator` | `1800_only` | `integer i;\n        arr = 0;\n        for (i = 0; i < A;` |
| sv2v | `test/core/unpacked_localparam.v` | 5 | `assignment_operator` | `1800_only` | `foo\n        integer i, j;\n        for (i = 3; i >= 0; ` |
| sv2v | `test/core/unsigned.v` | 5 | `inc_dec_operator` | `1800_only` | `ock_name\n        integer i;\n        for (i = 0; i < 4;` |

⛔ **This census adjudicates the CONSTRUCT, not the row.** Whether a given file is
`must_reject` or `divergence:explained` with an edition-contrast reason is `.13k`(a)
and `(c)`; editing the key so it agrees with the parser is the gaming
`EXTERNAL-CORPUS` forbids, and it is not done here.


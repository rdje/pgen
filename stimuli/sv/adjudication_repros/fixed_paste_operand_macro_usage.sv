// SVPP-EXPANSION.5(i) - a macro usage that is an OPERAND of a `` paste is expanded before the join,
// and the constructed name is looked up whole: `CHOICE_``A```SUFFIX with A=FOO is `CHOICE_FOO_MAGIC
// (3), not `CHOICE_FOO followed by `SUFFIX (1_MAGIC, which no parser accepts). sv2v's own reference
// (test/lex/macro_macro.v) and Verilator 5.050 read 3 , 4. Corpus row: sv2v test/lex/macro_macro.sv:16.
`define SUFFIX _MAGIC
`define CHOICE_FOO 1
`define CHOICE_BAR 2
`define CHOICE_FOO_MAGIC 3
`define CHOICE_BAR_MAGIC 4
`define MACRO2(A, B) `CHOICE_``A```SUFFIX , `CHOICE_``B```SUFFIX
module top;
    initial $display(`MACRO2(FOO, BAR));
endmodule

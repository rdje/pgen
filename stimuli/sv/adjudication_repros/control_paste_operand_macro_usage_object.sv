// SVPP-EXPANSION.5(i) - the CONTROL: the same paste-with-a-usage-operand inside an OBJECT macro
// (sv2v macro_macro.sv:17, `MACRO3) read 5 , 5 before this slice and must read it after: the object
// path already joined left to right; only the function-macro path had deleted its pastes early.
`define SUFFIX _MAGIC
`define CHOICE__MAGIC 5
`define MACRO3 `CHOICE_```SUFFIX , `CHOICE_```SUFFIX
module top;
    initial $display(`MACRO3);
endmodule

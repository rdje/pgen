// SV-CORPUS-GRAD.13c.2c — the FUNCTION-CALL leg of the same defect. IEEE 1364-2005 A.8.2:
//   function_call ::= hierarchical_function_identifier { attribute_instance } ( expression { , expression } )
// with `hierarchical_function_identifier ::= hierarchical_identifier` (A.9.3), so an indexed
// instance-array component in the receiver path is legal in plain Verilog.
// Expected before the fix: REJECT (three profiles).  After: ACCEPT (three profiles).
module sub; function integer f(input integer a); f = a; endfunction endmodule
module top; sub u1[1:0](); integer y; initial begin y = top.u1[0].f(1); end endmodule

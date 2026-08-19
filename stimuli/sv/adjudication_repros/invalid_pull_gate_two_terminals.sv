// SV-CORPUS-GRAD.13c.2k — IEEE 1364-2005 A.3.1 gives
// `pull_gate_instance ::= [ name_of_gate_instance ] ( output_terminal )` — exactly ONE terminal.
// `pulldown (pd1, pd2);` is two, and the legal spelling is `pulldown (pd1), (pd2);`.
// Shape from iverilog/ivtest/ivltests/pr1787423.v, accepted until .13c.2k closed the UDP route.
module top;
  wire pd1, pd2;
  pulldown (pd1, pd2);
endmodule

// SV-CORPUS-GRAD.20 — the legal spelling behind invalid_pull_gate_two_terminals.sv, with a strength:
// `pulldown [ pulldown_strength ] pull_gate_instance` and `pull_gate_instance ::= [ name_of_gate_instance ]
// ( output_terminal )` — exactly ONE terminal (1364-2005 A.3.1 / 1800-2017 A.3.1). Shape from
// iverilog/ivtest/ivltests/pr2834340.v, whose `pulldown (weak0) pd1 (r1a,r1c,r1o);` gives three.
module top;
  wire r1a;
  pulldown (weak0) pd1 (r1a);
endmodule

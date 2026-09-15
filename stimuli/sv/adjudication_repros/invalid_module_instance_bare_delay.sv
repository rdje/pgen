// SV-CORPUS-GRAD.20 — a bare `#1` between a MODULE name and its instance name.
// IEEE 1364-2005 A.4.1: `module_instantiation ::= module_identifier [ parameter_value_assignment ]
// module_instance { , module_instance } ;` with `parameter_value_assignment ::= # ( list_of_parameter_assignments )`
// — the parentheses are part of the production; a bare delay belongs to GATE and UDP instantiations
// (A.3.1 / A.5.4), whose instances need terminals. IEEE 1800-2017 A.4.1.1 is the same
// (`# ( [ list_of_parameter_assignments ] )`). Shape from iverilog/ivtest/ivltests/pr3194155.v, z1.v,
// z2.v (`foo #1 bar1(1'b0);`, `bar #345 bar1();`), which Icarus accepts as an extension.
module foo(input a);
endmodule
module top;
  foo #1 bar1(1'b0);
endmodule

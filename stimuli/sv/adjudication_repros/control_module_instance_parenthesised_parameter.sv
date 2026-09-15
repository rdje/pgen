// SV-CORPUS-GRAD.20 — the legal spelling of invalid_module_instance_bare_delay.sv:
// a parameter value assignment carries its parentheses (1364-2005 A.4.1.1 / 1800-2017 A.4.1.1).
module foo #(parameter D = 0) (input a);
endmodule
module top;
  foo #(1) bar1(1'b0);
endmodule

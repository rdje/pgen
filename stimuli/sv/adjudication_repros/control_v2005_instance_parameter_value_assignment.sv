// SV-CORPUS-GRAD.48 — the control for accepts_invalid_v2005_instance_empty_parameter_value_assignment.sv: the same
// instance with ONE ordered parameter assignment, IEEE 1364-2005 A.4.1.1 (docs/verilog/2005/txt/section-Annex_A-
// normative-formal-syntax-definition.txt:399-402). ACCEPT on every profile.
module mod #(parameter P = 1) (input a);
endmodule
module t;
  mod #(2) m1(1'b1);
endmodule

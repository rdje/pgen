// SV-CORPUS-GRAD.48 — ILLEGAL under IEEE 1364-2005, ACCEPTED by PGEN today (class accepts_invalid; WATCHED).
// `mod #() m1(1'b1);` — an EMPTY parameter value assignment on a module instance. IEEE 1364-2005 A.4.1.1
// parameter_value_assignment ::= # ( list_of_parameter_assignments ), list_of_parameter_assignments ::=
// ordered_parameter_assignment { , ordered_parameter_assignment } | named_parameter_assignment { , … }
// (docs/verilog/2005/txt/section-Annex_A-normative-formal-syntax-definition.txt:399-402): the list is never empty.
// IEEE 1800-2017 A.4.1.1 parameter_value_assignment ::= # ( [ list_of_parameter_assignments ] )
// (docs/systemverilog/2017/md/section-41-data-read-api.md:1795) admits it, and PGEN's parameter_value_assignment_sv_2017
// (gated sv_2017 + verilog_2005) carries the 1800 shape. iverilog 13.0 -g2005 ACCEPTS it too (rc 0) — a tool leniency;
// the clause decides. Measured by .48 while splitting the parameter PORT list; owned by SV-CORPUS-GRAD.50. Control:
// control_v2005_instance_parameter_value_assignment.sv.
module mod #(parameter P = 1) (input a);
endmodule
module t;
  mod #() m1(1'b1);
endmodule

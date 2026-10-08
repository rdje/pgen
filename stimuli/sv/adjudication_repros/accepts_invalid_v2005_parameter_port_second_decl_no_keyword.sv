// SV-CORPUS-GRAD.13e.14 — ILLEGAL under IEEE 1364-2005, ACCEPTED by PGEN today (class accepts_invalid; WATCHED).
// `module a #(parameter real A = 1.0, integer B = 2);` — the SECOND declaration omits `parameter` before its type.
// IEEE 1364-2005 A.1.3 module_parameter_port_list ::= # ( parameter_declaration { , parameter_declaration } )
// (docs/verilog/2005/txt/section-Annex_A-normative-formal-syntax-definition.txt:38): each comma-separated
// declaration is a parameter_declaration and begins with `parameter` (:126-128). Corpus row:
// ivtest/ivltests/parameter_omit3.v. Control: control_v2005_parameter_port_list.sv.
module a #(parameter real A = 1.0, integer B = 2);
endmodule

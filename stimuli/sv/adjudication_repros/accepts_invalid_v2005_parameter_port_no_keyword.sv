// SV-CORPUS-GRAD.13e.14 — ILLEGAL under IEEE 1364-2005, REFUSED from release 1.0.252 (class invalid; SV-CORPUS-GRAD.48).
// `module a #(A = 1);` — a parameter port list whose declaration omits `parameter`. IEEE 1364-2005 A.1.3
// module_parameter_port_list ::= # ( parameter_declaration { , parameter_declaration } )
// (docs/verilog/2005/txt/section-Annex_A-normative-formal-syntax-definition.txt:38) and A.2.1.1
// parameter_declaration ::= parameter [ signed ] [ range ] list_of_param_assignments | parameter parameter_type
// list_of_param_assignments (:126-128): every declaration BEGINS with `parameter`. IEEE 1800-2017 A.1.3
// parameter_port_list ::= # ( list_of_param_assignments { , parameter_port_declaration } ) | … admits it, and the
// grammar's single `parameter_port_list` rule serves every profile with that 1800 shape — the over-acceptance
// SV-CORPUS-GRAD.48 closed in 1.0.252 (parameter_port_list_v2005). Corpus row: ivtest/ivltests/parameter_omit1.v (the file's own header: "In Verilog this is
// not allowed and should result in an error"). Control: control_v2005_parameter_port_list.sv.
module a #(A = 1);
endmodule

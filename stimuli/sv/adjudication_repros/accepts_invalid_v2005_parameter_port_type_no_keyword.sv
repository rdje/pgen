// SV-CORPUS-GRAD.13e.14 — ILLEGAL under IEEE 1364-2005, REFUSED from release 1.0.252 (class invalid; SV-CORPUS-GRAD.48).
// `module a #(integer A = 1);` — a typed parameter port declaration without `parameter`. IEEE 1364-2005 A.1.3 /
// A.2.1.1 (docs/verilog/2005/txt/section-Annex_A-normative-formal-syntax-definition.txt:38, :126-128): every
// declaration begins with `parameter`; `parameter integer A = 1` is the 1364-2005 spelling. IEEE 1800-2017 A.1.3
// parameter_port_declaration ::= … | data_type list_of_param_assignments admits it. Corpus row:
// ivtest/ivltests/parameter_omit2.v. Control: control_v2005_parameter_port_list.sv.
module a #(integer A = 1);
endmodule

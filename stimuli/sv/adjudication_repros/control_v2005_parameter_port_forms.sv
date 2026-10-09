// SV-CORPUS-GRAD.48 — the control for the verilog_2005 twin's whole upper bound: every form IEEE 1364-2005 A.2.1.1
// parameter_declaration ::= parameter [ signed ] [ range ] list_of_param_assignments | parameter parameter_type
// list_of_param_assignments, parameter_type ::= integer | real | realtime | time
// (docs/verilog/2005/txt/section-Annex_A-normative-formal-syntax-definition.txt:126-130), five declarations in one
// A.1.3 module_parameter_port_list (:38). ACCEPT on every profile; the typed AST is the 1.0.251 tree byte for byte.
module a #(parameter signed [3:0] X = 1, parameter integer Y = 2, parameter real Z = 1.0, parameter realtime W = 1.0, parameter time V = 1);
endmodule

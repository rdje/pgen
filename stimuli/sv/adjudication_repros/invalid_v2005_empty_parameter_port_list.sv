// SV-CORPUS-GRAD.48 (release 1.0.252) — ILLEGAL under IEEE 1364-2005: an EMPTY parameter port list. IEEE 1364-2005 A.1.3
// module_parameter_port_list ::= # ( parameter_declaration { , parameter_declaration } )
// (docs/verilog/2005/txt/section-Annex_A-normative-formal-syntax-definition.txt:38) derives at least one declaration;
// IEEE 1800-2017 A.1.3 parameter_port_list ::= … | #( ) (docs/systemverilog/2017/md/section-41-data-read-api.md:231-236)
// admits the empty list, which is why the same text is the sv_2017 / sv_2023 control
// control_sv_empty_parameter_port_list.sv. iverilog 13.0 -g2005: `syntax error`. Through 1.0.251 the one
// parameter_port_list rule served every profile with the 1800 shape; parameter_port_list_v2005 refuses it.
module a #();
endmodule

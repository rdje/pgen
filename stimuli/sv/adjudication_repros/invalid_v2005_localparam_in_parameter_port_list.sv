// SV-CORPUS-GRAD.48 (release 1.0.252) — ILLEGAL under IEEE 1364-2005: a `localparam` declaration inside the parameter
// port list. IEEE 1364-2005 A.1.3 module_parameter_port_list ::= # ( parameter_declaration { , parameter_declaration } )
// (docs/verilog/2005/txt/section-Annex_A-normative-formal-syntax-definition.txt:38) derives only parameter_declaration
// (A.2.1.1 :126-128, `parameter`-led); local_parameter_declaration is derived only as a module item and a generate
// item (:67, :319). IEEE 1800-2017 A.1.3 parameter_port_declaration ::= … | local_parameter_declaration
// (docs/systemverilog/2017/md/section-41-data-read-api.md:238-243) admits it: the same text is the sv_2017 / sv_2023
// control control_sv_localparam_in_parameter_port_list.sv. iverilog 13.0 -g2005: `error: Local parameter in module
// parameter port list requires SystemVerilog.`
module a #(parameter X = 1, localparam Y = 2);
endmodule

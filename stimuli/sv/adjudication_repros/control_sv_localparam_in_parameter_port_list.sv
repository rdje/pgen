// SV-CORPUS-GRAD.48 — the IEEE 1800 control for invalid_v2005_localparam_in_parameter_port_list.sv: the same text, legal
// under IEEE 1800-2017 A.1.3 parameter_port_declaration ::= … | local_parameter_declaration
// (docs/systemverilog/2017/md/section-41-data-read-api.md:238-243). ACCEPT under sv_2017 / sv_2023.
module a #(parameter X = 1, localparam Y = 2);
endmodule

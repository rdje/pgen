// SV-CORPUS-GRAD.48 — the IEEE 1800 control for invalid_v2005_empty_parameter_port_list.sv: the same text, legal under
// IEEE 1800-2017 A.1.3 parameter_port_list ::= … | #( ) (docs/systemverilog/2017/md/section-41-data-read-api.md:231-236).
// ACCEPT under sv_2017 / sv_2023 (parameter_port_list_sv_only's `empty` alternative).
module a #();
endmodule

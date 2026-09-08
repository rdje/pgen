// SV-CORPUS-GRAD.3.52 - a NET TYPE on an ANSI port-expression declaration: `output wire .ao_rename(ao)` (verilator t_inst_port_complex_unsup.v:10). IEEE 1800-2017 A.1.3 ansi_port_declaration ::= [ net_port_header | interface_port_header ] port_identifier { unpacked_dimension } [ = constant_expression ] | [ variable_port_header ] port_identifier { variable_dimension } [ = constant_expression ] | [ port_direction ] . port_identifier ( [ expression ] ) (docs/systemverilog/2017/md/section-41-data-read-api.md:292-298) - the port-expression alternative takes a direction only, never a net type; 1800-2023 the same (docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:265-271).
// REJECTS at the `.` on both SV profiles; the control `output .ao_rename(ao)` ACCEPTS.
module m(output wire .ao_rename(ao));
  wire ao;
endmodule

// SV-CORPUS-GRAD.13e.13 — REJECT FOREVER under verilog_2005.
// `input wire real in` — IEEE 1364-2005 A.2.1.2 input_declaration ::= input [ net_type ] [ signed ]
// [ range ] list_of_port_identifiers (docs/verilog/2005/txt/section-Annex_A-normative-formal-syntax-definition.txt:135)
// has no real port of any kind; `real` is neither a net_type (A.2.2.1) nor a range. iverilog's real
// nets are its own extension. The same bytes are legal under IEEE 1800 (A.2.1.2 net_port_type ::=
// [ net_type ] data_type_or_implicit, docs/systemverilog/2017/md/section-41-data-read-api.md:900):
// control_ansi_input_wire_real_sv.sv.
// Corpus row: ivtest/ivltests/pr2976242.v (regress-ivl1.list, normal, the default generation).
module m(input wire real in);
endmodule

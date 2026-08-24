// SV-CORPUS-GRAD.13e.7 — an OVER-ACCEPTANCE under verilog_2005, expected ACCEPT until fixed.
// 1364-2005 A.1.2 module_declaration (docs/verilog/2005/txt/section-Annex_A-normative-formal-syntax-definition.txt:29) ::= { attribute_instance } module_keyword module_identifier [ module_parameter_port_list ] ... - no lifetime in either alternative
// WHERE: module_ansi_header:3679 carries the IEEE 1800 ( lifetime )? ungated. FOUND while PRICING .13e.7(a), not by the census - the L2 class again
// Pinned BEFORE the fix so the ratchet goes RED the day it lands (flip to class=invalid).
module automatic test;
endmodule

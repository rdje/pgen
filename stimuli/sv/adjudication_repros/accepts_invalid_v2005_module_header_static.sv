// SV-CORPUS-GRAD.13e.7 — an OVER-ACCEPTANCE under verilog_2005, expected ACCEPT until fixed.
// 1364-2005 A.1.2 module_declaration (docs/verilog/2005/txt/section-Annex_A-normative-formal-syntax-definition.txt:29) - no lifetime; and `static` is not even an Annex B keyword
// WHERE: the sibling spelling of module_header_automatic; pinned separately because `lifetime` has two branches and a gate could fix one
// Pinned BEFORE the fix so the ratchet goes RED the day it lands (flip to class=invalid).
module static test;
endmodule

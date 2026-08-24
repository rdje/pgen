// SV-CORPUS-GRAD.13e.7 — an OVER-ACCEPTANCE under verilog_2005, expected ACCEPT until fixed.
// 1364-2005 Annex A (docs/verilog/2005/txt/section-Annex_A-normative-formal-syntax-definition.txt) - `extern` occurs 0 times; A.1.2:29 module_declaration has no extern alternative
// WHERE: module_declaration_sv_2017 + udp_declaration_sv_2017; NOT reachable through any declaration rule, which is half the argument .13e.7(a) refuses the structural sibling on
// Pinned BEFORE the fix so the ratchet goes RED the day it lands (flip to class=invalid).
extern module test;

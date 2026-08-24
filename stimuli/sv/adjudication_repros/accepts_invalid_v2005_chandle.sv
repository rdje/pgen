// SV-CORPUS-GRAD.13e.7 — an OVER-ACCEPTANCE under verilog_2005, expected ACCEPT until fixed.
// 1364-2005 Annex A (docs/verilog/2005/txt/section-Annex_A-normative-formal-syntax-definition.txt) - `chandle` occurs 0 times in the whole Annex; the v2005 variable types are the A.2.1.3 set integer/time/real/realtime/reg/event only
// WHERE: reaches data_type/block_data_type wholesale because data_declaration_sv_2017 is @profiles-admitted to verilog_2005 and data_type is ungated
// Pinned BEFORE the fix so the ratchet goes RED the day it lands (flip to class=invalid).
module test;
  chandle c;
endmodule

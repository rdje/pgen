// SV-CORPUS-GRAD.13e.7 — an OVER-ACCEPTANCE under verilog_2005, expected ACCEPT until fixed.
// 1364-2005 Annex A (docs/verilog/2005/txt/section-Annex_A-normative-formal-syntax-definition.txt) - bare `const` occurs 0 times; A.2.1.3 integer_declaration:152 ::= integer list_of_variable_identifiers ; carries no qualifier
// WHERE: data_declaration_sv_2017's ( kw_const )? optional, slot const_keyword; the fix gates the OPTIONAL, not the keyword
// Pinned BEFORE the fix so the ratchet goes RED the day it lands (flip to class=invalid).
module test;
  const integer i = 1;
endmodule

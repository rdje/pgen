// SV-CORPUS-GRAD.13e.7 — an ACCEPTING CONTROL: legal IEEE 1364-2005 that must keep parsing.
// 1364-2005 A.2.6 function_declaration (docs/verilog/2005/txt/section-Annex_A-normative-formal-syntax-definition.txt:254) ::= function [ automatic ] [ function_range_or_type ] function_identifier ;
// WHY: pins the static/automatic gates to the DECLARATION-SITE optional: if `lifetime` itself were gated this control goes RED
module test;
  function automatic integer f;
    f = 1;
  endfunction
endmodule

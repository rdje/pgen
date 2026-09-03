// SV-CORPUS-GRAD.13x.b - an INVALID pin: text no Annex A production derives - must keep REJECTING.
// generate begin eh2_ram dccm_bank (); end endgenerate - a bare block directly in a generate region
// IEEE 1800-2017 A.4.2 generate_region ::= generate { generate_item } endgenerate section-41-data-read-api.md:1853; generate_item is a module_or_generate_item section-41-data-read-api.md:338-345; generate_block only via loop/conditional generate section-41-data-read-api.md:1903; IEEE 1364-2005 A.4.2 identical. t_var_dotted_dup_bad.v l.9-16.
module eh2_ram; endmodule
module t;
  generate
    begin
      eh2_ram dccm_bank ();
    end
  endgenerate
endmodule

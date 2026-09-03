// SV-CORPUS-GRAD.13x.b - a CONTROL: the nearest LEGAL twin (one token different) - must keep ACCEPTING.
// generate if (1) begin eh2_ram dccm_bank (); end endgenerate
// IEEE 1800-2017 A.4.2 conditional_generate_construct / if_generate_construct section-41-data-read-api.md:1880-1903. isolates the defect to the missing generate construct around the block.
module eh2_ram; endmodule
module t;
  generate
    if (1) begin
      eh2_ram dccm_bank ();
    end
  endgenerate
endmodule

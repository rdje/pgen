// SV-CORPUS-GRAD.13x.b - an INVALID pin: text no Annex A production derives - must keep REJECTING.
// import "DPI-BAD" task dpix_twice; - a spec string that is neither "DPI-C" nor "DPI"
// IEEE 1800-2017 A.2.6 dpi_spec_string ::= "DPI-C" | "DPI" section-41-data-read-api.md:1114. t_dpi_type_bad.v l.11.
module t;
  import "DPI-BAD" task dpix_twice;
endmodule

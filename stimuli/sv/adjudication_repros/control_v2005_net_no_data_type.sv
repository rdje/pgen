// SV-CORPUS-GRAD.13e.2 — the one-difference ACCEPT control for invalid_v2005_net_data_type.sv.
// The same net declaration with the data type removed: `wire [7:0] b;` is exactly A.2.1.3's
// `net_type [vectored|scalared] [signed] range [delay3] list_of_net_identifiers ;`.
// It parses, so the rejection above is attributable to the data type alone and not to the range,
// the net_type, or the module wrapper.
module test();
  wire [7:0] b;
endmodule

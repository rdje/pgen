// SV-CORPUS-GRAD.13e.2 — the one-difference ACCEPT control for
// invalid_v2005_unbased_unsized_literal.sv.
// The same comparison with the literal given a base, `2'bxx`, which is exactly A.8.7
// binary_number ::= [ size ] binary_base binary_value. It parses, so the rejection above is
// attributable to the MISSING BASE alone and not to `!==`, to the `if`, or to the reg width.
module test;
  reg [1:0] r;
  initial if (r !== 2'bxx) r = 0;
endmodule

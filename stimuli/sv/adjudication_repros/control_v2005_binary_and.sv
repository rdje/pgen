// SV-CORPUS-GRAD.13e.2 — the one-difference ACCEPT control for invalid_v2005_binary_nand_nor.sv.
// The same two statements with the operators replaced by their `&` / `|` forms, which A.8.6
// binary_operator DOES list. It parses, so the rejection above is attributable to the `~` prefix
// on a binary operator and not to the operands, the initial block, or the reg widths.
module test;
  reg [7:0] a, b, c;
  initial begin
    c = a & b;
    c = a | b;
  end
endmodule

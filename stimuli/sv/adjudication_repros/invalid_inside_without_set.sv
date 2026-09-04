// SV-CORPUS-GRAD.3.41 - INVALID: `inside` with no braced set - REJECTS forever.
// y = a inside & b; - A.8.3 inside_expression ::= expression inside { open_range_list } requires the literal braces;
// `inside` is reserved (Annex B), so it is not an operand either. Guards the chain element against a brace-less arm.
// IEEE 1800-2017 A.8.3 docs/systemverilog/2017/md/section-41-data-read-api.md:3043; Annex B Table B.1
module t;
  logic [1:0] a;
  logic b, y;
  always_comb y = a inside & b;
endmodule

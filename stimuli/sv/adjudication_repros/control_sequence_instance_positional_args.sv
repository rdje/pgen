// SV-CORPUS-GRAD.13c.2s control for `fixed_sequence_instance_mixed_args.sv`: the SAME sequence
// instance with the NAMED argument replaced by a positional one. It parsed before the fix and
// after, so the flip is attributable to the positional-then-named MIX alone — not to sequence
// arguments, nor to the `##1` delay, nor to the instance itself.
module m;
  logic clk, a, b, d;
  sequence sq(p, q); p ##1 q; endsequence
  assert property (@(posedge clk) sq(a ##1 b, d));
endmodule

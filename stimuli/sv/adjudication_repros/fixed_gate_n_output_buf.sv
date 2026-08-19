// SV-CORPUS-GRAD.13c.2r — `buf` is an `n_output_gatetype`. Its instance rule was unmatchable for
// EVERY input: `( comma output_terminal )* comma input_terminal` let the greedy star eat the
// mandatory trailing input terminal. Two outputs and one input is the shape that proves the star
// still iterates after the `&comma` guard.
module m;
  wire o, o2, i;
  buf g1(o, o2, i);
endmodule

// SV-CORPUS-GRAD.13c.2q/.13c.2r control — `and` is an `n_input_gatetype`, whose token kept its
// spelling and whose instance rule has its star at the END. It parsed through `gate_instantiation`
// BEFORE the fixes and still does, so the four rows above are attributable to their own defects.
module m;
  wire o, i, e;
  and g1(o, i, e);
endmodule

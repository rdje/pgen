// Control for `defect_constant_size_cast.sv`: the SAME cast in an ordinary (non-constant)
// expression parses, so the defect is the CONSTANT-expression path, not the cast syntax.
module m;
  logic [7:0] k;
  initial k = 8'(1);
endmodule

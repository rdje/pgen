// SV-CORPUS-GRAD.3.52 - the twin of invalid_range_select_in_hierarchical_path.sv with a bit-select (A.8.4 constant_bit_select): ACCEPT.
module m;
  I intfX[2:0]();
  M xs(intfX[1].ModportB);
endmodule

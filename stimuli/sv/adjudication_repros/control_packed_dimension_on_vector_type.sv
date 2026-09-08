// SV-CORPUS-GRAD.3.52 - the twin of invalid_packed_dimension_on_integer_atom_type.sv on an integer_vector_type, and the atom type with an unpacked dimension: ACCEPT.
module m;
  function f4(logic [1:0] t1 [3:0]);
  endfunction
  function f5(int t1 [3:0]);
  endfunction
endmodule

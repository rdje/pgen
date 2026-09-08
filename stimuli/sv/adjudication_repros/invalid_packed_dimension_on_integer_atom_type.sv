// SV-CORPUS-GRAD.3.52 - a PACKED dimension on an integer atom type: `function f4(int [var1:var2] t1 [var3:var4]);` (verible kythe testdata/function_port_dimensions.sv:13), `int [foo_pkg::DATA_WIDTH-1:foo_pkg::DATA_LENGTH] var5 […];` (primitive_type_with_dimensions.sv:22). IEEE 1800-2017 A.2.2.1 data_type ::= integer_vector_type [ signing ] { packed_dimension } | integer_atom_type [ signing ] | … (docs/systemverilog/2017/md/section-41-data-read-api.md:842-846) with integer_atom_type ::= byte | shortint | int | longint | integer | time (docs/systemverilog/2017/md/section-41-data-read-api.md:888) - the atom types take a signing and nothing else; 1800-2023 the same (docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:825-829, :885).
// REJECTS at the `[` on both SV profiles; the controls `logic [1:0] t1 [3:0]` and `int t1 [3:0]` ACCEPT.
module m;
  function f4(int [1:0] t1 [3:0]);
  endfunction
endmodule

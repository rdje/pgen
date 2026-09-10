// SV-CORPUS-GRAD.3.59 narrowness lock — a streaming concatenation whose slice_size is a type WITH A PACKED
// DIMENSION: IEEE 1800-2017 A.8.1 slice_size ::= simple_type | constant_expression, A.2.2.1 simple_type ::=
// integer_type | non_integer_type | ps_type_identifier | ps_parameter_identifier — `bit[1:0]` is neither
// (verilator t_stream_unpack.v:89, pinned must_reject).
module t;
  bit [5:0] b6;
  bit [1:0] arr2[3];
  initial begin
    { >> bit[1:0] {arr2}} = b6;
  end
endmodule

// SV-CORPUS-GRAD.3.38 - CONTROL: SIZED packed dimensions, one and several, at every packed_dimension site - the range arm's
// new `!unsized_dimension` lookahead refuses only a following `[]`, never a following `[msb:lsb]` or the identifier.
// Sites: a vector type with two packed dims, a typedef with three, a struct packed with one, an enum base type, an
// implicit-type port, a specparam, a net, a type identifier with a packed dim. Arm !unsized: no unsized node anywhere.
// docs/systemverilog/2017/md/section-41-data-read-api.md:1058-1062 (A.2.5 packed_dimension ::= [ constant_range ] | ...)
module t (input [3:0] p, output logic [1:0][1:0] o);
  logic [7:0][3:0] m;
  typedef logic [1:0][2:0][3:0] T3;
  T3 [1:0] t3;
  struct packed { logic [1:0] a; bit b; } [1:0] s;
  enum logic [1:0] {A, B} e;
  wire [3:0] w;
  reg [1:0] r [0:3];
  specparam [3:0] sp = 4'd1;
endmodule

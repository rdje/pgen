// SV-CORPUS-GRAD.3.38 - INVALID: an unsized packed dimension that is NOT the sole packed dimension - REJECTS forever.
// import "DPI-C" function void f(input logic [7:0] [] x); - footnote 20 permits unsized_dimension "only as the SOLE packed
// dimension in a DPI import declaration"; 35.5.6.1: "an unsized dimension shall be the sole packed dimension of a formal
// argument". The range arm of packed_dimension refuses a following `[]`. ACCEPTED through 1.0.213.
// docs/systemverilog/2017/md/section-41-data-read-api.md:3522-3523 (footnote 20);
// docs/systemverilog/2017/md/section-35-direct-programming-interface.md:561-562 (35.5.6.1)
module t;
  import "DPI-C" function void f(input logic [7:0] [] x);
endmodule

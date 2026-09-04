// SV-CORPUS-GRAD.3.35 - a CONTROL - must keep ACCEPTING.
// localparam X = 1; localparam type T = int; and #(parameter P, type U) - defaults present, and the port-list omission footnote 18 allows
// IEEE 1800-2017 Annex A footnote 18 docs/systemverilog/2017/md/section-41-data-read-api.md:3518-3520
module m #(parameter P, type U) ();
  localparam X = 1;
  localparam type T = int;
  localparam Y [2] = '{1, 2};
endmodule

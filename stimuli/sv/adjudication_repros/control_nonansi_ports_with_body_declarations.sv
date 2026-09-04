// SV-CORPUS-GRAD.13y - a CONTROL - must keep ACCEPTING.
// module t (clk, d); input clk; input d; - a pure non-ANSI list with body declarations
// IEEE 1800-2017 A.1.3 list_of_ports docs/systemverilog/2017/md/section-41-data-read-api.md:246; A.1.2 module_nonansi_header ... { module_item } docs/systemverilog/2017/md/section-41-data-read-api.md:108-113
module t (clk, d);
  input clk;
  input d;
  wire q = clk & d;
endmodule

// SV-CORPUS-GRAD.13x.b - a CONTROL: the nearest LEGAL twin (one token different) - must keep ACCEPTING.
// assert property (@(posedge clk) not val);
// IEEE 1800-2017 A.2.10 section-41-data-read-api.md:1324. isolates the defect to the ##1 before not.
module t; logic clk, val;
  assert property (@(posedge clk) not val);
endmodule

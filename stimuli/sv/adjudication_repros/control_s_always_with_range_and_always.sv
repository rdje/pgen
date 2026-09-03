// SV-CORPUS-GRAD.13x.b - a CONTROL: the nearest LEGAL twin (one token different) - must keep ACCEPTING.
// s_always [1:3] a and always a - the two legal forms
// IEEE 1800-2017 A.2.10 section-41-data-read-api.md:1344-1346. isolates the defect to the missing range on s_always.
module t; logic clk, a;
  assert property (@(posedge clk) s_always [1:3] a);
  assert property (@(posedge clk) always a);
endmodule

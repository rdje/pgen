// SV-CORPUS-GRAD.13c.2s control — the same call with the clocking event removed. It parses, so
// the rejection above is attributable to the clocking-event argument alone.
module m;
  logic clk, a, x;
  always @(posedge clk) x <= $rose(a);
endmodule

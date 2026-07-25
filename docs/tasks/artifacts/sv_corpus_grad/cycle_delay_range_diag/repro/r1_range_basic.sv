// LRM 1800-2017 §16 example: `data ##[1:3] gnt`
module m;
  logic clk, data, gnt;
  property p1; @(posedge clk) data ##[1:3] gnt; endproperty
endmodule

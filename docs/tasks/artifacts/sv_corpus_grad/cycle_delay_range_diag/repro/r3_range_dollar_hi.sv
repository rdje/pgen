// LRM 1800-2017 (:1439): `req ##[4:$] gnt`
module m;
  logic clk, req, gnt;
  property p1; @(posedge clk) req ##[4:$] gnt; endproperty
endmodule

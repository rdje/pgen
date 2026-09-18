import q::*;
module m;
  covergroup cg @(posedge clk);
    lbl : coverpoint cmd { bins a[] = (A,B => C); }
    logic [1:0] tlbl : coverpoint x;
    coverpoint y;
    xl : cross lbl, tlbl;
  endgroup
endmodule

import q::*;
module m;
  covergroup cg @(posedge clk);
    lbl : coverpoint cmd { bins a[] = (A,B => C); }
    xl : cross lbl, cmd;
  endgroup
endmodule

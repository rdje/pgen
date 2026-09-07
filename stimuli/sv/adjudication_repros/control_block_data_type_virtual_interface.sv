interface I; endinterface
module m;
  initial begin
    virtual interface I vi;
    vi = null;
  end
endmodule

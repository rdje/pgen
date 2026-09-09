module m;
  wire a, b, c, d;
  assign a = b;
//pragma translate_off
  ggggll
//pragma translate_on
  assign c = d;
endmodule

module m;
  reg q;
  reg a [3:0];
  initial foreach (a[i]) q = 1'b1;
endmodule

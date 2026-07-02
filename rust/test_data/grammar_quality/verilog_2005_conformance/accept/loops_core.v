module m;
  integer i;
  reg q;
  initial begin
    forever q = 0;
  end
  initial repeat (3) q = 1;
  initial while (0) q = 0;
  initial for (i = 0; i < 4; i = i + 1) q = 1;
endmodule

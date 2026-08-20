class C; rand int a; endclass
module m;
  C c;
  initial if (c.randomize(a) with { a < 5; }) $display("ok");
endmodule

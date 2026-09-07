module m;
  initial begin
    struct packed { int a; } s;
    s.a = 1;
  end
endmodule

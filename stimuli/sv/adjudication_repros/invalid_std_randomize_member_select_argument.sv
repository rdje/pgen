class C; int x; endclass
module m;
  int a; C obj;
  initial begin obj = new;
    if (std::randomize(a, obj.x) with { a > 10; obj.x > a; } != 1) $stop;
  end
endmodule

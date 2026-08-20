module m;
  int a, b;
  initial begin
    if (std::randomize(a, b) with { a < b; }) $display("ok");
    if (randomize(a) with { a > 0; }) $display("ok");
  end
endmodule

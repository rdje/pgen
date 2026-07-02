module m (d, clk);
  input d;
  input clk;
  specify
    $setup(d, posedge clk, 1);
    $hold(posedge clk, d, 1);
    $setuphold(posedge clk, d, 1, 1);
    $recovery(posedge clk, d, 1);
    $removal(posedge clk, d, 1);
    $recrem(posedge clk, d, 1, 1);
    $skew(posedge clk, posedge d, 1);
    $timeskew(posedge clk, posedge d, 1);
    $fullskew(posedge clk, posedge d, 1, 1);
    $period(posedge clk, 1);
    $width(posedge clk, 1, 0);
    $nochange(posedge clk, d, 0, 0);
  endspecify
endmodule

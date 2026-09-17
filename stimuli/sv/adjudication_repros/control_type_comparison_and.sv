module m(input clk); int a, b, c; initial if (type(b) == type(a) && $rose(c, @(posedge clk))) $display("x"); endmodule

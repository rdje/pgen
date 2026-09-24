module m(input clk, input [3:0] d, output [3:0] q); specify $width(posedge clk, 1); endspecify endmodule

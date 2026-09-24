module m(a[3:0], b); input [3:0] a; input b; wire [3:0] w; assign w[1] = 1'b0; assign w[1:0] = 2'b0; endmodule

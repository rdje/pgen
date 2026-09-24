module m; reg [7:0] a; reg [3:0] b [0:1]; reg [7:0] x; wire [7:0] w; assign w[1] = 1'b0; assign w[3:2] = 2'b0; initial begin x = a[1]; x = a[1:0]; x = a[1+:2]; x = b[1]; x = b[1][2]; end endmodule

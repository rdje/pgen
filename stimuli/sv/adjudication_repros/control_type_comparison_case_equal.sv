module m; int a; logic [3:0] b; initial if (type(a) === type(b)) $display("x"); endmodule

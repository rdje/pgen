`define FOO(arg) "foo``arg``bar"
module t; initial $display(`FOO(x)); endmodule

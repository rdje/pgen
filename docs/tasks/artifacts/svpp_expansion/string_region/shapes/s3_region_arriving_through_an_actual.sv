`define foo test
`define a x,y
`define bar(a, b) test a b
`define qux(x) string boo = x;
`define quux(x) `qux(`"x`")
module t;
`quux(`foo)
`quux(`bar(`a,`a))
endmodule

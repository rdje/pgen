`define Z z
`define pack_int(A) `"A`"
`define P(T, A) `pack_``T``(A)
module t; initial $display(`P(int, `Z)); endmodule

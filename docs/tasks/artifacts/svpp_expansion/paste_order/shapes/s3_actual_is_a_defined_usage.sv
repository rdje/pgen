`define X x
`define X_y from_whole_name
`define M(arg) arg``_y
module t; wire a = `M(`X); endmodule

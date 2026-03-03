`define ADD2(a,b) ((a)+(b))
module macro_function_args;
  localparam int VALUE = `ADD2(3,7);
endmodule

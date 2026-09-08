module memory(input logic clk); endmodule
module tbench_top;
  logic clk;
  memory dut(
    .clk(clk)
    );
endmodule

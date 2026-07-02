module counter #(parameter WIDTH = 8) (
  input wire clk,
  input wire rst_n,
  input wire enable,
  output reg [WIDTH-1:0] count,
  output wire parity
);

  wire [WIDTH-1:0] next_count;
  reg overflow;

  assign next_count = count + 1'b1;
  assign parity = ^count;

  always @(posedge clk or negedge rst_n) begin
    if (!rst_n) begin
      count <= {WIDTH{1'b0}};
      overflow <= 1'b0;
    end else if (enable) begin
      count <= next_count;
      if (next_count == {WIDTH{1'b1}})
        overflow <= 1'b1;
    end
  end

  function [WIDTH-1:0] invert;
    input [WIDTH-1:0] value;
    begin
      invert = ~value;
    end
  endfunction

  task clear_flag;
    output flag;
    begin
      flag = 1'b0;
    end
  endtask

endmodule

module top;
  wire p;
  wire [7:0] c;
  reg clk, rst_n, en;
  integer i;

  counter #(.WIDTH(8)) u_counter (
    .clk(clk),
    .rst_n(rst_n),
    .enable(en),
    .count(c),
    .parity(p)
  );

  and g1 (w_and, clk, en);

  always #5 clk = ~clk;

  initial begin
    clk = 0; rst_n = 0; en = 0;
    for (i = 0; i < 4; i = i + 1)
      $display("init %0d", i);
    #10 rst_n = 1;
    case (c[1:0])
      2'b00: $display("zero");
      2'b01, 2'b10: $display("mid");
      default: $display("high");
    endcase
    #100 $finish;
  end
endmodule

package deep_cfg_pkg;
  localparam logic [3:0] A = 4'b0100;
  localparam logic [3:0] B = 4'b1011;
endpackage

module deep_width_pipe_stage(input logic [3:0] a, input logic [3:0] b, output logic [3:0] y);
  assign y = a;
endmodule

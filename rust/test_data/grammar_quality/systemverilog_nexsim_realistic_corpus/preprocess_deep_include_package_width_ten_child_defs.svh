package deep_cfg_pkg;
  localparam logic [3:0] A = 4'b0010;
  localparam logic [3:0] B = 4'b1101;
endpackage

module deep_width_pipe_stage(input logic [3:0] a, input logic [3:0] b, output logic [3:0] y);
  assign y = a;
endmodule

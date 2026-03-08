package width_cfg_pkg;
  localparam logic [3:0] A = 4'b0010;
  localparam logic [3:0] B = 4'b1101;
endpackage

`define IMPORT_CFG import width_cfg_pkg::*;
module macro_width_pipe_stage(input logic [3:0] a, input logic [3:0] b, output logic [3:0] y);
  assign y = a;
endmodule

module preprocess_macro_import_multi_width_eight_child_pipeline(output logic [3:0] q);
  `IMPORT_CFG
  logic [3:0] s0;
  logic [3:0] s1;
  logic [3:0] mid0;
  logic [3:0] mid1;
  logic [3:0] mid2;
  logic [3:0] mid3;
  logic [3:0] mid4;
  logic [3:0] mid5;
  logic [3:0] mid6;
  assign s0 = A;
  assign s1 = B;
  macro_width_pipe_stage u0(.a(s0), .b(s1), .y(mid0));
  macro_width_pipe_stage u1(.a(mid0), .b(s1), .y(mid1));
  macro_width_pipe_stage u2(.a(mid1), .b(s1), .y(mid2));
  macro_width_pipe_stage u3(.a(mid2), .b(s1), .y(mid3));
  macro_width_pipe_stage u4(.a(mid3), .b(s1), .y(mid4));
  macro_width_pipe_stage u5(.a(mid4), .b(s1), .y(mid5));
  macro_width_pipe_stage u6(.a(mid5), .b(s1), .y(mid6));
  macro_width_pipe_stage u7(.a(mid6), .b(s1), .y(q));
endmodule

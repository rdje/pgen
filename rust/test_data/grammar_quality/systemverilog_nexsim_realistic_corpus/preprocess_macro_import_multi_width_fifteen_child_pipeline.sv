package width_cfg_pkg;
  localparam logic [3:0] A = 4'b1001;
  localparam logic [3:0] B = 4'b0110;
endpackage

`define IMPORT_CFG import width_cfg_pkg::*;
module macro_width_pipe_stage(input logic [3:0] a, input logic [3:0] b, output logic [3:0] y);
  assign y = a;
endmodule

module preprocess_macro_import_multi_width_fifteen_child_pipeline(output logic [3:0] q);
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
  logic [3:0] mid7;
  logic [3:0] mid8;
  logic [3:0] mid9;
  logic [3:0] mid10;
  logic [3:0] mid11;
  logic [3:0] mid12;
  logic [3:0] mid13;
  assign s0 = A;
  assign s1 = B;
  macro_width_pipe_stage u0(.a(s0), .b(s1), .y(mid0));
  macro_width_pipe_stage u1(.a(mid0), .b(s1), .y(mid1));
  macro_width_pipe_stage u2(.a(mid1), .b(s1), .y(mid2));
  macro_width_pipe_stage u3(.a(mid2), .b(s1), .y(mid3));
  macro_width_pipe_stage u4(.a(mid3), .b(s1), .y(mid4));
  macro_width_pipe_stage u5(.a(mid4), .b(s1), .y(mid5));
  macro_width_pipe_stage u6(.a(mid5), .b(s1), .y(mid6));
  macro_width_pipe_stage u7(.a(mid6), .b(s1), .y(mid7));
  macro_width_pipe_stage u8(.a(mid7), .b(s1), .y(mid8));
  macro_width_pipe_stage u9(.a(mid8), .b(s1), .y(mid9));
  macro_width_pipe_stage u10(.a(mid9), .b(s1), .y(mid10));
  macro_width_pipe_stage u11(.a(mid10), .b(s1), .y(mid11));
  macro_width_pipe_stage u12(.a(mid11), .b(s1), .y(mid12));
  macro_width_pipe_stage u13(.a(mid12), .b(s1), .y(mid13));
  macro_width_pipe_stage u14(.a(mid13), .b(s1), .y(q));
endmodule

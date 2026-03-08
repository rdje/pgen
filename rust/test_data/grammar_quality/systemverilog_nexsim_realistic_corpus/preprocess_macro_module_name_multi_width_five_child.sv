`define CHILD width_pipe_stage
module width_pipe_stage(input logic [3:0] a, input logic [3:0] b, output logic [3:0] y);
  assign y = a;
endmodule

module preprocess_macro_module_name_multi_width_five_child(input logic [3:0] d0, input logic [3:0] d1, output logic [3:0] q);
  logic [3:0] mid0;
  logic [3:0] mid1;
  logic [3:0] mid2;
  logic [3:0] mid3;
  `CHILD u0(.a(d0), .b(d1), .y(mid0));
  `CHILD u1(.a(mid0), .b(d1), .y(mid1));
  `CHILD u2(.a(mid1), .b(d1), .y(mid2));
  `CHILD u3(.a(mid2), .b(d1), .y(mid3));
  `CHILD u4(.a(mid3), .b(d1), .y(q));
endmodule

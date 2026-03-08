`define CHILD width_pipe_stage
module width_pipe_stage(input logic [3:0] a, input logic [3:0] b, output logic [3:0] y);
  assign y = a;
endmodule

module preprocess_macro_module_name_multi_width_seventeen_child(input logic [3:0] d0, input logic [3:0] d1, output logic [3:0] q);
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
  logic [3:0] mid14;
  logic [3:0] mid15;
  `CHILD u0(.a(d0), .b(d1), .y(mid0));
  `CHILD u1(.a(mid0), .b(d1), .y(mid1));
  `CHILD u2(.a(mid1), .b(d1), .y(mid2));
  `CHILD u3(.a(mid2), .b(d1), .y(mid3));
  `CHILD u4(.a(mid3), .b(d1), .y(mid4));
  `CHILD u5(.a(mid4), .b(d1), .y(mid5));
  `CHILD u6(.a(mid5), .b(d1), .y(mid6));
  `CHILD u7(.a(mid6), .b(d1), .y(mid7));
  `CHILD u8(.a(mid7), .b(d1), .y(mid8));
  `CHILD u9(.a(mid8), .b(d1), .y(mid9));
  `CHILD u10(.a(mid9), .b(d1), .y(mid10));
  `CHILD u11(.a(mid10), .b(d1), .y(mid11));
  `CHILD u12(.a(mid11), .b(d1), .y(mid12));
  `CHILD u13(.a(mid12), .b(d1), .y(mid13));
  `CHILD u14(.a(mid13), .b(d1), .y(mid14));
  `CHILD u15(.a(mid14), .b(d1), .y(mid15));
  `CHILD u16(.a(mid15), .b(d1), .y(q));
endmodule

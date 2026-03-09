`include "preprocess_deep_include_package_width_twenty_four_child_defs.svh"

module deep_width_bridge(input logic [3:0] a, input logic [3:0] b, output logic [3:0] y);
  logic [3:0] mid0;
  logic [3:0] mid1;
  deep_width_pipe_stage u0(.a(a), .b(b), .y(mid0));
  deep_width_pipe_stage u1(.a(mid0), .b(b), .y(mid1));
  deep_width_pipe_stage u2(.a(mid1), .b(b), .y(y));
endmodule

module deep_width_docosa_bridge(input logic [3:0] a, input logic [3:0] b, output logic [3:0] q);
  logic [3:0] bridge0;
  logic [3:0] bridge1;
  logic [3:0] bridge2;
  logic [3:0] bridge3;
  logic [3:0] bridge4;
  logic [3:0] bridge5;
  logic [3:0] bridge6;
  logic [3:0] bridge7;
  logic [3:0] bridge8;
  logic [3:0] bridge9;
  logic [3:0] bridge10;
  logic [3:0] bridge11;
  logic [3:0] bridge12;
  logic [3:0] bridge13;
  logic [3:0] bridge14;
  logic [3:0] bridge15;
  logic [3:0] bridge16;
  logic [3:0] bridge17;
  logic [3:0] bridge18;
  logic [3:0] bridge19;
  logic [3:0] bridge20;
  deep_width_bridge u0(.a(a), .b(b), .y(bridge0));
  deep_width_bridge u1(.a(bridge0), .b(b), .y(bridge1));
  deep_width_bridge u2(.a(bridge1), .b(b), .y(bridge2));
  deep_width_bridge u3(.a(bridge2), .b(b), .y(bridge3));
  deep_width_bridge u4(.a(bridge3), .b(b), .y(bridge4));
  deep_width_bridge u5(.a(bridge4), .b(b), .y(bridge5));
  deep_width_bridge u6(.a(bridge5), .b(b), .y(bridge6));
  deep_width_bridge u7(.a(bridge6), .b(b), .y(bridge7));
  deep_width_bridge u8(.a(bridge7), .b(b), .y(bridge8));
  deep_width_bridge u9(.a(bridge8), .b(b), .y(bridge9));
  deep_width_bridge u10(.a(bridge9), .b(b), .y(bridge10));
  deep_width_bridge u11(.a(bridge10), .b(b), .y(bridge11));
  deep_width_bridge u12(.a(bridge11), .b(b), .y(bridge12));
  deep_width_bridge u13(.a(bridge12), .b(b), .y(bridge13));
  deep_width_bridge u14(.a(bridge13), .b(b), .y(bridge14));
  deep_width_bridge u15(.a(bridge14), .b(b), .y(bridge15));
  deep_width_bridge u16(.a(bridge15), .b(b), .y(bridge16));
  deep_width_bridge u17(.a(bridge16), .b(b), .y(bridge17));
  deep_width_bridge u18(.a(bridge17), .b(b), .y(bridge18));
  deep_width_bridge u19(.a(bridge18), .b(b), .y(bridge19));
  deep_width_bridge u20(.a(bridge19), .b(b), .y(bridge20));
  deep_width_bridge u21(.a(bridge20), .b(b), .y(q));
endmodule

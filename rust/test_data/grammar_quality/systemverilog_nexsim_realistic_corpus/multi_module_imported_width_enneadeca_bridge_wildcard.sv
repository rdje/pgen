package defs_pkg;
  localparam logic [3:0] A = 4'b0100;
  localparam logic [3:0] B = 4'b1011;
endpackage

module imported_width_source(output logic [3:0] a, output logic [3:0] b);
  import defs_pkg::*;
  assign a = A;
  assign b = B;
endmodule

module width_chain_stage(input logic [3:0] a, input logic [3:0] b, output logic [3:0] y);
  assign y = a;
endmodule

module width_chain_bridge(input logic [3:0] a, input logic [3:0] b, output logic [3:0] y);
  logic [3:0] mid0;
  logic [3:0] mid1;
  width_chain_stage u0(.a(a), .b(b), .y(mid0));
  width_chain_stage u1(.a(mid0), .b(b), .y(mid1));
  width_chain_stage u2(.a(mid1), .b(b), .y(y));
endmodule

module width_chain_enneadeca_bridge(input logic [3:0] a, input logic [3:0] b, output logic [3:0] q);
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
  width_chain_bridge u0(.a(a), .b(b), .y(bridge0));
  width_chain_bridge u1(.a(bridge0), .b(b), .y(bridge1));
  width_chain_bridge u2(.a(bridge1), .b(b), .y(bridge2));
  width_chain_bridge u3(.a(bridge2), .b(b), .y(bridge3));
  width_chain_bridge u4(.a(bridge3), .b(b), .y(bridge4));
  width_chain_bridge u5(.a(bridge4), .b(b), .y(bridge5));
  width_chain_bridge u6(.a(bridge5), .b(b), .y(bridge6));
  width_chain_bridge u7(.a(bridge6), .b(b), .y(bridge7));
  width_chain_bridge u8(.a(bridge7), .b(b), .y(bridge8));
  width_chain_bridge u9(.a(bridge8), .b(b), .y(bridge9));
  width_chain_bridge u10(.a(bridge9), .b(b), .y(bridge10));
  width_chain_bridge u11(.a(bridge10), .b(b), .y(bridge11));
  width_chain_bridge u12(.a(bridge11), .b(b), .y(bridge12));
  width_chain_bridge u13(.a(bridge12), .b(b), .y(bridge13));
  width_chain_bridge u14(.a(bridge13), .b(b), .y(bridge14));
  width_chain_bridge u15(.a(bridge14), .b(b), .y(bridge15));
  width_chain_bridge u16(.a(bridge15), .b(b), .y(bridge16));
  width_chain_bridge u17(.a(bridge16), .b(b), .y(bridge17));
  width_chain_bridge u18(.a(bridge17), .b(b), .y(q));
endmodule

module multi_module_imported_width_enneadeca_bridge_wildcard(output logic [3:0] a, output logic [3:0] b, output logic [3:0] q);
  imported_width_source src(.a(a), .b(b));
  width_chain_enneadeca_bridge bridge(.*);
endmodule

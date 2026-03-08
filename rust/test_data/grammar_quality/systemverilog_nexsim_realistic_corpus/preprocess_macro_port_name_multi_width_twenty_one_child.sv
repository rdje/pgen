`define PORT_A a
`define PORT_B b
`define PORT_Y y

module macro_port_width_child(input logic [3:0] a, input logic [3:0] b, output logic [3:0] y);
  assign y = a;
endmodule

module preprocess_macro_port_name_multi_width_twenty_one_child(output logic [3:0] q);
  logic [3:0] d0;
  logic [3:0] d1;
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
  logic [3:0] mid16;
  logic [3:0] mid17;
  logic [3:0] mid18;
  logic [3:0] mid19;
  macro_port_width_child u0(.`PORT_A(d0), .`PORT_B(d1), .`PORT_Y(mid0));
  macro_port_width_child u1(.`PORT_A(mid0), .`PORT_B(d1), .`PORT_Y(mid1));
  macro_port_width_child u2(.`PORT_A(mid1), .`PORT_B(d1), .`PORT_Y(mid2));
  macro_port_width_child u3(.`PORT_A(mid2), .`PORT_B(d1), .`PORT_Y(mid3));
  macro_port_width_child u4(.`PORT_A(mid3), .`PORT_B(d1), .`PORT_Y(mid4));
  macro_port_width_child u5(.`PORT_A(mid4), .`PORT_B(d1), .`PORT_Y(mid5));
  macro_port_width_child u6(.`PORT_A(mid5), .`PORT_B(d1), .`PORT_Y(mid6));
  macro_port_width_child u7(.`PORT_A(mid6), .`PORT_B(d1), .`PORT_Y(mid7));
  macro_port_width_child u8(.`PORT_A(mid7), .`PORT_B(d1), .`PORT_Y(mid8));
  macro_port_width_child u9(.`PORT_A(mid8), .`PORT_B(d1), .`PORT_Y(mid9));
  macro_port_width_child u10(.`PORT_A(mid9), .`PORT_B(d1), .`PORT_Y(mid10));
  macro_port_width_child u11(.`PORT_A(mid10), .`PORT_B(d1), .`PORT_Y(mid11));
  macro_port_width_child u12(.`PORT_A(mid11), .`PORT_B(d1), .`PORT_Y(mid12));
  macro_port_width_child u13(.`PORT_A(mid12), .`PORT_B(d1), .`PORT_Y(mid13));
  macro_port_width_child u14(.`PORT_A(mid13), .`PORT_B(d1), .`PORT_Y(mid14));
  macro_port_width_child u15(.`PORT_A(mid14), .`PORT_B(d1), .`PORT_Y(mid15));
  macro_port_width_child u16(.`PORT_A(mid15), .`PORT_B(d1), .`PORT_Y(mid16));
  macro_port_width_child u17(.`PORT_A(mid16), .`PORT_B(d1), .`PORT_Y(mid17));
  macro_port_width_child u18(.`PORT_A(mid17), .`PORT_B(d1), .`PORT_Y(mid18));
  macro_port_width_child u19(.`PORT_A(mid18), .`PORT_B(d1), .`PORT_Y(mid19));
  macro_port_width_child u20(.`PORT_A(mid19), .`PORT_B(d1), .`PORT_Y(q));
endmodule

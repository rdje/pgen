`define PORT_A a
`define PORT_B b
`define PORT_Y y

module macro_port_width_child(input logic [3:0] a, input logic [3:0] b, output logic [3:0] y);
  assign y = a;
endmodule

module preprocess_macro_port_name_multi_width_nine_child(output logic [3:0] q);
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
  macro_port_width_child u0(.`PORT_A(d0), .`PORT_B(d1), .`PORT_Y(mid0));
  macro_port_width_child u1(.`PORT_A(mid0), .`PORT_B(d1), .`PORT_Y(mid1));
  macro_port_width_child u2(.`PORT_A(mid1), .`PORT_B(d1), .`PORT_Y(mid2));
  macro_port_width_child u3(.`PORT_A(mid2), .`PORT_B(d1), .`PORT_Y(mid3));
  macro_port_width_child u4(.`PORT_A(mid3), .`PORT_B(d1), .`PORT_Y(mid4));
  macro_port_width_child u5(.`PORT_A(mid4), .`PORT_B(d1), .`PORT_Y(mid5));
  macro_port_width_child u6(.`PORT_A(mid5), .`PORT_B(d1), .`PORT_Y(mid6));
  macro_port_width_child u7(.`PORT_A(mid6), .`PORT_B(d1), .`PORT_Y(mid7));
  macro_port_width_child u8(.`PORT_A(mid7), .`PORT_B(d1), .`PORT_Y(q));
endmodule

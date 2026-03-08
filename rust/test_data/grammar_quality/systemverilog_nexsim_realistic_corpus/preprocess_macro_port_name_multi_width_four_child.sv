`define PORT_A a
`define PORT_B b
`define PORT_Y y

module macro_port_width_child(input logic [3:0] a, input logic [3:0] b, output logic [3:0] y);
  assign y = a;
endmodule

module preprocess_macro_port_name_multi_width_four_child(output logic [3:0] q);
  logic [3:0] d0;
  logic [3:0] d1;
  logic [3:0] mid0;
  logic [3:0] mid1;
  logic [3:0] mid2;
  macro_port_width_child u0(.`PORT_A(d0), .`PORT_B(d1), .`PORT_Y(mid0));
  macro_port_width_child u1(.`PORT_A(mid0), .`PORT_B(d1), .`PORT_Y(mid1));
  macro_port_width_child u2(.`PORT_A(mid1), .`PORT_B(d1), .`PORT_Y(mid2));
  macro_port_width_child u3(.`PORT_A(mid2), .`PORT_B(d1), .`PORT_Y(q));
endmodule

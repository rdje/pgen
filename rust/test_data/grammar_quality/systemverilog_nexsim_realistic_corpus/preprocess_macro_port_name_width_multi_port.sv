`define PORT_A a
`define PORT_B b
`define PORT_Y y

module macro_port_width_child(input logic [3:0] a, input logic [3:0] b, output logic [3:0] y);
  assign y = a;
endmodule

module preprocess_macro_port_name_width_multi_port(input logic [3:0] d0, input logic [3:0] d1, output logic [3:0] q);
  macro_port_width_child u0(.`PORT_A(d0), .`PORT_B(d1), .`PORT_Y(q));
endmodule

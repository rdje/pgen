`define PORT_A a
`define PORT_B b
`define PORT_Y y

module macro_port_multi_child(input logic a, input logic b, output logic y);
  assign y = a;
endmodule

module preprocess_macro_port_name_multi_port(input logic d0, input logic d1, output logic q);
  macro_port_multi_child u0(.`PORT_A(d0), .`PORT_B(d1), .`PORT_Y(q));
endmodule

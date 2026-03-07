`define PORT_A a
`define PORT_Y y

module macro_port_packed_child(input logic [3:0] a, output logic [3:0] y);
  assign y = a;
endmodule

module preprocess_macro_port_name_internal_packed(output logic [3:0] q);
  logic [3:0] d;
  macro_port_packed_child u0(.`PORT_A(d), .`PORT_Y(q));
endmodule

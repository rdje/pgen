module child(input logic [3:0] a, output logic [3:0] y);
  assign y = a;
endmodule

module internal_packed_named_port_instantiation(output logic [3:0] q);
  logic [3:0] d;
  child u0(.a(d), .y(q));
endmodule

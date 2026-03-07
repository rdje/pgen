module child(input logic [3:0] a, output logic [3:0] y);
  assign y = a;
endmodule

module internal_packed_wildcard_instantiation(output logic [3:0] y);
  logic [3:0] a;
  child u0(.*);
endmodule

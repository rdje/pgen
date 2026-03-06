module child(input logic a, output logic y);
  assign y = a;
endmodule

module top(input logic d, output logic q);
  child u0(.a(d), .y(q));
endmodule

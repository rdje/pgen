module child(input logic a, input logic b, output logic y);
  assign y = a;
endmodule

module top(input logic d0, input logic d1, output logic q);
  child u0(.a(d0), .b(d1), .y(q));
endmodule

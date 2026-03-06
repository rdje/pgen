module child(input logic a, input logic b, output logic y);
  assign y = a;
endmodule

module top(input logic a, input logic b, output logic y);
  child u0(.*);
endmodule

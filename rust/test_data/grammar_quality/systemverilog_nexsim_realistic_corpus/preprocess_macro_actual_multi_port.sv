`define ID(x) x
module child(input logic a, input logic b, output logic y);
  assign y = a;
endmodule

module preprocess_macro_actual_multi_port(input logic d0, input logic d1, output logic q);
  child u0(.a(`ID(d0)), .b(`ID(d1)), .y(q));
endmodule

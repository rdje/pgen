`define ID(x) x
module child(input logic a, output logic y);
  assign y = a;
endmodule

module preprocess_macro_actual_named_port(input logic d, output logic q);
  child u0(.a(`ID(d)), .y(q));
endmodule

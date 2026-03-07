`define CHILD child
module child(input logic a, input logic b, output logic y);
  assign y = a;
endmodule

module preprocess_macro_module_name_wildcard(input logic a, input logic b, output logic y);
  `CHILD u0(.*);
endmodule

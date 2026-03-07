module child(input logic a);
endmodule

module directive_noise_named_port;
  logic sig;
  `define TAG 1
  child u0(.a(sig));
endmodule

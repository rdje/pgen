module directive_noise_always_comb;
  logic a;
  logic b;
  `define TAG 1
  always_comb begin
    b = a;
  end
endmodule

// SV-CORPUS-GRAD.13c.2k — a generate block labelled `byte`. `byte` is reserved in IEEE 1800 and is
// NOT in IEEE 1364-2005 Annex B. PAIRED with control_v2005_keyword_generate_label.sv.
// Shape from iverilog/ivtest/ivltests/generate_multi_loop.v.
module top;
  genvar i;
  generate
    for (i = 0; i < 4; i = i + 1) begin:byte
      wire [7:0] w;
    end
  endgenerate
endmodule

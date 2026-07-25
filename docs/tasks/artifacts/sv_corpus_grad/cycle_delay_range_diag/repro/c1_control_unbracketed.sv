// CONTROL (must ACCEPT at baseline): unbracketed `##2` — cycle_delay_range alt 1
module m;
  logic clk, te3, te4;
  sequence s1; @(posedge clk) te3 ##2 te4; endsequence
endmodule

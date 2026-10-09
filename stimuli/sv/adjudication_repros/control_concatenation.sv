// SV-CORPUS-GRAD.52 — the control for invalid_empty_concatenation_element.sv: the same concatenation with
// every element present (1364-2005 A.8.1 :841 / 1800-2017 A.8.1 :2880-2883). ACCEPT on every profile.
module t;
  reg [8:0] kuku;
  initial kuku = {3'd5,3'd5,3'd5};
endmodule

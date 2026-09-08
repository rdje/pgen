// SV-CORPUS-GRAD.3.51 - the reserved word `medium` bound as a module name and instance path: `assign o = medium.c;` with `module medium();` (Surelog tests/HierPathModule/dut.sv). IEEE 1800-2017 Annex B Table B.1 (docs/systemverilog/2017/md/section-41-data-read-api.md:3722; the table docs/systemverilog/2017/md/section-41-data-read-api.md:3599-3858) and IEEE 1364-2005 Annex B (docs/verilog/2005/md/section-Annex_B-normative-list-of-keywords.md:79) both reserve `medium` (a charge strength, A.2.2.2).
// REJECTS at `medium` on every profile; the control with the module named `med` ACCEPTS.
module top;
  wire o;
  assign o = medium.c;
  medium u1();
endmodule
module medium();
  wire c;
endmodule

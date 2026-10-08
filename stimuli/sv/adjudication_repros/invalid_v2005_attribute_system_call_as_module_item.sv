// SV-CORPUS-GRAD.13e.13 — REJECT FOREVER on every profile.
// `$attribute(clk, "PAD", "39");` directly inside a module — iverilog's attribute pragma as a
// MODULE ITEM. IEEE 1364-2005 A.1.4 module_or_generate_item (docs/verilog/2005/md/section-12-hierarchical-structures.md:83-94)
// derives declarations, assigns, instantiations, initial/always and generate constructs — no
// system task enable; IEEE 1800-2017 A.1.4 module_item / module_common_item
// (docs/systemverilog/2017/md/section-41-data-read-api.md:310-340) the same. The description-level
// twin is invalid_attribute_system_call_as_description.sv (SV-CORPUS-GRAD.3.51).
// Corpus row: ivtest/fpga_tests/sqrt.v (fpga_tests/fpga_reg.list; iverilog -tfpga, the default generation).
// Control: control_system_call_in_initial.sv (`initial $display("x");`) parses on every profile.
module m;
  $attribute(clk, "PAD", "39");
endmodule

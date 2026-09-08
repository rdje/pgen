// SV-CORPUS-GRAD.3.52 - a RANGE select inside a hierarchical path: `xs(intfX[2:0].ModportB)` (sv2v test/core/interface_array.sv:114). IEEE 1800-2017 A.9.3 hierarchical_identifier ::= [ $root . ] { identifier constant_bit_select . } identifier (docs/systemverilog/2017/md/section-41-data-read-api.md:3361) with A.8.4 constant_bit_select ::= { [ constant_expression ] } (docs/systemverilog/2017/md/section-41-data-read-api.md:3173) - a bit-select, never a range; 1800-2023 the same (docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:3310, :3120).
// REJECTS at the `:` on both SV profiles; the control `intfX[1].ModportB` ACCEPTS.
module m;
  I intfX[2:0]();
  M xs(intfX[2:0].ModportB);
endmodule

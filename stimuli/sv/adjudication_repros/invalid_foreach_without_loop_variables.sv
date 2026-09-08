// SV-CORPUS-GRAD.3.51 - a foreach with NO loop-variable brackets: `foreach (y) begin ... end` (verible kythe testdata/foreach.sv). IEEE 1800-2017 A.6.8 loop_statement ::= ... | foreach ( ps_or_hierarchical_array_identifier [ loop_variables ] ) statement (docs/systemverilog/2017/md/section-41-data-read-api.md:2334) - the brackets are the production's own tokens (loop_variables may be empty, the brackets may not); 1800-2023 the same (docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:2319).
// REJECTS at the `)` on both SV profiles; the control `foreach (y[i]) y[i] = 0;` ACCEPTS.
module m;
  int y[3];
  initial foreach (y) begin end
endmodule

// SV-CORPUS-GRAD.3.51 - a foreach whose body is the NULL statement: `foreach (a[i]) ;` (iverilog ivtest/ivltests/sf1289.v). IEEE 1800-2017 A.6.8 foreach ( ps_or_hierarchical_array_identifier [ loop_variables ] ) statement (docs/systemverilog/2017/md/section-41-data-read-api.md:2334) - `statement`, where every other loop takes statement_or_null (docs/systemverilog/2017/md/section-41-data-read-api.md:2326-2333); statement ::= [ block_identifier : ] { attribute_instance } statement_item (docs/systemverilog/2017/md/section-41-data-read-api.md:2087) and the bare `;` is statement_or_null's own alternative (docs/systemverilog/2017/md/section-41-data-read-api.md:2081-2084); 1800-2023 the same (docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:2311-2319).
// REJECTS at the `;` on both SV profiles; `for (...) ;` (statement_or_null) and `foreach (y[i]) begin end` (a statement) ACCEPT.
module m;
  int y[3];
  initial foreach (y[i]) ;
endmodule

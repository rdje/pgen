// SV-CORPUS-GRAD.3.48 - filed as an ACCEPTS-INVALID pin under the IEEE 1364-2005 profile (the unpacked size form is IEEE 1800's, not Verilog's, and through 1.0.221 the grammar's unpacked_dimension admitted it on every profile); SV-CORPUS-GRAD.3.50 (PGEN-SV-CORPUS-GRAD-0379) gated the size arm to the SV profiles (unpacked_dimension_size_sv_only) and flipped this row to REJECT / invalid - it guards the fix.
// reg x [2]; - IEEE 1364-2005 A.2.5 dimension ::= [ dimension_constant_expression : dimension_constant_expression ] (section-Annex_A-normative-formal-syntax-definition.md:309); IEEE 1800-2017 A.2.5 unpacked_dimension admits [ constant_expression ] (section-41-data-read-api.md:1046-1050), so the twin control_unpacked_dimension_size.sv ACCEPTS under sv_2017 / sv_2023.
module m;
  reg x [2];
endmodule

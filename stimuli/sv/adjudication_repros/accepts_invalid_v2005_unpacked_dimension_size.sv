// SV-CORPUS-GRAD.3.48 - an ACCEPTS-INVALID pin under the IEEE 1364-2005 profile (owned by SV-CORPUS-GRAD.3.50): the unpacked size form is IEEE 1800's, not Verilog's, and the shipped grammar's unpacked_dimension admits it on every profile - a known over-acceptance; the fix flips this row to REJECT / invalid.
// reg x [2]; - IEEE 1364-2005 A.2.5 dimension ::= [ dimension_constant_expression : dimension_constant_expression ] (section-Annex_A-normative-formal-syntax-definition.md:309); IEEE 1800-2017 A.2.5 unpacked_dimension admits [ constant_expression ] (section-41-data-read-api.md:1046-1050), so the twin control_unpacked_dimension_size.sv ACCEPTS under sv_2017 / sv_2023.
module m;
  reg x [2];
endmodule

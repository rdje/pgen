// SV-CORPUS-GRAD.13x.d - an INVALID pin under the profile it names: text the 1800-2017 edition derives nothing for - must keep REJECTING there.
// parameter type enum E_t; judged as IEEE 1800-2017
// IEEE 1800-2023 A.2.1.1 type_parameter_declaration ::= type [ forward_type ] ... docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:721, forward_type docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:803 (2023 only); IEEE 1800-2017 parameter_declaration docs/systemverilog/2017/md/section-41-data-read-api.md:739-741 has no forward_type. verilator t_param_type_fwd_bad.v l.10; the same text is the sv_2023 control beside this row.
module sub;
  parameter type enum E_t;
endmodule

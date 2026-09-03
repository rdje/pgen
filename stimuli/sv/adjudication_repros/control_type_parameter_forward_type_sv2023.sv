// SV-CORPUS-GRAD.13x.d - a CONTROL under the profile it names - must keep ACCEPTING there.
// parameter type enum E_t; judged as IEEE 1800-2023
// IEEE 1800-2023 A.2.1.1 docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:721, docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:803. the edition contrast: byte-identical text to the sv_2017 invalid pin, legal in 2023.
module sub;
  parameter type enum E_t;
endmodule

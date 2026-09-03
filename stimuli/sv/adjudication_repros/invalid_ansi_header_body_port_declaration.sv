// SV-CORPUS-GRAD.13x.b - an INVALID pin: text no Annex A production derives - must keep REJECTING.
// module t (input clk); input clk; endmodule - a body port declaration after an ANSI header
// IEEE 1800-2017 A.1.2 module_declaration section-41-data-read-api.md:108-113 (ANSI arm's body is non_port_module_item); module_item ::= port_declaration ; section-41-data-read-api.md:327 is the non-ANSI arm's. t_assert_imm_nz_bad.v l.11 and t_var_dup_bad.v l.62; 23.2.2.2 section-23-modules-and-hierarchy.md:407 says the same.
module t (input clk);
  input clk;
endmodule

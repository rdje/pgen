// SV-CORPUS-GRAD.13x.b - an INVALID pin: text no Annex A production derives - must keep REJECTING.
// assert #1 (clk); - a deferred immediate assertion with a delay other than #0
// IEEE 1800-2017 A.6.10 deferred_immediate_assert_statement ::= assert #0 ( expression ) action_block section-41-data-read-api.md:2421-2424. t_assert_imm_nz_bad.v l.14; sv2v keys the same class assert_deferred_nonzero must_reject.
module t (input clk);
  labeled_imas : assert #1 (clk);
endmodule

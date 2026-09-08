// SV-CORPUS-GRAD.3.51 - a deferred immediate assertion spelled `#0_0`: `assert #0_0 (1);` (sv2v test/nosim/assert.sv). IEEE 1800-2017 A.6.10 deferred_immediate_assert_statement ::= assert #0 ( expression ) action_block | assert final ( expression ) action_block (docs/systemverilog/2017/md/section-41-data-read-api.md:2421-2424) - the `#0` is the production's own token, not a delay value; 1800-2023 the same (docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:2406-2409).
// REJECTS at `0_0` on both SV profiles; the control `assert #0 (1);` ACCEPTS.
module m;
  initial assert #0_0 (1);
endmodule

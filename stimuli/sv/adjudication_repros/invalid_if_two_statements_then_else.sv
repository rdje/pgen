// SV-CORPUS-GRAD.27 - an INVALID pin - must keep REJECTING on every profile.
// IEEE 1800-2017 A.6.6 conditional_statement takes ONE statement_or_null before `else`: the first statement closes the `if`,
// so this `else` has no `if` to pair with (opentitan otbn_mask_accelerator_sca_wrapper.sv lines 48-51, a RESIDUAL dark row).
module m;
  reg c, a, b;
  always @(posedge c) begin
    if (c)
      a <= 0;
      b <= 1;
    else
      a <= 1;
  end
endmodule

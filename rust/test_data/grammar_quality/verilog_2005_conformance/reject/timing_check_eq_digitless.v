// SV-0030 reject-lock (SV-DOLLAR-LRM-FIDELITY.3): the digit-less text `1'b`
// is not a scalar_constant (nor a number) — the compare condition must REJECT
// everywhere, before and after the fix.
module m(input d, input clk, input e);
  specify
    $setup(d &&& e == 1'b, posedge clk, 1);
  endspecify
endmodule

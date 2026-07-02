// SV-0030 accept-lock (SV-DOLLAR-LRM-FIDELITY.3): the IEEE scalar timing-check
// compare forms (1364-2005 / 1800-2017 A.7.5.3) and the flat fallback forms
// must all accept under every profile. The compare forms now emit typed
// eq/case_eq/ne/case_ne shapes; the `&& f` line locks the !binary_operator
// follow-guard fallback to the flat expression branch.
module m(input d, input clk, input e, input f);
  specify
    $setup(d &&& e == 1'b0, posedge clk, 1);
    $setup(d &&& e === 1'b1, posedge clk, 1);
    $setup(d &&& e != 'b0, posedge clk, 1);
    $setup(d &&& e !== 1'B1, posedge clk, 1);
    $setup(d &&& e == 1'b0 && f, posedge clk, 1);
  endspecify
endmodule

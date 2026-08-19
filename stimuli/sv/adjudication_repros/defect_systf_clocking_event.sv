// SV-CORPUS-GRAD.13c.2s — found by `.13c.2r`'s starving-star census, not by looking for it.
// IEEE 1800-2017 A.8.2: system_tf_call ::= … | system_tf_identifier ( expression { , [ expression ] }
// [ , [ clocking_event ] ] ). PGEN spells that third alternative with a greedy
// `( comma ( expression )? )*` in front of `( comma ( clocking_event )? )?`, so the star eats the
// comma and the clocking-event group can never fire. REJECTS at the `@`.
module m;
  logic clk, a, x;
  always @(posedge clk) x <= $rose(a, @(posedge clk));
endmodule

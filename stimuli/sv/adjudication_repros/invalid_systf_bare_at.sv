// SV-CORPUS-GRAD.13c.2s negative. A bare `@` is neither an expression nor a clocking_event
// (A.6.5 requires `@ identifier` or `@ ( event_expression )`), so this must REJECT. It is the
// probe that separates the chosen `!at_sign` guard from the semantically exact `!( clocking_event )`
// one: `!at_sign` refuses a strictly larger set, and this row pins that the extra refusals are
// texts that were rejected anyway. Expected REJECT FOREVER.
module m;
  logic clk, a, x;
  always @(posedge clk) x <= $rose(a, @);
endmodule

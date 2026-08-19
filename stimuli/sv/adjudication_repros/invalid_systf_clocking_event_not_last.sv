// SV-CORPUS-GRAD.13c.2s negative. IEEE 1800-2017 A.8.2 puts the clocking event LAST:
// system_tf_identifier ( expression { , [ expression ] } [ , [ clocking_event ] ] ). A clocking
// event followed by a further argument is not derivable, and the `!at_sign` fix must not make it
// so — a guard that simply let the star skip `@` would have. Expected REJECT FOREVER.
module m;
  logic clk, a, b, x;
  always @(posedge clk) x <= $rose(a, @(posedge clk), b);
endmodule

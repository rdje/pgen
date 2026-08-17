// §30.7.1's specify block with the specparam removed — parsed before the fix and after,
// so it isolates the rejection to the PATHPULSE$ specparam rather than to the paths.
module m (input clk, data, clr, pre, output q);
  specify
    (clk => q) = 12;
    (data => q) = 10;
    (clr, pre *> q) = 4;
  endspecify
endmodule

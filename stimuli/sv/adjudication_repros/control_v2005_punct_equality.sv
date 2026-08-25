// SV-CORPUS-GRAD.13e.10 (c3) — CONTROL. Legal IEEE 1364-2005 that must keep parsing
// under verilog_2005 AFTER (c4)'s _sv_only gates land. This is the arm that catches a
// gate made TOO WIDE: the over-acceptance repros prove the fix works, this proves it
// did not take legal Verilog with it.
module m;
  initial begin
    if (a == b) ;
  end
endmodule

// SV-CORPUS-GRAD.13e.10 (c3) — an OVER-ACCEPTANCE under verilog_2005, expected ACCEPT until fixed.
// WHAT: the IEEE 1800-only operator `&=` reaches through the `verilog_2005` profile.
// LRM: `&=` occurs 0 times in the whole IEEE 1364-2005 LRM; 1364-2005 has no compound assignment (A.6.2)
// LEG 3: the operator's own terminal rule `and_assign` COMMITS on this text under
//        verilog_2005 (committed=1), and reads 0 on the matched
//        control with the operator removed — so the ACCEPT is carried by the operator
//        itself, not by the surrounding text parsing as something else.
// Pinned BEFORE the fix so the ratchet goes RED the day it lands (flip to class=invalid).
module m;
  initial begin
    a &= 1;
  end
endmodule

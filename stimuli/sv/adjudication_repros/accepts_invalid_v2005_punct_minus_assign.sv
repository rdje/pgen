// SV-CORPUS-GRAD.13e.10 (c3) — an OVER-ACCEPTANCE under verilog_2005, expected ACCEPT until fixed.
// WHAT: the IEEE 1800-only operator `-=` reaches through the `verilog_2005` profile.
// LRM: `-=` appears in IEEE 1364-2005 ONLY inside the clause-14 specify-path polarity operator `-=>` (`(In1 -=> q)`), never as an assignment operator; A.6.2 has no compound assignment
// LEG 3: the operator's own terminal rule `minus_assign` COMMITS on this text under
//        verilog_2005 (committed=1), and reads 0 on the matched
//        control with the operator removed — so the ACCEPT is carried by the operator
//        itself, not by the surrounding text parsing as something else.
// Pinned BEFORE the fix so the ratchet goes RED the day it lands (flip to class=invalid).
module m;
  initial begin
    a -= 1;
  end
endmodule

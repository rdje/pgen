// SV-CORPUS-GRAD.13e.10 (c3) — an OVER-ACCEPTANCE under verilog_2005, expected ACCEPT until fixed.
// WHAT: the IEEE 1800-only operator `.*` reaches through the `verilog_2005` profile.
// LRM: `.*` appears in IEEE 1364-2005 ONLY as a clause-13 library-map FILENAME GLOB (`library aLib adder.*;`) — a different grammar and a different token role; A.4.1 module_instantiation has no implicit port connection form
// LEG 3: the operator's own terminal rule `dot_star` COMMITS on this text under
//        verilog_2005 (committed=3), and reads 0 on the matched
//        control with the operator removed — so the ACCEPT is carried by the operator
//        itself, not by the surrounding text parsing as something else.
// Pinned BEFORE the fix so the ratchet goes RED the day it lands (flip to class=invalid).
module s;
endmodule
module m;
  s u(.*);
endmodule

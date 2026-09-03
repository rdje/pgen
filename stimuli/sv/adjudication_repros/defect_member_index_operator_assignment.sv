// SV-CORPUS-GRAD.13x.f -> FIXED by SV-CORPUS-GRAD.3.31 (release 1.0.208, ledger SV-0081): pinned as a DEFECT (REJECT) one commit before the fix, now a FIXED pin (ACCEPT) that guards the fix against regression.
// clist[(b[0] += 1)].m <= 1; - a parenthesised operator_assignment as the index before .member
// IEEE 1800-2017 A.8.3 expression ::= ... | ( operator_assignment ) section-41-data-read-api.md:272-280. owned by SV-CORPUS-GRAD.3.31 - the second expression alternative the index path lacks; FLIPS TO ACCEPT WHEN THE FIX LANDS.
module t; class Cls; int m; endclass Cls clist[1]; int b[2], x; logic clk;
  always @(posedge clk) clist[(b[0] += 1)].m <= 1;
endmodule

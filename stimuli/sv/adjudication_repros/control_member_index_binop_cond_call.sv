// SV-CORPUS-GRAD.13x.f - a CONTROL: the nearest construct that DOES parse - must keep ACCEPTING.
// clist[b[0]+1].m, clist[b[0] ? 0 : 1].m, clist[f()].m - the same member access with a binary, conditional and call index
// IEEE 1800-2017 A.8.3 expression section-41-data-read-api.md:272-280. isolates the .3.31 defect to the two missing alternatives: every other expression shape indexes the member access today.
module t; class Cls; int m; endclass Cls clist[1]; int b[2]; logic clk; function int f(); return 0; endfunction
  always @(posedge clk) clist[b[0]+1].m <= 1;
  always @(posedge clk) clist[b[0] ? 0 : 1].m <= 1;
  always @(posedge clk) clist[f()].m <= 1;
endmodule

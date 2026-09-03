// SV-CORPUS-GRAD.13x.f - a DEFECT pin: LEGAL text PGEN rejects today - expected REJECT, and this runner FAILS with 'flip it to fixed' the day the owning fix lands.
// clist[b[0]++].m <= 1; - an inc_or_dec_expression as the index of an array element whose member is then selected
// IEEE 1800-2017 A.8.3 expression ::= ... | inc_or_dec_expression section-41-data-read-api.md:272-280; A.9.3 hierarchical_identifier section-41-data-read-api.md:3361 (the letter says constant_bit_select, and the parser already admits non-constant indices there - see .3.31). owned by SV-CORPUS-GRAD.3.31 (verilator t_assign_automatic_bad.v l.58-60) - FLIPS TO ACCEPT (class fixed) WHEN THE GRAMMAR FIX LANDS; the same index without the member (control) and a binary index with the member (control) parse today.
module t; class Cls; int m; endclass Cls clist[1]; int b[2], x; logic clk;
  always @(posedge clk) clist[b[0]++].m <= 1;
  always @(posedge clk) x = clist[b[0]++].m;
endmodule

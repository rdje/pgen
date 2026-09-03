// SV-CORPUS-GRAD.13x.f - a CONTROL: the nearest construct that DOES parse - must keep ACCEPTING.
// clist[b[0]++] = null; a[b[0]++] = 1; - the same inc_or_dec index with NO member selected after it
// IEEE 1800-2017 A.8.3 section-41-data-read-api.md:272-280; A.8.4 bit_select ::= { [ expression ] } section-41-data-read-api.md:3157. isolates the .3.31 defect to the member-access path: the plain bit-select path already takes the full expression.
module t; class Cls; int m; endclass Cls clist[1]; int a[2], b[2]; logic clk;
  always @(posedge clk) clist[b[0]++] = null;
  always @(posedge clk) a[b[0]++] = 1;
endmodule

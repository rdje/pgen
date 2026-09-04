// SV-CORPUS-GRAD.13y - a CONTROL - must keep ACCEPTING.
// foreach (a[i, j]) - the LRM's own spelling
// IEEE 1800-2017 12.7.3 docs/systemverilog/2017/md/section-12-procedural-programming-statements.md:1082-1083; A.6.8 loop_variables docs/systemverilog/2017/md/section-41-data-read-api.md:2360
module t;
  int a[2][3];
  initial foreach (a[i, j]) a[i][j] = 0;
endmodule

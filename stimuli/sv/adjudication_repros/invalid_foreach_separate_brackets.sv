// SV-CORPUS-GRAD.13y - an INVALID pin - text IEEE 1800-2017 derives nothing for; must keep REJECTING on every profile.
// foreach (a[i][j]) - loop variables in separate bracket pairs
// IEEE 1800-2017 A.6.8 docs/systemverilog/2017/md/section-41-data-read-api.md:915, loop_variables docs/systemverilog/2017/md/section-41-data-read-api.md:2360; 12.7.3 docs/systemverilog/2017/md/section-12-procedural-programming-statements.md:1045, :1082-1083
module t;
  int a[2][3];
  initial foreach (a[i][j]) a[i][j] = 0;
endmodule

// SV-CORPUS-GRAD.3.37 - a CONTROL - must keep ACCEPTING.
// static int at package / module / class scope, automatic and static inside a function and a named block - footnote 10's legal side
// IEEE 1800-2017 Annex A footnote 10 docs/systemverilog/2017/md/section-41-data-read-api.md:3503-3505: in a data_declaration that is not within a procedural context, it shall be illegal to use the automatic keyword
package p; static int ps = 1; int q; endpackage
module t; static int ms; function automatic int f(input int a); automatic int x = a; static int y = 0; return x + y; endfunction
  initial begin : b automatic int z = 1; static int w; end
  class K; static int cnt; int v; endclass
endmodule

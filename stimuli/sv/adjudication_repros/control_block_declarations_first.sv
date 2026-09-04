// SV-CORPUS-GRAD.13y - a CONTROL - must keep ACCEPTING.
// initial begin int y = 0; x = 1; end - the declaration first, as A.6.3 orders it
// IEEE 1800-2017 A.6.3 seq_block docs/systemverilog/2017/md/section-41-data-read-api.md:2064-2066
module t;
  int x;
  initial begin
    int y = 0;
    x = 1;
    x = y;
  end
endmodule

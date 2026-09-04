// SV-CORPUS-GRAD.3.35 - a CONTROL - must keep ACCEPTING.
// begin x = 1; x = 2; end - a block whose first item is a statement (its tree changes from block_data to statement; the verdict does not)
// IEEE 1800-2017 A.6.3 seq_block docs/systemverilog/2017/md/section-41-data-read-api.md:2147-2149
module t;
  int x;
  initial begin
    x = 1;
    x = 2;
  end
endmodule

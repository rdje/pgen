// SV-CORPUS-GRAD.3.59 control — the forms every release accepted; byte-identical before and after the slice.
module t;
  int q[$];
  int x;
  initial begin
    q[0].delete;
    q[x-1].delete();
    x = q[$-1];
    q[$-1] = 1;
  end
endmodule

module m;
  typedef struct { int a; int b; } st;
  st s;
  int arr [0:1];
  initial begin
    s = '{1, 2};
    arr = '{0, 1};
  end
endmodule

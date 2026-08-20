module m;
  int q[$], r[$];
  initial begin
    r = q.find with (item > 3);
    r = q.find_index with (item == 2);
    r = q.sum with (item * 2);
  end
endmodule

module foreach_array_assign;
  logic [3:0] arr;
  always_comb begin
    foreach (arr[idx]) begin
      arr[idx] = 1'b1;
    end
  end
endmodule

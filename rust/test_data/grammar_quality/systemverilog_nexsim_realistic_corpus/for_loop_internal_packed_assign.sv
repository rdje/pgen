module for_loop_internal_packed_assign;
  logic [3:0] arr;
  always_comb begin
    for (int i = 0; i < 4; i = i + 1) begin
      arr[i] = 1'b0;
    end
  end
endmodule

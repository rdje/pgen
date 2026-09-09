module m;
  logic a;
  begin
    logic s;
    begin
      always_ff @(posedge a) s <= a;
    end
  end
endmodule

import q::*;
module m;
  initial begin
    randsequence(main)
      main : first second;
      first : add | dec := (1 + 1);
      second : { $display("x"); };
      add : { x = 1; };
      dec : { x = 0; };
    endsequence
  end
endmodule

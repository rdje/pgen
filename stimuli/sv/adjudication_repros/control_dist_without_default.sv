// SV-CORPUS-GRAD.3.51 - the same default disable iff item without the default dist item: ACCEPT on both SV profiles (the item takes no parentheses in either edition).
module m;
  logic x;
  default disable iff 1 dist { [1:2] :/ 3, 2 };
endmodule

import q::*;
module m;
  localparam p::t X = 1;
  parameter p::u Y = 2;
  function automatic void f(ref p::t a, input p::u b);
  endfunction
endmodule

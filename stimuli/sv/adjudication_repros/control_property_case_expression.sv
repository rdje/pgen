module m(input clk, a, b);
  property p;
    case (a) 1: b; endcase
  endproperty
endmodule

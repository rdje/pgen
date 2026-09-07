module m(input clk, a, b);
  property p;
    case (a) 1: b; default: 1; endcase
  endproperty
endmodule

// SV-CORPUS-GRAD.3.60 CONTROL - the typed readings the .3.60 declarator guard must NOT disturb: a $unit typedef used AS A TYPE
// in a net, a function return, an ANSI port, a tf port and a parameter port. Byte-identical before and after.
typedef int T;
module m1;
  wire T x = 1;
endmodule
module m2;
  function T f;
    f = 1;
  endfunction
endmodule
module m3(input T x);
endmodule
module m4;
  function int f;
    input T x;
    return 1;
  endfunction
endmodule
module m5 #(T typed_value = 8'ha5) ();
endmodule

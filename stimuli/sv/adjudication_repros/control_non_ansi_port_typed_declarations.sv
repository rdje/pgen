// SV-CORPUS-GRAD.3.60 CONTROL - the typed and the plain non-ANSI port declarations the .3.60 declarator guard must NOT disturb.
// Byte-identical before and after.
typedef logic [3:0] T;
module m(x);
  input T x;
endmodule
module m2(x, y);
  input T x, y;
endmodule
module m3(x);
  input T [1:0] x;
endmodule
module m4(a);
  input a;
endmodule
module m5(a, b, c);
  input a, b;
  output logic c;
endmodule
module m6(a, b);
  input wire a;
  output reg [3:0] b;
endmodule

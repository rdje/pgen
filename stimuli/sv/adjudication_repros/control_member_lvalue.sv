// SV-CORPUS-GRAD.3.51 - the twin of invalid_method_call_in_lvalue.sv with a plain member target: ACCEPT.
class C;
  int x;
endclass
module m;
  C c;
  initial c.x = 4;
endmodule

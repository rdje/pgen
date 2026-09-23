// SV-CORPUS-GRAD.30 - an IDENTITY control (ACCEPT before and after 1.0.242, typed AST byte-identical): a DECLARED
// parameterized class as a block local under an open wildcard import keeps the known arm - the escape ties it on the same
// span and loses the tie by declaration order.
import q::*;
class C #(type T = int);
endclass
class d;
  function void f();
    C #(int) x;
  endfunction
endclass
module m;
  initial begin
    C #(int) y;
  end
endmodule

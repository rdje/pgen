// SV-CORPUS-GRAD.31 - an IDENTITY control (ACCEPT before and after 1.0.243, typed AST byte-identical): every class-scope
// shape the escape reaches, with the class DECLARED under an open wildcard import - the known `class_scope` arm ties the
// escape on the same span at every site and wins by declaration order.
import q::*;
class C #(type T = int);
  static int count;
  static int arr [4];
  typedef enum { A, B } e_t;
  typedef int width_t;
  localparam int WIDTH = 8;
  static function int f(int a); return a; endfunction
  extern function void m();
  extern function new();
  extern constraint cc;
endclass
function void C#(int)::m();
endfunction
function C#(int)::new();
endfunction
constraint C#(int)::cc { count > 0; }
module m;
  wire int x;
  assign x = C#(int)::f(1);
  localparam int W = C#(int)::WIDTH;
  initial begin
    C#(int)::count = 1;
  end
endmodule
class d;
  int y;
  C#(int) h;
  function void g();
    y = C#(int)::count + 1;
    y = C#(int)::A;
    y = C#(int)::arr[0];
    h = C#(int)::new();
    y = C#(int)::f(1).abs();
  endfunction
endclass

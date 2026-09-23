// SV-CORPUS-GRAD.31 - a CONTROL pin (ACCEPT from 1.0.243 on both SV profiles): the class-scope wildcard escape at every
// expression / statement / parameter site - a call in a continuous assign, the UVM factory chain, a primary, a class-scoped
// parameter, an enum id, an array through the scope, a scoped `new`, a method-call receiver, a static-member lvalue in a
// module block. IEEE 1800-2017 A.1.9 class_scope ::= class_type ::; A.8.4 primary with class_qualifier; A.8.5
// variable_lvalue; A.8.6 tf_call; A.9.3 ps_parameter_identifier; 26.3 / 26.4.
import q::*;
module m;
  wire int x;
  assign x = C#(int)::f(1);
  localparam int W = C#(int)::WIDTH;
  initial begin
    C#(int)::count = 1;
    uvm_config_db#(int)::set(null, "*", "w", W);
  end
endmodule
class d;
  int y;
  C#(int) h;
  function void g();
    d hh;
    hh = C#(int)::type_id::create("hh");
    y = C#(int)::count + 1;
    y = C#(int)::A;
    y = C#(int)::arr[0];
    h = C#(int)::new();
    y = C#(int)::f(1).abs();
  endfunction
endclass

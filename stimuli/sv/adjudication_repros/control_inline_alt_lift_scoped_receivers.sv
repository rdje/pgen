// SV-CORPUS-GRAD.3.42 - CONTROL (byte-identical): package- and class-scoped method-call RECEIVERS and a package-scoped
// foreach array. `p::q.m()` reaches `method_call_initial` through `split_direct_callable_method_call`
// (`split_hierarchical_callable_receiver`, whose `pkg::` / `C::` branches were never shadowed), so `direct_method_call`'s
// receiver alternative 2 - the one the census named - is a successful-but-LOSING branch on this text (TOOLBOX 3.5, C3-B).
// Every tree in this file is byte-identical between release 1.0.214 and 1.0.215 (interpreter AST JSON, HEAD grammar vs
// candidate). Arm `split_hierarchical`: the receiver route that wins.
package p;
  class C;
    static function int m(); return 1; endfunction
  endclass
  int arr[4];
  C q;
endpackage
module t;
  int z;
  initial begin
    z = p::q.m();
    z = p::C::m();
    foreach (p::arr[i]) z = i;
  end
endmodule

// SV-CORPUS-GRAD.3.40 - FIXED in 1.0.212 - must keep ACCEPTING.
// local:: before a SUBROUTINE CALL inside randomize() with { ... }: IEEE 1800-2017 18.7.1 "When applied to an identifier within an in-line constraint, the local:: qualifier bypasses the scope of the [randomize() with object] class and resolves the identifier in the local scope" - a subroutine name is an identifier; "The local:: prefix may be used to qualify class scopes and type names" (docs/systemverilog/2017/md/section-18-constrained-random-value-generation.md:1708-1726). A.8.4 class_qualifier (section-41-data-read-api.md:3140) reaches primary but no tf_call production - an Annex gap, not a prohibition. verilator t_randomize_method_with_scoping.v:70-71.
class C;
  int c1_f;
  int c2_f;
endclass
class K;
  static function int m(); return 1; endfunction
endclass
class F;
  rand int x;
  C e;
  function int get_x(); return x; endfunction
  function C get_c(); return e; endfunction
  function int f(); return x; endfunction
  function void g(int x);
    // local::this — 18.7.1: "including the special names this or super (i.e., local::this)"
    void'(randomize() with { x == local::this.f(); });
  endfunction
endclass
module t;
  F f;
  C e;
  int doit;
  function int get_x(); return 1; endfunction
  function C get_c(); return e; endfunction
  initial begin
    f = new;
    doit = f.randomize() with { get_x() == local::get_x(); };
    doit &= f.randomize() with { get_c().c1_f == local::get_c().c2_f; };
    doit &= f.randomize() with { (get_c).c1_f == (local::get_c).c2_f; };
    // local:: qualifying a CLASS SCOPE — 18.7.1: "The local:: prefix may be used to qualify class scopes and type names"
    doit &= f.randomize() with { x == local::K::m(); };
  end
endmodule

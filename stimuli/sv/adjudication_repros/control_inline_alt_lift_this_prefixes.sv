// SV-CORPUS-GRAD.3.42 - CONTROL: the `this .` prefix at every single-template site the inline-alternation census
// named - IEEE 1800-2017 A.9.3 `ps_or_hierarchical_array_identifier` (`foreach (this.arr[i])`), A.1.10
// `constraint_primary` (`solve this.x before this.y;`, `disable soft this.x;`), A.8.5 `nonrange_variable_lvalue`
// (`this.arr = new[4];`). ACCEPT before and after. Until release 1.0.214 each `scope` field carried the HOST's own
// template wrapped around `this .` - `{name: {kind: "dot"}, scope: {kind: "this"}[, select: "<invalid_sequence_access>"
// [, call: "<invalid_sequence_access>"]]}` - in an ACCEPTING file (4 invalid_sequence_access on sv_2017, 7 on sv_2023);
// since 1.0.215 `scope` is the pair `[{kind: "this"}, {kind: "dot"}]`. No instrument pins that NESTED shape byte-exactly:
// an `arm` chain and the AST-shape contract both key on `kind`, and the pair's kinds (`this`, `dot`) were present in the
// corrupted node too - the gap is routed to ENGINE-UNIVERSAL-SERVICES.97; the corpus AST identity sweep (TOOLBOX 1.5e)
// measured the move on the release. This row pins the verdict and the arm `dynamic_array_new` (the `new[]` lvalue route).
class C;
  int arr[];
  int n;
  rand int x, y;
  constraint c1 { solve this.x before this.y; }
  constraint c2 { disable soft this.x; }
  function void f();
    this.arr = new[4];
    foreach (this.arr[i]) this.n = i;
  endfunction
endclass

// SV-CORPUS-GRAD.3.37 - a CONTROL - must keep ACCEPTING.
// x < local::x inside randomize() with { ... } (IEEE 1800-2017 18.7.1's own example) and local::x as a primary - the [ local :: ] half of A.8.4 class_qualifier
// IEEE 1800-2017 A.8.4 class_qualifier ::= [ local :: ] [ implicit_class_handle . | class_scope ] docs/systemverilog/2017/md/section-41-data-read-api.md:3140 (footnote 43); 18.7.1 docs/systemverilog/2017/md/section-18-constrained-random-value-generation.md
class C;
  rand int x;
  function void f(int x);
    void'(randomize() with { x < local::x; });
  endfunction
endclass
module t;
  int x;
  initial $display(local::x);
endmodule

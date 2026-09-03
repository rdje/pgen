// SV-CORPUS-GRAD.3.29 — an ACCEPTING CONTROL: the same cross-body function returning a type the file
// DECLARES itself. Parses before and after the fix, which isolates the defect to the two implicit
// typedefs (IEEE 1800-2017 §19.6.1.3, section-19-functional-coverage.md:1320): a function in a cross
// body was never the problem (`.13c.2d`), an undeclared return-type name was.
// WHY: if this control goes RED the cross-body scope broke ordinary declared-type visibility.
module m;
  logic [31:0] a, b;
  typedef int q_t[$];
  covergroup cg(int cg_lim);
    coverpoint a;
    coverpoint b;
    aXb : cross a, b
    {
      function q_t myFunc1(int f_lim);
        for (int i = 0; i < f_lim; ++i)
          myFunc1.push_back(i);
      endfunction
      bins one = myFunc1(cg_lim);
    }
  endgroup
endmodule

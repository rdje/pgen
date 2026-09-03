// SV-CORPUS-GRAD.3.29 — FIXED in PGEN-SV-CORPUS-GRAD-0347 (release 1.0.206, ledger SV-0079).
// IEEE 1800-2017 §19.6.1.3 (docs/systemverilog/2017/md/section-19-functional-coverage.md:1307-1308):
// "The types are named CrossValType and CrossQueueType. The scope of the type names is the cross itself
// and the types are not accessible outside of this scope." and :1320 "The cross types shall be considered
// as implicit typedefs in the body of the cross". §19.6.1.4 (:1373) is the standard's own example of
// their USE, reproduced by ispras ieee-1800-2012/19/19.06.01.04_02.sv and Surelog tests/CrossFunc/dut.sv.
// WHY it rejected: `function CrossQueueType myFunc1(...)` routes its return type through
// checked_type_identifier, whose post predicate has_fact(type_name, CrossQueueType) found no fact — the
// grammar declared the two implicit typedefs nowhere. FIX: cross_body_open opens a `cross` scope and
// emits both type_name facts (declaration_family typedef); cross_body_close pops it, so the two names
// are type names exactly in the body of the cross and nowhere else.
module m;
  logic [31:0] a, b;
  covergroup cg(int cg_lim);
    coverpoint a;
    coverpoint b;
    aXb : cross a, b
    {
      function CrossQueueType myFunc1(int f_lim);
        for (int i = 0; i < f_lim; ++i)
          myFunc1.push_back('{i, i});
      endfunction
      bins one = myFunc1(cg_lim);
    }
  endgroup
endmodule

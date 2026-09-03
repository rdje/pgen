// SV-CORPUS-GRAD.3.29 — FIXED in PGEN-SV-CORPUS-GRAD-0347 (release 1.0.206, ledger SV-0079).
// The SECOND implicit typedef: CrossValType used as a variable's type inside a cross-body function.
// IEEE 1800-2017 §19.6.1.3 (docs/systemverilog/2017/md/section-19-functional-coverage.md:1314-1318):
// "The definition of CrossValType is a SystemVerilog struct consisting of one member for each coverpoint in
// the cross [...] The definition of CrossQueueType is an unbounded queue of CrossValType elements."
// WHY: pins that BOTH names are emitted — a fix that declared only CrossQueueType leaves this RED.
module m;
  logic [31:0] a, b;
  covergroup cg(int cg_lim);
    coverpoint a;
    coverpoint b;
    aXb : cross a, b
    {
      function CrossQueueType myFunc(int f_lim);
        CrossValType v;
        for (int i = 0; i < f_lim; ++i) begin
          v = '{i, i};
          myFunc.push_back(v);
        end
      endfunction
      bins one = myFunc(cg_lim);
    }
  endgroup
endmodule

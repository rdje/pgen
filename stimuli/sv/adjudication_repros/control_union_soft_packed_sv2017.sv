// SV-CORPUS-GRAD.14b - a CONTROL: the 2017 spelling of the same construct, legal in both editions - must keep ACCEPTING.
// typedef union soft packed { ... } u_t; - a soft packed union - the 2017 spelling
// IEEE 1800-2017 A.2.2.1 struct_union ::= struct | union [ tagged ] docs/systemverilog/2017/md/section-41-data-read-api.md:917 - no `soft`.
package p;
  typedef union packed {
    logic [7:0] a;
    logic [7:0] b;
  } u_t;
endpackage

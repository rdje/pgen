// SV-CORPUS-GRAD.14b - a CONTROL under the profile it names - must keep ACCEPTING there.
// typedef union soft packed { ... } u_t; - a soft packed union, judged as IEEE 1800-2023
// IEEE 1800-2023 A.2.2.1 struct_union ::= struct | union [ soft | tagged ] docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:916-920 with data_type ::= struct_union [ packed [ signing ] ] .... the edition contrast: byte-identical text to the sv_2017 invalid pin, legal in 2023.
package p;
  typedef union soft packed {
    logic [7:0] a;
    logic [7:0] b;
  } u_t;
endpackage

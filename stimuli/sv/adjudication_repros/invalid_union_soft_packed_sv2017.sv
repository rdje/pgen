// SV-CORPUS-GRAD.14b - an INVALID pin under the profile it names: text the 1800-2017 edition derives nothing for - must keep REJECTING there.
// typedef union soft packed { ... } u_t; - a soft packed union, judged as IEEE 1800-2017
// IEEE 1800-2023 A.2.2.1 struct_union ::= struct | union [ soft | tagged ] docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:916-920 with data_type ::= struct_union [ packed [ signing ] ] ... (2023 only); IEEE 1800-2017 A.2.2.1 struct_union ::= struct | union [ tagged ] docs/systemverilog/2017/md/section-41-data-read-api.md:917 - no `soft`. verilator t_export_packed_struct.v l.9, t_export_packed_struct2.v l.9; the same bytes are the sv_2023 control beside this row.
package p;
  typedef union soft packed {
    logic [7:0] a;
    logic [7:0] b;
  } u_t;
endpackage

// SV-CORPUS-GRAD.14b - an INVALID pin under the profile it names: text the 1800-2017 edition derives nothing for - must keep REJECTING there.
// union soft { ... } u; - a soft union, judged as IEEE 1800-2017
// IEEE 1800-2023 A.2.2.1 struct_union ::= struct | union [ soft | tagged ] docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:916-920 (2023 only); IEEE 1800-2017 A.2.2.1 struct_union ::= struct | union [ tagged ] docs/systemverilog/2017/md/section-41-data-read-api.md:917 - no `soft`. verilator t_union_soft.v l.9; the same bytes are the sv_2023 control beside this row.
module t;
  union soft {
    bit [7:0] val1;
    bit [7:0] val2;
  } u;
endmodule

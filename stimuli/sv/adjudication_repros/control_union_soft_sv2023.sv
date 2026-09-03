// SV-CORPUS-GRAD.14b - a CONTROL under the profile it names - must keep ACCEPTING there.
// union soft { ... } u; - a soft union, judged as IEEE 1800-2023
// IEEE 1800-2023 A.2.2.1 struct_union ::= struct | union [ soft | tagged ] docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:916-920. the edition contrast: byte-identical text to the sv_2017 invalid pin, legal in 2023.
module t;
  union soft {
    bit [7:0] val1;
    bit [7:0] val2;
  } u;
endmodule

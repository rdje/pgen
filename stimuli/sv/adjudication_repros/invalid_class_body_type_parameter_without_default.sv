// SV-CORPUS-GRAD.3.52 - a class-body TYPE parameter without a default: `parameter type AT;` (Surelog tests/ClassTypeParamAlias/dut.sv:4). IEEE 1800-2017 Annex A footnote 18: "It shall be legal to omit the constant_param_expression from a param_assignment or the data_type from a type_assignment only within a parameter_port_list" (docs/systemverilog/2017/md/section-41-data-read-api.md:3518-3519); 1800-2023 the same footnote (docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:3483-3484) - the .3.37 ruling (footnote 18's parameter half) on a type_assignment.
// REJECTS at the `;` on both SV profiles; the controls `parameter type AT = logic;` in the body and `class C #(parameter type AT);` in the port list ACCEPT.
class C;
  parameter type AT;
endclass

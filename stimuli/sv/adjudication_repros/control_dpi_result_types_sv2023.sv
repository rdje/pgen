// SV-CORPUS-GRAD.3.43 - CONTROL (sv_2023): IEEE 1800-2023 A.2.6 `function_prototype ::= function [ dynamic_override_specifiers ]
// data_type_or_void function_identifier ...` and `dpi_function_proto ::= function_prototype`, so the 2023 twin of the DPI
// prototype carries the override slot; `dpi_function_prototype_sv_2023` mirrors it. Accepted before and after with the
// typed AST BYTE-IDENTICAL (interpreter AST JSON, HEAD grammar vs 1.0.216).
// docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:1158 (dpi_function_proto), :1210-1217
// (dynamic_override_specifiers); §35.5.5 :493-501.
module t;
  import "DPI-C" function :initial int f_ini(input int a);
  import "DPI-C" function :final void f_fin(input int a);
endmodule

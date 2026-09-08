// SV-CORPUS-GRAD.3.51 - a METHOD CALL inside an assignment target: `baz.get_bar().f.x = 4;` (verilator t_class_static_member_sel.v). IEEE 1800-2017 A.8.5 variable_lvalue ::= [ implicit_class_handle . | package_scope ] hierarchical_variable_identifier select | { variable_lvalue { , variable_lvalue } } | [ assignment_pattern_expression_type ] assignment_pattern_variable_lvalue | streaming_concatenation (docs/systemverilog/2017/md/section-41-data-read-api.md:3199-3204) - a hierarchical identifier with selects, never a call; 1800-2023 the same (docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:3146-3151).
// REJECTS at the `(` of the call on both SV profiles; the control `c.x = 4;` ACCEPTS.
class C;
  int x;
  function C get(); return this; endfunction
endclass
module m;
  C c;
  initial c.get().x = 4;
endmodule

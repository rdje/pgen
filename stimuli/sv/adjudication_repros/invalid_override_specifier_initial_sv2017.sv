// SV-CORPUS-GRAD.14b - an INVALID pin under the profile it names: text the 1800-2017 edition derives nothing for - must keep REJECTING there.
// function :initial int get_i; ... in a class - the :initial dynamic-override specifier (sibling of the .13x.d :extends pin), judged as IEEE 1800-2017
// IEEE 1800-2023 A.1.9 dynamic_override_specifiers ::= [ initial_or_extends_specifier ] [ final_specifier ], initial_or_extends_specifier ::= : initial | : extends docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:1210-1217 (2023 only); IEEE 1800-2017 A.1.9 class_method docs/systemverilog/2017/md/section-41-data-read-api.md:210-215 has no such production (the 2017 spelling is control_plain_method_sv2017.sv). verilator t_class_override.v l.26; the same bytes are the sv_2023 control beside this row.
class C;
  function :initial int get_i; return 1; endfunction
endclass

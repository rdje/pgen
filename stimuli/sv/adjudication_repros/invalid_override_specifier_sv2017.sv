// SV-CORPUS-GRAD.13x.d - an INVALID pin under the profile it names: text the 1800-2017 edition derives nothing for - must keep REJECTING there.
// function :extends int get_e; ... in a class, judged as IEEE 1800-2017
// IEEE 1800-2023 A.1.9 dynamic_override_specifiers docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:1210-1217 (2023 only); IEEE 1800-2017 class_method docs/systemverilog/2017/md/section-41-data-read-api.md:210-215 has no such production. verilator t_class_override_bad.v l.23; the same text is the sv_2023 control beside this row.
class C;
  function :extends int get_e; return 1; endfunction
endclass

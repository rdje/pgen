// SV-CORPUS-GRAD.13x.d - a CONTROL under the profile it names - must keep ACCEPTING there.
// function :extends int get_e; ... in a class, judged as IEEE 1800-2023
// IEEE 1800-2023 A.1.9 docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:1210-1217. the edition contrast: byte-identical text to the sv_2017 invalid pin, legal in 2023.
class C;
  function :extends int get_e; return 1; endfunction
endclass

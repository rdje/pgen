// SV-CORPUS-GRAD.14b - a CONTROL under the profile it names - must keep ACCEPTING there.
// function :initial int get_i; ... in a class - the :initial dynamic-override specifier (sibling of the .13x.d :extends pin), judged as IEEE 1800-2023
// IEEE 1800-2023 A.1.9 dynamic_override_specifiers ::= [ initial_or_extends_specifier ] [ final_specifier ], initial_or_extends_specifier ::= : initial | : extends docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:1210-1217. the edition contrast: byte-identical text to the sv_2017 invalid pin, legal in 2023.
class C;
  function :initial int get_i; return 1; endfunction
endclass

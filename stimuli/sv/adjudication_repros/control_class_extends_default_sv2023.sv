// SV-CORPUS-GRAD.14b - a CONTROL under the profile it names - must keep ACCEPTING there.
// class Cls1 extends Base1(default); - the `default` argument list on extends, judged as IEEE 1800-2023
// IEEE 1800-2023 A.1.9 class_declaration ::= ... [ extends class_type [ ( [ list_of_arguments | default ] ) ] ] docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:179. the edition contrast: byte-identical text to the sv_2017 invalid pin, legal in 2023.
class Base1;
  function new(int d = 1); endfunction
endclass
class Cls1 extends Base1(default);
endclass

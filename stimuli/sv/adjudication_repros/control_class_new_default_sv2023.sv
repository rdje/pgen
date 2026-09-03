// SV-CORPUS-GRAD.14b - a CONTROL under the profile it names - must keep ACCEPTING there.
// function new(default); - a constructor declaration whose argument is `default`, judged as IEEE 1800-2023
// IEEE 1800-2023 A.1.9 class_constructor_declaration ::= function [ class_scope ] new [ ( [ class_constructor_arg_list ] ) ] ; docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:546-549, class_constructor_arg ::= tf_port_item | default docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:557. the edition contrast: byte-identical text to the sv_2017 invalid pin, legal in 2023.
class C;
  int i;
  function new(default);
    i = 1;
  endfunction
endclass

// SV-CORPUS-GRAD.14b - an INVALID pin under the profile it names: text the 1800-2017 edition derives nothing for - must keep REJECTING there.
// function new(default); - a constructor declaration whose argument is `default`, judged as IEEE 1800-2017
// IEEE 1800-2023 A.1.9 class_constructor_declaration ::= function [ class_scope ] new [ ( [ class_constructor_arg_list ] ) ] ; docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:546-549, class_constructor_arg ::= tf_port_item | default docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:557 (2023 only); IEEE 1800-2017 A.1.9 class_constructor_declaration ::= function [ class_scope ] new [ ( [ tf_port_list ] ) ] ; docs/systemverilog/2017/md/section-41-data-read-api.md:622-625 - tf_port_item admits no `default`. verilator t_class_new_default.v l.10; the same bytes are the sv_2023 control beside this row.
class C;
  int i;
  function new(default);
    i = 1;
  endfunction
endclass

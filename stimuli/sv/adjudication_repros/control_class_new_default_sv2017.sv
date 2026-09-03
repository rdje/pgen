// SV-CORPUS-GRAD.14b - a CONTROL: the 2017 spelling of the same construct, legal in both editions - must keep ACCEPTING.
// function new(default); - a constructor declaration whose argument is `default` - the 2017 spelling
// IEEE 1800-2017 A.1.9 class_constructor_declaration ::= function [ class_scope ] new [ ( [ tf_port_list ] ) ] ; docs/systemverilog/2017/md/section-41-data-read-api.md:622-625 - tf_port_item admits no `default`.
class C;
  int i;
  function new(int j);
    i = j;
  endfunction
endclass

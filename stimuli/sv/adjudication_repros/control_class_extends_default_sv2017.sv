// SV-CORPUS-GRAD.14b - a CONTROL: the 2017 spelling of the same construct, legal in both editions - must keep ACCEPTING.
// class Cls1 extends Base1(default); - the `default` argument list on extends - the 2017 spelling
// IEEE 1800-2017 A.1.9 class_declaration ::= ... [ extends class_type [ ( list_of_arguments ) ] ] docs/systemverilog/2017/md/section-41-data-read-api.md:187 - no `default`.
class Base1;
  function new(int d = 1); endfunction
endclass
class Cls1 extends Base1(1);
endclass

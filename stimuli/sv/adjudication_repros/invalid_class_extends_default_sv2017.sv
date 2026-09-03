// SV-CORPUS-GRAD.14b - an INVALID pin under the profile it names: text the 1800-2017 edition derives nothing for - must keep REJECTING there.
// class Cls1 extends Base1(default); - the `default` argument list on extends, judged as IEEE 1800-2017
// IEEE 1800-2023 A.1.9 class_declaration ::= ... [ extends class_type [ ( [ list_of_arguments | default ] ) ] ] docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:179 (2023 only); IEEE 1800-2017 A.1.9 class_declaration ::= ... [ extends class_type [ ( list_of_arguments ) ] ] docs/systemverilog/2017/md/section-41-data-read-api.md:187 - no `default`. verilator t_class_extends_default.v l.14; the same bytes are the sv_2023 control beside this row.
class Base1;
  function new(int d = 1); endfunction
endclass
class Cls1 extends Base1(default);
endclass

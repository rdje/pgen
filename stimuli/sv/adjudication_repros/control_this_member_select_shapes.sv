// SV-CORPUS-GRAD.29 - a CONTROL (ACCEPT from 1.0.241 under sv_2017; accepted under sv_2023 before) - must keep ACCEPTING.
// Every shape where the `this.`-rooted member chain ends in a SELECT: IEEE 1800-2017 A.8.4 primary ::= [ class_qualifier | package_scope ]
// hierarchical_identifier select, class_qualifier ::= [ local :: ] [ implicit_class_handle . | class_scope ] - `super.` and `this.super.`
// are implicit_class_handle too (A.8.4 implicit_class_handle ::= this | super | this . super); the part-select is A.8.4 select's
// [ range_expression ] tail; `this.obj.z[0]` puts the select after a member chain; `this.arr[0] + 1` is the primary as a binary operand.
class p;
  int y;
  int z[2];
endclass
class c;
  int arr[4];
  int id;
  p obj;
  p objs[4];
  function void f();
    id = super.arr[0];
    id = this.super.arr[0];
    id = this.arr[1:0];
    id = this.obj.z[0];
    id = this.objs[0].z[1];
    id = this.arr[0] + 1;
    if (this.arr[0]) id = {this.arr[0], this.arr[1]};
  endfunction
endclass

// SV-CORPUS-GRAD.29 - a CONTROL (ACCEPT before and after 1.0.241, tree byte-identical) - must keep ACCEPTING.
// The `this.`-rooted expressions the primary's earlier `call` arm reads to the same end as the new class_qualifier prefix; under
// longest_match the tie goes to declaration order, so every one keeps its `call` tree: a bare member, a member chain, a bare and a
// parenthesised built-in method, a method call after a select, a select then a member, and a method after a select-then-member.
class p;
  int y;
  int z[2];
  function int g();
    return 1;
  endfunction
endclass
class c;
  int arr[4];
  int q[$];
  int id;
  p obj;
  p objs[4];
  function void f();
    id = this.id;
    id = this.obj.y;
    id = this.obj.obj.y;
    id = this.q.size;
    id = this.q.size();
    id = this.objs[0].g();
    id = this.objs[0].y;
    id = this.objs[0].z.size();
    obj = this;
  endfunction
endclass

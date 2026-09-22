// SV-CORPUS-GRAD.28 - the CONTROL for defect_this_member_select_rvalue.sv - must keep ACCEPTING.
// A `this.member[select]` FOLLOWED by a further member parses today: the select is read on the hierarchical path, not on the primary.
class p;
  int y;
endclass
class c;
  p arr[4];
  int id;
  function void f();
    id = this.arr[0].y;
  endfunction
endclass

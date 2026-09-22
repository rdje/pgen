// SV-CORPUS-GRAD.28 - the CONTROL for defect_this_member_select_rvalue.sv - must keep ACCEPTING.
// The same selects as LVALUES, and a bare member select, parse today (IEEE 1800-2017 A.8.5 variable_lvalue, A.8.4 primary).
class c;
  int arr[4];
  string names[string];
  string n;
  int id;
  function void f();
    this.arr[0] = 1;
    this.names[n] = "x";
    id = arr[0];
    id = this.id;
  endfunction
endclass

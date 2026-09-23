// SV-CORPUS-GRAD.30 - a CONTROL pin (ACCEPT today, both SV profiles): the same class-scoped static call and static member
// access through a DECLARED parameterized class - the nearest construct that parses, isolating the .31 defect to the
// unresolved-under-wildcard-import case.
class C #(type T = int);
  static int count;
  static function void set(int a);
  endfunction
endclass
class d;
  int x;
  function void f();
    C#(int)::set(1);
    C#(int)::count = 1;
    x = C#(int)::count;
  endfunction
endclass

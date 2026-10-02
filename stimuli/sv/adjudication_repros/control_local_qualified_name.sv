class c;
  int q[$];
  int x;
  c o;
  function int g(); return 0; endfunction
  function void f();
    if (!o.randomize() with { x == local::x; q[0] == local::q[0]; x == local::g(); }) ;
  endfunction
endclass

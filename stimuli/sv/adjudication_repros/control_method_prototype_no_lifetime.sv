// SV-CORPUS-GRAD.41 - a CONTROL - the legal neighbour; must keep ACCEPTING on every profile.
// pure virtual task t(); and extern function int f(); with function automatic int c::f(); out of block - the same prototypes without the lifetime
// IEEE 1800-2017 A.1.9 method_prototype; 8.24 docs/systemverilog/2017/txt/section-8-classes.txt:1011-1013
virtual class b;
  pure virtual task t();
endclass
class c;
  extern function int f();
endclass
function automatic int c::f();
  return 1;
endfunction

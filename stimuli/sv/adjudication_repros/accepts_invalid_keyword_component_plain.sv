// SV-CORPUS-GRAD.13c.2k — the same over-acceptance with NO bit-select, which proves the index is
// not what admits the keyword: the component loop itself is unguarded. Expected today: ACCEPT.
class inner; function int g(); return 0; endfunction endclass
class rb; inner arr[2]; endclass
class w; function int f(rb ral); return ral.module.g(); endfunction endclass

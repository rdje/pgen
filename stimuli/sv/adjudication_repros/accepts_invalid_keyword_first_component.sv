// SV-CORPUS-GRAD.13c.2k — the same over-acceptance in the FIRST component, which proves it is not
// specific to a member position: every NON-FINAL component of a hierarchical path is unguarded.
// Expected today: ACCEPT.
class inner; function int g(); return 0; endfunction endclass
class rb; inner arr[2]; endclass
class w; function int f(rb ral); return module.g(); endfunction endclass

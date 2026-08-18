// SV-CORPUS-GRAD.13c.2k — the GUARDED counterpart: the same reserved keyword as the FINAL component
// of a hierarchical name. `hierarchical_identifier`'s tail is `non_keyword_identifier`, so this
// REJECTS — correctly.
class inner; function int g(); return 0; endfunction endclass
class rb; inner arr[2]; endclass
class w; function int f(rb ral); return ral.arr[0].module; endfunction endclass

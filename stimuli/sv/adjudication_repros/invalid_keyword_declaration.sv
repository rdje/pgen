// SV-CORPUS-GRAD.13c.2k — the GUARDED counterpart in a declaration. `declaration_identifier :=
// non_keyword_identifier`, so this REJECTS — correctly, and it shows the keyword table itself is
// live: the leak is positional, not a missing keyword.
class inner; function int g(); return 0; endfunction endclass
class rb; inner arr[2]; endclass
class w; function int f(rb ral); int module; return 0; endfunction endclass

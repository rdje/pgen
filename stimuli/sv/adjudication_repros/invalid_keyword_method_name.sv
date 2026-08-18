// SV-CORPUS-GRAD.13c.2k — the GUARDED counterpart: the same reserved keyword as the METHOD NAME.
// `callable_identifier := non_keyword_identifier`, so this REJECTS — correctly. It is what makes
// the sibling ACCEPTs a statement about WHICH positions are unguarded rather than about the keyword.
class inner; function int g(); return 0; endfunction endclass
class rb; inner arr[2]; endclass
class w; function int f(rb ral); return ral.arr[0].module(); endfunction endclass

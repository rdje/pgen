// SV-CORPUS-GRAD.13c.2k — the one-identifier CONTROL: `ral.module[0].g()` with `module` spelled
// `modxx`. Legal SV, parses before and after the fix, so the sibling's flip is attributable to the
// reserved word alone and not to the shape of the expression.
class inner; function int g(); return 0; endfunction endclass
class rb; inner arr[2]; endclass
class w; function int f(rb ral); return ral.modxx[0].g(); endfunction endclass

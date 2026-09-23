// SV-CORPUS-GRAD.31 - a CONTROL pin (ACCEPT from 1.0.243 on both SV profiles): the class-scope wildcard escape at the
// out-of-block sites - an out-of-block method body, an out-of-block constructor, an extern constraint block - whose class
// is an unresolved parameterized class under an open wildcard import (IEEE 1800-2017 8.24 out-of-block declarations, 18.5.1
// extern constraint blocks; A.1.9 class_scope).
import q::*;
function void C#(int)::m();
endfunction
function C#(int)::new();
endfunction
constraint C#(int)::cc { count > 0; }

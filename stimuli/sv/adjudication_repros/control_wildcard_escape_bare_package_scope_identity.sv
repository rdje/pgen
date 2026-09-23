// SV-CORPUS-GRAD.31 - an IDENTITY control (ACCEPT before and after 1.0.243, typed AST byte-identical): a BARE package scope
// under an open wildcard import keeps its package_scope reading - the escape consumes a parameter list, so a bare `ident ::`
// never takes it (a bare unresolved scope may be a package scope, a different node; not admitted here).
package p;
  int y = 1;
endpackage
import q::*;
module m;
  int x;
  initial x = p::y;
endmodule

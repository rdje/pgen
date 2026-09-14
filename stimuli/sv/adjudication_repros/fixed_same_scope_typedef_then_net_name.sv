// SV-CORPUS-GRAD.3.60 - the SAME-SCOPE twin: `typedef int T;` then `wire T = 1'b1;` in one module. IEEE 1800-2017 6.4
// makes the redeclaration a SEMANTIC error ("within a name space it shall be illegal to redeclare a name already
// declared"), not a syntax one - Annex A derives the text exactly as it derives `typedef int T; int T;`, which every
// release accepted; the parser reads syntax and leaves 6.4 to the elaborator.
module m;
  typedef int T;
  wire T = 1'b1;
endmodule

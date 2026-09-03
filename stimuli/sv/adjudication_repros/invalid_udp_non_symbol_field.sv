// SV-CORPUS-GRAD.3.30 — an INVALID pin: table fields that no symbol production derives — the digit `2`
// and the identifier `a` — must keep rejecting on every profile (IEEE 1800-2017 A.5.3 level_symbol ::=
// 0 | 1 | x | X | ? | b | B; output_symbol ::= 0 | 1 | x | X). WHY: the boundary-free terminals must
// admit exactly the LRM's symbol characters and nothing wider; a terminal that matched a digit run
// or an identifier would start ACCEPTING this file — over-acceptance.
primitive bad2(y, a, b);
  output y; input a, b;
  table
    2 0 : 0 ;
  endtable
endprimitive
primitive bad3(y, a, b);
  output y; input a, b;
  table
    0 a : 0 ;
  endtable
endprimitive

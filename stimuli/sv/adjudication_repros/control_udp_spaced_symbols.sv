// SV-CORPUS-GRAD.3.30 — an ACCEPTING CONTROL: the same tables with one space between every symbol.
// Parses before and after the fix, and its typed AST must be byte-identical across the fix — the symbol
// rules keep their `{kind: "…"}` shapes and only their terminals lose a boundary assertion.
// WHY: if this control goes RED, or its AST moves, the new terminals changed more than adjacency.
primitive dff(q, clk, d);
  output q; reg q; input clk, d;
  table
    (0 1) 0 : ? : 0 ;
    (0 1) 1 : ? : 1 ;
    (1 ?) ? : ? : - ;
  endtable
endprimitive
primitive jk(q, clk, j, k, pc);
  output q; reg q; input clk, j, k, pc;
  table
    ? ? ? 0 1 : ? : 1 ;
    ? ? ? * 1 : 1 : 1 ;
    r 0 0 1 : 0 : 1 ;
  endtable
endprimitive
primitive andp(y, a, b);
  output y; input a, b;
  table
    0 0 : 0 ;
    0 x : 0 ;
    1 1 : 1 ;
  endtable
endprimitive

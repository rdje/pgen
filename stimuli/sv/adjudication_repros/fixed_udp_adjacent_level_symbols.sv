// SV-CORPUS-GRAD.3.30 — FIXED in PGEN-SV-CORPUS-GRAD-0349 (release 1.0.207, ledger SV-0080).
// Adjacent LEVEL symbols outside any parentheses — `??`, `01`, `00`, `0x`, `(bx)` — in sequential AND
// combinational tables, the shape of ispras 29.09_01.sv / 1364-2005 test_08_07_00_1.v (`? ?? 01 : ? : 1`)
// and iverilog udp_bx.v (`? ? (bx): ? : x`). WHY: pins that EVERY table symbol lost its `\b`, not only the
// two inside an edge indicator — a fix that touched edge_indicator alone leaves this RED.
primitive jk(q, clk, j, k, pc);
  output q; reg q; input clk, j, k, pc;
  table
    ? ?? 01 : ? : 1 ;
    ? ?? *1 : 1 : 1 ;
    r 00 1 : 0 : 1 ;
    ? ? (bx) 1 : ? : x ;
  endtable
endprimitive
primitive andp(y, a, b);
  output y; input a, b;
  table
    00 : 0 ;
    0x : 0 ;
    11 : 1 ;
  endtable
endprimitive

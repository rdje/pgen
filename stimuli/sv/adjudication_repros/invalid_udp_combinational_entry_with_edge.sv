// SV-CORPUS-GRAD.13x.b - an INVALID pin: text no Annex A production derives - must keep REJECTING.
// 0 ? 1 : 1; (01) 1 0 : 0; - an edge indicator in a one-colon table row
// IEEE 1800-2017 A.5.3 combinational_entry section-29-user-defined-primitives.md:119 admits level symbols; an edge belongs to sequential_entry with two colons section-29-user-defined-primitives.md:123-126; section-41-data-read-api.md:1975-1982. t_udp_bad_comb_trigger.v l.14.
primitive t_gate(dout, a, b, c); output dout; input a, b, c;
  table
    0 ? 1 : 1;
    (01) 1 0 : 0;
  endtable
endprimitive

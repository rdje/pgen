// SV-CORPUS-GRAD.13x.b - an INVALID pin: text no Annex A production derives - must keep REJECTING.
// a primitive whose file ends inside table - no endtable, no endprimitive
// IEEE 1800-2017 A.5.3 combinational_body ::= table combinational_entry { combinational_entry } endtable section-29-user-defined-primitives.md:118; A.5.1 udp_declaration ... endprimitive section-41-data-read-api.md:1930-1934. t_udp_tableeof_bad.v.
primitive udp_x (a, b); output a; input b;
  table
    0 : 1;

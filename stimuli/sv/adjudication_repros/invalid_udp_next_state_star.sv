// SV-CORPUS-GRAD.13x.b - an INVALID pin: text no Annex A production derives - must keep REJECTING.
// 1 1 : ? : *; - * as a next state
// IEEE 1800-2017 A.5.3 next_state ::= output_symbol | - section-29-user-defined-primitives.md:129-130; section-41-data-read-api.md:1985-1986; IEEE 1364-2005 A.5.3 identical. t_udp_bad_illegal_output.v l.16.
primitive p(dout, a, b); output reg dout; input a, b;
  table
    1 1 : ? : *;
  endtable
endprimitive

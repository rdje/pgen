// SV-CORPUS-GRAD.13x.b - an INVALID pin: text no Annex A production derives - must keep REJECTING.
// 0 : ?; - ? as a combinational UDP output
// IEEE 1800-2017 A.5.3 combinational_entry ::= level_input_list : output_symbol ; and output_symbol ::= 0 | 1 | x | X section-29-user-defined-primitives.md:119, :130; section-41-data-read-api.md:1975, :1986; IEEE 1364-2005 A.5.3 identical. t_timescale_udp.v l.39 (the row .3.30 named).
primitive p(dout, a); output dout; input a;
  table
    0 : ?;
  endtable
endprimitive

// SV-CORPUS-GRAD.13x.b - an INVALID pin: text no Annex A production derives - must keep REJECTING.
// output dout1, dout2; inside a primitive - two identifiers where the production takes one
// IEEE 1800-2017 A.5.2 udp_output_declaration ::= { attribute_instance } output port_identifier section-29-user-defined-primitives.md:105-107; section-41-data-read-api.md:1956-1958; IEEE 1364-2005 A.5.2 identical. t_udp_bad_multi_output.v l.8.
primitive p(dout1, dout2, a); output dout1, dout2; input a;
  table
    1 : 1;
  endtable
endprimitive

// SV-CORPUS-GRAD.13x.b - an INVALID pin: text no Annex A production derives - must keep REJECTING.
// tri a_bad; inside a primitive - a net declaration among UDP port declarations
// IEEE 1800-2017 A.5.2 udp_port_declaration ::= udp_output_declaration ; | udp_input_declaration ; | udp_reg_declaration ; section-29-user-defined-primitives.md:98-101; section-41-data-read-api.md:1949-1953; IEEE 1364-2005 A.5.2 identical. t_udp_bad.v l.15.
primitive udp_x (a_bad, b, c_bad); tri a_bad; output b; output c_bad;
  table
    0 : 1;
  endtable
endprimitive

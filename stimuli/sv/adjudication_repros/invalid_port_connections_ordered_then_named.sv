// SV-CORPUS-GRAD.13x.b - an INVALID pin: text no Annex A production derives - must keep REJECTING.
// m1 m0 (j, .b(j)); - an ordered connection followed by a named one (t_inst_2star_bad.v: foo, .*)
// IEEE 1800-2017 A.4.1.1 list_of_port_connections ::= ordered { , ordered } | named { , named } section-41-data-read-api.md:1811-1820; .* is a named_port_connection. t_fuzz_genintf_bad.v l.23, t_inst_2star_bad.v l.14; sv2v keys the class binding_mix_port must_reject.
module m1(input a, input b); endmodule
module t; logic j;
  m1 m0 (j, .b(j));
  m1 m2 (j, .*);
endmodule

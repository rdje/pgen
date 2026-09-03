// SV-CORPUS-GRAD.13x.b - an INVALID pin: text no Annex A production derives - must keep REJECTING.
// m(); - a module instance with no instance name
// IEEE 1800-2017 A.4.1.1 hierarchical_instance ::= name_of_instance ( ... ), name_of_instance ::= instance_identifier ... section-41-data-read-api.md:1809-1810; IEEE 1364-2005 A.4.1 identical. t_inst_noname_bad.v l.8.
module m; endmodule
module t;
  m();
endmodule

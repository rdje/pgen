// SV-CORPUS-GRAD.13x.b - an INVALID pin: text no Annex A production derives - must keep REJECTING.
// module sub1 #([7:0] PAR1 = 1); - a packed dimension where a data_type or a keyword is required
// IEEE 1800-2017 A.1.3 parameter_port_declaration section-41-data-read-api.md:238-243; IEEE 1364-2005 A.1.3 parameter_port_list ::= # ( parameter_declaration { , parameter_declaration } ). t_param_implicit_bad.v l.9.
module sub1 #([7:0] PAR1 = 1); endmodule

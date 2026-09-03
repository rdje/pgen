// SV-CORPUS-GRAD.13x.b - an INVALID pin: text no Annex A production derives - must keep REJECTING.
// localparam type bad2 = 2; - a constant where a data_type is required
// IEEE 1800-2017 A.2.1.1 local_parameter_declaration section-41-data-read-api.md:733-737; A.2.4 type_assignment ::= type_identifier [ = data_type ] section-41-data-read-api.md:1013. t_param_type_bad.v l.9.
module t;
  localparam type bad2 = 2;
endmodule

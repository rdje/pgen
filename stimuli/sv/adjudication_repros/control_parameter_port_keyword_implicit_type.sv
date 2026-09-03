// SV-CORPUS-GRAD.13x.b - a CONTROL: the nearest LEGAL twin (one token different) - must keep ACCEPTING.
// module sub1 #(parameter [7:0] PAR1 = 1);
// IEEE 1800-2017 A.1.3 section-41-data-read-api.md:238-243; A.2.1.1 parameter_declaration ::= parameter data_type_or_implicit list_of_param_assignments section-41-data-read-api.md:739. isolates the defect to the missing keyword.
module sub1 #(parameter [7:0] PAR1 = 1); endmodule

// SV-CORPUS-GRAD.3.37 - an INVALID pin - must keep REJECTING on every profile.
// class K; parameter type T; - a class-body type parameter without its data_type
// IEEE 1800-2017 Annex A footnote 18 docs/systemverilog/2017/md/section-41-data-read-api.md:3518-3520: the default may be omitted only within a parameter_port_list
class K;
  parameter type T;
endclass

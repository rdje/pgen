// SV-CORPUS-GRAD.3.37 - a CONTROL - must keep ACCEPTING.
// trireg unsup; trireg (small) unsup_s; trireg (medium) ...; trireg (large) ... - a net declaration with a charge_strength
// IEEE 1800-2017 A.2.1.3 net_declaration ::= net_type [ drive_strength | charge_strength ] [ vectored | scalared ] data_type_or_implicit [ delay3 ] list_of_net_decl_assignments ; docs/systemverilog/2017/md/section-41-data-read-api.md:805; charge_strength ::= ( small ) | ( medium ) | ( large ) docs/systemverilog/2017/md/section-41-data-read-api.md:937
module t;
  trireg unsup;
  trireg (small) unsup_s;
  trireg (medium) unsup_m;
  trireg (large) unsup_l;
endmodule

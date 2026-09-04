// SV-CORPUS-GRAD.13y - an INVALID pin - text IEEE 1800-2017 derives nothing for; must keep REJECTING on every profile.
// localparam type p0_t = if0.rq_t; - a typedef reached through an interface INSTANCE
// IEEE 1800-2017 A.2.2.1 data_type docs/systemverilog/2017/md/section-41-data-read-api.md:842 ([ class_scope | package_scope ] type_identifier only); IEEE 1800-2023 docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:825 the same
interface if_t;
  typedef logic [3:0] rq_t;
endinterface
module t;
  if_t if0();
  localparam type p0_t = if0.rq_t;
endmodule

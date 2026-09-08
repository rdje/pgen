// SV-CORPUS-GRAD.3.51 - a type named THROUGH an interface instance: `localparam type t = if0.rq_t;` (verilator t_lparam_assign_iface_typedef4.v, _nested.v, _nested6.v, t_param_type_from_iface_struct.v). IEEE 1800-2017 A.2.2.1 data_type ::= ... | [ class_scope | package_scope ] type_identifier { packed_dimension } | ... (docs/systemverilog/2017/md/section-41-data-read-api.md:842-866, the type_identifier alternative docs/systemverilog/2017/md/section-41-data-read-api.md:855) - a type is reached through a class scope or a package scope, never a hierarchical path; 1800-2023 the same (docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:825-849, :838).
// REJECTS at the `.` after the instance name on both SV profiles; the control `pkg::rq_t` ACCEPTS.
module m;
  localparam type t = if0.rq_t;
endmodule

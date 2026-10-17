// SV-CORPUS-GRAD.60 — CORRECT REJECTION under IEEE 1800-2017. A param_assignment takes `{ unpacked_dimension }` (A.2.4,
// docs/systemverilog/2017/md/section-41-data-read-api.md:1002-1005) and an unpacked_dimension is `[ constant_range ]` or
// `[ constant_expression ]` (:1047-1051) — there is no `[ ]` form. IEEE 1800-2023 reads `{ variable_dimension }` (A.2.4,
// docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:1042) whose unsized_dimension is `[ ]` (:1097-1101, :1106):
// an EDITION CONTRAST; the twin control_param_unsized_dimension_2023.sv holds that side. The corpus rows: OpenTitan's three
// rstmgr_env_pkg.sv (`:37`) and three flash_ctrl_env_pkg.sv (`:38`) copies. Expected on sv_2017 FOREVER: REJECT.
package p;
  parameter string L[] = {"a", "b"};
endpackage

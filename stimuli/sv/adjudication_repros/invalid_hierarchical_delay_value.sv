// SV-CORPUS-GRAD.3.51 - a HIERARCHICAL name as a delay value: `assign #sub.delay dly4 = dly1 + 1;` (verilator t_delay.v). IEEE 1800-2017 A.6.1 delay_value ::= unsigned_number | real_number | ps_identifier | time_literal | 1step (docs/systemverilog/2017/md/section-41-data-read-api.md:948-954) with A.9.3 ps_identifier ::= [ package_scope ] identifier (docs/systemverilog/2017/md/section-41-data-read-api.md:3404) - package-scoped, never hierarchical; 1800-2023 the same (docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:989-995, :3355); IEEE 1364-2005 A.6.1 delay_value ::= unsigned_number | real_number | identifier.
// REJECTS at the `.` after `sub` on every profile; the control `#pkg::delay` ACCEPTS on the SV profiles.
module m;
  logic a, b;
  assign #sub.delay a = b;
endmodule

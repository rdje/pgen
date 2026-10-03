// SV-CORPUS-GRAD.43 - a TOLERATED pin - text no IEEE 1800 edition derives, ACCEPTED under `sv_2023_tolerant` only; its strict twin must keep REJECTING.
// foreach (a[i][j]) - a bracket pair before the loop-variable bracket, read as a select
// no IEEE 1800 production derives it (the strict twin invalid_foreach_separate_brackets.sv carries the citations); admitted by DIRECTOR DECISION 2026-10-02, docs/decisions/project_sv_opt_in_tolerance_profile.md
module t;
  int a[2][3];
  initial foreach (a[i][j]) a[i][j] = 0;
endmodule

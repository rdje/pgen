// SV-CORPUS-GRAD.43 - a TOLERATED pin - text no IEEE 1800 edition derives, ACCEPTED under `sv_2023_tolerant` only; its strict twin must keep REJECTING.
// foreach (q[i][j]) n++; - the second strict pin of the bracket-pair spelling (.3.52), read as a select
// no IEEE 1800 production derives it (the strict twin invalid_foreach_bracket_per_variable.sv carries the citations); admitted by DIRECTOR DECISION 2026-10-02, docs/decisions/project_sv_opt_in_tolerance_profile.md
module m;
  int q[2][2];
  int n;
  initial foreach (q[i][j]) n++;
endmodule

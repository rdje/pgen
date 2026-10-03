// SV-CORPUS-GRAD.43 - a TOLERATED pin - text no IEEE 1800 edition derives, ACCEPTED under `sv_2023_tolerant` only; its strict twin must keep REJECTING.
// foreach (x[i]) { solve y before x[i]; } - a solve ... before directly inside a foreach constraint
// no IEEE 1800 production derives it (the strict twin invalid_solve_before_inside_foreach.sv carries the citations); admitted by DIRECTOR DECISION 2026-10-02, docs/decisions/project_sv_opt_in_tolerance_profile.md
class c;
  rand int x[2];
  rand int y;
  constraint k {
    foreach (x[i]) {
      solve y before x[i];
    }
  }
endclass

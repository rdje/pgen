// SV-CORPUS-GRAD.43 - a TOLERATED pin - text no IEEE 1800 edition derives, ACCEPTED under `sv_2023_tolerant` only; its strict twin must keep REJECTING.
// xc: cross cp_x, cfg.a; - a dotted name as a cross item
// no IEEE 1800 production derives it (the strict twin invalid_cross_item_hierarchical_name.sv carries the citations); admitted by DIRECTOR DECISION 2026-10-02, docs/decisions/project_sv_opt_in_tolerance_profile.md
class k;
  bit a;
endclass
module m;
  k cfg = new;
  bit x;
  covergroup cg;
    cp_x: coverpoint x;
    xc: cross cp_x, cfg.a;
  endgroup
endmodule

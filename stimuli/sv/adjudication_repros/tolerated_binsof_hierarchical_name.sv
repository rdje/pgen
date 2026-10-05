// SV-CORPUS-GRAD.45 - a TOLERATED pin - text no IEEE 1800 edition derives, ACCEPTED under sv_2023_tolerant only; its strict twin must keep REJECTING.
// bins hit = binsof(cfg.vif.a) intersect {1}; - a hierarchical name of three parts inside binsof(...)
// no IEEE 1800 production derives it (the strict twin invalid_binsof_hierarchical_name.sv carries the citations); admitted by a DELEGATED call 2026-10-05, docs/decisions/project_sv_opt_in_tolerance_profile.md (addendum)
class v;
  bit a;
endclass
class k;
  v vif = new;
endclass
module m;
  k cfg = new;
  bit x;
  covergroup cg;
    cp_x: coverpoint x;
    cp_a: coverpoint cfg.vif.a;
    xc: cross cp_x, cp_a {
      bins hit = binsof(cfg.vif.a) intersect {1};
    }
  endgroup
endmodule

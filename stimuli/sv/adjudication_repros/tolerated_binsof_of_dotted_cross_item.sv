// SV-CORPUS-GRAD.45 - a TOLERATED pin - text no IEEE 1800 edition derives, ACCEPTED under sv_2023_tolerant only; its strict twin must keep REJECTING.
// xc: cross cp_x, cfg.vif.a { bins hit = binsof(cp_x) ... && binsof(cfg.vif.a) ...; } - a dotted cross item and its reference inside binsof
// no IEEE 1800 production derives it (the strict twin invalid_binsof_of_dotted_cross_item.sv carries the citations); docs/decisions/project_sv_opt_in_tolerance_profile.md
class v;
  bit a;
  bit [1:0] w;
  function bit f(); return a; endfunction
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
    xc: cross cp_x, cfg.vif.a {
      bins hit = binsof(cp_x) intersect {0} && binsof(cfg.vif.a) intersect {1};
    }
  endgroup
endmodule

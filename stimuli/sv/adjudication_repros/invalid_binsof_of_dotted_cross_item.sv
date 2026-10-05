// SV-CORPUS-GRAD.45 - an INVALID pin - text neither IEEE 1800-2017 nor 1800-2023 derives; must keep REJECTING on the strict profiles.
// xc: cross cp_x, cfg.vif.a { bins hit = binsof(cp_x) ... && binsof(cfg.vif.a) ...; } - a dotted cross item and its reference inside binsof (sysrst_ctrl_in_out_inverted_vseq.sv:58-62)
// IEEE 1800-2017 19.6 docs/systemverilog/2017/txt/section-19-functional-coverage.txt:889-890 (the cross item) and 19.6.1 docs/systemverilog/2017/txt/section-19-functional-coverage.txt:924-926, :1048-1049 (the binsof operand)
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

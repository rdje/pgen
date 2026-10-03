// SV-CORPUS-GRAD.44 - a CONTROL pin - the two forms 19.6.1 names, must keep ACCEPTING on every profile.
// bins hit = binsof(cp_a) intersect {1}; bins one = binsof(cp_a.one); - a coverage point, and a coverage point bin (cover_point_identifier . bin_identifier)
// IEEE 1800-2017 19.6.1 docs/systemverilog/2017/txt/section-19-functional-coverage.txt:924-926, :1048-1049; IEEE 1800-2023 A.2.11 docs/systemverilog/2023/txt/section-Annex_A-normative-formal-syntax.txt:1069-1071
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
    cp_a: coverpoint cfg.vif.a { bins one = {1}; bins zero = {0}; }
    xc: cross cp_x, cp_a {
      bins hit = binsof(cp_a) intersect {1};
      bins one = binsof(cp_a.one);
    }
  endgroup
endmodule

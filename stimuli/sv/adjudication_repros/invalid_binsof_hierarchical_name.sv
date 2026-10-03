// SV-CORPUS-GRAD.44 - an INVALID pin - text neither IEEE 1800-2017 nor 1800-2023 derives; must keep REJECTING on every profile, the tolerant one included (not in its closed list).
// bins hit = binsof(cfg.vif.a) intersect {1}; - a HIERARCHICAL name (three parts) inside binsof(...)
// IEEE 1800-2017 19.6.1 Syntax 19-4 bins_expression ::= variable_identifier | cover_point_identifier [ . bin_identifier ] (docs/systemverilog/2017/txt/section-19-functional-coverage.txt:924-926) and "The binsof construct yields the bins of its expression, which can be either a coverage point ... or a coverage point bin" (:1048-1049); IEEE 1800-2023 A.2.11 docs/systemverilog/2023/txt/section-Annex_A-normative-formal-syntax.txt:1069-1071, 19.6.1 docs/systemverilog/2023/txt/section-19-functional-coverage.txt:1356-1357
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

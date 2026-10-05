// SV-CORPUS-GRAD.45 - an INVALID pin - the boundary of the tolerant profile's closed list; must keep REJECTING on every profile, sv_2023_tolerant included.
// bins hit = binsof(cfg.vif.w[0]) intersect {1}; - a hierarchical name with a TRAILING SELECT inside binsof(...)
// IEEE 1800-2017 19.6.1 Syntax 19-4 bins_expression ::= variable_identifier | cover_point_identifier [ . bin_identifier ] (docs/systemverilog/2017/txt/section-19-functional-coverage.txt:924-926), "either a coverage point ... or a coverage point bin" (:1048-1049); IEEE 1800-2023 A.2.11 docs/systemverilog/2023/txt/section-Annex_A-normative-formal-syntax.txt:1069-1071; the tolerant profile admits a hierarchical NAME, nothing else
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
    xc: cross cp_x, cp_a {
      bins hit = binsof(cfg.vif.w[0]) intersect {1};
    }
  endgroup
endmodule

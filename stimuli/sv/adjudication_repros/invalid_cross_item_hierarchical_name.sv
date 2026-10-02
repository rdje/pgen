// SV-CORPUS-GRAD.41 - an INVALID pin - text neither IEEE 1800-2017 nor 1800-2023 derives; must keep REJECTING on every profile.
// xc: cross cp_x, cfg.a; - a dotted (hierarchical) name as a cross item
// IEEE 1800-2017 19.6 docs/systemverilog/2017/txt/section-19-functional-coverage.txt:889-890 (Expressions cannot be used directly in a cross; a coverage point must be explicitly defined first) and cross_item ::= cover_point_identifier | variable_identifier (:901-903); IEEE 1800-2023 docs/systemverilog/2023/txt/section-19-functional-coverage.txt:1197, A.2.11 docs/systemverilog/2023/txt/section-Annex_A-normative-formal-syntax.txt:1046-1048
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

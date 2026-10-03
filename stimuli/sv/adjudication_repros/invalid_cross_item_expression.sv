// SV-CORPUS-GRAD.43 - an INVALID pin - the boundary of the tolerant profile's closed list; must keep REJECTING on every profile, `sv_2023_tolerant` included.
// xc: cross cp_x, y + 1; - an EXPRESSION as a cross item
// IEEE 1800-2017 19.6 docs/systemverilog/2017/txt/section-19-functional-coverage.txt:889-890; the tolerant profile admits a dotted NAME, not an expression
module m;
  bit x, y;
  covergroup cg;
    cp_x: coverpoint x;
    xc: cross cp_x, y + 1;
  endgroup
endmodule

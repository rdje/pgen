// SV-CORPUS-GRAD.41 - a CONTROL - the legal neighbour; must keep ACCEPTING on every profile.
// xc: cross cp_x, y; - a coverage point and a plain variable, the two forms 19.6 names
// IEEE 1800-2017 19.6 docs/systemverilog/2017/txt/section-19-functional-coverage.txt:886-890, :901-903
module m;
  bit x, y;
  covergroup cg;
    cp_x: coverpoint x;
    xc: cross cp_x, y;
  endgroup
endmodule

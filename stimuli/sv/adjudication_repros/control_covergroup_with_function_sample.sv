// SV-CORPUS-GRAD.13x.b - a CONTROL: the nearest LEGAL twin (one token different) - must keep ACCEPTING.
// covergroup cg with function sample(int x);
// IEEE 1800-2017 A.2.11 section-41-data-read-api.md:1520-1523. isolates the defect to the function name.
module t;
  covergroup cg with function sample(int x); endgroup
endmodule

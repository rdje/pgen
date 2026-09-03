// SV-CORPUS-GRAD.13x.b - a CONTROL: the nearest LEGAL twin (one token different) - must keep ACCEPTING.
// bins auto[size_var] = {[0:3]};
// IEEE 1800-2017 A.2.11 section-41-data-read-api.md:1553-1566. isolates the defect to the missing assignment.
module t; int x, size_var;
  covergroup cg; cp: coverpoint x { bins auto[size_var] = {[0:3]}; } endgroup
endmodule

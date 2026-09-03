// SV-CORPUS-GRAD.13x.b - an INVALID pin: text no Annex A production derives - must keep REJECTING.
// bins auto[size_var]; - a bins declaration with no = ...
// IEEE 1800-2017 A.2.11 bins_or_options: every bins arm continues with = section-41-data-read-api.md:1553-1566. t_covergroup_autobins_bad.v l.17.
module t; int x, size_var;
  covergroup cg; cp: coverpoint x { bins auto[size_var]; } endgroup
endmodule

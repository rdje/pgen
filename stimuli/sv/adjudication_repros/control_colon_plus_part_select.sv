// SV-CORPUS-GRAD.13x.b - a CONTROL: the nearest LEGAL twin (one token different) - must keep ACCEPTING.
// wire [1:0] z = r[2 :+ 1]; - the :+ the lint warns about is 2 : +1, a legal constant range
// IEEE 1800-2017 A.8.4 part_select_range / constant_range section-41-data-read-api.md:3160-3166. proves the t_lint_colonplus_bad.v defect is the port initializer, not the range.
module t; reg [3:0] r; wire [1:0] z = r[2 :+ 1];
endmodule

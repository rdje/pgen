// SV-CORPUS-GRAD.3.63 — the three-argument `$width` both editions derive (1364-2005 A.7.5.1 / 1800-2017 A.7.5.1):
// accepts on every profile before and after the slice; isolates the fix to the missing threshold.
module m(input clr);
  specify
    specparam tWidth = 10;
    $width(posedge clr, tWidth, 0);
  endspecify
endmodule

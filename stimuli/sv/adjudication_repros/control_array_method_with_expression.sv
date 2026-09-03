// SV-CORPUS-GRAD.13x.b - a CONTROL: the nearest LEGAL twin (one token different) - must keep ACCEPTING.
// total = q.sum() with (item > 0);
// IEEE 1800-2017 A.8.2 section-41-data-read-api.md:2954-2957. isolates the defect to the block after with.
module t; int q[$], total;
  initial total = q.sum() with (item > 0);
endmodule

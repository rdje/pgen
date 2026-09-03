// SV-CORPUS-GRAD.13x.b - a CONTROL: the nearest LEGAL twin (one token different) - must keep ACCEPTING.
// foreach (array[i]) $display(i);
// IEEE 1800-2017 A.6.8 section-41-data-read-api.md:2334. isolates the defects to the brackets and to the null body.
module t; int array[2];
  initial foreach (array[i]) $display(i);
endmodule

// SV-CORPUS-GRAD.13x.b - a CONTROL: the nearest LEGAL twin (one token different) - must keep ACCEPTING.
// dim2[1][0+:1] = 0; a[1][2][3:0] = b[1][2][3:0]; - the range last
// IEEE 1800-2017 A.8.4 section-41-data-read-api.md:3157-3159, section-41-data-read-api.md:3174-3176. isolates the defect to the select order.
module t; logic [3:0] dim2 [2]; logic [1:0][2:0][3:0] a, b;
  initial dim2[1][0+:1] = 0;
  assign a[1][2][3:0] = b[1][2][3:0];
endmodule

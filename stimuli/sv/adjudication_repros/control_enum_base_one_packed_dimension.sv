// SV-CORPUS-GRAD.13x.b - a CONTROL: the nearest LEGAL twin (one token different) - must keep ACCEPTING.
// typedef enum logic [1:0] { A, B } e_t;
// IEEE 1800-2017 A.2.2.1 section-41-data-read-api.md:868-871. isolates the defect to the second dimension.
module t;
  typedef enum logic [1:0] { A, B } e_t;
endmodule

// SV-CORPUS-GRAD.3.63 — the bare delayed terminals (`D`, `BL_X`) both editions derive; accepts on every
// profile before and after the slice; isolates the fix to the bracketed select.
module m(input A, B, C, D, BL, BL_X);
  specify
    $setuphold(posedge A, BL, 0, 0, C,,,D, BL_X);
  endspecify
endmodule

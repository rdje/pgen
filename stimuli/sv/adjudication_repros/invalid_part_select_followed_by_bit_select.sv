// SV-CORPUS-GRAD.13x.b - an INVALID pin: text no Annex A production derives - must keep REJECTING.
// dim2[0+:1][1] = 0; and a[1:0][2:0][3:0] - a range that is not the last select
// IEEE 1800-2017 A.8.4 select section-41-data-read-api.md:3157-3159 and constant_select section-41-data-read-api.md:3174-3176: bit_selects then ONE optional part_select_range. t_mem_multi_ref_bad.v l.19, t_mem_slice_bad.v l.36.
module t; logic [3:0] dim2 [2]; logic [1:0][2:0][3:0] a, b;
  initial dim2[0+:1][1] = 0;
  assign a[1:0][2:0][3:0] = b;
endmodule

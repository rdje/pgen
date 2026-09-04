// SV-CORPUS-GRAD.3.37 - an INVALID pin - must keep REJECTING on every profile.
// module t; automatic int x; - an automatic lifetime on a module-level data declaration
// IEEE 1800-2017 Annex A footnote 10 docs/systemverilog/2017/md/section-41-data-read-api.md:3503-3505: in a data_declaration that is not within a procedural context, it shall be illegal to use the automatic keyword
module t;
  automatic int x;
endmodule

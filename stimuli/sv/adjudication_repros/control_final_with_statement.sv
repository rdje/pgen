// SV-CORPUS-GRAD.13y - a CONTROL - must keep ACCEPTING.
// final $display("x"); / final begin end - a final construct with a function statement
// IEEE 1800-2017 A.6.2 final_construct docs/systemverilog/2017/md/section-41-data-read-api.md:2022
module t;
  final $display("x");
  final begin
  end
endmodule

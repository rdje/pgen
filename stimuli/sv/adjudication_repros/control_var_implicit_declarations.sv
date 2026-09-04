// SV-CORPUS-GRAD.3.35 - a CONTROL - must keep ACCEPTING.
// var x = 1; var [1:0] y; var signed z; - implicit data types WITH the var keyword
// IEEE 1800-2017 Annex A footnote 6 docs/systemverilog/2017/md/section-41-data-read-api.md:3504-3505 (the exception)
module top;
  var x = 1;
  var [1:0] y;
  var signed z;
  initial begin
    var w = 2;
  end
endmodule

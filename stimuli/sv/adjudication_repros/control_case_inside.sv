// SV-CORPUS-GRAD.13x.b - a CONTROL: the nearest LEGAL twin (one token different) - must keep ACCEPTING.
// case (1'bx) inside default: $stop; endcase
// IEEE 1800-2017 A.6.7 section-41-data-read-api.md:2228-2238. isolates the defect to the case keyword.
module t;
  initial begin
    case (1'bx) inside
      default: $stop;
    endcase
  end
endmodule

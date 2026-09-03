// SV-CORPUS-GRAD.13x.b - an INVALID pin: text no Annex A production derives - must keep REJECTING.
// casex (1'bx) inside default: $stop; endcase - the inside form with casex
// IEEE 1800-2017 A.6.7 case_statement: the inside arm is the literal keyword case section-41-data-read-api.md:2228-2238. t_case_inside_bad.v l.9.
module t;
  initial begin
    casex (1'bx) inside
      default: $stop;
    endcase
  end
endmodule

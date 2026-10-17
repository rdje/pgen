// SV-CORPUS-GRAD.58a — the control for invalid_delay_control_member_select.sv: the same delay through the standard's
// parenthesized form, `# ( mintypmax_expression )` (IEEE 1800-2017 A.6.5). Expected: ACCEPT.
module m;
  class c; int offset; endclass
  c s;
  initial begin
    #(s.offset);
  end
endmodule

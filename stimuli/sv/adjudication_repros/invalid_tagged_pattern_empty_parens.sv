// SV-CORPUS-GRAD.3.34 - an INVALID pin: `tagged JmpU ()` - empty parentheses where a pattern is required - must keep REJECTING on every profile.
// IEEE 1800-2023 A.6.7.1 pattern ::= ( pattern ) | ... docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:2273 - the parentheses enclose a pattern, never nothing; 1800-2017 docs/systemverilog/2017/md/section-41-data-read-api.md:2279-2286 has no parenthesised form at all.
module t;
  typedef union tagged { bit [9:0] JmpU; } Jmp;
  Jmp j; bit [9:0] pc;
  initial begin
    case (j) matches
      tagged JmpU () : pc = 1;
    endcase
  end
endmodule

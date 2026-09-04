// SV-CORPUS-GRAD.3.34 - a FIXED pin: the standard's own §12.6 example, a parenthesised nested tagged pattern - must keep ACCEPTING on every SV profile.
// `tagged Jmp (tagged JmpU .a)` - IEEE 1800-2017 §12.6.1 docs/systemverilog/2017/md/section-12-procedural-programming-statements.md:826, :828, :840 (the LRM's own example); 1800-2017 Annex A A.6.7.1 pattern docs/systemverilog/2017/md/section-41-data-read-api.md:2279-2286 omits `( pattern )` (an erratum) and 1800-2023 A.6.7.1 pattern ::= ( pattern ) | ... docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:2273-2281 adds it first.
module t;
  typedef union tagged { void Invalid; int Valid; } VInt;
  typedef union tagged { struct { bit [4:0] reg1, reg2, regd; } Add; union tagged { bit [9:0] JmpU; struct { bit [1:0] cc; bit [9:0] addr; } JmpC; } Jmp; } Instr;
  Instr instr; bit [9:0] pc, rf; bit [1:0] c; bit [9:0] a;
  initial begin
    case (instr) matches
      tagged Jmp (tagged JmpU .a) : pc = pc + a;
      tagged Jmp (tagged JmpC '{.c, .a}) : if (rf[c]) pc = a;
    endcase
  end
endmodule

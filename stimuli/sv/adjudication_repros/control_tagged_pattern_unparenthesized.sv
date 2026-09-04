// SV-CORPUS-GRAD.3.34 - a CONTROL: the same tagged patterns without parentheses, legal by the letter of both editions - must keep ACCEPTING, AST byte-identical.
// IEEE 1800-2017 A.6.7.1 pattern ::= ... | tagged member_identifier [ pattern ] | '{ pattern { , pattern } } docs/systemverilog/2017/md/section-41-data-read-api.md:2279-2286.
module t;
  typedef union tagged { bit [9:0] JmpU; struct { bit [1:0] cc; bit [9:0] addr; } JmpC; } Jmp;
  Jmp j; bit [9:0] pc, a; bit [1:0] c;
  initial begin
    case (j) matches
      tagged JmpU .a : pc = pc + a;
      tagged JmpC '{.c, .a} : pc = a;
    endcase
  end
endmodule

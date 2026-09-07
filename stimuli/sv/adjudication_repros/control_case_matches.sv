module m;
  typedef union tagged { void Invalid; int Valid; } u_t;
  u_t u;
  initial case (u) matches
    tagged Valid .n: ;
    default: ;
  endcase
endmodule

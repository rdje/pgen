// SV-CORPUS-GRAD.13x.c - an INVALID pin: text no Annex A production derives - must keep REJECTING.
// void'(1+2); - an expression where void'( ) takes a function_subroutine_call
// IEEE 1800-2017 A.6.9 subroutine_call_statement ::= subroutine_call ; | void ' ( function_subroutine_call ) ; section-41-data-read-api.md:2365-2367. iverilog ivtest/ivltests/sv_void_cast_fail3.v l.6.
module test;
  initial void'(1+2);
endmodule

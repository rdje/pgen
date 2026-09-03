// SV-CORPUS-GRAD.13x.c - a CONTROL: the nearest LEGAL twin (one token different) - must keep ACCEPTING.
// void'(f()); - a subroutine call inside void'( )
// IEEE 1800-2017 A.6.9 section-41-data-read-api.md:2365-2367. isolates the defect to the operand kind; sv_void_cast_fail1/2 (a void function, a task) are the same derivable shape.
module test; function int f(); return 1; endfunction
  initial void'(f());
endmodule

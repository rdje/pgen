// §30.7.1's own third line. `PATHPULSE$` is a legal simple_identifier (A.9.3 admits `$`),
// so this is an ORDINARY specparam_assignment and parsed BEFORE the fix too.
// Its `!pulse` arm is the RED control for the sibling rows' arm claims: if `pulse>general`
// could be satisfied by the ordinary route, this row would fail.
module m;
  specparam PATHPULSE$ = 3;
endmodule

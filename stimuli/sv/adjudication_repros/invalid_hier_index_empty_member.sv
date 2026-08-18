// A.9.3 spells the hierarchy `{ identifier [ [ constant_expression ] ] . } identifier` — every
// `.` is followed by an identifier, so an EMPTY component between two dots has no derivation in
// 1364-2005 nor in 1800-2017/2023. This is the over-acceptance guard for the revived
// `split_hierarchical_callable_receiver` loop: the new lookahead admits `X . Y[sel] .` only when
// a further component follows, and this row proves it does not admit nothing at all.
// Expected FOREVER: REJECT (three profiles).
module sub; task t; begin end endtask endmodule
module top; sub u1[1:0](); initial begin top.u1[0]..t; end endmodule

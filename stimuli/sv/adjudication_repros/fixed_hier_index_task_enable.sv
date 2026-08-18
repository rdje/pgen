// SV-CORPUS-GRAD.13c.2c — an INDEXED component in the middle of a hierarchical name, in a CALL
// position. IEEE 1364-2005 A.9.3 / §12.4:
//   hierarchical_identifier ::= { identifier [ [ constant_expression ] ] . } identifier
// and A.6.9 `task_enable ::= hierarchical_task_identifier [ ( expression { , expression } ) ] ;`
// so `top.u1[0].t;` is legal in PLAIN VERILOG — as it is in IEEE 1800-2017/2023 A.9.3, whose
// `hierarchical_identifier ::= [ $root . ] { identifier constant_bit_select . } identifier` is the
// same shape. The rule that rejected it (`split_hierarchical_callable_receiver`) carries no
// `@profiles` gate, so this reproducer binds on all three profiles.
// Expected before the fix: REJECT (three profiles).  After: ACCEPT (three profiles).
module sub; task t; begin end endtask endmodule
module top; sub u1[1:0](); initial begin top.u1[0].t; end endmodule

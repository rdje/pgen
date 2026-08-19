// SV-CORPUS-GRAD.13c.2s — FIXED in PGEN-SV-CORPUS-GRAD-0242 (release 1.0.191, ledger SV-0064).
// IEEE 1800-2017 A.2.10: sequence_list_of_arguments ::= [ sequence_actual_arg ]
// { , [ sequence_actual_arg ] } { , . identifier ( [ sequence_actual_arg ] ) } — ordered arguments
// THEN named ones. PGEN spelled the middle group as a bare `( comma ( sequence_actual_arg )? )*`,
// so the greedy star ate the comma introducing the first NAMED argument and the named-argument star
// then needed a comma and saw `.`.
//
// ⛔ THE POSITIONAL ARGUMENT HERE IS A SEQUENCE (`a ##1 b`), WHICH IS THE WHOLE POINT. The general
// `list_of_arguments` route rescues a mixed list only while every positional argument is ALSO a
// plain expression; `a ##1 b` is not, so before the fix BOTH routes failed and the construct was
// unparseable. Its control is `control_sequence_instance_positional_args.sv`, the same call with
// the named argument removed, which parsed on both sides.
// Expected FOREVER: ACCEPT on sv_2017 + sv_2023.
module m;
  logic clk, a, b, d;
  sequence sq(p, q); p ##1 q; endsequence
  assert property (@(posedge clk) sq(a ##1 b, .q(d)));
endmodule

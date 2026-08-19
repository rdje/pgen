// SV-CORPUS-GRAD.13c.2s — FIXED in PGEN-SV-CORPUS-GRAD-0242 (release 1.0.191, ledger SV-0064).
// The property sibling of `fixed_sequence_instance_mixed_args.sv`: IEEE 1800-2017 A.2.10 gives
// property_list_of_arguments the same "ordered then named" shape, PGEN spelled it with the same
// starving star, and the positional argument here (`a ##1 b`) is one only property_actual_arg can
// parse — so the general list_of_arguments route could not rescue it either.
// Expected FOREVER: ACCEPT on sv_2017 + sv_2023.
module m;
  logic clk, a, b, d;
  property pr(p, q); p |-> q; endproperty
  assert property (@(posedge clk) pr(a ##1 b, .q(d)));
endmodule

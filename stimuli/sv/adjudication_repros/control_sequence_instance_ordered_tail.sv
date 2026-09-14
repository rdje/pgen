// SV-CORPUS-GRAD.3.61 - the sequence twin of control_property_instance_ordered_tail.sv: `c ##1 d` in the ordered tail.
module m;
  assert property (s(a, c ##1 d, .q(e)) |-> 1);
endmodule

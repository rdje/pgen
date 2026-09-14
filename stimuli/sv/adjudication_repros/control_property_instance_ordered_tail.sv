// SV-CORPUS-GRAD.3.61 - a property instance with a positional argument AFTER the first and a named one last:
// `( comma !dot ( property_actual_arg )? )*` sliced its `!dot` guard (empty) into ordered_tail from 1.0.191 to 1.0.229,
// dropping `c |-> d`. The implication is reachable only through property_list_of_arguments (a tf call refuses `|->`).
module m;
  assert property (p(a, c |-> d, .q(e)));
endmodule

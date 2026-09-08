// SV-CORPUS-GRAD.3.52 - the twin of invalid_event_control_indexed_identifier.sv as an event_expression: ACCEPT.
module m;
  event x[2];
  int y;
  initial @(x[y]);
endmodule

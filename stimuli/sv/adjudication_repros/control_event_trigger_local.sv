// SV-CORPUS-GRAD.3.51 - the twin of invalid_package_scoped_event_trigger.sv with a local event: ACCEPT on every profile.
module m;
  event e;
  initial #1 ->e;
endmodule

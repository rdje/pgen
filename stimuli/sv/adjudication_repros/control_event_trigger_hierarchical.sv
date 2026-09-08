// SV-CORPUS-GRAD.3.52 - the twin of invalid_event_trigger_on_method_call.sv on a hierarchical event: ACCEPT.
class B;
  event e;
endclass
module m;
  B b;
  initial ->b.e;
endmodule

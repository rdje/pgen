// SV-CORPUS-GRAD.60 — the trigger both editions derive — a hierarchical_event_identifier with no select. The control for
// invalid_event_trigger_select_2017.sv.
module m;
  event ev;
  initial begin
    -> ev;
  end
endmodule

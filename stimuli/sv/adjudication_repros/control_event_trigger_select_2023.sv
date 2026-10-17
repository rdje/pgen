// SV-CORPUS-GRAD.60 — the SAME text under IEEE 1800-2023, where A.6.5 reads event_trigger ::= -> hierarchical_event_identifier nonrange_select ;
// (docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:2189-2192; 15.5 :293). The 2023 side of
// invalid_event_trigger_select_2017.sv. Expected: ACCEPT.
module m;
  event ev[2];
  initial begin
    -> ev[0];
  end
endmodule

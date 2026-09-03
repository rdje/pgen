// SV-CORPUS-GRAD.14b - a CONTROL under the profile it names - must keep ACCEPTING there.
// ->ev[index]; - an event trigger with a select on the event, judged as IEEE 1800-2023
// IEEE 1800-2023 A.6.5 event_trigger ::= -> hierarchical_event_identifier nonrange_select ; docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:2189-2192, nonrange_select docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:3115. the edition contrast: byte-identical text to the sv_2017 invalid pin, legal in 2023.
module t;
  event ev[2];
  task trig(int index);
    ->ev[index];
  endtask
endmodule

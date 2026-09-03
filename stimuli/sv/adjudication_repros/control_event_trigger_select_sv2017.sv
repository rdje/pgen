// SV-CORPUS-GRAD.14b - a CONTROL: the 2017 spelling of the same construct, legal in both editions - must keep ACCEPTING.
// ->ev[index]; - an event trigger with a select on the event - the 2017 spelling
// IEEE 1800-2017 A.6.5 event_trigger ::= -> hierarchical_event_identifier ; docs/systemverilog/2017/md/section-41-data-read-api.md:2191-2195 with hierarchical_event_identifier ::= hierarchical_identifier docs/systemverilog/2017/md/section-41-data-read-api.md:3360, whose final identifier carries no select ({ identifier constant_bit_select . } identifier, docs/systemverilog/2017/md/section-41-data-read-api.md:3361) - measured: the 2017 path parses `[index]` inside the loop and fails at `;` where a `.` is required.
module t;
  event e;
  task trig();
    ->e;
  endtask
endmodule

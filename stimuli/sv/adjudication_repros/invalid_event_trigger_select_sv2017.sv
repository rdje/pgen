// SV-CORPUS-GRAD.14b - an INVALID pin under the profile it names: text the 1800-2017 edition derives nothing for - must keep REJECTING there.
// ->ev[index]; - an event trigger with a select on the event, judged as IEEE 1800-2017
// IEEE 1800-2023 A.6.5 event_trigger ::= -> hierarchical_event_identifier nonrange_select ; docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:2189-2192, nonrange_select docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:3115 (2023 only); IEEE 1800-2017 A.6.5 event_trigger ::= -> hierarchical_event_identifier ; docs/systemverilog/2017/md/section-41-data-read-api.md:2191-2195 with hierarchical_event_identifier ::= hierarchical_identifier docs/systemverilog/2017/md/section-41-data-read-api.md:3360, whose final identifier carries no select ({ identifier constant_bit_select . } identifier, docs/systemverilog/2017/md/section-41-data-read-api.md:3361) - measured: the 2017 path parses `[index]` inside the loop and fails at `;` where a `.` is required. verilator t_array_event.v l.15; the same bytes are the sv_2023 control beside this row.
module t;
  event ev[2];
  task trig(int index);
    ->ev[index];
  endtask
endmodule

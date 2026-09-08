// SV-CORPUS-GRAD.3.52 - a method CALL as an event trigger target: `->b.get_event();` (verilator t_event_control_pass.v:21). IEEE 1800-2017 A.6.5 event_trigger ::= -> hierarchical_event_identifier ; | ->> [ delay_or_event_control ] hierarchical_event_identifier ; (docs/systemverilog/2017/md/section-41-data-read-api.md:2191-2194) with A.9.3 hierarchical_identifier (docs/systemverilog/2017/md/section-41-data-read-api.md:3361) - no call; 1800-2023 the same with a trailing nonrange_select (docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:2189-2192), still no call.
// REJECTS at the `(` on both SV profiles; the control `->b.e;` ACCEPTS.
class B;
  event e;
  function event get_event(); return e; endfunction
endclass
module m;
  B b;
  initial ->b.get_event();
endmodule

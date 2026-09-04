// SV-CORPUS-GRAD.3.35 - a CONTROL - must keep ACCEPTING.
// event e = (a, b) - a parenthesised comma-separated event list as a port default stays an event_expression_primary
// IEEE 1800-2017 A.6.5 event_expression ::= ... | ( event_expression ) docs/systemverilog/2017/md/section-41-data-read-api.md:2197-2204
module t;
  logic a, b, c;
  property p(event e = (a, b));
    @e c;
  endproperty
  assert property (p((a, b)));
endmodule

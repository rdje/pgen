// SV-CORPUS-GRAD.3.35 - a FIXED pin - must keep ACCEPTING.
// clk = $inferred_clock, rst = $inferred_disable / string msg = "violation", coverage_level clevel = cover_all - a port default followed by the next port
// IEEE 1800-2017 A.2.10 property_port_item ::= ... [ = property_actual_arg ] docs/systemverilog/2017/md/section-41-data-read-api.md:1300-1310; A.1.8 checker_port_item docs/systemverilog/2017/md/section-41-data-read-api.md:450-460; the comma is the port list's, not an event_expression's (A.6.5 event_expression ::= ... | event_expression , event_expression docs/systemverilog/2017/md/section-41-data-read-api.md:2197-2204)
typedef enum {cover_none, cover_all} coverage_level;
checker c(event clk, string msg = "violation", coverage_level clevel = cover_all);
endchecker
module t;
  logic clk, rst, start_event, form;
  property p_triggers(start_event, form, event clk_e = $inferred_clock, rst = $inferred_disable);
    @clk_e disable iff (rst) start_event |=> form;
  endproperty
  assert property (p_triggers(start_event, form, posedge clk, rst));
endmodule

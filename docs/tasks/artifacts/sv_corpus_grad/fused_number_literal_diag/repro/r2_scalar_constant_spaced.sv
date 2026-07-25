// DEFECT (shape-degraded at baseline): §5.7.1-spaced `1 'b 1` scalar_constant.
// The file still ACCEPTS via the fallback `expression` branch, but the typed
// `{kind:"eq", rhs:<scalar_constant>}` shape is lost.
module m (input clk, input d, input cond);
  specify
    $setup(posedge clk &&& cond == 1 'b 1, d, 10);
  endspecify
endmodule

// SV-CORPUS-GRAD.3.49 - the two-sided twin under IEEE 1364-2005: a checker instantiation is SystemVerilog-only (IEEE 1364-2005 Annex A has no checker, no concurrent assertion item and no sequence syntax - A.1.4 module_item / A.6.x), so the forward-instantiation control must REJECT under verilog_2005 whatever the gate does: the sequence actual `a[*1:$] ##1 b` is not a 1364-2005 expression.
module top;
  reg a, b, clk;
  my_checker check(@(posedge clk) a[*1:$] ##1 b);
endmodule

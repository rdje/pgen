// Control for `fixed_hier_index_task_enable.sv`: the SAME hierarchical task enable with the
// bit-select REMOVED. It parsed before the fix and after, so the flip is attributable to the
// indexed middle component alone — not to hierarchical task enables, nor to the instance array
// declaration (which this file still carries).
module sub; task t; begin end endtask endmodule
module top; sub u1[1:0](); initial begin top.u1.t; end endmodule

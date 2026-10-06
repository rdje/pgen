// SVPP-EXPANSION.5(t) - a CONTROL pin (preprocess=1) - a directive name is PREDEFINED to the conditionals, so a guarded redefinition is skipped, not executed; must keep ACCEPTING on the SV profiles (the ispras ieee-1800-2012/22/22.13_01.sv shape, keyed must_accept).
// IEEE 1800-2017 22.5.1 "All compiler directives shall be considered predefined macro names" (docs/systemverilog/2017/md/section-22-compiler-directives.md:139-140): `ifndef __FILE__ is FALSE. Before this a directive name was "not defined" to `ifdef / `ifndef and the guard let the redefinition run.
`ifndef __FILE__
  `define __FILE__ "unknown"
`endif
`ifdef define
module define_is_predefined; endmodule
`endif
module top; initial $display("at %s, line %d.", `__FILE__, `__LINE__); endmodule

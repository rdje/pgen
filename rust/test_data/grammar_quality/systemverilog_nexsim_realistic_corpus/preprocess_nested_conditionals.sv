`define OUTER_CFG
`ifdef OUTER_CFG
  `define INNER_CFG
  `ifdef INNER_CFG
module nested_conditionals_true;
endmodule
  `else
module nested_conditionals_inner_false;
endmodule
  `endif
`else
module nested_conditionals_outer_false;
endmodule
`endif

`define TEMP_GUARD
`undef TEMP_GUARD
`ifdef TEMP_GUARD
module preprocess_macro_define_undef_guard_true;
endmodule
`else
module preprocess_macro_define_undef_guard_false;
endmodule
`endif

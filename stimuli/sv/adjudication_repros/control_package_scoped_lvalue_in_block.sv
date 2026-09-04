// SV-CORPUS-GRAD.3.37 - a CONTROL - must keep ACCEPTING.
// P::Q = 1; in an initial block - A.8.5 variable_lvalue admits [ package_scope ] hierarchical_variable_identifier
// IEEE 1800-2017 A.8.5 variable_lvalue ::= [ implicit_class_handle . | package_scope ] hierarchical_variable_identifier select docs/systemverilog/2017/md/section-41-data-read-api.md:3199-3202
module top;
  initial begin
    P::Q = 1;
  end
endmodule

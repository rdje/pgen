// SV-CORPUS-GRAD.13x.b - an INVALID pin: text no Annex A production derives - must keep REJECTING.
// covergroup cg_bad with function foo(int x); - a sampling function not named sample
// IEEE 1800-2017 A.2.11 coverage_event ::= ... | with function sample ( [ tf_port_list ] ) section-41-data-read-api.md:1520-1523. t_covergroup_with_function_foo_bad.v l.8.
module t;
  covergroup cg_bad with function foo(int x); endgroup
endmodule

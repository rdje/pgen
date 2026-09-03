// SV-CORPUS-GRAD.13x.c - an INVALID pin: text no Annex A production derives - must keep REJECTING.
// $fatal(.x("x")); at module level - a named argument where finish_number is required
// IEEE 1800-2017 A.1.4 module_common_item -> elaboration_system_task ::= $fatal [ ( finish_number [ , list_of_arguments ] ) ] ; section-41-data-read-api.md:301-309, finish_number ::= 0 | 1 | 2. sv2v test/error/severity_task_arg.sv l.4; the .8b.2 pin's A.8.2 reading applies inside procedural code only.
module top;
  $fatal(.x("x"));
endmodule

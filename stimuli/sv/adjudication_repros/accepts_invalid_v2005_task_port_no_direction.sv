// SV-CORPUS-GRAD.13e.3(b) — an OVER-ACCEPTANCE under verilog_2005, expected ACCEPT until fixed.
// A task port item MUST carry its direction in IEEE 1364-2005. A.2.7
// (docs/verilog/2005/txt/section-Annex_A-normative-formal-syntax-definition.txt:290) is
//   task_port_item ::= {attribute_instance} tf_input_declaration
//                    | {attribute_instance} tf_output_declaration
//                    | {attribute_instance} tf_inout_declaration
// and each of those (:300/:303/:306) BEGINS with its `input` / `output` / `inout` keyword — there
// is no directionless alternative. `task t(a, b);` therefore has no derivation, and iverilog says
// so from its own grammar action: `error: Missing task/function port direction.` (parse.y).
// ⛔ IEEE 1800 A.2.7 makes the direction OPTIONAL, so this is a verilog_2005-ONLY defect and the
// fix must be profile-gated (the `scope_randomize_sv_only` idiom), not an inline tightening.
// Expected: ACCEPT today (the defect). FLIPS TO REJECT — and this row to `class=invalid` — when
// SV-CORPUS-GRAD.13e.5 lands. Owner of the corpus rows: ivtest br1027a / br1027c.
module test();
  task t(a, b);
    $display(a,,b);
  endtask
  initial t(0, 1);
endmodule

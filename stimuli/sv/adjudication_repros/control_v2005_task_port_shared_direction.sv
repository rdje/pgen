// SV-CORPUS-GRAD.13e.5 — the control that REFUTED THE FIRST FIX, kept so it cannot happen twice.
// `task t(input integer a, b);` is LEGAL IEEE 1364-2005: A.2.7's
// `tf_input_declaration ::= input task_port_type list_of_port_identifiers`
// (docs/verilog/2005/txt/section-Annex_A-normative-formal-syntax-definition.txt:300) takes a LIST
// of names, so ONE `input integer` covers both `a` and `b`. PGEN renders a task port list the IEEE
// 1800 way — one `tf_port_item` per comma — so a continuation name arrives as its own item with no
// direction, and the obvious v2005 fix ("every item must carry a direction") REJECTS this file.
// ⛔ That first fix was written, regenerated and MEASURED rejecting this text before it shipped;
// the corrected shape admits a directionless item only when it is a BARE port identifier — no data
// type, no `var` — which is exactly a `list_of_port_identifiers` continuation, and requires the
// FIRST item of the list to carry a direction.
// Expected FOREVER: ACCEPT on verilog_2005. It is the half of the construct the fix must not touch.
module test();
  task t(input integer a, b);
    $display(a,,b);
  endtask
  initial t(0, 1);
endmodule

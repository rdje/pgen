// SV-CORPUS-GRAD.13e.3(b) — the SECOND shape of the same over-acceptance, expected ACCEPT until
// fixed. Here the FIRST item is well formed and the second drops the direction:
// `task t(input integer a, integer b);`. It matters as a distinct row because
// `tf_input_declaration ::= input task_port_type list_of_port_identifiers` DOES let one `input`
// cover several names — `task t(input integer a, b);` is legal — so the defect is specifically
// that a NEW `task_port_item` may start without a direction, not that a name may lack one.
// IEEE 1364-2005 A.2.7:290/:300; iverilog: `error: Missing task/function port direction.`
// Expected: ACCEPT today. FLIPS TO REJECT when SV-CORPUS-GRAD.13e.5 lands.
// Owner of the corpus row: ivtest br1027e.
module test();
  task t(input integer a, integer b);
    $display(a,,b);
  endtask
  initial t(0, 1);
endmodule

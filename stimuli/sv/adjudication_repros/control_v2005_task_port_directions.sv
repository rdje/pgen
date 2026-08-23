// SV-CORPUS-GRAD.13e.3(b) — the one-difference ACCEPT control for the two task-port rows above.
// The same task with a direction on every item, which is exactly IEEE 1364-2005 A.2.7's
// `task_port_item ::= {attribute_instance} tf_input_declaration` twice over. It parses today and
// must STILL parse after 13e.5 tightens the rule — that is what makes this a control rather than
// a duplicate: it is the half of the construct the fix must not touch.
module test();
  task t(input integer a, input integer b);
    $display(a,,b);
  endtask
  initial t(0, 1);
endmodule

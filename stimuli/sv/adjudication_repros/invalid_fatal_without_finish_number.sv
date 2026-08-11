// SV-CORPUS-GRAD.13c.2 — CORRECT REJECTION. IEEE 1800-2017 A.9.3:
//   elaboration_system_task ::= $fatal [ ( finish_number [ , list_of_arguments ] ) ] ;
//   finish_number ::= 0 | 1 | 2
// so `$fatal` takes a finish_number FIRST; `$error`/`$warning`/`$info` do not.
// ⛔ Accepting this would be an OVER-ACCEPTANCE defect. Expected FOREVER: REJECT.
module m;
  if (1) begin
    $fatal("boom");
  end
endmodule

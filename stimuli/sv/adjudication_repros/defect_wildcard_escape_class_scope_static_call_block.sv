// SV-CORPUS-GRAD.30 - a DEFECT pin (REJECT through 1.0.242, both SV profiles) - FIXED by SV-CORPUS-GRAD.31 (release 1.0.243: class_scope's ordered escape alternative + the class_scoped_call_prefix escape arm; class fixed / ACCEPT).
// The same class-scoped static call, and a static member assignment through the scope, as statements of a module's
// initial block under an open wildcard import.
import q::*;
module m;
  initial begin
    uvm_config_db#(int)::set(null, "a", "b", 1);
  end
endmodule

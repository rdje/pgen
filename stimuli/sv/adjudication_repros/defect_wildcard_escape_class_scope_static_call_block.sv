// SV-CORPUS-GRAD.30 - a DEFECT pin (REJECT today, both SV profiles) - routed to SV-CORPUS-GRAD.31.
// The same class-scoped static call, and a static member assignment through the scope, as statements of a module's
// initial block under an open wildcard import.
import q::*;
module m;
  initial begin
    uvm_config_db#(int)::set(null, "a", "b", 1);
  end
endmodule

// ENGINE-UNIVERSAL-SERVICES.111 - FIXED in release 1.0.246 (was a defect pin: REJECT through 1.0.245) - now class fixed / ACCEPT on both SV profiles.
// SV-CORPUS-GRAD.28 - a DEFECT pin (REJECT today) - routed to ENGINE-UNIVERSAL-SERVICES.111.
// A class-typed member at MODULE scope whose parameterized class is unresolved under an open wildcard import (IEEE 1800-2017 A.1.9
// class_type, 26.3): the .28 escape is reached from class bodies and typedefs only, because at module scope `ident #(` is also an
// instantiation head and the class route's parameter list, parsed in a branch that rolls back, is not answered from the memo on
// the instantiation route (measured: entries +5.18 % over the pinned sample). Flips to fixed when .111 lands.
import q::*;
module m;
  uvm_tlm_fifo#(int) fifo;
endmodule

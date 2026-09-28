// ENGINE-UNIVERSAL-SERVICES.111 - FIXED in release 1.0.246 (was a defect pin: REJECT through 1.0.245) - now class fixed / ACCEPT on both SV profiles.
// SV-CORPUS-GRAD.31 - a DEFECT pin (REJECT on both SV profiles) - ROUTED to ENGINE-UNIVERSAL-SERVICES.111, like the .28
// module-scope member: a class-scoped TYPE at MODULE scope whose class is unresolved under an open wildcard import. The .31
// escape is deliberately NOT reached from the type-position prefixes: at a module item's start `ident #( ... )` is also an
// instantiation head, and an escape that consumes the parameter list there re-parses it on the instantiation route after the
// rollback (the .28 first candidate, entries +5.18 %). A memoized success surviving a no-fact rollback lets the escape be
// admitted here at no cost. Inside class bodies and blocks the same type parses (the .28 / .30 escapes).
import q::*;
module m;
  C#(int)::width_t x;
endmodule

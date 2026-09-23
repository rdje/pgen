// SV-CORPUS-GRAD.30 - an IDENTITY control (ACCEPT before and after 1.0.242): a type PARAMETER with a parameter list is
// admitted by the known type-parameter arm on both sides (a type parameter may be bound to a parameterized class), so the
// typedef exclusion added to the escape head must not touch it.
import q::*;
class c #(type T = int);
  T #(int) x;
endclass

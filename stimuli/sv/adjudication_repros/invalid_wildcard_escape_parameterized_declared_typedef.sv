// SV-CORPUS-GRAD.30 - an INVALID pin (REJECT from 1.0.242 on both SV profiles; ACCEPTED by 1.0.240 / 1.0.241 through the .28
// escape): a name the unit DECLARES as a typedef is provably not a class - IEEE 1800-2017 26.3: a local declaration takes
// precedence over a wildcard-imported one - so `T #(int)` is illegal with or without the import (the same text without the
// import is REJECTED on every release). The escape head now carries known_unscoped_block_class_type's typedef exclusion.
import q::*;
typedef int T;
class c;
  T #(int) x;
endclass

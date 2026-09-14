// SV-CORPUS-GRAD.3.61 - a type parameter's DEFAULT TYPE and the SECOND assignment of a type list. Through 1.0.229
// type_assignment bound `name` to its leading `!kw_type` lookahead (empty) and `value` to the identifier, so `int`
// vanished; and the .3.35 list sliced the guard's comma in place of `U = real`. The arms pin both: an integer_atom
// (the bound default `int`) and a non_integer (the `real` of the second assignment surviving the slice) - the leaves,
// because the enclosing kind is `type` on sv_2017 and `type_parameter` on sv_2023.
module m #(parameter type T = int, U = real) ();
  localparam type L = int, M = real;
endmodule
class c #(type T = int, U = real);
endclass

// SV-CORPUS-GRAD.13e.4 — the half of the construct the fix must NOT touch.
// IEEE 1364-2005 A.2.1.3 DOES derive both of these:
//   integer_declaration ::= integer list_of_variable_identifiers ;   (…Annex_A…txt:152)
//   time_declaration    ::= time list_of_variable_identifiers ;      (…Annex_A…txt:174)
// The .13e.4 gate is on the optional `signing` beside `integer_atom_type`, never on
// `integer_atom_type` itself, so these must still parse under verilog_2005 forever. A fix that
// gated the atom rule instead of the optional would go RED here.
// Expected FOREVER: ACCEPT on verilog_2005.
module test;
  integer i;
  time t;
  integer j = 1;
  initial begin
    i = 0;
    t = 0;
  end
endmodule

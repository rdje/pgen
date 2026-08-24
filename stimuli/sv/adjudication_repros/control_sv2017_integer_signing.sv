// SV-CORPUS-GRAD.13e.4 — the OTHER SIDE of the profile gate, and byte-for-byte the same
// declaration as accepts_invalid_v2005_integer_signing.sv. The two rows differ ONLY in the
// profile they bind on, which is exactly the claim the fix makes.
// IEEE 1800-2017 A.2.2.1 `data_type ::= integer_atom_type [ signing ] | …` DOES derive this, so
// gating `signing` for verilog_2005 must leave sv_2017 / sv_2023 untouched — in the accept set
// AND in the typed AST, which is what the `arm` claim pins.
// Expected FOREVER: ACCEPT on sv_2017 and sv_2023, through integer_atom>unsigned.
module test;
  integer unsigned u;
  initial u = 0;
endmodule

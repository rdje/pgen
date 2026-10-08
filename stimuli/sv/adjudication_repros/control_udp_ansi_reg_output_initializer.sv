// SV-CORPUS-GRAD.13e.14 — the control for the two invalid_udp_*initial* rows: an ANSI `output reg` with an
// initializer and a three-field sequential table. ACCEPT on every profile.
primitive p (output reg o = 1'b0, input i);
  table
    0 : 0 : 0;
    1 : 0 : 1;
  endtable
endprimitive

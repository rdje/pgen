// SV-CORPUS-GRAD.13e.2 — the one-difference ACCEPT control for invalid_v2005_udp_empty_table.sv.
// The identical primitive with two combinational_entry rows in its table, which is exactly
// A.5.3's `table combinational_entry { combinational_entry } endtable`. It parses, so the
// rejection above is attributable to the EMPTINESS of the table alone and not to the primitive
// header, the port declarations, or the table keywords.
primitive p (o, i);
  output o;
  input i;
  table
    0 : 1;
    1 : 0;
  endtable
endprimitive

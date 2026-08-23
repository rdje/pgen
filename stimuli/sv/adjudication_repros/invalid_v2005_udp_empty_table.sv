// SV-CORPUS-GRAD.13e.2 — REJECT FOREVER under verilog_2005.
// A UDP table must hold at least ONE entry. IEEE 1364-2005 A.5.3
// (…/section-Annex_A-normative-formal-syntax-definition.txt:483/:485) is
//   combinational_body ::= table combinational_entry { combinational_entry } endtable
//   sequential_body    ::= [ udp_initial_statement ] table sequential_entry
//                             { sequential_entry } endtable
// — the first entry is mandatory in both, so `table endtable` has no derivation at all.
// The upstream row ivtest/ivltests/udp_empty_table_fail.v agrees in both of its own artifacts:
// its header comment reads "Check that an empty old-style UDP table generates an error" and its
// golden is "error: Empty UDP table."
// Expected FOREVER: REJECT on verilog_2005.
primitive p (o, i);
  output o;
  input i;
  table
  endtable
endprimitive

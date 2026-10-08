// SV-CORPUS-GRAD.13e.14 — REJECT FOREVER on every profile.
// An old-style UDP with `initial o = 1'b0;` and a two-field table. IEEE 1364-2005 A.5.3 udp_body ::=
// combinational_body | sequential_body, sequential_body ::= [ udp_initial_statement ] table sequential_entry
// { sequential_entry } endtable, sequential_entry ::= seq_input_list : current_state : next_state ;
// (docs/verilog/2005/txt/section-Annex_A-normative-formal-syntax-definition.txt:482-494): the initial statement
// makes the body sequential, and `0 : 0;` is not a three-field sequential_entry. IEEE 1800-2017 A.5.3 reads the
// same. iverilog 13.0 asserts (pform.cc:2008) on the row instead of refusing it.
// Corpus row: ivtest/ivltests/udp_initial_nonreg_fail.v. Control: control_udp_ansi_reg_output_initializer.sv.
primitive p (o, i);
  output o;
  input i;
  initial o = 1'b0;
  table
    0 : 0;
    1 : 1;
  endtable
endprimitive

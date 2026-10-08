// SV-CORPUS-GRAD.13e.14 — REJECT FOREVER on every profile.
// `primitive p (output o = 1'b0, input i);` — an ANSI UDP output with an initializer and no `reg`. IEEE 1364-2005
// A.5.1 udp_output_declaration ::= { attribute_instance } output port_identifier | { attribute_instance } output reg
// port_identifier [ = constant_expression ] (docs/verilog/2005/txt/section-Annex_A-normative-formal-syntax-definition.txt:476-478):
// only the `reg` form carries an initializer. IEEE 1800-2017 A.5.1 reads the same. iverilog 13.0 asserts
// (pform.cc:2089) on the row instead of refusing it, so the suite's CE carries no usable message.
// Corpus row: ivtest/ivltests/udp_ansi_initial_nonreg_fail.v. Control: control_udp_ansi_reg_output_initializer.sv.
primitive p (output o = 1'b0, input i);
  table
    0 : 0;
    1 : 1;
  endtable
endprimitive

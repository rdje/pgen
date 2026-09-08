// SV-CORPUS-GRAD.3.51 - a parameter value assignment WITHOUT parentheses: `m1 #PAR m1();` / `mreal #1.2 mr ();` (verilator t_param_no_parentheses.v, t_param.v). IEEE 1800-2017 A.4.1.1 parameter_value_assignment ::= # ( [ list_of_parameter_assignments ] ) (docs/systemverilog/2017/md/section-41-data-read-api.md:1795); 1800-2023 A.4.1.1 (docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:1804); IEEE 1364-2005 A.4.1 parameter_value_assignment ::= # ( list_of_parameter_assignments ) - the parentheses are the production, a bare `#PAR` is a delay control no instantiation admits.
// The parser REJECTS at the byte after `#` on every profile; the control control_parameter_value_parenthesised.sv ACCEPTS.
module m;
  m1 #PAR u1();
  mreal #1.2 mr ();
endmodule

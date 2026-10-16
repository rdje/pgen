// SV-CORPUS-GRAD.50 (release 1.0.253) — the IEEE 1800 control for accepts_invalid_v2005_instance_empty_parameter_value_assignment.sv
// (REJECT under verilog_2005 from 1.0.253): the same text, an EMPTY parameter value assignment on a module instance, which IEEE
// 1800-2017 A.4.1.1 parameter_value_assignment ::= # ( [ list_of_parameter_assignments ] ) (docs/systemverilog/2017/md/section-41-data-read-api.md:1795) derives, as does IEEE
// 1800-2023 A.4.1.1 (docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:1804). ACCEPT under sv_2017 / sv_2023.
module mod #(parameter P = 1) (input a);
endmodule
module t;
  mod #() m1(1'b1);
endmodule

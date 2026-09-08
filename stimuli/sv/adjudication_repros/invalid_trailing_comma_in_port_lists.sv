// SV-CORPUS-GRAD.3.51 - a TRAILING COMMA in a parameter port list and a port list: `module ModuleA #(P=1,) (inp,);` (sv2v test/core/trailing_commas.sv). IEEE 1800-2017 A.1.3 parameter_port_list ::= # ( list_of_param_assignments { , parameter_port_declaration } ) | ... (docs/systemverilog/2017/md/section-41-data-read-api.md:231-236) and list_of_ports ::= ( port { , port } ) (docs/systemverilog/2017/md/section-41-data-read-api.md:246); 1800-2023 (docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:208-213, :223); IEEE 1364-2005 A.1.3 the same shapes - every comma is followed by an item.
// REJECTS at the `)` after the trailing comma on every profile; the control without the commas ACCEPTS.
module ModuleA #(P=1,) (inp,);
  input inp;
endmodule

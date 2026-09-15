// SV-CORPUS-GRAD.20 — a port declaration in the body of a module whose header lists NO ports.
// IEEE 1364-2005 A.1.2: a module with `list_of_ports` holds `{ module_item }` (which admits
// `port_declaration ;`); a module with `[ list_of_port_declarations ]` — the only form `module test;`
// can take — holds `{ non_port_module_item }`, which does not. IEEE 1800-2017 A.1.2/A.1.4 are the
// same split (`module_nonansi_header … { module_item }` vs `module_ansi_header … { non_port_module_item }`).
// Shape from iverilog/ivtest/ivltests/module_output_port_var2.v (`module test; output reg a;`), which
// Icarus accepts as an extension.
module test;
  output reg a;
endmodule

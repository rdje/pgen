// SV-CORPUS-GRAD.20 — the legal spelling of invalid_port_declaration_in_portless_module.sv:
// the header lists the port, so the body may declare it (1364-2005 A.1.2 / 1800-2017 A.1.4 module_item).
module test(a);
  output reg a;
endmodule

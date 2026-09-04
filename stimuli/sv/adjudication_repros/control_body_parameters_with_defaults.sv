// SV-CORPUS-GRAD.3.37 - a CONTROL - must keep ACCEPTING.
// parameter with defaults at $unit / module body / generate block / class body / function body / interface class / package, and #(parameter A = 1, B, type T = int, type U) omitting them in the port list
// IEEE 1800-2017 Annex A footnote 18 docs/systemverilog/2017/md/section-41-data-read-api.md:3518-3520: the default may be omitted only within a parameter_port_list
parameter UNIT_P = 4;
module t #(parameter A = 1, B, type T = int, type U) (); parameter C = 2; parameter int D = 3; parameter type E = logic; localparam F = 4;
  generate if (A) begin : g parameter G = 5; end endgenerate
  class K #(parameter W = 8); parameter H = 9; endclass
  function int f(); parameter I = 1; return I; endfunction
endmodule
interface class IC; parameter J = 1; endclass
package pk; parameter K = 3; endpackage

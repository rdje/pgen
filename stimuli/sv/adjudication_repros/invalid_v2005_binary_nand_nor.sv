// SV-CORPUS-GRAD.13e.2 — REJECT FOREVER under verilog_2005.
// `~&` and `~|` are REDUCTION (unary) operators in IEEE 1364-2005, never binary ones.
// A.8.6 binary_operator (…/section-Annex_A-normative-formal-syntax-definition.txt:954) lists
// neither; both appear only under unary_operator (:952) and unary_module_path_operator (:957).
// Upstream says the same in its own words: ivtest/ivltests/br_gh552.v passes `-gno-icarus-misc`
// and its golden reads "The binary NAND operator is an Icarus Verilog extension. Use
// -gicarus-misc to enable it." — i.e. accepting it is a documented NON-STANDARD extension.
// Expected FOREVER: REJECT on verilog_2005.
module test;
  reg [7:0] a, b, c;
  initial begin
    c = a ~& b;
    c = a ~| b;
  end
endmodule

// SV-CORPUS-GRAD.13y - an INVALID pin - text IEEE 1800-2017 derives nothing for; must keep REJECTING on every profile.
// module t (clk, input d); input clk; - a first port with everything omitted makes the list non-ANSI (23.2.2.3); the typed item after it and the body redeclaration both have no derivation
// IEEE 1800-2017 23.2.2.3 docs/systemverilog/2017/md/section-23-modules-and-hierarchy.md:472-474; A.1.3 list_of_ports docs/systemverilog/2017/md/section-41-data-read-api.md:246, port docs/systemverilog/2017/md/section-41-data-read-api.md:266; A.1.2 docs/systemverilog/2017/md/section-41-data-read-api.md:108-113
module t (clk, input d);
  input clk;
endmodule

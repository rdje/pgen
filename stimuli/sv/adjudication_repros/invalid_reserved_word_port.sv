// SV-CORPUS-GRAD.13c.2 — CORRECT REJECTION. `do` is a reserved SystemVerilog keyword and
// cannot name a port (IEEE 1800-2017 Table B-1). The corpus row is a verilator language-
// extension test that opts out with `// verilator lint_off SYMRSVDWORD`.
// ⛔ Accepting this would be an OVER-ACCEPTANCE defect. Expected FOREVER: REJECT.
module m(input do);
endmodule

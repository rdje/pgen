// SV-CORPUS-GRAD.13c.2 — CORRECT REJECTION. `$_DLATCH_P_` is a Yosys techmap cell name, not a
// legal SV identifier: `$` may not start one (IEEE 1800-2017 A.9.3 identifier / 5.6).
// ⛔ Accepting this would be an OVER-ACCEPTANCE defect. Expected FOREVER: REJECT.
module $_DLATCH_P_ (input E, input D, output Q);
endmodule

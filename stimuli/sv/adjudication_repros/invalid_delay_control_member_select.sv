// SV-CORPUS-GRAD.58a — CORRECT REJECTION. A delay control is `# delay_value` or `# ( mintypmax_expression )`
// (IEEE 1800-2017 A.6.5, docs/systemverilog/2017/md/section-41-data-read-api.md:2141-2146) and a delay_value is an
// unsigned_number, real_number, ps_identifier, time_literal or 1step (:948-958) — a ps_identifier carries no `.`, so a
// member select after `#` needs the parenthesized form. IEEE 1800-2023 the same (Annex A :2134-2138, :989-999).
// The corpus rows are the UVM 1800.2-2017 reference library's `#setting.offset;` (verilator
// t/uvm/uvm_pkg_all_v2017_1_0_{dpi,nodpi}.svh), which verilator accepts as an extension — a tolerance candidate for
// SV-CORPUS-GRAD.43's closed list, not in it; under the IEEE 1800 profiles the standard outranks the tool.
// ⛔ Accepting this would be an OVER-ACCEPTANCE defect. Expected FOREVER: REJECT.
module m;
  class c; int offset; endclass
  c s;
  initial begin
    #s.offset;
  end
endmodule

// SV-CORPUS-GRAD.13c.2a — FIXED in PGEN-SV-CORPUS-GRAD-0212 (was `defect_cross_body_item.sv`).
// A non-empty `cross_body` rejected every item it may contain, because the `;` was spelled TWICE:
// once in `cross_body_sv_2017` and once in `cross_body_item_sv_2017`. IEEE 1800-2017 A.2.11 does
// write it in both places — and §19.6.2/§19.6.3 of the same standard write ONE, as does every row
// in the vendored corpus. Fix: the `;` now lives only on the item, matching the sv_2023 pair.
// Expected FOREVER (a fixed defect must not regress): ACCEPT.
//
// ⚠️ This reproducer deliberately no longer uses the `select_expression … with ( … )` form it was
// first written with. That form still REJECTS, for a DIFFERENT reason now pinned separately by
// `defect_select_expression_with.sv` (`.13c.2a.2`) — a reproducer that fails for two reasons
// isolates neither, and while it held both it was crediting the `with` defect to the `;` defect.
module m;
  bit a, b;
  covergroup cg;
    ca: coverpoint a;
    cb: coverpoint b;
    x: cross ca, cb {
      option.weight = 2;
      ignore_bins ib = binsof(ca) intersect { 1 };
    }
  endgroup
endmodule

// SV-CORPUS-GRAD.13c.2a — CORRECT REJECTION, and the OVER-ACCEPTANCE the `.2a` fix closed.
// Before that fix the doubled `;` (in `cross_body_sv_2017` AND `cross_body_item_sv_2017`) meant
// this form — and ONLY this form — parsed, while every single-semicolon spelling the standard's
// own examples use did not. `cross_body_item` is non-nullable, so `( cross_body_item semi )*` can
// no longer absorb a stray `;`.
// ⛔ Accepting this again would mean the fix was undone in the widening direction, which a
// pass-rate metric cannot see. Expected FOREVER: REJECT.
module m;
  bit a, b;
  covergroup cg;
    ca: coverpoint a;
    cb: coverpoint b;
    x: cross ca, cb { option.weight = 2;; }
  endgroup
endmodule

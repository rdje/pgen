// SV-CORPUS-GRAD.13c.2 — CONFIRMED DEFECT (rejects-valid).
// A non-empty `cross_body` rejects every `bins_selection_or_option` item.
// IEEE 1800-2017 A.2.11:
//   cross_body      ::= { { cross_body_item ; } } | ;
//   cross_body_item ::= function_declaration | bins_selection_or_option ;
// so both an `ignore_bins` selection and an `option.` assignment are legal here.
// Expected today: REJECT.  Expected once fixed: ACCEPT.
module m;
  bit a, b;
  covergroup cg;
    ca: coverpoint a;
    cb: coverpoint b;
    x: cross ca, cb {
      ignore_bins ib = ca with (a == 1);
    }
  endgroup
endmodule

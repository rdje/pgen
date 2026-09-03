// SV-CORPUS-GRAD.13x.b - an INVALID pin: text no Annex A production derives - must keep REJECTING.
// total = q.sum() with (item) { item > 0; }; - a constraint block on an array method's with
// IEEE 1800-2017 A.8.2 array_manipulation_call ... [ with ( expression ) ] section-41-data-read-api.md:2954-2957; the constraint_block form is randomize_call's section-41-data-read-api.md:2961-2964. t_randomize_with_idlist_method_bad.v l.13.
module t; int q[$], total;
  initial total = q.sum() with (item) { item > 0; };
endmodule

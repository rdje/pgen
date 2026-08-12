// ⭐⭐ PROMOTED TO A GENUINE `&&` CONTROL by `.13c.2e` (`PGEN-SV-CORPUS-GRAD-0214`) — and it
// predicted its own promotion. From `PGEN-SV-CORPUS-GRAD-0213` to `.13c.2e` this file was a
// MASKING PIN carrying the arm `condition,!and`: the text parsed, but through exactly one
// `condition` node and ZERO `and` nodes, with `binsof(cb)` absorbed as a plain subroutine call
// inside `select_condition`'s `covergroup_range_list*` — because `intersect`'s LITERAL braces
// had been transcribed as EBNF repetition. Its comment then said: *"When `.13c.2a.1` restores
// the literal braces the seed will stop over-consuming, this row will FAIL, and that failure is
// the signal to re-adjudicate it as a genuine `&&` control."* That is precisely what happened —
// the oracle failed on `!and` the moment the braces landed, which is a two-sided ratchet doing
// its job rather than a regression.
// ⭐ THE LESSON THAT COST A LEAF, kept because it is what made the prediction possible: ruling
// out ONE accidental route is not ruling out the accidental route. Read the arm out of the AST.
// The 0212 claim — *"both operands carry the KEYWORD `intersect`, so this cannot be slipping
// through as one ordinary SV expression"* — was FALSE, and only the AST said so.
// ⛔ WHAT THE ARM PINS NOW: `and>condition` — an `and` node with a `condition` beneath it, so
// the `&&` continuation is what joined the operands; `!concat` forbids the old swallowing route,
// so a revert to `covergroup_range_list*` turns this row red again instead of silently green.
module m;
  bit [2:0] a, b;
  covergroup cg;
    ca: coverpoint a;
    cb: coverpoint b;
    x: cross ca, cb { ignore_bins ib = binsof(ca) intersect { 1 } && binsof(cb) intersect { 2 }; }
  endgroup
endmodule

// SV-CORPUS-GRAD.13c.2d — FIXED in PGEN-SV-CORPUS-GRAD-0220 (release 1.0.184, ledger SV-0054).
// IEEE 1800-2017 §19.6.1's OWN worked example: a `cross` body that DECLARES a function and uses it
// in a `bins`. It REJECTED under sv_2017 (furthest_position=65) while sv_2023 accepted it, because
// A.2.11 of the 2017 edition MISSPELLS the nonterminal as `function_declaraton` and the LRM->EBNF
// extraction transcribed the typo as a literal keyword token — an alternative no source can reach.
//
// ⛔ PINNED BY .13c.2l, WHICH IS THE POINT OF THE ROW. The fix shipped on 2026-08-17 with NO pinned
// witness, so the accept-set-transition sweep that derives this contract's release list reported
// `-0220` as moving nothing. An unpinned widening is invisible to every oracle in this repository.
// Expected FOREVER (a fixed defect must not regress): ACCEPT under sv_2017.
module m;
  int x, y;
  covergroup cg;
    a: coverpoint x;
    b: coverpoint y;
    aXb : cross a, b
    {
      function int myFunc1(int p, int q);
        return p + q;
      endfunction
      bins one = myFunc1(2, 5);
    }
  endgroup
endmodule

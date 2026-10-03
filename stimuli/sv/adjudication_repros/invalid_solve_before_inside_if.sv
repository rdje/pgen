// SV-CORPUS-GRAD.43 - an INVALID pin - the boundary of the tolerant profile's closed list; must keep REJECTING on every profile, `sv_2023_tolerant` included.
// foreach (x[i]) { if (y > 0) { solve y before x[i]; } } - a solve ... before under an if, not directly in the foreach braces
// IEEE 1800-2017 A.1.10 docs/systemverilog/2017/txt/section-18-constrained-random-value-generation.txt:328-330, :339-341; the tolerant profile admits solve only as a direct item of a foreach constraint set
class c;
  rand int x[2];
  rand int y;
  constraint k {
    foreach (x[i]) {
      if (y > 0) {
        solve y before x[i];
      }
    }
  }
endclass

// SV-CORPUS-GRAD.41 - a CONTROL - the legal neighbour; must keep ACCEPTING on every profile.
// solve y before x; as a constraint BLOCK item, beside a foreach constraint set
// IEEE 1800-2017 A.1.10 docs/systemverilog/2017/txt/section-18-constrained-random-value-generation.txt:328-330
class c;
  rand int x[2];
  rand int y;
  constraint k {
    solve y before x;
    foreach (x[i]) {
      x[i] > y;
    }
  }
endclass

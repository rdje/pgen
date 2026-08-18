// The control for the .13c.2o pair: a plain covergroup in the same class, no `extends`. It parses
// on both SV profiles before and after the fix, so the flip is attributable to the extends branch
// alone — not to the class carrier, not to the empty covergroup body.
class c;
  covergroup base;
  endgroup
endclass

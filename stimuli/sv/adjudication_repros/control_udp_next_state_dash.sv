// SV-CORPUS-GRAD.13x.b - a CONTROL: the nearest LEGAL twin (one token different) - must keep ACCEPTING.
// 1 1 : ? : -;
// IEEE 1800-2017 A.5.3 section-29-user-defined-primitives.md:129. isolates the defect to the next-state symbol.
primitive p(dout, a, b); output reg dout; input a, b;
  table
    1 1 : ? : -;
  endtable
endprimitive

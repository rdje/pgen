// SV-CORPUS-GRAD.13x.b - a CONTROL: the nearest LEGAL twin (one token different) - must keep ACCEPTING.
// 0 ? 1 : 1; 1 1 0 : 0; - level symbols only, 0/1 outputs
// IEEE 1800-2017 A.5.3 section-29-user-defined-primitives.md:119, :130-131. isolates the ? output and the edge defects to their one symbol each.
primitive t_gate(dout, a, b, c); output dout; input a, b, c;
  table
    0 ? 1 : 1;
    1 1 0 : 0;
  endtable
endprimitive

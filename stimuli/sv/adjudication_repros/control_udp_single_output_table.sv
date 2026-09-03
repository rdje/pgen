// SV-CORPUS-GRAD.13x.b - a CONTROL: the nearest LEGAL twin (one token different) - must keep ACCEPTING.
// primitive udp_x (a, b); output a; input b; table 0 : 1; 1 : 0; endtable endprimitive
// IEEE 1800-2017 A.5.1-A.5.3 section-41-data-read-api.md:1930-1988. the closed, single-output, port-declared twin of the tri / two-output / unterminated invalid pins.
primitive udp_x (a, b); output a; input b;
  table
    0 : 1;
    1 : 0;
  endtable
endprimitive

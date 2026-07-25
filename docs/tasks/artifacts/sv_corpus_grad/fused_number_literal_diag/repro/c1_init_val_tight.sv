// CONTROL (ACCEPT at baseline, must HOLD): tight `1'b1` init_val — IEEE 1800-2017 A.5.2
primitive p (q, clk, d);
  output q;
  reg q;
  input clk, d;
  initial q = 1'b1;
  table
    r 0 : ? : 0 ;
  endtable
endprimitive

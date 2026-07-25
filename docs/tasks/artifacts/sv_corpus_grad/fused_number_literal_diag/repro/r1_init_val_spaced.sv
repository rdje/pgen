// DEFECT (REJECT at baseline): §5.7.1-spaced `1 'b 1` init_val
// IEEE 1800-2017 §5.7.1: white space is legal between the size and the `'`,
// and between the base format and the value.
primitive p (q, clk, d);
  output q;
  reg q;
  input clk, d;
  initial q = 1 'b 1;
  table
    r 0 : ? : 0 ;
  endtable
endprimitive

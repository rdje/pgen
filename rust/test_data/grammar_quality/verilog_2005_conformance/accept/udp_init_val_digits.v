// SV-0030 accept-lock (SV-DOLLAR-LRM-FIDELITY.3): the IEEE UDP init_val digit
// alternatives (1364-2005 / 1800-2017 A.5.1) must accept under every profile.
// Before that fix the digit-less "1'b" token prefix-stole the spelling and
// `initial q = 1'b0;` REJECTED everywhere.
primitive p0 (q, d);
  output q; reg q;
  input d;
  initial q = 1'b0;
  table
    0 : ? : 0 ;
  endtable
endprimitive

primitive p1 (q, d);
  output q; reg q;
  input d;
  initial q = 1'b1;
  table
    1 : ? : 1 ;
  endtable
endprimitive

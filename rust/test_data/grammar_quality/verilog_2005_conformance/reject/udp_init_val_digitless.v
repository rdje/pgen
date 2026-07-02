// SV-0030 reject-lock (SV-DOLLAR-LRM-FIDELITY.3): the digit-less non-language
// text `1'b` is NOT an IEEE init_val alternative and must REJECT everywhere
// (it wrongly ACCEPTED before the fix).
primitive p (q, d);
  output q; reg q;
  input d;
  initial q = 1'b;
  table
    0 : ? : 0 ;
  endtable
endprimitive

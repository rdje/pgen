package p;
  typedef struct { real field1; bit field2; } T;
  function automatic T Tsum(input T driver[]);
    Tsum.field1 = 0.0;
  endfunction
  nettype T wTsum with Tsum;
  nettype real realNet;
endpackage

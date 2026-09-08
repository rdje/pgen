interface test_if;
  extern function myfunc (input logic val);
  logic data;
  modport mp_e(export myfunc, output data);
endinterface

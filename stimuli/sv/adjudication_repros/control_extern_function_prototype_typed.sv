class Cls;
  extern function int ext_f_i(int in);
  extern function int ext_f_np;
endclass
function int Cls::ext_f_i(int in);
  return in;
endfunction
function int Cls::ext_f_np;
  return 1;
endfunction

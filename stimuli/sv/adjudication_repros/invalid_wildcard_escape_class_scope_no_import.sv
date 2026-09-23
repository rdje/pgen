// SV-CORPUS-GRAD.31 - an INVALID pin (REJECT on both SV profiles): the class-scoped static call with NO wildcard import open -
// the parser can prove the class undeclared, so the escape must not fire (SV-PARSE-STRICT.2, sound-not-complete).
class d;
  function void g();
    uvm_config_db#(int)::set(null, "*", "w", 1);
  endfunction
endclass

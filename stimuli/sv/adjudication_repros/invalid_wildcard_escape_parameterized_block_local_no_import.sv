// SV-CORPUS-GRAD.30 - an INVALID pin (REJECT on both SV profiles): the block-local parameterized class type with NO wildcard
// import open - the parser can prove the name undeclared, so the escape must not fire (SV-PARSE-STRICT.2, sound-not-complete).
module m;
  initial begin
    uvm_tlm_b_target_socket_base #(int) bt;
  end
endmodule

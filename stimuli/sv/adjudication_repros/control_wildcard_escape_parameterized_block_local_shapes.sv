// SV-CORPUS-GRAD.30 - a CONTROL pin (ACCEPT from 1.0.242 on both SV profiles): the parameterized-class wildcard escape at
// every BLOCK ITEM type position - a task local, a class-constructor local, a fork-join local in a module, two declarators
// with an initializer, a named parameter list in an always block, and a scope-chain type `C #(int)::T x;`.
// IEEE 1800-2017 A.2.8 block_item_declaration ::= { attribute_instance } data_declaration; A.1.9 class_type; 26.3 / 26.4.
import q::*;
class c;
  task run();
    uvm_tlm_fifo #(int) fifo;
  endtask
  function new();
    uvm_tlm_analysis_fifo #(int) af;
  endfunction
  function void f();
    uvm_tlm_fifo #(int) a, b = new("b");
    uvm_reg_predictor #(int)::T t;
  endfunction
endclass
module m;
  initial fork
    uvm_tlm_fifo #(int) fifo;
  join
  always begin
    uvm_tlm_fifo #(.T(int), .N(2)) nf;
  end
endmodule

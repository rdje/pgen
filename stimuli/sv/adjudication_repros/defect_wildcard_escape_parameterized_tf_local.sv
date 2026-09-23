// SV-CORPUS-GRAD.29 - a DEFECT pin (REJECT through 1.0.241, both SV profiles) - FIXED by SV-CORPUS-GRAD.30 (release 1.0.242: wildcard_escape_block_class_type, the LAST arm of block_data_type; class fixed / ACCEPT).
// A PARAMETERIZED class type whose class is unresolved under an open wildcard import, declaring a LOCAL of a function body
// inside a class: the .28 escape (wildcard_escape_class_type, on class_type's second arm, gated on the class_body scope) is
// reached by data_type - a class MEMBER parses (control_wildcard_escape_parameterized_class_member.sv) - but a block item's
// type is read by block_data_type, whose *_block_class_type twins carry no escape, so the bare-name escape reads the class
// name and the declaration stops before `#(`. IEEE 1800-2017 A.2.8 block_item_declaration ::= { attribute_instance }
// data_declaration; A.1.9 class_type ::= ps_class_identifier [ parameter_value_assignment ] ...; 26.3 / 26.4.
// Surelog tests/SplitFile/amiq_eth_ve_top.v line 10592 `uvm_tlm_b_target_socket_base #(T) bt;` (the RESIDUAL dark row after .29).
import q::*;
class c;
  static function void connect(int port);
    uvm_tlm_b_target_socket_base #(int) bt;
  endfunction
endclass

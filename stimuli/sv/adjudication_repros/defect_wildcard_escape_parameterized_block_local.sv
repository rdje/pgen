// SV-CORPUS-GRAD.29 - a DEFECT pin (REJECT through 1.0.241, both SV profiles) - FIXED by SV-CORPUS-GRAD.30 (release 1.0.242: wildcard_escape_block_class_type, the LAST arm of block_data_type; class fixed / ACCEPT).
// The same parameterized class type, unresolved under an open wildcard import, declaring a LOCAL of a begin-end block in a
// module (IEEE 1800-2017 A.6.3 seq_block ::= begin [ : block_identifier ] { block_item_declaration } { statement_or_null } end;
// A.2.8 block_item_declaration ::= { attribute_instance } data_declaration): block_data_type carries no escape.
import q::*;
module m;
  initial begin
    uvm_tlm_b_target_socket_base #(int) bt;
  end
endmodule

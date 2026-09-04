// SV-CORPUS-GRAD.3.37 - an INVALID pin - must keep REJECTING on every profile.
// function new; input x; - a tf_port_declaration inside a class constructor body (was parsed as an ordinary function NAMED new)
// IEEE 1800-2017 Annex B Table B.1 docs/systemverilog/2017/md/section-41-data-read-api.md:3599-3858 (248 reserved words; 1800-2023 Annex B lists the same 248, docs/systemverilog/2023/md/section-Annex_B-normative-keywords.md); 5.6.2 docs/systemverilog/2017/md/section-1024-characters-if-an-identifier-exceeds-the-implementation-specific-length-limit-an-error-shall-be-reported.md:52; A.2.7 class_constructor_declaration ::= function [ class_scope ] new [ ( [ tf_port_list ] ) ] ; { block_item_declaration } ... docs/systemverilog/2017/md/section-41-data-read-api.md:622-626 - block_item_declaration, not tf_item_declaration
module test;
  class C;
    function new;
      input x;
      $display("FAILED");
    endfunction
  endclass
  C c = new;
endmodule

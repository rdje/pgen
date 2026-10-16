// SV-CORPUS-GRAD.50 (release 1.0.253) — the UPPER bound of packed_dimension_list_v2005 and the shape-neutrality control: the
// same six sites (a reg variable, a named-block reg, a function range, an ANSI input port, an output reg port, a task port),
// each with ONE packed range, plus one with none — every form IEEE 1364-2005 derives (reg_declaration docs/verilog/2005/txt/section-Annex_A-normative-formal-syntax-definition.txt:172, A.2.8
// block_item_declaration, function_range_or_type :268, input_declaration :135, output_declaration :137-139, tf_input_declaration
// :296-305; range :252). ACCEPT on every profile; the typed AST byte-identical to the 1.0.252 tree (measured by the interpreter
// A/B and the AST identity sweep): the v2005 twin renders the same dims list as the 1800 repetition for 0 and 1 dimension.
module m(input [7:0] a, output reg [1:0] q, input signed [3:0] c, input d);
  reg [7:0] r;
  reg s;
  initial begin : b
    reg [1:0] t;
  end
  function [1:0] f;
    input x;
    f = x;
  endfunction
  task k(input [1:0] v);
    ;
  endtask
endmodule

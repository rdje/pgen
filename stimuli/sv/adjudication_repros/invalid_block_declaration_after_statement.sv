// SV-CORPUS-GRAD.13y - an INVALID pin - text IEEE 1800-2017 derives nothing for; must keep REJECTING on every profile.
// initial begin x = 1; int y = 0; end - a block item declaration after a statement
// IEEE 1800-2017 A.6.3 seq_block ::= begin [ : block_identifier ] { block_item_declaration } { statement_or_null } end docs/systemverilog/2017/md/section-41-data-read-api.md:2064-2066; A.2.6 function_body_declaration docs/systemverilog/2017/md/section-41-data-read-api.md:1091-1099
module t;
  int x;
  initial begin
    x = 1;
    int y = 0;
    x = y;
  end
endmodule

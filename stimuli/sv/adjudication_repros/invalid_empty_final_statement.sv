// SV-CORPUS-GRAD.13y - an INVALID pin - text IEEE 1800-2017 derives nothing for; must keep REJECTING on every profile.
// final ; - a final construct with no statement
// IEEE 1800-2017 A.6.2 final_construct ::= final function_statement docs/systemverilog/2017/md/section-41-data-read-api.md:2022; function_statement ::= statement docs/systemverilog/2017/md/section-41-data-read-api.md:2117; statement_or_null docs/systemverilog/2017/md/section-41-data-read-api.md:2081-2083
module t;
  final ;
endmodule

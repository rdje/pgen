// SV-CORPUS-GRAD.3.37 - a CONTROL - must keep ACCEPTING.
// integer x [] = 1; - A.2.4's first alternative: variable_identifier { variable_dimension } [ = expression ], and unsized_dimension IS a variable_dimension
// IEEE 1800-2017 A.2.4 variable_decl_assignment docs/systemverilog/2017/md/section-41-data-read-api.md:1028-1034; A.2.5 variable_dimension ::= unsized_dimension | unpacked_dimension | associative_dimension | queue_dimension docs/systemverilog/2017/md/section-41-data-read-api.md:1070, unsized_dimension ::= [ ] docs/systemverilog/2017/md/section-41-data-read-api.md:1079
module top;
  integer x [] = 1;
endmodule

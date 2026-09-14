// SVPP-EXPANSION.8 - a region naming the profile's OWN version unreserves nothing, so ordinary text
// inside it is untouched and the two directives themselves parse (A.1.2 / §22.14 Syntax 22-10).
// The neighbouring construct for every fixed_begin_keywords_* row.
`begin_keywords "1800-2017"
module top;
  reg [63:0] ordinary_name;
  logic [1:0] still_a_keyword;
endmodule
`end_keywords

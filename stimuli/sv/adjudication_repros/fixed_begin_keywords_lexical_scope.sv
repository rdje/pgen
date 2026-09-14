// SVPP-EXPANSION.8 - the rendering is LEXICAL. A word that is not a token of the design - inside a
// line comment, a block comment, a string literal, or an escaped identifier the author already
// wrote - is left exactly as it stands; only the real identifier is rendered. §5.4, §5.9, §5.6.1.
`begin_keywords "1364-2001"
module top;
  // logic in a line comment
  /* logic in a block comment */
  reg [63:0] logic;
  wire \logic_already = 1'b1;
  initial $display("logic in a string");
endmodule
`end_keywords

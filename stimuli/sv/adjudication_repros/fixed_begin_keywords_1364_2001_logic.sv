// SVPP-EXPANSION.8 - IEEE 1800-2017 §22.14.1's own example. Inside a `begin_keywords "1364-2001"
// region `logic` is not a reserved word, so it is a legal net name; §22.14: "only the identifiers
// listed as reserved keywords in the specified version are considered to be reserved words".
// The preprocessor hands it to the parser as the escaped identifier `\logic ` (§5.6.2: "A
// SystemVerilog keyword preceded by an escape character is not interpreted as a keyword").
// Corpus rows: ispras-sv-tests 22/22.14.01_01.sv, sv-tests chapter-5/5.6.4--compiler-directives-begin-keywords.sv.
`begin_keywords "1364-2001"
module top;
  reg [63:0] logic;
endmodule
`end_keywords

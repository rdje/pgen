// SVPP-EXPANSION.6f - a CONTROL pin (preprocess=1) - a backtick inside a string literal or a comment is the text's own, and the LRM directives the parser reads are not unresolved usages; must keep ACCEPTING on every profile.
`timescale 1ns/1ps
`default_nettype none
module top;
  // a `comment_word here
  /* and `another one */
  initial $display("a `string_word too");
endmodule
`default_nettype wire

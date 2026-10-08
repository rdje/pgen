// SVPP-EXPANSION.6h - a CONTROL pin (preprocess=1) - an escaped identifier carrying a backtick is ONE token of the language; must keep ACCEPTING on every profile.
// IEEE 1800-2017 5.6.1 "Escaped identifiers shall start with the backslash character (\) and end with white space (space, tab, newline). They provide a means of including any of the printable ASCII characters in an identifier" (docs/systemverilog/2017/md/section-1024-characters-if-an-identifier-exceeds-the-implementation-specific-length-limit-an-error-shall-be-reported.md:36-38); the grammar's escaped_identifier := /\\[!-~]+/.
// Through 1.0.251 the stage's scan read the backtick INSIDE \a`b as a usage of `b and refused the file ("`b is neither a defined macro nor a compiler directive"); iverilog ivltests/mangle.v writes an escaped identifier of every printable character.
module top;
  wire \a`b ;
  assign \a`b = 1'b1;
endmodule

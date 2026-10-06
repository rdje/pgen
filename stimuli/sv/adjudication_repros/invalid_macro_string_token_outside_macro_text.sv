// SVPP-EXPANSION.6f - an INVALID pin (preprocess=1) - a `" macro-string token in SOURCE text, outside any `define; must keep REJECTING on every profile.
// IEEE 1800-2017 22.5.1 gives `" (and `\`", ``) meaning only within the text of a `define (docs/systemverilog/2017/md/section-22-compiler-directives.md: the macro-text paragraphs); at the top level it is no token of the language. sv2v test/lex/string_macro.sv:9 is pinned must-reject on exactly this (SVPP-EXPANSION.6c.2); before this the stage left the bytes and the PARSER rejected them - the refusal is now the stage's, with the clause.
module top;
initial $display(`"standalone`");
endmodule

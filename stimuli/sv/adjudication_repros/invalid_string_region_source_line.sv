// SVPP-EXPANSION.5(j) - INVALID, the negative control: a `"…`" region in a SOURCE line is NOT rendered - §22.5.1 defines
// the form for macro text ("The `define macro text can also include `\", `\\`\", and ``"), and sv2v
// test/lex/string_macro.sv:9 is pinned must-reject on exactly this shape (SVPP-EXPANSION.6c.2). It must
// stay rejected through the pipeline as well as raw: the rescan's second pass over a source line used to
// carry expansion-text provenance and rendered the author's own region the moment a usage shared its line.
`define FOO BAR
module top;
    initial $display(`"standalone `FOO`");
endmodule

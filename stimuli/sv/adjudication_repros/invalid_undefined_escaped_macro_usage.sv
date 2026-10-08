// SVPP-EXPANSION.6h - an INVALID pin (preprocess=1) - a text-macro usage whose name is an escaped identifier with no definition; must keep REJECTING on every profile.
// IEEE 1800-2017 22.5.1 Syntax 22-2 `text_macro_identifier ::= identifier` and A.9.3 identifier ::= simple_identifier | escaped_identifier (SVPP-EXPANSION.5(q)(ii): \esc and esc name ONE macro); 22.5.1 gives a usage its meaning by substitution (docs/systemverilog/2017/md/section-22-compiler-directives.md:272-273) - no definition, no substitution.
// Through 1.0.251 the refusal named the wrong clause (the `\ read as the prefix of the `\`" macro-string token); the verdict was right, the sentence is now the undefined-name one.
module top;
endmodule
`\not_defined

// SVPP-EXPANSION.5(q) - a FIXED pin (preprocess=1) - an ESCAPED identifier as a macro name, defined and used in both spellings, with formals after the single terminating space; must keep ACCEPTING on every profile.
// define \M@CRO3 (a=5, b=0, c="C") ... (the 1800-2023 example) and define \escaped ... used as `escaped and `\escaped (iverilog ivltests/escaped_macro_name.v); before: "missing macro name in `define"
// IEEE 1800-2017 22.5.1 Syntax 22-2 text_macro_identifier ::= identifier (docs/systemverilog/2017/txt/section-22-compiler-directives.txt:131-132) with A.9.3 identifier ::= simple_identifier | escaped_identifier; IEEE 1800-2023 22.5.1 "The macro name may be either a simple_identifier or an escaped_identifier (see 5.6)" (docs/systemverilog/2023/txt/section-22-compiler-directives.txt:130), "only the single white space character terminating the identifier name ... shall separate the opening parenthesis from the macro name" (docs/systemverilog/2023/txt/section-22-compiler-directives.txt:166-167), the \M@CRO3 example (docs/systemverilog/2023/txt/section-22-compiler-directives.txt:268-275); 5.6.1 "an escaped identifier \cpu3 is treated the same as a nonescaped identifier cpu3" (docs/systemverilog/2017/txt/section-5-lexical-conventions.txt)
`define \M@CRO3 (a=5, b=0, c="C") $display(a,,b,,c);
`define \escaped "escaped name"
`define simple "simple name"
module m;
  initial begin
    `\M@CRO3 ( 1 )
    `\M@CRO3 ( )
    $display(`escaped);
    $display(`\escaped );
    $display(`\simple );
  end
endmodule

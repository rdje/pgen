// SVPP-EXPANSION.6f - an INVALID pin (preprocess=1) - a function-like macro whose argument list the file never closes; must keep REJECTING on every profile.
// IEEE 1800-2017 22.5.1 Syntax 22-3 `text_macro_usage ::= `text_macro_identifier [ ( list_of_actual_arguments ) ]` (docs/systemverilog/2017/md/section-22-compiler-directives.md:197-201): the closing parenthesis is part of the usage. verilator t_preproc_eof3_bad.v is the shape (`FOO(1, at end of file); before this the incomplete usage was kept as written and accepted as an opaque directive line.
`define FOO(a,b) a b
`FOO(1,

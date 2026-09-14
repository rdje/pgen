// SVPP-EXPANSION.8 - §22.14.4: "1364-2001-noconfig" behaves like "1364-2001" "with the exception
// that the following identifiers are excluded from the reserved list in Table 22-2" - the ten
// configuration keywords, `cell` among them. `automatic` is NOT excluded and stays a keyword, which
// is what separates this row from fixed_begin_keywords_1364_1995_automatic.sv.
// Corpus row: sv2v test/lex/begin_keywords.sv:15.
`begin_keywords "1364-2001-noconfig"
task automatic baz;
    integer cell = 3;
    $display(cell ** cell);
endtask
`end_keywords

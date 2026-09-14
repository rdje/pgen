// SVPP-EXPANSION.8 - §22.14: "The `begin_keywords...`end_keywords directive pair can be nested.
// Each nested pair is stacked so that when an `end_keywords directive is encountered, the
// implementation returns to using the version_specifier that was in effect prior to the matching
// `begin_keywords directive." The inner "1364-2005" region keeps `automatic` a KEYWORD (it is in
// Table 22-2) while unreserving `logic`; the outer "1364-1995" region unreserves `automatic`.
// Corpus row: sv2v test/lex/begin_keywords.sv - the whole file.
`begin_keywords "1364-1995"
task foo;
    integer automatic = 2;
endtask
`begin_keywords "1364-2005"
task automatic bar;
    integer logic = 3;
endtask
`end_keywords
task baz;
    integer automatic = 4;
endtask
`end_keywords

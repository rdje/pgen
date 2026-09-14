// SVPP-EXPANSION.8 - `automatic` enters the reserved list in IEEE 1364-2001 (Table 22-2), so inside
// a `begin_keywords "1364-1995" region it is an ordinary identifier (§22.14.2, Table 22-1).
// Corpus row: sv2v test/lex/begin_keywords.sv:3.
`begin_keywords "1364-1995"
task foo;
    integer automatic = 2;
    $display(automatic * automatic);
endtask
`end_keywords

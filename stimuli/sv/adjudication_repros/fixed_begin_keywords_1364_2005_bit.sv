// SVPP-EXPANSION.8 - `bit` enters the reserved list in IEEE 1800-2005 (Table 22-4), so inside a
// `begin_keywords "1364-2005" region it is an ordinary net name (§22.14.5, Tables 22-1..22-3).
// Corpus row: Surelog tests/1364_2005/dut.v:8.
`begin_keywords "1364-2005"
module main;
   reg [3:0] foo, bar;
   reg       bit, rst, clk;
endmodule
`end_keywords

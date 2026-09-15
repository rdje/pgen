`define message(TOTO) TATA
`define MAKE_MEM_PATH(filename) `"`MEM_ROOT/filename`message`"
module t; initial $display(`MAKE_MEM_PATH(rom.mem)); endmodule

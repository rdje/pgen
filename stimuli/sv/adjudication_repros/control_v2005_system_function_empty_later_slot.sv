module m; integer code, fd, count; reg [7:0] mem [0:3]; initial code = $fread(mem, fd, , count); endmodule

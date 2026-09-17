module m; reg a, b; function type; input x; type = x; endfunction initial if (type(a) == type(b)) $display("same"); endmodule

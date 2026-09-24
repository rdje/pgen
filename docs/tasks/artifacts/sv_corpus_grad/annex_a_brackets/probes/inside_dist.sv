module m; int x; class c; rand int y; constraint k { y dist {[1:3] := 1, 5 :/ 2}; } endclass initial if (x inside {[1:3], 7}) ; endmodule

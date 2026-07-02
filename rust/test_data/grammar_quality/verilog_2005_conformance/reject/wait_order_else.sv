module m;
  event a, b;
  initial wait_order (a, b) else $display("bad");
endmodule

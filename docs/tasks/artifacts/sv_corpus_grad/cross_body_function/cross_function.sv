module m;
  int x, y;
  covergroup cg;
    a: coverpoint x;
    b: coverpoint y;
    aXb : cross a, b
    {
      function int myFunc1(int p, int q);
        return p + q;
      endfunction
      bins one = myFunc1(2, 5);
    }
  endgroup
endmodule

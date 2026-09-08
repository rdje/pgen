module top;
  interface class IntfD;
    pure virtual function bit[1:0] funcD();
  endclass : IntfD
  class ClassB;
    virtual function bit[1:0] funcD(); return 0; endfunction
  endclass : ClassB
endmodule

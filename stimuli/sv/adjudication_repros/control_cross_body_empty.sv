// Control for `defect_cross_body_item.sv`: the SAME cross with an EMPTY body parses, so the
// defect is the cross_body ITEM, not the cross, the coverpoints or the covergroup.
module m;
  bit a, b;
  covergroup cg;
    ca: coverpoint a;
    cb: coverpoint b;
    x: cross ca, cb { }
  endgroup
endmodule

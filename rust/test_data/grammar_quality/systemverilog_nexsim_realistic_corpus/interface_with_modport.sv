interface bus_if(input logic clk);
  logic req;
  logic gnt;
  modport m(output req, input gnt);
endinterface

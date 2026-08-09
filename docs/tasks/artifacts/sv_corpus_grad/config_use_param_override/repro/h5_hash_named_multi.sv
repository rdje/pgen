// SV-CORPUS-GRAD.3.19 repro — the config `use` clause parameter-override spelling.
module adder #(parameter ID = "id", W = 8, D = 512) ();
endmodule: adder
module top(); adder a1(); endmodule
config cfg;
  design rtlLib.top;
  instance top.a1 use #(.P1(), .P2("override.u_a.p2"));
endconfig

// control — the ORDINARY parameter_value_assignment path, which must keep accepting
// positional overrides. Only the config `use` clause forbids them (LRM 33.4.3).
module adder #(parameter ID = "id", W = 8, D = 512) ();
endmodule: adder
module top(); adder #(8, 16) a1(); adder #(.W(8)) a2(); endmodule
